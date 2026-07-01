use crate::audit;
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

pub fn handle(action: &str, _params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("system_info", action);
    let result = match action {
        "cpu_info" => Ok(fs::read_to_string("/proc/cpuinfo").unwrap_or_default()),
        "cpu_topology" => Ok(run("lscpu", &["-e"])),
        "memory_info" => Ok(fs::read_to_string("/proc/meminfo").unwrap_or_default()),
        "disk_list" => Ok(run("lsblk", &["-o", "NAME,SIZE,TYPE,MOUNTPOINT"])),
        "disk_partitions" => Ok(run("fdisk", &["-l"])),
        "os_info" => Ok(run("uname", &["-a"])),
        "os_release" => Ok(fs::read_to_string("/etc/os-release").unwrap_or_default()),
        "kernel_info" => Ok(run("uname", &["-r"])),
        "kernel_modules" => Ok(run("lsmod", &[])),
        "uptime" => Ok(fs::read_to_string("/proc/uptime").unwrap_or_default()),
        "load_avg" => Ok(fs::read_to_string("/proc/loadavg").unwrap_or_default()),
        "hostname" => Ok(fs::read_to_string("/proc/sys/kernel/hostname").unwrap_or_default()),
        "pci_list" => Ok(run("lspci", &[])),
        "usb_list" => Ok(run("lsusb", &[])),
        "sensors_list" => Ok(run("sensors", &[])),
        "gpu_list" => Ok(run("lspci", &["-v"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("system_info", action, "ok"),
        Err(e) => audit::log_failure("system_info", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
