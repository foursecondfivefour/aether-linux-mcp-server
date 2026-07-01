use crate::config::FeatureGates;
use crate::error::{require_force, AetherError, ErrorContext};
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Safe,
    Moderate,
    Dangerous,
    Critical,
}

impl RiskLevel {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Moderate => "moderate",
            Self::Dangerous => "dangerous",
            Self::Critical => "critical",
        }
    }
}

const ALLOWED_COMMANDS: &[&str] = &[
    "aa-status",
    "acpi",
    "apt",
    "apt-cache",
    "atq",
    "auditctl",
    "bluetoothctl",
    "bridge",
    "chmod",
    "chown",
    "cpupower",
    "crontab",
    "dconf",
    "df",
    "dmesg",
    "dmidecode",
    "dnf",
    "dpkg",
    "env",
    "ethtool",
    "faillock",
    "fdisk",
    "find",
    "findmnt",
    "flatpak",
    "free",
    "getenforce",
    "getent",
    "getsebool",
    "grep",
    "getfacl",
    "gpasswd",
    "gsettings",
    "groupadd",
    "groupdel",
    "hostnamectl",
    "id",
    "import",
    "ip",
    "journalctl",
    "kdump-config",
    "last",
    "locale",
    "localectl",
    "loginctl",
    "ls",
    "lsblk",
    "lsmod",
    "lspci",
    "lsusb",
    "lvs",
    "modprobe",
    "ln",
    "mokutil",
    "mount",
    "nft",
    "nix",
    "nmcli",
    "nvidia-smi",
    "pactl",
    "pacman",
    "parted",
    "passwd",
    "ps",
    "pvs",
    "rc-status",
    "renice",
    "resolvectl",
    "rocm-smi",
    "rpm",
    "sensors",
    "sestatus",
    "setfacl",
    "sha256sum",
    "snap",
    "ss",
    "sv",
    "swapon",
    "systemctl",
    "systemd-analyze",
    "sysctl",
    "timedatectl",
    "tpm2_getcap",
    "tpm2_pcrread",
    "truncate",
    "udevadm",
    "ufw",
    "umount",
    "uname",
    "useradd",
    "userdel",
    "usermod",
    "vgs",
    "vulkaninfo",
    "w",
    "whoami",
    "wg",
    "wmctrl",
    "xclip",
    "xdotool",
    "xrandr",
    "zfs",
    "zypper",
];

#[must_use]
pub fn is_allowed_command(cmd: &str) -> bool {
    ALLOWED_COMMANDS.contains(&cmd)
}

#[must_use]
pub fn command_exists(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(cmd).is_file()))
        .unwrap_or(false)
}

#[must_use]
pub fn dry_run(params: &serde_json::Value) -> bool {
    params.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(false)
}

