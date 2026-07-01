use crate::audit;
use crate::config::FeatureGates;
use crate::error::{AetherError, ErrorContext};
use std::fs;
use std::process::Command;

fn run(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
        .unwrap_or_else(|_| format!("'{}' not available", cmd))
}

pub fn handle(_gates: &FeatureGates, action: &str, _params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("security_audit", action);
    let result = match action {
        "selinux_status" => Ok(run("getenforce", &[])),
        "selinux_mode" => Ok(run("sestatus", &[])),
        "apparmor_status" => Ok(run("aa-status", &[])),
        "auditd_status" => Ok(run("auditctl", &["-s"])),
        "auditd_rules_list" => Ok(run("auditctl", &["-l"])),
        "firewall_status" => Ok(run("ufw", &["status"])),
        "tpm2_info" => Ok(run("tpm2_getcap", &["properties-fixed"])),
        "secure_boot_status" => Ok(run("mokutil", &["--sb-state"])),
        "lockdown_status" => Ok(fs::read_to_string("/sys/kernel/security/lockdown").unwrap_or_default()),
        "kernel_lsms" => Ok(fs::read_to_string("/sys/kernel/security/lsm").unwrap_or_default()),
        "aslr_status" => Ok(fs::read_to_string("/proc/sys/kernel/randomize_va_space").unwrap_or_default()),
        "kptr_restrict" => Ok(fs::read_to_string("/proc/sys/kernel/kptr_restrict").unwrap_or_default()),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("security_audit", action, "ok"),
        Err(e) => audit::log_failure("security_audit", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
