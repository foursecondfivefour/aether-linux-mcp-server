use crate::audit;
use crate::error::{self, AetherError, ErrorContext, require_force};
use std::fs;
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
    let ctx = ErrorContext::new("package_manager", action);
    let result = match action {
        "list_installed" => Ok(run("dpkg", &["-l"])),
        "list_upgradable" => Ok(run("apt", &["list", "--upgradable"])),
        "search" => Ok(run("apt-cache", &["search", &ps(&params, "package", &ctx)])),
        "info" => Ok(run("apt-cache", &["show", &ps(&params, "package", &ctx)])),
        "install" => force_check(&params, &ctx).map(|_| run("apt", &["install", "-y", &ps(&params, "package", &ctx)])),
        "remove" => force_check(&params, &ctx).map(|_| run("apt", &["remove", "-y", &ps(&params, "package", &ctx)])),
        "upgrade_all" => force_check(&params, &ctx).map(|_| run("apt", &["upgrade", "-y"])),
        "clean_cache" => Ok(run("apt", &["clean"])),
        "history" => Ok(fs::read_to_string("/var/log/apt/history.log").unwrap_or_default()),
        "flatpak_list" => Ok(run("flatpak", &["list"])),
        "snap_list" => Ok(run("snap", &["list"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("package_manager", action, "ok"),
        Err(e) => audit::log_failure("package_manager", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
