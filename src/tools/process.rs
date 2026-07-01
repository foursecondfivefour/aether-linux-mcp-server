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
        "list" => Ok(helpers::cmd("ps", &["aux"])),
        "tree" => Ok(helpers::cmd("ps", &["aux", "--forest"])),
        "kill" => signal(&params, &ctx, Signal::SIGKILL).map(|_| "Process killed".to_string()),
        "signal" => {
            let sig_str = helpers::p(&params, "signal");
            let sig_val: i32 = sig_str.parse().unwrap_or(15);
            match Signal::try_from(sig_val) {
                Ok(sig) => signal(&params, &ctx, sig).map(|_| "Signal sent".to_string()),
                Err(e) => Err(AetherError::invalid_param(ctx.clone(), "signal", format!("{}", e))),
            }
        }
        "suspend" => signal(&params, &ctx, Signal::SIGSTOP).map(|_| "Process suspended".to_string()),
        "resume" => signal(&params, &ctx, Signal::SIGCONT).map(|_| "Process resumed".to_string()),
        "create" => create(&params, &ctx).map(|pid_val| format!("Process created: PID {}", pid_val)),
        "set_priority" => (|| -> Result<String, AetherError> {
            let pid_val = pid(&params)?;
            let prio_str = helpers::p(&params, "priority");
            let prio_val: i32 = prio_str.parse().unwrap_or(0);
            set_priority_impl(pid_val, prio_val, &ctx)?;
            Ok("Priority set".to_string())
        })(),
        "set_nice" => {
            let pid = helpers::p(&params, "pid");
            let nice = helpers::p(&params, "nice");
            Ok(helpers::cmd("renice", &["-n", nice.as_str(), "-p", pid.as_str()]))
        }
        "query_info" => {
            let pid = helpers::p(&params, "pid");
            Ok(helpers::cmd("ps", &["-p", pid.as_str(), "-o", "pid,ppid,user,%cpu,%mem,cmd", "--no-headers"]))
        }
        "threads" => {
            let pid = helpers::p(&params, "pid");
            Ok(helpers::cmd("ps", &["-T", "-p", pid.as_str()]))
        }
        "memory_info" => {
            let path = format!("/proc/{}/status", helpers::p(&params, "pid"));
            Ok(helpers::read(&path))
        }
        "fd_list" => {
            let path = format!("/proc/{}/fd", helpers::p(&params, "pid"));
            Ok(helpers::cmd("ls", &["-la", path.as_str()]))
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

fn signal(params: &serde_json::Value, ctx: &ErrorContext, sig: Signal) -> Result<(), AetherError> {
    let pid_val = pid(params)?;
    kill(Pid::from_raw(pid_val as i32), sig)
        .map_err(|e| AetherError::system_error(ctx.clone(), format!("signal failed: {}", e)))
}

fn create(params: &serde_json::Value, ctx: &ErrorContext) -> Result<u32, AetherError> {
    let program = helpers::p(params, "program");
    if program.is_empty() {
        return Err(AetherError::invalid_param(ctx.clone(), "program", "required"));
    }
    let mut cmd = Command::new(program.as_str());
    if let Some(args) = params.get("args") {
        if let Some(arr) = args.as_array() {
            for a in arr {
                if let Some(s) = a.as_str() {
                    cmd.arg(s);
                }
            }
        }
    }
    let child = cmd.spawn().map_err(|e| AetherError::system_error(ctx.clone(), format!("spawn failed: {}", e)))?;
    Ok(child.id())
}

fn set_priority_impl(pid_val: u32, prio: i32, ctx: &ErrorContext) -> Result<(), AetherError> {
    let ret = unsafe { libc::setpriority(libc::PRIO_PROCESS, pid_val, prio) };
    if ret != 0 {
        Err(AetherError::system_error(ctx.clone(), "setpriority failed"))
    } else {
        Ok(())
    }
}
