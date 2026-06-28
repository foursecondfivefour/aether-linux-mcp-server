use crate::audit;
use crate::error::{self, AetherError, ErrorContext, require_force};
use std::process::Command;

fn run(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd).args(args).output()
        .map(|o| format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
        .unwrap_or_else(|_| format!("'{}' not available", cmd))
}

fn ps(params: &serde_json::Value, key: &str, ctx: &ErrorContext) -> String {
    error::get_string_param(params, key, ctx).unwrap_or("").to_string()
}

fn force_check(params: &serde_json::Value, ctx: &ErrorContext) -> Result<(), AetherError> {
    if !require_force(params) { Err(AetherError::ForceRequired { ctx: ctx.clone() }) } else { Ok(()) }
}

pub fn handle(action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("service_manager", action);
    let result = match action {
        "list" => Ok(run("systemctl", &["list-units"])),
        "status" => Ok(run("systemctl", &["status", &ps(&params, "service", &ctx)])),
        "start" => force_check(&params, &ctx).map(|_| run("systemctl", &["start", &ps(&params, "service", &ctx)])),
        "stop" => force_check(&params, &ctx).map(|_| run("systemctl", &["stop", &ps(&params, "service", &ctx)])),
        "restart" => force_check(&params, &ctx).map(|_| run("systemctl", &["restart", &ps(&params, "service", &ctx)])),
        "enable" => force_check(&params, &ctx).map(|_| run("systemctl", &["enable", &ps(&params, "service", &ctx)])),
        "disable" => force_check(&params, &ctx).map(|_| run("systemctl", &["disable", &ps(&params, "service", &ctx)])),
        "list_timers" => Ok(run("systemctl", &["list-timers"])),
        "daemon_reload" => Ok(run("systemctl", &["daemon-reload"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("service_manager", action, "ok"),
        Err(e) => audit::log_failure("service_manager", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
