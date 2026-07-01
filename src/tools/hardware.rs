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

pub fn handle(_gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("hardware_control", action);
    let result = match action {
        "gpu_nvidia_info" | "gpu_nvidia_smi" => Ok(run(&params, "nvidia-smi", &[])),
        "gpu_amdgpu_info" => Ok(run(&params, "rocm-smi", &[])),
        "gpu_vulkan_info" => Ok(run(&params, "vulkaninfo", &["--summary"])),
        "pci_list" => Ok(run(&params, "lspci", &[])),
        "pci_info" => Ok(run(&params, "lspci", &["-vv", "-s", &ps(&params, "slot", &ctx)])),
        "usb_devices" => Ok(run(&params, "lsusb", &["-t"])),
        "usb_authorize" => write_sysfs(&params, &ctx, "authorized", "1"),
        "usb_deauthorize" => write_sysfs(&params, &ctx, "authorized", "0"),
        "acpi_info" => Ok(run(&params, "acpi", &["-V"])),
        "acpi_wakeup" => Ok(fs::read_to_string("/proc/acpi/wakeup").unwrap_or_default()),
        "cpufreq_governor_list" => {
            Ok(fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors")
                .unwrap_or_default())
        }
        "cpufreq_governor_get" => {
            Ok(fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor").unwrap_or_default())
        }
        "cpufreq_governor_set" => force_check(&params, &ctx)
            .map(|_| run(&params, "cpupower", &["frequency-set", "-g", &ps(&params, "governor", &ctx)])),
        "thermal_zones" => {
            Ok(run(&params, "find", &["/sys/class/thermal", "-maxdepth", "2", "-type", "f", "-name", "temp"]))
        }
        "thermal_temperature" => Ok(run(&params, "sensors", &[])),
        "thermal_cooling" => Ok(run(
            &params,
            "find",
            &["/sys/class/thermal", "-maxdepth", "2", "-type", "d", "-name", "cooling_device*"],
        )),
        "cpu_microcode_version" => Ok(run(&params, "grep", &["microcode", "/proc/cpuinfo"])),
        "iommu_groups" => Ok(run(&params, "find", &["/sys/kernel/iommu_groups", "-maxdepth", "2", "-type", "l"])),
        "iommu_info" => Ok(run(&params, "dmesg", &["--level=info,warn,err"])),
        "irq_list" => Ok(fs::read_to_string("/proc/interrupts").unwrap_or_default()),
        "irq_set_affinity" => set_irq_affinity(&params, &ctx),
        "hugepages_info" => Ok(fs::read_to_string("/proc/sys/vm/nr_hugepages").unwrap_or_default()),
        "hugepages_alloc" => set_hugepages(&params, &ctx),
        "kdump_status" => Ok(run(&params, "kdump-config", &["status"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("hardware_control", action, "ok"),
        Err(e) => audit::log_failure("hardware_control", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}

fn write_sysfs(params: &serde_json::Value, ctx: &ErrorContext, file: &str, value: &str) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let device = ps(params, "device", ctx);
    let path = format!("/sys/bus/usb/devices/{device}/{file}");
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Critical, format!("write {value} to {path}")));
    }
    fs::write(&path, value).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })?;
    Ok(format!("Wrote {value} to {path}"))
}

fn set_irq_affinity(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let irq = ps(params, "irq", ctx);
    let mask = ps(params, "mask", ctx);
    let path = format!("/proc/irq/{irq}/smp_affinity");
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Critical, format!("write {mask} to {path}")));
    }
    fs::write(&path, &mask).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })?;
    Ok(format!("Set IRQ {irq} affinity to {mask}"))
}

fn set_hugepages(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let count = ps(params, "count", ctx);
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Critical, format!("set nr_hugepages to {count}")));
    }
    fs::write("/proc/sys/vm/nr_hugepages", &count).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })?;
    Ok(format!("Set nr_hugepages to {count}"))
}
