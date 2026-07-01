//! AetherServer — the central MCP server with 12 tools, 3 prompts, 20 resources.
use crate::config::FeatureGates;

use crate::tools;
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router, ErrorData, RoleServer, ServerHandler,
};
use serde::Deserialize;
use std::future::Future;

#[derive(Clone)]
pub struct AetherServer {
    pub gates: FeatureGates,
    tool_router: ToolRouter<Self>,
}

impl AetherServer {
    #[must_use]
    pub fn new(gates: FeatureGates) -> Self {
        Self { gates, tool_router: Self::tool_router() }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ActionParams {
    pub action: String,
    #[serde(default)]
    #[schemars(with = "serde_json::Value")]
    pub params: serde_json::Value,
}

// ─── Tool Router ──────────────────────────────────────────────────────────

#[tool_router(router = tool_router)]
impl AetherServer {
    #[tool(
        description = "Process management: list, tree, kill, signal, create, set_priority, set_nice, query_info, threads, set_affinity, memory_info, memory_limits, fd_list, fd_info, limits, environ, suspend, resume, cgroup_list, cgroup_info, cgroup_create, cgroup_delete, cgroup_set_limit, cgroup_move, namespace_list, namespace_enter, oom_score, oom_kill, seccomp_info, ptrace_attach, ptrace_detach, container_list, container_info, container_start, container_stop, container_logs, container_exec, perf_top, perf_stat"
    )]
    async fn process_control(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::process::handle(&self.gates, &args.action, args.params).await
    }

