# Project Structure

The repository is organized so the root stays small: only language/package entry points and root-required tool config remain there. Detailed docs, install helpers, community files, agent instructions, and marketplace metadata live in dedicated folders.

```text
.
├── .agents/                         # Agent instructions and skill metadata
│   ├── instructions/                 # AGENTS.md, CLAUDE.md
│   └── skills/aether-linux-mcp/      # Skill manifest/documentation
├── .cargo/                          # Cargo/rustc hardening flags
├── .cursor/                         # Cursor project rules
├── .github/                         # GitHub templates and community docs
│   ├── ISSUE_TEMPLATE/
│   └── community/                    # CONTRIBUTING, SECURITY, CODE_OF_CONDUCT
├── config/                          # Runtime/config examples
│   └── env/.env.example
├── docs/                            # Detailed documentation split by topic
├── integrations/                    # External marketplace/client metadata
│   └── lobehub/lhm.plugin.json
├── scripts/                         # Install and smoke-test helper scripts
│   ├── install/                     # npm and shell installers
│   └── smoke_stdio.py
├── src/                             # Rust implementation
│   ├── main.rs                      # stdio server bootstrap
│   ├── lib.rs                       # Public module declarations
│   ├── server.rs                    # MCP router, prompts, resources
│   ├── config.rs                    # Feature gates
│   ├── error.rs                     # AetherError and parameter helpers
│   ├── audit.rs                     # Structured audit logging
│   └── tools/                       # 12 Linux tool groups
├── tests/                           # Rust tests
├── Cargo.toml / Cargo.lock          # Rust package metadata
├── package.json                     # npm package metadata
├── rust-toolchain.toml              # Rust toolchain pin
├── rustfmt.toml / clippy.toml       # Rust quality-tool config
├── README.md                        # Short overview and quick start
└── LICENSE                          # License
```

## Source modules

| Path | Responsibility |
|------|----------------|
| `src/main.rs` | Load `.env`, initialize stderr logging, start stdio MCP service |
| `src/server.rs` | Register 12 tools, 3 prompts, 20 resources |
| `src/config.rs` | Load disabled-by-default feature gates |
| `src/error.rs` | Shared error type and JSON parameter helpers |
| `src/audit.rs` | Success/failure/security/forced-operation audit logs |
| `src/tools/process.rs` | Process, signal, cgroup, namespace, container, perf actions |
| `src/tools/filesystem.rs` | Files, permissions, mounts, disks, LVM, swap, fsck, NFS, btrfs, ZFS |
| `src/tools/packages.rs` | apt/dnf/pacman/zypper/flatpak/snap/AppImage/nix actions |
| `src/tools/sysconfig.rs` | sysctl, modprobe, hostname, timezone, locale, dconf/gsettings |
| `src/tools/service.rs` | systemd/OpenRC/runit/s6 service management |
| `src/tools/gui.rs` | X11/Wayland GUI automation, display, audio, clipboard |
| `src/tools/sysinfo.rs` | CPU, memory, disk, kernel, hardware, sensors information |
| `src/tools/network.rs` | iproute2, nftables, NetworkManager, WireGuard, WiFi, Bluetooth |
| `src/tools/user.rs` | Users, groups, sessions, PAM, sudoers, SSH keys |
| `src/tools/security.rs` | SELinux/AppArmor/auditd/firewall/TPM/Secure Boot/hardening |
| `src/tools/hardware.rs` | GPU, PCI, USB, ACPI, cpufreq, thermal, IOMMU, kdump |
| `src/tools/automation.rs` | journald, timers, cron, at, tmpfiles, sysusers, boot analysis |
| `src/tools/helpers.rs` | Shared read/command helpers |

## Root file policy

Keep only files that tools conventionally expect at the root:

- `README.md`, `LICENSE`
- `Cargo.toml`, `Cargo.lock`, `package.json`
- Rust tool configuration (`rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`)
- repository-wide editor files (`.editorconfig`, `.gitignore`, `.cursorignore`, `.windsurfrules`)

Everything else should live under `docs/`, `scripts/`, `config/`, `.github/community/`, `.agents/`, or `integrations/`.
