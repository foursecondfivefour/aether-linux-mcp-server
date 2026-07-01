use crate::audit;
use crate::error::{AetherError, ErrorContext};
use std::fs;

fn run(params: &serde_json::Value, cmd: &str, args: &[&str]) -> String {
    crate::tools::helpers::cmd_params(params, cmd, args)
}

pub fn handle(action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("system_info", action);
    let result = match action {
        "cpu_info" => Ok(fs::read_to_string("/proc/cpuinfo").unwrap_or_default()),
        "cpu_topology" => Ok(run(&params, "lscpu", &["-e"])),
        "memory_info" => Ok(fs::read_to_string("/proc/meminfo").unwrap_or_default()),
        "disk_list" => Ok(run(&params, "lsblk", &["-o", "NAME,SIZE,TYPE,MOUNTPOINT"])),
        "disk_partitions" => Ok(run(&params, "fdisk", &["-l"])),
        "os_info" => Ok(run(&params, "uname", &["-a"])),
        "os_release" => Ok(fs::read_to_string("/etc/os-release").unwrap_or_default()),
        "kernel_info" => Ok(run(&params, "uname", &["-r"])),
        "kernel_modules" => Ok(run(&params, "lsmod", &[])),
        "uptime" => Ok(fs::read_to_string("/proc/uptime").unwrap_or_default()),
        "load_avg" => Ok(fs::read_to_string("/proc/loadavg").unwrap_or_default()),
        "hostname" => Ok(fs::read_to_string("/proc/sys/kernel/hostname").unwrap_or_default()),
        "pci_list" => Ok(run(&params, "lspci", &[])),
        "usb_list" => Ok(run(&params, "lsusb", &[])),
        "sensors_list" => Ok(run(&params, "sensors", &[])),
        "gpu_list" => Ok(run(&params, "lspci", &["-v"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("system_info", action, "ok"),
        Err(e) => audit::log_failure("system_info", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