    #[tool(
        description = "File system operations: read, write, append, delete, copy, move, rename, list_dir, stat, statvfs, mkdir, exists, acl_get, acl_set, xattr_list, xattr_get, xattr_set, xattr_delete, symlink_read, symlink_create, symlink_delete, chmod, chown, truncate, mount_list, mount, umount, remount, bind_mount, tmpfs_mount, disk_list, disk_info, part_list, part_create, part_delete, part_resize, mdraid_list, mdraid_info, mdraid_create, lvm_pvs, lvm_vgs, lvm_lvs, lvm_create, swap_list, swap_on, swap_off, swap_create, fsck_check, fsck_repair, nfs_list, nfs_mount, inotify_watch, inotify_read, btrfs_subvol_list, btrfs_snapshot, zfs_list, zfs_snapshot, overlay_mount"
    )]
    async fn file_system(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::filesystem::handle(&self.gates, &args.action, args.params)
    }

    #[tool(
        description = "Package management: list_installed, list_upgradable, search, info, install, remove, upgrade, upgrade_all, hold, unhold, list_repos, add_repo, remove_repo, clean_cache, history, verify, download, flatpak_list, flatpak_install, flatpak_remove, snap_list, snap_install, snap_remove, appimage_list, nix_list, nix_install, nix_remove"
    )]
    async fn package_manager(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::packages::handle(&args.action, args.params)
    }

    #[tool(
        description = "System configuration: sysctl_list, sysctl_get, sysctl_set, sysctl_load, kernel_cmdline_get, kernel_cmdline_set, modprobe_list, modprobe_load, modprobe_unload, modprobe_blacklist, dracut_rebuild, dconf_list, dconf_get, dconf_set, gsettings_list, gsettings_get, gsettings_set, limits_get, limits_set, hostname_get, hostname_set, timezone_list, timezone_get, timezone_set, locale_list, locale_get, locale_set, keyboard_layout, etc_config_read, etc_config_write, udev_info, udev_rules_list, udev_trigger, environment_list, environment_set, os_release"
    )]
    async fn system_config(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::sysconfig::handle(&self.gates, &args.action, args.params)
    }

    #[tool(
        description = "Service management: list, list_units, list_unit_files, status, start, stop, restart, reload, enable, disable, mask, unmask, cat, edit, show, list_dependencies, list_timers, list_sockets, list_paths, list_mounts, isolate, default, rescue, emergency, daemon_reload, reset_failed, user_services, analyze. Supports systemd, OpenRC, runit, s6."
    )]
    async fn service_manager(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::service::handle(&args.action, args.params)
    }

    #[tool(
        description = "GUI automation: mouse_move, mouse_click, mouse_scroll, mouse_position, mouse_drag, keyboard_type, keyboard_press, keyboard_combo, keyboard_state, find_window, list_windows, set_window_pos, set_window_size, focus_window, get_window_rect, get_window_title, get_window_class, close_window, minimize_window, maximize_window, screenshot, screenshot_window, clipboard_read, clipboard_write, clipboard_clear, display_list, display_info, display_mode_set, display_scale, display_rotation, display_brightness_get, display_brightness_set, display_night_light, display_color_profile, compositor_info, audio_list_sinks, audio_list_sources, audio_volume_get, audio_volume_set, audio_mute, audio_default_sink, input_list_devices, input_device_info, input_remap, notification_send, notification_history, screensaver_lock, screensaver_unlock, inhibit_idle"
    )]
    async fn gui_automation(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::gui::handle(&self.gates, &args.action, args.params)
    }

    #[tool(
        description = "System information: cpu_info, cpu_topology, cpu_freq, cpu_governor, cpu_vulnerabilities, cpu_microcode, cpu_cache, cpu_perf_flags, memory_info, memory_topology, memory_hugepages, memory_numa, memory_slab, disk_list, disk_info, disk_smart, disk_iostat, disk_partitions, os_info, os_release, kernel_info, kernel_modules, kernel_tainted, uptime, load_avg, env_vars, hostname, boot_time, gpu_list, gpu_info, gpu_usage, gpu_temperature, gpu_driver, pci_list, pci_info, usb_list, usb_info, dmi_bios, dmi_system, dmi_board, dmi_chassis, acpi_tables, acpi_power, edid_info, iommu_groups, irq_list, irq_affinity, sensors_list, sensors_temperature, sensors_fan, sensors_voltage, device_list, driver_list, firmware_list"
    )]
    async fn system_info(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::sysinfo::handle(&args.action, args.params)
    }

    #[tool(
        description = "Network management: adapter_list, adapter_info, adapter_up, adapter_down, adapter_stats, adapter_speed, address_list, address_add, address_delete, route_list, route_add, route_delete, connection_list, connection_info, connection_create, connection_modify, connection_delete, connection_up, connection_down, dns_servers, dns_cache, dns_flush, dns_set, firewall_rules_list, firewall_rule_add, firewall_rule_delete, firewall_chain_list, nftables_table_list, nftables_chain_list, nftables_rule_add, nftables_rule_delete, wireguard_list, wireguard_info, wireguard_config, bridge_list, bridge_create, bridge_add_port, bond_list, bond_create, vlan_list, vlan_create, network_namespace_list, network_namespace_create, tc_qdisc_list, tc_qdisc_add, socket_list, socket_info, wifi_list, wifi_connect, wifi_disconnect, wifi_scan, bluetooth_list, bluetooth_info, bluetooth_connect, bluetooth_pair, proxy_list, proxy_set, hosts_read, hosts_write"
    )]
    async fn network_manager(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::network::handle(&self.gates, &args.action, args.params)
    }

    #[tool(
        description = "User management: users_list, user_info, user_create, user_delete, user_modify, user_password_set, user_lock, user_unlock, groups_list, group_info, group_create, group_delete, group_add_user, group_remove_user, sessions_list, session_info, session_terminate, current_user, whoami, id, nsswitch_conf, pam_list_modules, pam_conf_read, pam_conf_write, polkit_list, polkit_info, sudoers_read, sudoers_write, sudoers_list_users, ssh_authorized_keys_list, ssh_authorized_keys_add, ssh_authorized_keys_remove, faillock_list, faillock_reset, subuid_list, subgid_list, acl_users, last_logins, loginctl_list, loginctl_user_status"
    )]
    async fn user_management(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::user::handle(&self.gates, &args.action, args.params)
    }

    #[tool(
        description = "Security audit: selinux_status, selinux_mode, selinux_set_mode, selinux_booleans, selinux_policy, selinux_context_list, selinux_set_context, apparmor_status, apparmor_profiles, apparmor_set_mode, auditd_status, auditd_rules_list, auditd_rule_add, auditd_rule_delete, firewall_status, firewall_profile, tpm2_info, tpm2_pcrs, tpm2_quote, secure_boot_status, ima_status, ima_policy, evm_status, fapolicyd_status, lockdown_status, kernel_lsms, kernel_mitigations, kernel_hardening, aslr_status, mmap_min_addr, kptr_restrict, dmesg_restrict, ptrace_scope, core_pattern, file_integrity_list, file_integrity_check, aide_check, lynis_check, cve_mitigations, uefi_variables, boot_chain"
    )]
    async fn security_audit(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::security::handle(&self.gates, &args.action, args.params)
    }

    #[tool(
        description = "Hardware control: gpu_nvidia_info, gpu_nvidia_smi, gpu_nvidia_power, gpu_nvidia_fan, gpu_amdgpu_info, gpu_intel_info, gpu_vulkan_info, pci_rescan, pci_remove, usb_devices, usb_authorize, usb_deauthorize, acpi_info, acpi_wakeup, acpi_event, cpufreq_governor_list, cpufreq_governor_get, cpufreq_governor_set, thermal_zones, thermal_temperature, thermal_cooling, cpu_microcode_version, cpu_microcode_reload, iommu_groups, iommu_info, irq_list, irq_set_affinity, hugepages_info, hugepages_alloc, kdump_status, kdump_config, kdump_test, memory_error_count"
    )]
    async fn hardware_control(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::hardware::handle(&self.gates, &args.action, args.params)
    }

    #[tool(
        description = "System automation: journal_list, journal_query, journal_follow, journal_boot, journal_vacuum, journal_disk_usage, timer_list, timer_info, timer_enable, timer_disable, timer_create, timer_delete, cron_list, cron_user_list, cron_add, cron_remove, anacron_list, at_list, at_create, at_delete, tmpfiles_list, tmpfiles_create, tmpfiles_clean, sysusers_list, sysusers_create, binfmt_list, binfmt_register, modules_load_list, modules_load_add, sysctl_d_list, sysctl_d_apply, boot_analyze, boot_chart"
    )]
    async fn system_automation(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::automation::handle(&self.gates, &args.action, args.params)
    }
}

