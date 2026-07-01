---
description:
alwaysApply: true
---

# CLAUDE.md — AETHER_02

## Project Identity

- **Name:** AETHER_02
- **Type:** MCP server over stdio, 12 tools, 99.9% Linux management
- **Language:** Rust (edition 2021, nightly toolchain for hardening flags)
- **Target:** Linux x86-64/arm64
- **Transport:** stdio — stdout is JSON-RPC ONLY, never print to stdout

## Build

```bash
cargo check    # verify
cargo build    # binary → target/debug/aether-linux-mcp-server
cargo test
```

Release hardening: `lto=fat`, `codegen-units=1`, `panic=abort`, `strip=true`, `target-cpu=native`, Full RELRO, NX stack, PIE, CET/BTI where available.

## Architecture

```
main.rs → dotenvy → FeatureGates → AetherServer → serve((stdin,stdout))

server.rs: AetherServer { gates, tool_router }
  #[tool_router(router = tool_router)] → 12 tools
  #[tool_handler(router = self.tool_router)] → ServerHandler

tools/*.rs → pub fn handle_*(action, params) -> Result<String, AetherError>
error.rs  → AetherError + POSIX/nix helpers
audit.rs  → log_success/failure/forced/security
config.rs → FeatureGates from .env (all disabled by default)
```

## 12 Tools

1. `process_control` — processes, signals, cgroups, namespaces, containers, perf
2. `file_system` — files, ACL/xattr, symlinks, mount, disks, LVM, swap, fsck, NFS, btrfs, ZFS
3. `package_manager` — apt, dnf, pacman, zypper, flatpak, snap, AppImage, nix
4. `system_config` — sysctl, kernel cmdline, modprobe, dconf/gsettings, hostname, timezone, locale, udev
5. `service_manager` — systemd, OpenRC, runit, s6
6. `gui_automation` — X11/Wayland, mouse, keyboard, windows, screenshots, clipboard, display, audio
7. `system_info` — CPU, memory, disk, kernel, GPU, PCI, USB, DMI, ACPI, IOMMU, sensors
8. `network_manager` — iproute2, nftables, NetworkManager, WireGuard, bridges, WiFi, Bluetooth
9. `user_management` — passwd/group/shadow, PAM, polkit, sudoers, SSH keys, sessions
10. `security_audit` — SELinux, AppArmor, auditd, firewall, TPM2, Secure Boot, IMA/EVM, hardening
11. `hardware_control` — GPU, PCI, USB authorization, ACPI, cpufreq, thermal, IOMMU, kdump
12. `system_automation` — journald, timers, cron/anacron, at, tmpfiles, sysusers, boot analysis

* = disabled by default, enabled via `.env` feature gate or requires `force: true`.

## Key Patterns

### Tool Registration

```rust
#[tool_router(router = tool_router)]
impl AetherServer {
    #[tool(description = "...")]
    async fn my_tool(&self, Parameters(args): Parameters<ActionParams>) -> String {
        tools::module::handle_*(&args.action, args.params)
            .unwrap_or_else(|e| format!("Error: {e}"))
    }
}
#[tool_handler(router = self.tool_router)]
impl ServerHandler for AetherServer { fn get_info(&self) -> ServerInfo { ... } }
```

### Error Handling

```rust
let ctx = ErrorContext::new("file_system", "read");
let path = get_string_param(&params, "path", &ctx)?;

if !require_force(&params) {
    return Err(AetherError::force_required(ctx));
}

audit::log_success("file_system", "read", path);
```

### Feature Gates

```rust
gates.check(ctx, gates.partition_edit, "AETHER_PARTITION_EDIT")?;
```

All gates: `AETHER_KEXEC_LOAD`, `AETHER_MODULE_LOAD`, `AETHER_BPF_LOAD`, `AETHER_PTRACE_ATTACH`, `AETHER_NAMESPACE_CREATE`, `AETHER_OFFLINE_MOUNT`, `AETHER_PARTITION_EDIT`, `AETHER_TOKEN_MANIPULATION`.

## Conventions

- NEVER print to stdout; MCP uses it for JSON-RPC
- NEVER spawn shell with interpolated user input — direct syscalls or argv vectors only
- ALWAYS use `// SAFETY:` comments on unsafe blocks
- `snake_case` for Rust, `camelCase` for JSON where the protocol requires it
- Log via `tracing` to stderr with `.with_ansi(false).with_writer(std::io::stderr)`
