use crate::audit;
use crate::error::{self, AetherError, ErrorContext};
use crate::tools::helpers;
use std::fs;

#[derive(Debug, Clone, Copy)]
enum PackageManager {
    Apt,
    Dnf,
    Pacman,
    Zypper,
}

impl PackageManager {
    fn detect(params: &serde_json::Value) -> Self {
        match params.get("manager").and_then(|v| v.as_str()) {
            Some("dnf") => Self::Dnf,
            Some("pacman") => Self::Pacman,
            Some("zypper") => Self::Zypper,
            Some("apt") => Self::Apt,
            _ if helpers::command_exists("apt") => Self::Apt,
            _ if helpers::command_exists("dnf") => Self::Dnf,
            _ if helpers::command_exists("pacman") => Self::Pacman,
            _ if helpers::command_exists("zypper") => Self::Zypper,
            _ => Self::Apt,
        }
    }
}

fn run(params: &serde_json::Value, cmd: &str, args: &[&str]) -> String {
    helpers::cmd_params(params, cmd, args)
}

fn ps(params: &serde_json::Value, key: &str, ctx: &ErrorContext) -> String {
    error::get_string_param(params, key, ctx).unwrap_or("").to_string()
}

fn force_check(params: &serde_json::Value, ctx: &ErrorContext) -> Result<(), AetherError> {
    helpers::require_force_or_dry_run(params, ctx)
}

