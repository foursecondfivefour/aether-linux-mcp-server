use crate::audit;
use crate::config::FeatureGates;
use crate::error::{self, require_force, AetherError, ErrorContext};
use std::fs;
use std::process::Command;

fn run(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
        .unwrap_or_else(|_| format!("'{}' not available", cmd))
}

fn ps(params: &serde_json::Value, key: &str, ctx: &ErrorContext) -> String {
    error::get_string_param(params, key, ctx).unwrap_or("").to_string()
}

fn force_check(params: &serde_json::Value, ctx: &ErrorContext) -> Result<(), AetherError> {
    if !require_force(params) {
        Err(AetherError::ForceRequired { ctx: ctx.clone() })
    } else {
        Ok(())
    }
}

pub fn handle(gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("system_config", action);
    let result =
        match action {
            "sysctl_list" => Ok(run("sysctl", &["-a"])),
            "sysctl_get" => Ok(run("sysctl", &[&ps(&params, "key", &ctx)])),
            "kernel_cmdline_get" => Ok(fs::read_to_string("/proc/cmdline").unwrap_or_default()),
            "modprobe_list" => Ok(run("lsmod", &[])),
            "modprobe_load" => force_check(&params, &ctx)
                .and_then(|_| gates.check(ctx.clone(), gates.module_load, "AETHER_MODULE_LOAD"))
                .map(|_| run("modprobe", &[&ps(&params, "module", &ctx)])),
            "hostname_get" => Ok(fs::read_to_string("/proc/sys/kernel/hostname").unwrap_or_default()),
            "hostname_set" => force_check(&params, &ctx)
                .map(|_| run("hostnamectl", &["set-hostname", &ps(&params, "hostname", &ctx)])),
            "timezone_list" => Ok(run("timedatectl", &["list-timezones"])),
            "timezone_get" => Ok(run("timedatectl", &[])),
            "timezone_set" => force_check(&params, &ctx)
                .map(|_| run("timedatectl", &["set-timezone", &ps(&params, "timezone", &ctx)])),
            "locale_list" => Ok(run("locale", &["-a"])),
            "locale_get" => Ok(run("localectl", &[])),
            "os_release" => Ok(fs::read_to_string("/etc/os-release").unwrap_or_default()),
            "dconf_list" => Ok(run("dconf", &["list", "/"])),
            "dconf_get" => Ok(run("dconf", &["read", &ps(&params, "key", &ctx)])),
            "dconf_set" => Ok(run("dconf", &["write", &ps(&params, "key", &ctx), &ps(&params, "value", &ctx)])),
            other => Err(AetherError::not_implemented(ctx.clone(), other)),
        };
    match &result {
        Ok(_) => audit::log_success("system_config", action, "ok"),
        Err(e) => audit::log_failure("system_config", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