// ─── Resource definitions ────────────────────────────────────────────────

const RESOURCES: &[(&str, &str, &str)] = &[
    ("aether://system/cpu", "cpu_info", "Live CPU metrics — model, cores, frequency, governor, vulnerabilities, cache"),
    (
        "aether://system/memory",
        "memory_info",
        "Live memory metrics — total, used, available, buffer/cache, swap, hugepages",
    ),
    ("aether://system/disk", "disk_info", "Disk layout — lsblk output with partitions, filesystems, mount points"),
    ("aether://system/os-release", "os_release", "OS identification — distribution, version, kernel, architecture"),
    ("aether://system/uptime", "uptime_info", "System uptime, load averages, and boot time"),
    ("aether://system/kernel", "kernel_info", "Kernel version, modules loaded, tainted state"),
    ("aether://system/gpu", "gpu_info", "GPU list from lspci — vendor, model, driver"),
    ("aether://system/pci", "pci_devices", "Full PCI bus enumeration with device details"),
    ("aether://system/usb", "usb_devices", "USB device tree with vendor/product IDs"),
    ("aether://system/dmi", "dmi_system", "DMI/SMBIOS system information — manufacturer, product, serial"),
    ("aether://system/thermal", "thermal_zones", "Thermal zone temperatures from /sys/class/thermal and sensors"),
    (
        "aether://system/network-adapters",
        "network_adapters",
        "Network interfaces — IP addresses, MAC, speed, driver, statistics",
    ),
    ("aether://system/firewall", "firewall_status", "Firewall state — ufw/firewalld/nftables rules overview"),
    ("aether://system/services", "services_status", "Systemd service status — active, enabled, failed, masked units"),
    ("aether://system/users", "users_groups", "Local users and groups from /etc/passwd and /etc/group"),
    (
        "aether://system/security",
        "security_status",
        "Security subsystem status — SELinux/AppArmor, auditd, Secure Boot, kernel lockdown",
    ),
    ("aether://config/feature-gates", "feature_gates", "Current feature gate configuration from .env"),
    ("aether://system/environment", "environment", "System environment variables"),
    (
        "aether://system/installed-packages",
        "installed_packages",
        "List of installed packages via detected package manager",
    ),
    ("aether://system/journal", "journal_recent", "Recent journal entries since last boot"),
];

