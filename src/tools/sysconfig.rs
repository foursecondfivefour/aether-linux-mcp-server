use crate::audit;
use crate::config::FeatureGates;
use crate::error::{self, AetherError, ErrorContext};
use crate::tools::helpers::{self, RiskLevel};
use std::fs;

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
    let ctx = ErrorContext::new("system_config", action);
    let result = match action {
        "sysctl_list" => Ok(run(&params, "sysctl", &["-a"])),
        "sysctl_get" => Ok(run(&params, "sysctl", &[&ps(&params, "key", &ctx)])),
        "sysctl_set" => force_check(&params, &ctx).map(|_| {
            run(&params, "sysctl", &["-w", &format!("{}={}", ps(&params, "key", &ctx), ps(&params, "value", &ctx))])
        }),
        "sysctl_load" => {
            force_check(&params, &ctx).map(|_| run(&params, "sysctl", &["-p", &ps(&params, "path", &ctx)]))
        }
        "kernel_cmdline_get" => Ok(fs::read_to_string("/proc/cmdline").unwrap_or_default()),
        "modprobe_list" => Ok(run(&params, "lsmod", &[])),
        "modprobe_load" => force_check(&params, &ctx)
            .and_then(|_| gates.check(ctx.clone(), gates.module_load, "AETHER_MODULE_LOAD"))
            .map(|_| run(&params, "modprobe", &[&ps(&params, "module", &ctx)])),
        "modprobe_unload" => force_check(&params, &ctx)
            .and_then(|_| gates.check(ctx.clone(), gates.module_load, "AETHER_MODULE_LOAD"))
            .map(|_| run(&params, "modprobe", &["-r", &ps(&params, "module", &ctx)])),
        "hostname_get" => Ok(fs::read_to_string("/proc/sys/kernel/hostname").unwrap_or_default()),
        "hostname_set" => force_check(&params, &ctx)
            .map(|_| run(&params, "hostnamectl", &["set-hostname", &ps(&params, "hostname", &ctx)])),
        "timezone_list" => Ok(run(&params, "timedatectl", &["list-timezones"])),
        "timezone_get" => Ok(run(&params, "timedatectl", &[])),
        "timezone_set" => force_check(&params, &ctx)
            .map(|_| run(&params, "timedatectl", &["set-timezone", &ps(&params, "timezone", &ctx)])),
        "locale_list" => Ok(run(&params, "locale", &["-a"])),
        "locale_get" => Ok(run(&params, "localectl", &[])),
        "locale_set" => {
            force_check(&params, &ctx).map(|_| run(&params, "localectl", &["set-locale", &ps(&params, "locale", &ctx)]))
        }
        "keyboard_layout" => Ok(run(&params, "localectl", &["status"])),
        "dconf_list" => Ok(run(&params, "dconf", &["list", "/"])),
        "dconf_get" => Ok(run(&params, "dconf", &["read", &ps(&params, "key", &ctx)])),
        "dconf_set" => force_check(&params, &ctx)
            .map(|_| run(&params, "dconf", &["write", &ps(&params, "key", &ctx), &ps(&params, "value", &ctx)])),
        "gsettings_list" => Ok(run(&params, "gsettings", &["list-schemas"])),
        "gsettings_get" => {
            Ok(run(&params, "gsettings", &["get", &ps(&params, "schema", &ctx), &ps(&params, "key", &ctx)]))
        }
        "gsettings_set" => force_check(&params, &ctx).map(|_| {
            run(
                &params,
                "gsettings",
                &["set", &ps(&params, "schema", &ctx), &ps(&params, "key", &ctx), &ps(&params, "value", &ctx)],
            )
        }),
        "limits_get" => Ok(fs::read_to_string("/etc/security/limits.conf").unwrap_or_default()),
        "etc_config_read" => read_config(&params, &ctx),
        "etc_config_write" => write_config(&params, &ctx),
        "udev_rules_list" => Ok(run(&params, "ls", &["/etc/udev/rules.d"])),
        "udev_trigger" => force_check(&params, &ctx).map(|_| run(&params, "udevadm", &["trigger"])),
        "environment_list" => Ok(run(&params, "env", &[])),
        "environment_set" => write_environment(&params, &ctx),
        "os_release" => Ok(fs::read_to_string("/etc/os-release").unwrap_or_default()),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("system_config", action, "ok"),
        Err(e) => audit::log_failure("system_config", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}

fn read_config(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    let path = ps(params, "path", ctx);
    if !path.starts_with("/etc/") {
        return Err(AetherError::permission_denied(ctx.clone(), "etc_config_read is limited to /etc paths"));
    }
    fs::read_to_string(path).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
}

fn write_config(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let path = ps(params, "path", ctx);
    let content = ps(params, "content", ctx);
    if !path.starts_with("/etc/") {
        return Err(AetherError::permission_denied(ctx.clone(), "etc_config_write is limited to /etc paths"));
    }
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Critical, format!("write {} bytes to {}", content.len(), path)));
    }
    fs::write(&path, content).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })?;
    Ok(format!("Wrote {}", path))
}

fn write_environment(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let key = ps(params, "key", ctx);
    let value = ps(params, "value", ctx);
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Moderate, format!("set process env {key}")));
    }
    std::env::set_var(&key, &value);
    Ok(format!("Set process environment variable {}", key))
}
