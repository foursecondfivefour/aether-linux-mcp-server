use crate::audit;
use crate::config::FeatureGates;
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

pub fn handle(gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("system_config", action);
    let result = match action {
        "sysctl_list" => Ok(run(&params, "sysctl", &["-a"])),
        "sysctl_get" => Ok(run(&params, "sysctl", &[&ps(&params, "key", &ctx)])),
        "kernel_cmdline_get" => Ok(fs::read_to_string("/proc/cmdline").unwrap_or_default()),
        "modprobe_list" => Ok(run(&params, "lsmod", &[])),
        "modprobe_load" => force_check(&params, &ctx)
            .and_then(|_| gates.check(ctx.clone(), gates.module_load, "AETHER_MODULE_LOAD"))
            .map(|_| run(&params, "modprobe", &[&ps(&params, "module", &ctx)])),
        "hostname_get" => Ok(fs::read_to_string("/proc/sys/kernel/hostname").unwrap_or_default()),
        "hostname_set" => force_check(&params, &ctx)
            .map(|_| run(&params, "hostnamectl", &["set-hostname", &ps(&params, "hostname", &ctx)])),
        "timezone_list" => Ok(run(&params, "timedatectl", &["list-timezones"])),
        "timezone_get" => Ok(run(&params, "timedatectl", &[])),
        "timezone_set" => force_check(&params, &ctx)
            .map(|_| run(&params, "timedatectl", &["set-timezone", &ps(&params, "timezone", &ctx)])),
        "locale_list" => Ok(run(&params, "locale", &["-a"])),
        "locale_get" => Ok(run(&params, "localectl", &[])),
        "os_release" => Ok(fs::read_to_string("/etc/os-release").unwrap_or_default()),
        "dconf_list" => Ok(run(&params, "dconf", &["list", "/"])),
        "dconf_get" => Ok(run(&params, "dconf", &["read", &ps(&params, "key", &ctx)])),
        "dconf_set" => force_check(&params, &ctx)
            .map(|_| run(&params, "dconf", &["write", &ps(&params, "key", &ctx), &ps(&params, "value", &ctx)])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("system_config", action, "ok"),
        Err(e) => audit::log_failure("system_config", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