const PROMPT_INFO: &[(&str, &str)] = &[
    ("system_health", "Analyze CPU, memory, disk, OS, and process health. Check load, uptime, swap usage, disk space, and top processes with optional service status."),
    ("security_posture", "Audit Linux security posture — SELinux/AppArmor status, firewall rules, kernel hardening, ASLR, Secure Boot, TPM, file integrity. Generate a risk-scored security report."),
    ("diagnostic_report", "Generate a structured diagnostic report — journal errors, failing services, disk health, OOM events, network issues, and recent crashes. Include actionable remediation steps."),
];

// ─── Helper functions ────────────────────────────────────────────────────

fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|_| format!("Cannot read {}", path))
}

fn read_cmd(cmd: &str, args: &[&str]) -> String {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_else(|_| format!("'{}' not available", cmd))
}

fn resolve_resource(p: &str) -> String {
    match p {
        "cpu" => read_cmd("lscpu", &[]),
        "memory" => read_cmd("free", &["-h"]),
        "disk" => read_cmd("lsblk", &["-o", "NAME,SIZE,TYPE,MOUNTPOINT,FSTYPE"]),
        "os-release" => read_file("/etc/os-release"),
        "uptime" => read_file("/proc/uptime"),
        "kernel" => read_cmd("uname", &["-a"]),
        "gpu" => read_cmd("sh", &["-c", "lspci | grep -iE 'vga|3d|display'"]),
        "pci" => read_cmd("lspci", &[]),
        "usb" => read_cmd("lsusb", &["-t"]),
        "dmi" => read_cmd("dmidecode", &["-t", "system"]),
        "thermal" => read_cmd("sensors", &[]),
        "network-adapters" => read_cmd("ip", &["-s", "link", "show"]),
        "firewall" => read_cmd("ufw", &["status", "verbose"]),
        "services" => read_cmd("systemctl", &["list-units", "--type=service", "--no-pager"]),
        "users" => read_file("/etc/passwd"),
        "security" => {
            format!(
                "SELinux: {}\nAppArmor: {}\nSecure Boot: {}\nASLR: {}\nLockdown: {}",
                read_cmd("getenforce", &[]).trim(),
                read_cmd("aa-status", &["--enabled"]).lines().next().unwrap_or("not enabled"),
                read_cmd("mokutil", &["--sb-state"]).trim(),
                read_file("/proc/sys/kernel/randomize_va_space").trim(),
                read_file("/sys/kernel/security/lockdown").trim(),
            )
        }
        "feature-gates" => "See AETHER_02 .env — all gates default to 0 (disabled)".to_string(),
        "environment" => read_cmd("env", &[]),
        "installed-packages" => read_cmd("dpkg", &["-l"]),
        "journal" => read_cmd("journalctl", &["-n", "50", "--no-pager"]),
        _ => "Unknown resource".to_string(),
    }
}

