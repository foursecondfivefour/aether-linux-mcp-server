use crate::audit;
use crate::config::FeatureGates;
use crate::error::{self, AetherError, ErrorContext};
use crate::tools::helpers::{self, RiskLevel};
use std::fs;
use std::path::PathBuf;

fn run(params: &serde_json::Value, cmd: &str, args: &[&str]) -> String {
    helpers::cmd_params(params, cmd, args)
}

fn ps(params: &serde_json::Value, key: &str, ctx: &ErrorContext) -> String {
    error::get_string_param(params, key, ctx).unwrap_or("").to_string()
}

fn force_check(params: &serde_json::Value, ctx: &ErrorContext) -> Result<(), AetherError> {
    helpers::require_force_or_dry_run(params, ctx)
}

pub fn handle(gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("user_management", action);
    let result = match action {
        "users_list" => Ok(fs::read_to_string("/etc/passwd").unwrap_or_default()),
        "user_info" => Ok(run(&params, "getent", &["passwd", &ps(&params, "username", &ctx)])),
        "user_create" => force_check(&params, &ctx)
            .and_then(|_| gates.check(ctx.clone(), gates.token_manipulation, "AETHER_TOKEN_MANIPULATION"))
            .map(|_| run(&params, "useradd", &["-m", &ps(&params, "username", &ctx)])),
        "user_delete" => force_check(&params, &ctx)
            .and_then(|_| gates.check(ctx.clone(), gates.token_manipulation, "AETHER_TOKEN_MANIPULATION"))
            .map(|_| run(&params, "userdel", &[&ps(&params, "username", &ctx)])),
        "user_modify" => force_check(&params, &ctx)
            .map(|_| run(&params, "usermod", &[&ps(&params, "args", &ctx), &ps(&params, "username", &ctx)])),
        "user_lock" => {
            force_check(&params, &ctx).map(|_| run(&params, "passwd", &["-l", &ps(&params, "username", &ctx)]))
        }
        "user_unlock" => {
            force_check(&params, &ctx).map(|_| run(&params, "passwd", &["-u", &ps(&params, "username", &ctx)]))
        }
        "groups_list" => Ok(fs::read_to_string("/etc/group").unwrap_or_default()),
        "group_info" => Ok(run(&params, "getent", &["group", &ps(&params, "name", &ctx)])),
        "group_create" => force_check(&params, &ctx).map(|_| run(&params, "groupadd", &[&ps(&params, "name", &ctx)])),
        "group_delete" => force_check(&params, &ctx).map(|_| run(&params, "groupdel", &[&ps(&params, "name", &ctx)])),
        "group_add_user" => force_check(&params, &ctx)
            .map(|_| run(&params, "usermod", &["-aG", &ps(&params, "group", &ctx), &ps(&params, "username", &ctx)])),
        "group_remove_user" => force_check(&params, &ctx)
            .map(|_| run(&params, "gpasswd", &["-d", &ps(&params, "username", &ctx), &ps(&params, "group", &ctx)])),
        "sessions_list" => Ok(run(&params, "loginctl", &["list-sessions"])),
        "session_info" => Ok(run(&params, "loginctl", &["session-status", &ps(&params, "session", &ctx)])),
        "session_terminate" => force_check(&params, &ctx)
            .map(|_| run(&params, "loginctl", &["terminate-session", &ps(&params, "session", &ctx)])),
        "current_user" | "whoami" => Ok(run(&params, "whoami", &[])),
        "id" => Ok(run(&params, "id", &[])),
        "nsswitch_conf" => Ok(fs::read_to_string("/etc/nsswitch.conf").unwrap_or_default()),
        "pam_list_modules" => Ok(run(&params, "ls", &["/etc/pam.d"])),
        "sudoers_read" => Ok(fs::read_to_string("/etc/sudoers").unwrap_or_default()),
        "ssh_authorized_keys_list" => read_authorized_keys(&params, &ctx),
        "ssh_authorized_keys_add" => add_authorized_key(&params, &ctx),
        "ssh_authorized_keys_remove" => remove_authorized_key(&params, &ctx),
        "last_logins" => Ok(run(&params, "last", &["-n", "20"])),
        "faillock_list" => Ok(run(&params, "faillock", &[])),
        "faillock_reset" => force_check(&params, &ctx)
            .map(|_| run(&params, "faillock", &["--user", &ps(&params, "username", &ctx), "--reset"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("user_management", action, "ok"),
        Err(e) => audit::log_failure("user_management", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}

fn authorized_keys_path(params: &serde_json::Value, ctx: &ErrorContext) -> PathBuf {
    let home = params
        .get("home")
        .and_then(|v| v.as_str())
        .map_or_else(|| format!("/home/{}", ps(params, "username", ctx)), ToString::to_string);
    PathBuf::from(home).join(".ssh/authorized_keys")
}

fn read_authorized_keys(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    fs::read_to_string(authorized_keys_path(params, ctx)).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
}

fn add_authorized_key(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let path = authorized_keys_path(params, ctx);
    let key = ps(params, "key", ctx);
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Critical, format!("add SSH key to {}", path.display())));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })?;
    }
    let mut existing = fs::read_to_string(&path).unwrap_or_default();
    if !existing.ends_with('\n') && !existing.is_empty() {
        existing.push('\n');
    }
    existing.push_str(&key);
    existing.push('\n');
    fs::write(&path, existing).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })?;
    Ok(format!("Added SSH key to {}", path.display()))
}

fn remove_authorized_key(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let path = authorized_keys_path(params, ctx);
    let key = ps(params, "key", ctx);
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Critical, format!("remove SSH key from {}", path.display())));
    }
    let existing = fs::read_to_string(&path).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })?;
    let filtered = existing.lines().filter(|line| line.trim() != key.trim()).collect::<Vec<_>>().join("\n") + "\n";
    fs::write(&path, filtered).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })?;
    Ok(format!("Removed SSH key from {}", path.display()))
}
