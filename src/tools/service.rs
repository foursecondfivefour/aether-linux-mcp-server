use crate::audit;
use crate::error::{self, AetherError, ErrorContext};

fn run(params: &serde_json::Value, cmd: &str, args: &[&str]) -> String {
    crate::tools::helpers::cmd_params(params, cmd, args)
}

fn ps(params: &serde_json::Value, key: &str, ctx: &ErrorContext) -> String {
    error::get_string_param(params, key, ctx).unwrap_or("").to_string()
}

fn force_check(params: &serde_json::Value, ctx: &ErrorContext) -> Result<(), AetherError> {
    crate::tools::helpers::require_force_or_dry_run(params, ctx)
}

pub fn handle(action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("service_manager", action);
    let result =
        match action {
            "list" => Ok(run(&params, "systemctl", &["list-units"])),
            "status" => Ok(run(&params, "systemctl", &["status", &ps(&params, "service", &ctx)])),
            "start" => {
                force_check(&params, &ctx).map(|_| run(&params, "systemctl", &["start", &ps(&params, "service", &ctx)]))
            }
            "stop" => {
                force_check(&params, &ctx).map(|_| run(&params, "systemctl", &["stop", &ps(&params, "service", &ctx)]))
            }
            "restart" => force_check(&params, &ctx)
                .map(|_| run(&params, "systemctl", &["restart", &ps(&params, "service", &ctx)])),
            "enable" => force_check(&params, &ctx)
                .map(|_| run(&params, "systemctl", &["enable", &ps(&params, "service", &ctx)])),
            "disable" => force_check(&params, &ctx)
                .map(|_| run(&params, "systemctl", &["disable", &ps(&params, "service", &ctx)])),
            "list_timers" => Ok(run(&params, "systemctl", &["list-timers"])),
            "daemon_reload" => Ok(run(&params, "systemctl", &["daemon-reload"])),
            other => Err(AetherError::not_implemented(ctx.clone(), other)),
        };
    match &result {
        Ok(_) => audit::log_success("service_manager", action, "ok"),
        Err(e) => audit::log_failure("service_manager", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
