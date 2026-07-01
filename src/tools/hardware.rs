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

pub fn handle(_gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("hardware_control", action);
    let result = match action {
        "gpu_nvidia_info" => Ok(run(&params, "nvidia-smi", &[])),
        "gpu_amdgpu_info" => Ok(run(&params, "rocm-smi", &[])),
        "usb_devices" => Ok(run(&params, "lsusb", &["-t"])),
        "acpi_info" => Ok(run(&params, "acpi", &["-V"])),
        "cpufreq_governor_get" => {
            Ok(fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor").unwrap_or_default())
        }
        "cpufreq_governor_set" => force_check(&params, &ctx)
            .map(|_| run(&params, "cpupower", &["frequency-set", "-g", &ps(&params, "governor", &ctx)])),
        "thermal_zones" => Ok(run(&params, "sensors", &[])),
        "thermal_temperature" => Ok(run(&params, "sensors", &[])),
        "iommu_groups" => Ok(run(&params, "ls", &["/sys/kernel/iommu_groups"])),
        "irq_list" => Ok(fs::read_to_string("/proc/interrupts").unwrap_or_default()),
        "hugepages_info" => Ok(fs::read_to_string("/proc/sys/vm/nr_hugepages").unwrap_or_default()),
        "kdump_status" => Ok(run(&params, "kdump-config", &["status"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("hardware_control", action, "ok"),
        Err(e) => audit::log_failure("hardware_control", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
