# AETHER_02 — Skill Configuration

```json
{
  "tools": [
    "system_info",
    "hardware_control",
    "security_audit",
    "network_manager",
    "user_management",
    "process_control",
    "service_manager",
    "gui_automation",
    "system_automation",
    "file_system",
    "package_manager",
    "system_config"
  ],
  "prompts": [
    "system_health",
    "security_posture",
    "diagnostic_report"
  ],
  "resources": [
    "aether://system/cpu",
    "aether://system/memory",
    "aether://system/disk",
    "aether://system/os-release",
    "aether://system/uptime",
    "aether://system/kernel",
    "aether://system/gpu",
    "aether://system/pci",
    "aether://system/usb",
    "aether://system/dmi",
    "aether://system/thermal",
    "aether://system/network-adapters",
    "aether://system/firewall",
    "aether://system/services",
    "aether://system/users",
    "aether://system/security",
    "aether://config/feature-gates",
    "aether://system/environment",
    "aether://system/installed-packages",
    "aether://system/journal"
  ]
}
```

## AETHER_02 — Full-Spectrum Linux MCP Server

**12 tools. 3 prompts. 20 resources. 99.9% Linux coverage. Zero compromises.**

AETHER_02 gives AI assistants full control over Linux via standard input/output.
From process management to GUI automation, from kernel security audit to hardware control.
Separate repository from [AETHER_01](https://github.com/foursecondfivefour/aether-mcp-server) (Windows).
Same philosophy, different OS.

### Tools (12)

| # | Tool | Coverage |
|---|------|----------|
| 1 | `process_control` | Processes, signals, cgroups, namespaces, containers, Docker/Podman, perf |
| 2 | `file_system` | Files, ACL, xattr, symlinks, mount, LVM, swap, fsck, NFS, btrfs, ZFS |
| 3 | `package_manager` | apt, dnf, pacman, zypper, flatpak, snap, AppImage, nix |
| 4 | `system_config` | sysctl, kernel cmdline, modprobe, dconf, hostname, timezone, locale, udev |
| 5 | `service_manager` | systemd, OpenRC, runit, s6 — start/stop/restart/enable/disable |
| 6 | `gui_automation` | X11 + Wayland, mouse/keyboard, windows, screenshots, audio, display |
| 7 | `system_info` | CPU, memory, disk, OS, GPU, PCI, USB, DMI, ACPI, IOMMU, NUMA, sensors |
| 8 | `network_manager` | iproute2, nftables, NetworkManager, WireGuard, bridges, WiFi, Bluetooth |
| 9 | `user_management` | passwd/group/shadow, useradd/userdel, PAM, polkit, sudoers, SSH keys |
| 10 | `security_audit` | SELinux, AppArmor, auditd, ufw, TPM2, Secure Boot, IMA/EVM, kernel hardening |
| 11 | `hardware_control` | NVIDIA/AMD/Intel GPU, PCI, USB, ACPI, cpufreq, thermal, IOMMU, kdump |
| 12 | `system_automation` | journald, systemd timers, cron, at, tmpfiles, sysusers, boot analysis |

### Prompts (3)

| Prompt | Description |
|--------|-------------|
| `system_health` | Analyze CPU, memory, disk, OS, and process health with a scored report |
| `security_posture` | Audit security: SELinux/AppArmor, firewall, kernel hardening, Secure Boot, TPM |
| `diagnostic_report` | Structured diagnostic: journal errors, failed services, OOM events, remediation |

### Resources (20)

20 real-time system resources via `aether://` URIs — CPU, memory, disk, OS, kernel, GPU, PCI, USB, DMI, thermal, network, firewall, services, users, security, feature gates, environment, packages, journal.

### Security Model

- **stdio-only** — no network, no HTTP, no TCP
- **No shell injection** — direct syscalls via nix/libc, CLI args as Vec<String>
- **force: true** — destructive operations require explicit confirmation
- **Feature gates** — 8 dangerous capabilities disabled by default in `.env`
- **Audit log to stderr** — every invocation logged
- **Compiler hardening**: PIE, Full RELRO, BIND_NOW, NX, CET (IBT+SHSTK), stack-protector=strong, fat LTO, strip

### Installation

```bash
curl -sSL https://raw.githubusercontent.com/foursecondfivefour/aether-linux-mcp-server/main/scripts/install/install.sh | bash
```

Or build from source:
```bash
git clone https://github.com/foursecondfivefour/aether-linux-mcp-server
cd aether-linux-mcp-server
cp config/env/.env.example .env
cargo build --release
```

### MCP Configuration

```json
{
  "mcpServers": {
    "aether-02": {
      "command": "/path/to/aether-linux-mcp-server/target/release/aether-mcp-server",
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Feature Gates (.env)

| Gate | Env Var | Description |
|------|---------|-------------|
| Kexec | `AETHER_KEXEC_LOAD=0` | Load new kernel via kexec |
| Module Load | `AETHER_MODULE_LOAD=0` | Load/unload kernel modules |
| BPF Load | `AETHER_BPF_LOAD=0` | Load BPF programs |
| Ptrace | `AETHER_PTRACE_ATTACH=0` | ptrace attach to non-child |
| Namespace | `AETHER_NAMESPACE_CREATE=0` | Creating new namespaces |
| Offline Mount | `AETHER_OFFLINE_MOUNT=0` | Mount from raw block devices |
| Partition | `AETHER_PARTITION_EDIT=0` | Create/delete/resize partitions |
| Token | `AETHER_TOKEN_MANIPULATION=0` | Capability setuid/setgid changes |

### License

MIT — see [LICENSE](LICENSE)