#[must_use]
pub fn shell_quote(s: &str) -> String {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':' | '@' | '=' | '+')) {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

#[must_use]
pub fn command_line(cmd: &str, args: &[&str]) -> String {
    std::iter::once(cmd).chain(args.iter().copied()).map(shell_quote).collect::<Vec<_>>().join(" ")
}

#[must_use]
pub fn preview_command(cmd: &str, args: &[&str]) -> String {
    format!("DRY-RUN: {}", command_line(cmd, args))
}

#[must_use]
pub fn preview_action(risk: RiskLevel, detail: impl AsRef<str>) -> String {
    let message = format!("DRY-RUN [{}]: {}", risk.as_str(), detail.as_ref());
    serde_json::json!({
        "ok": true,
        "type": "action_preview",
        "dry_run": true,
        "risk": risk.as_str(),
        "message": message,
    })
    .to_string()
}

#[must_use]
pub fn tool_response(tool: &str, action: &str, raw: String) -> String {
    let parsed = serde_json::from_str::<serde_json::Value>(&raw).ok();
    let nested_ok = parsed.as_ref().and_then(|v| v.get("ok")).and_then(|v| v.as_bool());
    let ok = nested_ok.unwrap_or_else(|| !raw.starts_with("Error:"));
    let result = parsed.unwrap_or(serde_json::Value::String(raw));
    let spec = crate::actions::get(tool, action);
    let spec_json = spec.map(|s| s.to_json());

    serde_json::json!({
        "ok": ok,
        "tool": tool,
        "action": action,
        "risk": spec.map(|s| s.risk).unwrap_or("unknown"),
        "requires_force": spec.map(|s| s.requires_force).unwrap_or(false),
        "supports_dry_run": spec.map(|s| s.supports_dry_run).unwrap_or(false),
        "feature_gate": spec.and_then(|s| s.feature_gate),
        "required_params": spec.map(|s| s.required_params).unwrap_or(&[]),
        "spec": spec_json,
        "result": result,
    })
    .to_string()
}

pub fn require_force_or_dry_run(params: &serde_json::Value, ctx: &ErrorContext) -> Result<(), AetherError> {
    if dry_run(params) || require_force(params) {
        Ok(())
    } else {
        Err(AetherError::force_required(ctx.clone()))
    }
}

pub fn cmd(cmd: &str, args: &[&str]) -> String {
    command_response(cmd, args, false)
}

pub fn cmd_params(params: &serde_json::Value, cmd: &str, args: &[&str]) -> String {
    command_response(cmd, args, dry_run(params))
}

fn command_response(cmd: &str, args: &[&str], is_dry_run: bool) -> String {
    let args_vec = args.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
    let command_line = command_line(cmd, args);

    if !is_allowed_command(cmd) {
        return serde_json::json!({
            "ok": false,
            "type": "command",
            "error": "not_allowed",
            "message": format!("Command not allowed by AETHER allowlist: {cmd}"),
            "command": cmd,
            "args": args_vec,
            "command_line": command_line,
        })
        .to_string();
    }

    if is_dry_run {
        return serde_json::json!({
            "ok": true,
            "type": "command",
            "dry_run": true,
            "mock": false,
            "command": cmd,
            "args": args_vec,
            "command_line": command_line,
            "stdout": preview_command(cmd, args),
            "stderr": "",
            "exit_code": null,
        })
        .to_string();
    }

    if std::env::var("AETHER_MOCK_COMMANDS").ok().as_deref() == Some("1") {
        return serde_json::json!({
            "ok": true,
            "type": "command",
            "dry_run": false,
            "mock": true,
            "command": cmd,
            "args": args_vec,
            "command_line": command_line,
            "stdout": format!("MOCK: {command_line}"),
            "stderr": "",
            "exit_code": 0,
        })
        .to_string();
    }

    match Command::new(cmd).args(args).output() {
        Ok(o) => serde_json::json!({
            "ok": o.status.success(),
            "type": "command",
            "dry_run": false,
            "mock": false,
            "command": cmd,
            "args": args_vec,
            "command_line": command_line,
            "stdout": String::from_utf8_lossy(&o.stdout).to_string(),
            "stderr": String::from_utf8_lossy(&o.stderr).to_string(),
            "exit_code": o.status.code(),
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "ok": false,
            "type": "command",
            "dry_run": false,
            "mock": false,
            "command": cmd,
            "args": args_vec,
            "command_line": command_line,
            "error": "spawn_failed",
            "message": e.to_string(),
        })
        .to_string(),
    }
}

pub fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| format!("Cannot read {}", path))
}

pub fn p(params: &serde_json::Value, key: &str) -> String {
    crate::error::get_string_param(params, key, &ErrorContext::new("helpers", "p")).unwrap_or("").to_string()
}

pub fn check_force(params: &serde_json::Value, ctx: &ErrorContext) -> Result<(), AetherError> {
    require_force_or_dry_run(params, ctx)
}

pub fn check_gate(_gates: &FeatureGates, ctx: &ErrorContext, enabled: bool, gate: &str) -> Result<(), AetherError> {
    if !enabled {
        Err(AetherError::feature_disabled(ctx.clone(), gate))
    } else {
        Ok(())
    }
}
