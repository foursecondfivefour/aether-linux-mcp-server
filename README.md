# AETHER_02 — Full-Spectrum Linux MCP Server

[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![npm](https://img.shields.io/npm/v/%40foursecondfivefour%2Faether-linux-mcp-server?color=red)](https://www.npmjs.com/package/@foursecondfivefour/aether-linux-mcp-server)

[![Add AETHER_02 MCP server to Cursor](https://cursor.com/deeplink/mcp-install-dark.png)](cursor://anysphere.cursor-deeplink/mcp/install?name=aether-02&config=eyJjb21tYW5kIjoibnB4IiwiYXJncyI6WyIteSIsIkBmb3Vyc2Vjb25kZml2ZWZvdXIvYWV0aGVyLWxpbnV4LW1jcC1zZXJ2ZXIiXSwiZW52Ijp7IlJVU1RfTE9HIjoiaW5mbyJ9fQ==)
[![Install in VS Code](https://img.shields.io/badge/Install_in-VS_Code-007ACC?style=for-the-badge&logo=visualstudiocode&logoColor=white)](https://vscode.dev/redirect/mcp/install?name=aether-02&config=%7B%22command%22%3A%22npx%22%2C%22args%22%3A%5B%22-y%22%2C%22%40foursecondfivefour%2Faether-linux-mcp-server%22%5D%2C%22env%22%3A%7B%22RUST_LOG%22%3A%22info%22%7D%7D)
[![Install in VS Code Insiders](https://img.shields.io/badge/Install_in-VS_Code_Insiders-24BFA5?style=for-the-badge&logo=visualstudiocode&logoColor=white)](https://insiders.vscode.dev/redirect/mcp/install?name=aether-02&config=%7B%22command%22%3A%22npx%22%2C%22args%22%3A%5B%22-y%22%2C%22%40foursecondfivefour%2Faether-linux-mcp-server%22%5D%2C%22env%22%3A%7B%22RUST_LOG%22%3A%22info%22%7D%7D&quality=insiders)

[![npm install](https://img.shields.io/badge/npm%20install-g%20aether--linux--mcp--server-CB3837?logo=npm&style=for-the-badge)](https://www.npmjs.com/package/@foursecondfivefour/aether-linux-mcp-server)
[![Shell install](https://img.shields.io/badge/Shell-curl%20%7C%20bash-4EAA25?logo=gnubash&logoColor=white&style=for-the-badge)](https://raw.githubusercontent.com/foursecondfivefour/aether-linux-mcp-server/main/scripts/install/install.sh)

**12 tools. 3 prompts. 20 resources. 99.9% Linux coverage. stdio-only.**

AETHER_02 is a Rust MCP server that lets AI assistants operate a local Linux system through the Model Context Protocol. It covers process management, filesystem operations, package managers, system configuration, services, GUI automation, system info, networking, users, security, hardware control, and automation.

AETHER_02 is the Linux counterpart to [AETHER_01 for Windows](https://github.com/foursecondfivefour/aether-mcp-server).

---

## Quick install

### npm

```bash
npm install -g @foursecondfivefour/aether-linux-mcp-server
```

### Shell installer

```bash
curl -sSL https://raw.githubusercontent.com/foursecondfivefour/aether-linux-mcp-server/main/scripts/install/install.sh | bash
```

### Build from source

```bash
git clone https://github.com/foursecondfivefour/aether-linux-mcp-server
cd aether-linux-mcp-server
cp config/env/.env.example .env
cargo build --release
./scripts/install/install.sh target/release/aether-mcp-server
```

More install/client details: [docs/INSTALL.md](docs/INSTALL.md), [docs/MCP_CLIENTS.md](docs/MCP_CLIENTS.md).

---

## Tool overview

| # | Tool | Scope |
|---|------|-------|
| 1 | `process_control` | Processes, signals, priorities, cgroups, namespaces, containers, perf |
| 2 | `file_system` | Files, ACL/xattr, symlinks, mounts, disks, LVM, swap, fsck, NFS, btrfs, ZFS |
| 3 | `package_manager` | apt, dnf, pacman, zypper, flatpak, snap, AppImage, nix |
| 4 | `system_config` | sysctl, kernel cmdline, modprobe, hostname, timezone, locale, udev, dconf |
| 5 | `service_manager` | systemd, OpenRC, runit, s6 |
| 6 | `gui_automation` | X11/Wayland, mouse, keyboard, windows, screenshots, clipboard, display, audio |
| 7 | `system_info` | CPU, memory, disk, kernel, GPU, PCI, USB, DMI, ACPI, IOMMU, sensors |
| 8 | `network_manager` | iproute2, nftables, NetworkManager, WireGuard, bridges, WiFi, Bluetooth |
| 9 | `user_management` | users, groups, sessions, PAM, polkit, sudoers, SSH keys |
| 10 | `security_audit` | SELinux, AppArmor, auditd, firewall, TPM2, Secure Boot, IMA/EVM, hardening |
| 11 | `hardware_control` | GPU, PCI, USB authorization, ACPI, cpufreq, thermal, IOMMU, kdump |
| 12 | `system_automation` | journald, timers, cron/anacron, at, tmpfiles, sysusers, boot analysis |

Full action list: [docs/TOOLS.md](docs/TOOLS.md).

---

## Safety model

AETHER_02 is an administrative tool. It is intentionally powerful, so risky actions are guarded:

- **stdio-only** — no HTTP server and no listening TCP port.
- **stdout stays JSON-RPC only** — logs and audit output go to stderr.
- **No shell interpolation** — use syscalls or explicit argv vectors.
- **Command allowlist** blocks arbitrary command execution through command-backed handlers.
- **`force: true`** required for destructive operations.
- **`dry_run: true`** previews dangerous operations without mutating the host.
- **Feature gates** in `.env` protect critically dangerous capabilities.
- **Audit logging** records tool invocations.

Feature gates and hardening details: [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md).

---

## Repository map

```text
.
├── docs/                 # Detailed docs split by topic
├── scripts/              # Install and smoke-test helper scripts
├── config/               # Runtime/config examples
├── integrations/         # Marketplace/client metadata
├── src/                  # Rust MCP server implementation
│   └── tools/            # 12 Linux tool groups
├── tests/                # Config, error, dispatch, and smoke tests
├── .agents/              # Agent instructions and skill metadata
├── .cursor/              # Cursor project rules
├── .github/              # Issues, PR template, community docs
├── Cargo.toml            # Rust crate metadata
└── package.json          # npm package metadata
```

Detailed map: [docs/PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md).

---

## Development

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

More: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md), [docs/RELEASE.md](docs/RELEASE.md).

---

## License

MIT — see [LICENSE](LICENSE).
