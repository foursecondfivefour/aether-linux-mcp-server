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

pub fn handle(_gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("security_audit", action);
    let result = match action {
        "selinux_status" => Ok(run(&params, "getenforce", &[])),
        "selinux_mode" => Ok(run(&params, "sestatus", &[])),
        "selinux_booleans" => Ok(run(&params, "getsebool", &["-a"])),
        "apparmor_status" => Ok(run(&params, "aa-status", &[])),
        "apparmor_profiles" => Ok(fs::read_to_string("/sys/kernel/security/apparmor/profiles").unwrap_or_default()),
        "auditd_status" => Ok(run(&params, "auditctl", &["-s"])),
        "auditd_rules_list" => Ok(run(&params, "auditctl", &["-l"])),
        "firewall_status" => Ok(run(&params, "ufw", &["status", "verbose"])),
        "firewall_profile" => Ok(run(&params, "nft", &["list", "ruleset"])),
        "tpm2_info" => Ok(run(&params, "tpm2_getcap", &["properties-fixed"])),
        "tpm2_pcrs" => Ok(run(&params, "tpm2_pcrread", &[])),
        "secure_boot_status" => Ok(run(&params, "mokutil", &["--sb-state"])),
        "ima_status" => Ok(read_many(&[
            "/sys/kernel/security/ima/ascii_runtime_measurements",
            "/sys/kernel/security/ima/binary_runtime_measurements",
        ])),
        "lockdown_status" => Ok(fs::read_to_string("/sys/kernel/security/lockdown").unwrap_or_default()),
        "kernel_lsms" => Ok(fs::read_to_string("/sys/kernel/security/lsm").unwrap_or_default()),
        "kernel_mitigations" => Ok(run(&params, "grep", &[".", "/sys/devices/system/cpu/vulnerabilities/*"])),
        "kernel_hardening" => Ok(kernel_hardening_report()),
        "aslr_status" => Ok(fs::read_to_string("/proc/sys/kernel/randomize_va_space").unwrap_or_default()),
        "mmap_min_addr" => Ok(fs::read_to_string("/proc/sys/vm/mmap_min_addr").unwrap_or_default()),
        "kptr_restrict" => Ok(fs::read_to_string("/proc/sys/kernel/kptr_restrict").unwrap_or_default()),
        "dmesg_restrict" => Ok(fs::read_to_string("/proc/sys/kernel/dmesg_restrict").unwrap_or_default()),
        "ptrace_scope" => Ok(fs::read_to_string("/proc/sys/kernel/yama/ptrace_scope").unwrap_or_default()),
        "core_pattern" => Ok(fs::read_to_string("/proc/sys/kernel/core_pattern").unwrap_or_default()),
        "file_integrity_check" => Ok(run(&params, "sha256sum", &[&ps(&params, "path", &ctx)])),
        "cve_mitigations" => Ok(run(&params, "grep", &[".", "/sys/devices/system/cpu/vulnerabilities/*"])),
        "uefi_variables" => Ok(run(&params, "ls", &["/sys/firmware/efi/efivars"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("security_audit", action, "ok"),
        Err(e) => audit::log_failure("security_audit", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}

fn read_many(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|p| format!("== {p} ==\n{}", fs::read_to_string(p).unwrap_or_else(|_| "unavailable\n".to_string())))
        .collect::<Vec<_>>()
        .join("\n")
}

fn kernel_hardening_report() -> String {
    let keys = [
        ("ASLR", "/proc/sys/kernel/randomize_va_space"),
        ("kptr_restrict", "/proc/sys/kernel/kptr_restrict"),
        ("dmesg_restrict", "/proc/sys/kernel/dmesg_restrict"),
        ("ptrace_scope", "/proc/sys/kernel/yama/ptrace_scope"),
        ("mmap_min_addr", "/proc/sys/vm/mmap_min_addr"),
        ("unprivileged_bpf_disabled", "/proc/sys/kernel/unprivileged_bpf_disabled"),
    ];
    keys.iter()
        .map(|(name, path)| {
            format!("{name}: {}", fs::read_to_string(path).unwrap_or_else(|_| "unavailable".to_string()).trim())
        })
        .collect::<Vec<_>>()
        .join("\n")
}
