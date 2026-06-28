use crate::audit;
use crate::config::FeatureGates;
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

pub fn handle(gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("user_management", action);
    let result = match action {
        "users_list" => Ok(fs::read_to_string("/etc/passwd").unwrap_or_default()),
        "user_info" => Ok(run("getent", &["passwd", &ps(&params, "username", &ctx)])),
        "user_create" => force_check(&params, &ctx).and_then(|_| gates.check(ctx.clone(), gates.token_manipulation, "AETHER_TOKEN_MANIPULATION")).map(|_| run("useradd", &["-m", &ps(&params, "username", &ctx)])),
        "user_delete" => force_check(&params, &ctx).and_then(|_| gates.check(ctx.clone(), gates.token_manipulation, "AETHER_TOKEN_MANIPULATION")).map(|_| run("userdel", &[&ps(&params, "username", &ctx)])),
        "user_lock" => force_check(&params, &ctx).map(|_| run("passwd", &["-l", &ps(&params, "username", &ctx)])),
        "user_unlock" => force_check(&params, &ctx).map(|_| run("passwd", &["-u", &ps(&params, "username", &ctx)])),
        "groups_list" => Ok(fs::read_to_string("/etc/group").unwrap_or_default()),
        "group_create" => force_check(&params, &ctx).map(|_| run("groupadd", &[&ps(&params, "name", &ctx)])),
        "group_delete" => force_check(&params, &ctx).map(|_| run("groupdel", &[&ps(&params, "name", &ctx)])),
        "sessions_list" => Ok(run("w", &[])),
        "current_user" => Ok(run("whoami", &[])),
        "id" => Ok(run("id", &[])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("user_management", action, "ok"),
        Err(e) => audit::log_failure("user_management", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
