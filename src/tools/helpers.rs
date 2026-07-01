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
    "crontab",
    "dconf",
    "dmidecode",
    "dpkg",
    "env",
    "fdisk",
    "flatpak",
    "free",
    "getenforce",
    "getent",
    "groupadd",
    "groupdel",
    "hostnamectl",
    "id",
    "import",
    "ip",
    "journalctl",
    "kdump-config",
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
    "mokutil",
    "mount",
    "nft",
    "nmcli",
    "nvidia-smi",
    "pactl",
    "parted",
    "passwd",
    "ps",
    "pvs",
    "renice",
    "resolvectl",
    "rocm-smi",
    "sensors",
    "sestatus",
    "snap",
    "ss",
    "swapon",
    "systemctl",
    "systemd-analyze",
    "sysctl",
    "timedatectl",
    "ufw",
    "umount",
    "uname",
    "useradd",
    "userdel",
    "vgs",
    "w",
    "whoami",
    "wmctrl",
    "xclip",
    "xdotool",
    "xrandr",
    "zfs",
];

#[must_use]
pub fn is_allowed_command(cmd: &str) -> bool {
    ALLOWED_COMMANDS.contains(&cmd)
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
    format!("DRY-RUN [{}]: {}", risk.as_str(), detail.as_ref())
}

pub fn require_force_or_dry_run(params: &serde_json::Value, ctx: &ErrorContext) -> Result<(), AetherError> {
    if dry_run(params) || require_force(params) {
        Ok(())
    } else {
        Err(AetherError::force_required(ctx.clone()))
    }
}

pub fn cmd(cmd: &str, args: &[&str]) -> String {
    if !is_allowed_command(cmd) {
        return format!("Command not allowed by AETHER allowlist: {cmd}");
    }

    if std::env::var("AETHER_MOCK_COMMANDS").ok().as_deref() == Some("1") {
        return format!("MOCK: {}", command_line(cmd, args));
    }

    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
        .unwrap_or_else(|_| format!("'{}' not available", cmd))
}

pub fn cmd_params(params: &serde_json::Value, cmd: &str, args: &[&str]) -> String {
    if dry_run(params) {
        preview_command(cmd, args)
    } else {
        self::cmd(cmd, args)
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
