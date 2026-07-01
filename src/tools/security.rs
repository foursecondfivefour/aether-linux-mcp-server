use crate::audit;
use crate::config::FeatureGates;
use crate::error::{AetherError, ErrorContext};
use std::fs;

fn run(params: &serde_json::Value, cmd: &str, args: &[&str]) -> String {
    crate::tools::helpers::cmd_params(params, cmd, args)
}

pub fn handle(_gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("security_audit", action);
    let result = match action {
        "selinux_status" => Ok(run(&params, "getenforce", &[])),
        "selinux_mode" => Ok(run(&params, "sestatus", &[])),
        "apparmor_status" => Ok(run(&params, "aa-status", &[])),
        "auditd_status" => Ok(run(&params, "auditctl", &["-s"])),
        "auditd_rules_list" => Ok(run(&params, "auditctl", &["-l"])),
        "firewall_status" => Ok(run(&params, "ufw", &["status"])),
        "tpm2_info" => Ok(run(&params, "tpm2_getcap", &["properties-fixed"])),
        "secure_boot_status" => Ok(run(&params, "mokutil", &["--sb-state"])),
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
