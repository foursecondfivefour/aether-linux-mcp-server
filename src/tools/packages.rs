use crate::audit;
use crate::error::{self, AetherError, ErrorContext};
use std::fs;

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
    let ctx = ErrorContext::new("package_manager", action);
    let result =
        match action {
            "list_installed" => Ok(run(&params, "dpkg", &["-l"])),
            "list_upgradable" => Ok(run(&params, "apt", &["list", "--upgradable"])),
            "search" => Ok(run(&params, "apt-cache", &["search", &ps(&params, "package", &ctx)])),
            "info" => Ok(run(&params, "apt-cache", &["show", &ps(&params, "package", &ctx)])),
            "install" => force_check(&params, &ctx)
                .map(|_| run(&params, "apt", &["install", "-y", &ps(&params, "package", &ctx)])),
            "remove" => force_check(&params, &ctx)
                .map(|_| run(&params, "apt", &["remove", "-y", &ps(&params, "package", &ctx)])),
            "upgrade_all" => force_check(&params, &ctx).map(|_| run(&params, "apt", &["upgrade", "-y"])),
            "clean_cache" => force_check(&params, &ctx).map(|_| run(&params, "apt", &["clean"])),
            "history" => Ok(fs::read_to_string("/var/log/apt/history.log").unwrap_or_default()),
            "flatpak_list" => Ok(run(&params, "flatpak", &["list"])),
            "snap_list" => Ok(run(&params, "snap", &["list"])),
            other => Err(AetherError::not_implemented(ctx.clone(), other)),
        };
    match &result {
        Ok(_) => audit::log_success("package_manager", action, "ok"),
        Err(e) => audit::log_failure("package_manager", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
