use crate::audit;
use crate::config::FeatureGates;
use crate::error::{AetherError, ErrorContext};
use crate::tools::helpers;

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::process::Command;

pub async fn handle(gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let _ = gates;
    let ctx = ErrorContext::new("process_control", action);
    let r: Result<String, AetherError> = match action {
        "list" => Ok(helpers::cmd_params(&params, "ps", &["aux"])),
        "tree" => Ok(helpers::cmd_params(&params, "ps", &["aux", "--forest"])),
        "kill" => signal_action(&params, &ctx, Signal::SIGKILL, "Process killed"),
        "signal" => {
            let sig_str = helpers::p(&params, "signal");
            let sig_val: i32 = sig_str.parse().unwrap_or(15);
            match Signal::try_from(sig_val) {
                Ok(sig) => signal_action(&params, &ctx, sig, "Signal sent"),
                Err(e) => Err(AetherError::invalid_param(ctx.clone(), "signal", format!("{}", e))),
            }
        }
        "suspend" => signal_action(&params, &ctx, Signal::SIGSTOP, "Process suspended"),
        "resume" => signal_action(&params, &ctx, Signal::SIGCONT, "Process resumed"),
        "create" => create(&params, &ctx),
        "set_priority" => (|| -> Result<String, AetherError> {
            helpers::require_force_or_dry_run(&params, &ctx)?;
            let pid_val = pid(&params)?;
            let prio_str = helpers::p(&params, "priority");
            let prio_val: i32 = prio_str.parse().unwrap_or(0);
            if helpers::dry_run(&params) {
                return Ok(helpers::preview_action(
                    helpers::RiskLevel::Dangerous,
                    format!("set priority of pid {pid_val} to {prio_val}"),
                ));
            }
            set_priority_impl(pid_val, prio_val, &ctx)?;
            Ok("Priority set".to_string())
        })(),
        "set_nice" => helpers::require_force_or_dry_run(&params, &ctx).map(|_| {
            let pid = helpers::p(&params, "pid");
            let nice = helpers::p(&params, "nice");
            helpers::cmd_params(&params, "renice", &["-n", nice.as_str(), "-p", pid.as_str()])
        }),
        "query_info" => {
            let pid = helpers::p(&params, "pid");
            Ok(helpers::cmd_params(
                &params,
                "ps",
                &["-p", pid.as_str(), "-o", "pid,ppid,user,%cpu,%mem,cmd", "--no-headers"],
            ))
        }
        "threads" => {
            let pid = helpers::p(&params, "pid");
            Ok(helpers::cmd_params(&params, "ps", &["-T", "-p", pid.as_str()]))
        }
        "memory_info" => {
            let path = format!("/proc/{}/status", helpers::p(&params, "pid"));
            Ok(helpers::read(&path))
        }
        "fd_list" => {
            let path = format!("/proc/{}/fd", helpers::p(&params, "pid"));
            Ok(helpers::cmd_params(&params, "ls", &["-la", path.as_str()]))
        }
        "limits" => {
            let path = format!("/proc/{}/limits", helpers::p(&params, "pid"));
            Ok(helpers::read(&path))
        }
        "environ" => {
            let path = format!("/proc/{}/environ", helpers::p(&params, "pid"));
            Ok(helpers::read(&path))
        }
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &r {
        Ok(_) => audit::log_success("process_control", action, "ok"),
        Err(e) => audit::log_failure("process_control", action, &e.to_string()),
    }
    r.unwrap_or_else(|e| format!("Error: {e}"))
}

fn pid(params: &serde_json::Value) -> Result<u32, AetherError> {
    let ctx = ErrorContext::new("process_control", "pid");
    let s = helpers::p(params, "pid");
    s.parse::<u32>().map_err(|_| AetherError::invalid_param(ctx, "pid", "invalid PID"))
}

fn signal_action(
    params: &serde_json::Value,
    ctx: &ErrorContext,
    sig: Signal,
    success: &str,
) -> Result<String, AetherError> {
    helpers::require_force_or_dry_run(params, ctx)?;
    let pid_val = pid(params)?;
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(
            helpers::RiskLevel::Dangerous,
            format!("send signal {sig:?} to pid {pid_val}"),
        ));
    }
    kill(Pid::from_raw(pid_val as i32), sig)
        .map_err(|e| AetherError::system_error(ctx.clone(), format!("signal failed: {}", e)))?;
    Ok(success.to_string())
}

fn create(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    helpers::require_force_or_dry_run(params, ctx)?;
    let program = helpers::p(params, "program");
    if program.is_empty() {
        return Err(AetherError::invalid_param(ctx.clone(), "program", "required"));
    }
    if !helpers::is_allowed_command(&program) {
        return Err(AetherError::permission_denied(
            ctx.clone(),
            format!("program '{program}' is not in the AETHER command allowlist"),
        ));
    }

    let args = params
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();

    if helpers::dry_run(params) {
        let detail = helpers::command_line(&program, &args);
        return Ok(helpers::preview_action(helpers::RiskLevel::Dangerous, detail));
    }

    let mut cmd = Command::new(program.as_str());
    cmd.args(args);
    let child = cmd.spawn().map_err(|e| AetherError::system_error(ctx.clone(), format!("spawn failed: {}", e)))?;
    Ok(format!("Process created: PID {}", child.id()))
}

fn set_priority_impl(pid_val: u32, prio: i32, ctx: &ErrorContext) -> Result<(), AetherError> {
    // SAFETY: libc::setpriority is called with primitive integer arguments derived from validated JSON params.
    let ret = unsafe { libc::setpriority(libc::PRIO_PROCESS, pid_val, prio) };
    if ret != 0 {
        Err(AetherError::system_error(ctx.clone(), "setpriority failed"))
    } else {
        Ok(())
    }
}