pub fn handle(action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("package_manager", action);
    let pm = PackageManager::detect(&params);
    let result = match action {
        "list_installed" => Ok(match pm {
            PackageManager::Apt => run(&params, "dpkg", &["-l"]),
            PackageManager::Dnf => run(&params, "dnf", &["list", "installed"]),
            PackageManager::Pacman => run(&params, "pacman", &["-Q"]),
            PackageManager::Zypper => run(&params, "zypper", &["se", "--installed-only"]),
        }),
        "list_upgradable" => Ok(match pm {
            PackageManager::Apt => run(&params, "apt", &["list", "--upgradable"]),
            PackageManager::Dnf => run(&params, "dnf", &["check-update"]),
            PackageManager::Pacman => run(&params, "pacman", &["-Qu"]),
            PackageManager::Zypper => run(&params, "zypper", &["list-updates"]),
        }),
        "search" => {
            let pkg = ps(&params, "package", &ctx);
            Ok(match pm {
                PackageManager::Apt => run(&params, "apt-cache", &["search", &pkg]),
                PackageManager::Dnf => run(&params, "dnf", &["search", &pkg]),
                PackageManager::Pacman => run(&params, "pacman", &["-Ss", &pkg]),
                PackageManager::Zypper => run(&params, "zypper", &["search", &pkg]),
            })
        }
        "info" => {
            let pkg = ps(&params, "package", &ctx);
            Ok(match pm {
                PackageManager::Apt => run(&params, "apt-cache", &["show", &pkg]),
                PackageManager::Dnf => run(&params, "dnf", &["info", &pkg]),
                PackageManager::Pacman => run(&params, "pacman", &["-Si", &pkg]),
                PackageManager::Zypper => run(&params, "zypper", &["info", &pkg]),
            })
        }
        "install" => force_check(&params, &ctx).map(|_| {
            let pkg = ps(&params, "package", &ctx);
            match pm {
                PackageManager::Apt => run(&params, "apt", &["install", "-y", &pkg]),
                PackageManager::Dnf => run(&params, "dnf", &["install", "-y", &pkg]),
                PackageManager::Pacman => run(&params, "pacman", &["-S", "--noconfirm", &pkg]),
                PackageManager::Zypper => run(&params, "zypper", &["--non-interactive", "install", &pkg]),
            }
        }),
        "remove" => force_check(&params, &ctx).map(|_| {
            let pkg = ps(&params, "package", &ctx);
            match pm {
                PackageManager::Apt => run(&params, "apt", &["remove", "-y", &pkg]),
                PackageManager::Dnf => run(&params, "dnf", &["remove", "-y", &pkg]),
                PackageManager::Pacman => run(&params, "pacman", &["-R", "--noconfirm", &pkg]),
                PackageManager::Zypper => run(&params, "zypper", &["--non-interactive", "remove", &pkg]),
            }
        }),
        "upgrade_all" => force_check(&params, &ctx).map(|_| match pm {
            PackageManager::Apt => run(&params, "apt", &["upgrade", "-y"]),
            PackageManager::Dnf => run(&params, "dnf", &["upgrade", "-y"]),
            PackageManager::Pacman => run(&params, "pacman", &["-Syu", "--noconfirm"]),
            PackageManager::Zypper => run(&params, "zypper", &["--non-interactive", "update"]),
        }),
        "clean_cache" => force_check(&params, &ctx).map(|_| match pm {
            PackageManager::Apt => run(&params, "apt", &["clean"]),
            PackageManager::Dnf => run(&params, "dnf", &["clean", "all"]),
            PackageManager::Pacman => run(&params, "pacman", &["-Sc", "--noconfirm"]),
            PackageManager::Zypper => run(&params, "zypper", &["clean", "--all"]),
        }),
        "list_repos" => Ok(match pm {
            PackageManager::Apt => fs::read_to_string("/etc/apt/sources.list").unwrap_or_default(),
            PackageManager::Dnf => run(&params, "dnf", &["repolist"]),
            PackageManager::Pacman => fs::read_to_string("/etc/pacman.conf").unwrap_or_default(),
            PackageManager::Zypper => run(&params, "zypper", &["repos"]),
        }),
        "verify" => Ok(match pm {
            PackageManager::Apt => run(&params, "dpkg", &["-V"]),
            PackageManager::Dnf => run(&params, "rpm", &["-Va"]),
            PackageManager::Pacman => run(&params, "pacman", &["-Qk"]),
            PackageManager::Zypper => run(&params, "rpm", &["-Va"]),
        }),
        "history" => Ok(fs::read_to_string("/var/log/apt/history.log").unwrap_or_default()),
        "download" => {
            let pkg = ps(&params, "package", &ctx);
            Ok(match pm {
                PackageManager::Apt => run(&params, "apt", &["download", &pkg]),
                PackageManager::Dnf => run(&params, "dnf", &["download", &pkg]),
                PackageManager::Pacman => run(&params, "pacman", &["-Sw", "--noconfirm", &pkg]),
                PackageManager::Zypper => run(&params, "zypper", &["download", &pkg]),
            })
        }
        "flatpak_list" => Ok(run(&params, "flatpak", &["list"])),
        "flatpak_install" => force_check(&params, &ctx)
            .map(|_| run(&params, "flatpak", &["install", "-y", &ps(&params, "package", &ctx)])),
        "flatpak_remove" => force_check(&params, &ctx)
            .map(|_| run(&params, "flatpak", &["uninstall", "-y", &ps(&params, "package", &ctx)])),
        "snap_list" => Ok(run(&params, "snap", &["list"])),
        "snap_install" => {
            force_check(&params, &ctx).map(|_| run(&params, "snap", &["install", &ps(&params, "package", &ctx)]))
        }
        "snap_remove" => {
            force_check(&params, &ctx).map(|_| run(&params, "snap", &["remove", &ps(&params, "package", &ctx)]))
        }
        "nix_list" => Ok(run(&params, "nix", &["profile", "list"])),
        "nix_install" => force_check(&params, &ctx)
            .map(|_| run(&params, "nix", &["profile", "install", &ps(&params, "package", &ctx)])),
        "nix_remove" => force_check(&params, &ctx)
            .map(|_| run(&params, "nix", &["profile", "remove", &ps(&params, "package", &ctx)])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("package_manager", action, "ok"),
        Err(e) => audit::log_failure("package_manager", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