fn prompt_content(name: &str) -> String {
    match name {
        "system_health" => "Perform a comprehensive system health check on this Linux machine:\n\
            1. CPU: model, cores, frequency, load averages, throttling\n\
            2. Memory: total, used, available, swap, top consumers\n\
            3. Disks: filesystems, usage %, near-full partitions\n\
            4. Processes: top 10 by CPU/memory, zombies, process count\n\
            5. Services: failed systemd units, degraded state\n\
            6. Network: active interfaces, IPs, connection stats\n\
            7. Kernel: version, uptime, tainted flags\n\
            Summarize with a health score (1-10) and actionable recommendations."
            .to_string(),
        "security_posture" => "Audit the security posture of this Linux system:\n\
            1. MAC: SELinux/AppArmor mode and profiles\n\
            2. Firewall: active rules, open ports, default policy\n\
            3. Kernel hardening: ASLR, kptr_restrict, dmesg_restrict, ptrace_scope\n\
            4. Secure Boot & TPM: status, PCR values\n\
            5. Integrity: IMA/EVM, AIDE\n\
            6. Vulnerabilities: CPU mitigations active\n\
            7. Audit daemon: status and rules\n\
            Output a risk-scored report (Critical/High/Medium/Low) with remediation steps."
            .to_string(),
        "diagnostic_report" => "Generate a structured diagnostic report:\n\
            1. Journal: errors/warnings from last boot\n\
            2. Services: failed, inactive, masked units\n\
            3. OOM: recent Out-of-Memory killer events\n\
            4. Disk health: S.M.A.R.T., filesystem errors\n\
            5. Network: errors, dropped packets, connectivity\n\
            6. Crashes: core dumps, kernel oops, segfaults\n\
            7. Packages: broken, held, available updates\n\
            Format as structured report with severity and actionable remediation."
            .to_string(),
        _ => "Unknown prompt".to_string(),
    }
}

// ─── Tool + Prompt + Resource Handler ────────────────────────────────────

#[tool_handler(router = self.tool_router)]
impl ServerHandler for AetherServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .enable_logging()
                .build(),
            server_info: Implementation { name: "AETHER_02".to_string(), version: "1.0.1".to_string() },
            instructions: Some(
                r#"AETHER_02 — Full-spectrum Linux management server.

12 tools covering 99.9% of system administration including GUI automation.
3 prompts for system health, security audit, and diagnostics.
20 resources for real-time system information.

Dangerous operations require `force: true` parameter.
Feature gates in `.env` control critically dangerous capabilities.

Get started:
- Use the "system_info" tool with action "cpu_info" for CPU details
- Use the "system_health" prompt for a full health assessment
- Read resource "aether://system/cpu" for live CPU metrics
"#
                .to_string(),
            ),
        }
    }

    /// List all 3 prompts
    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        Ok(ListPromptsResult {
            prompts: PROMPT_INFO
                .iter()
                .map(|(name, desc)| Prompt {
                    name: name.to_string(),
                    description: Some(desc.to_string()),
                    arguments: None,
                })
                .collect(),
            ..Default::default()
        })
    }

    /// Get a specific prompt content
    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        let desc = PROMPT_INFO.iter().find(|(n, _)| *n == request.name).map(|(_, d)| d.to_string());

        Ok(GetPromptResult {
            description: desc,
            messages: vec![PromptMessage {
                role: PromptMessageRole::User,
                content: PromptMessageContent::Text { text: prompt_content(&request.name) },
            }],
        })
    }

    /// List all 20 resources
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult {
            resources: RESOURCES
                .iter()
                .map(|(uri, name, desc)| Annotated {
                    raw: RawResource {
                        uri: uri.to_string(),
                        name: name.to_string(),
                        description: Some(desc.to_string()),
                        mime_type: Some("text/plain".to_string()),
                        size: None,
                    },
                    annotations: None,
                })
                .collect(),
            ..Default::default()
        })
    }

    /// Read a specific resource by URI
    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let key = request
            .uri
            .strip_prefix("aether://system/")
            .or_else(|| request.uri.strip_prefix("aether://config/"))
            .unwrap_or(&request.uri);

        let content = if key == "feature-gates" {
            format!(
                "{{\"kexec_load\":{},\"module_load\":{},\"bpf_load\":{},\"ptrace_attach\":{},\"namespace_create\":{},\"offline_mount\":{},\"partition_edit\":{},\"token_manipulation\":{}}}",
                self.gates.kexec_load, self.gates.module_load, self.gates.bpf_load,
                self.gates.ptrace_attach, self.gates.namespace_create,
                self.gates.offline_mount, self.gates.partition_edit, self.gates.token_manipulation,
            )
        } else {
            resolve_resource(key)
        };

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::TextResourceContents {
                uri: request.uri.clone(),
                text: content,
                mime_type: Some("text/plain".to_string()),
            }],
        })
    }
}
