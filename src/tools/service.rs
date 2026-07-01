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
            "list" | "list_units" => Ok(run(&params, "systemctl", &["list-units", "--no-pager"])),
            "list_unit_files" => Ok(run(&params, "systemctl", &["list-unit-files", "--no-pager"])),
            "status" => Ok(run(&params, "systemctl", &["status", &ps(&params, "service", &ctx), "--no-pager"])),
            "start" => {
                force_check(&params, &ctx).map(|_| run(&params, "systemctl", &["start", &ps(&params, "service", &ctx)]))
            }
            "stop" => {
                force_check(&params, &ctx).map(|_| run(&params, "systemctl", &["stop", &ps(&params, "service", &ctx)]))
            }
            "restart" => force_check(&params, &ctx)
                .map(|_| run(&params, "systemctl", &["restart", &ps(&params, "service", &ctx)])),
            "reload" => force_check(&params, &ctx)
                .map(|_| run(&params, "systemctl", &["reload", &ps(&params, "service", &ctx)])),
            "enable" => force_check(&params, &ctx)
                .map(|_| run(&params, "systemctl", &["enable", &ps(&params, "service", &ctx)])),
            "disable" => force_check(&params, &ctx)
                .map(|_| run(&params, "systemctl", &["disable", &ps(&params, "service", &ctx)])),
            "mask" => {
                force_check(&params, &ctx).map(|_| run(&params, "systemctl", &["mask", &ps(&params, "service", &ctx)]))
            }
            "unmask" => force_check(&params, &ctx)
                .map(|_| run(&params, "systemctl", &["unmask", &ps(&params, "service", &ctx)])),
            "cat" => Ok(run(&params, "systemctl", &["cat", &ps(&params, "service", &ctx)])),
            "show" => Ok(run(&params, "systemctl", &["show", &ps(&params, "service", &ctx)])),
            "list_dependencies" => {
                Ok(run(&params, "systemctl", &["list-dependencies", &ps(&params, "service", &ctx), "--no-pager"]))
            }
            "list_timers" => Ok(run(&params, "systemctl", &["list-timers", "--all", "--no-pager"])),
            "list_sockets" => Ok(run(&params, "systemctl", &["list-sockets", "--all", "--no-pager"])),
            "list_paths" => Ok(run(&params, "systemctl", &["list-units", "--type=path", "--all", "--no-pager"])),
            "list_mounts" => Ok(run(&params, "systemctl", &["list-units", "--type=mount", "--all", "--no-pager"])),
            "isolate" => force_check(&params, &ctx)
                .map(|_| run(&params, "systemctl", &["isolate", &ps(&params, "target", &ctx)])),
            "default" => force_check(&params, &ctx).map(|_| run(&params, "systemctl", &["default"])),
            "rescue" => force_check(&params, &ctx).map(|_| run(&params, "systemctl", &["rescue"])),
            "emergency" => force_check(&params, &ctx).map(|_| run(&params, "systemctl", &["emergency"])),
            "daemon_reload" => force_check(&params, &ctx).map(|_| run(&params, "systemctl", &["daemon-reload"])),
            "reset_failed" => Ok(run(&params, "systemctl", &["reset-failed"])),
            "user_services" => Ok(run(&params, "systemctl", &["--user", "list-units", "--no-pager"])),
            "analyze" => Ok(run(&params, "systemd-analyze", &["blame"])),
            "openrc_list" => Ok(run(&params, "rc-status", &[])),
            "runit_list" => Ok(run(&params, "sv", &["status", "/etc/service/*"])),
            other => Err(AetherError::not_implemented(ctx.clone(), other)),
        };
    match &result {
        Ok(_) => audit::log_success("service_manager", action, "ok"),
        Err(e) => audit::log_failure("service_manager", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
