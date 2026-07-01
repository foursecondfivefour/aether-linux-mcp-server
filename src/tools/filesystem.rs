//! file_system - File system operations.
use crate::audit;
use crate::config::FeatureGates;
use crate::error::{self, require_force, AetherError, ErrorContext};
use std::fs;
use std::process::Command;

pub fn handle(gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("file_system", action);
    let result = match action {
        "read" => {
            let path = ps(&params, "path", &ctx);
            fs::read_to_string(&path).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
        }
        "write" => force_check(&params, &ctx).and_then(|_| {
            let path = ps(&params, "path", &ctx);
            let content = ps(&params, "content", &ctx);
            fs::write(&path, &content)
                .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
                .map(|_| format!("Wrote {} bytes to {}", content.len(), path))
        }),
        "delete" => force_check(&params, &ctx).and_then(|_| {
            let path = ps(&params, "path", &ctx);
            fs::remove_file(&path)
                .or_else(|_| fs::remove_dir_all(&path))
                .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
                .map(|_| format!("Deleted {}", path))
        }),
        "list_dir" => Ok(run("ls", &["-la", &ps(&params, "path", &ctx)])),
        "stat" => {
            let path = ps(&params, "path", &ctx);
            fs::metadata(&path).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e }).map(|meta| {
                format!(
                    "path: {}\ntype: {}\nsize: {} bytes",
                    path,
                    if meta.is_dir() { "dir" } else { "file" },
                    meta.len()
                )
            })
        }
        "mkdir" => {
            let path = ps(&params, "path", &ctx);
            fs::create_dir_all(&path)
                .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
                .map(|_| format!("Created {}", path))
        }
        "exists" => Ok(format!("{}", std::path::Path::new(&ps(&params, "path", &ctx)).exists())),
        "mount_list" => Ok(run("mount", &[])),
        "mount" => force_check(&params, &ctx)
            .map(|_| run("mount", &[&ps(&params, "device", &ctx), &ps(&params, "target", &ctx)])),
        "umount" => force_check(&params, &ctx).map(|_| run("umount", &[&ps(&params, "path", &ctx)])),
        "disk_list" => Ok(run("lsblk", &["-o", "NAME,SIZE,TYPE,MOUNTPOINT"])),
        "part_list" => Ok(run("fdisk", &["-l"])),
        "part_create" => force_check(&params, &ctx)
            .and_then(|_| gates.check(ctx.clone(), gates.partition_edit, "AETHER_PARTITION_EDIT"))
            .map(|_| {
                run(
                    "parted",
                    &[
                        &ps(&params, "device", &ctx),
                        "mkpart",
                        "primary",
                        &ps(&params, "start", &ctx),
                        &ps(&params, "end", &ctx),
                    ],
                )
            }),
        "lvm_pvs" => Ok(run("pvs", &[])),
        "lvm_vgs" => Ok(run("vgs", &[])),
        "lvm_lvs" => Ok(run("lvs", &[])),
        "swap_list" => Ok(run("swapon", &["--show"])),
        "swap_on" => force_check(&params, &ctx).map(|_| run("swapon", &[&ps(&params, "device", &ctx)])),
        "zfs_list" => Ok(run("zfs", &["list"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("file_system", action, "ok"),
        Err(e) => audit::log_failure("file_system", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}

fn run(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
        .unwrap_or_else(|_| format!("'{}' not available", cmd))
}

fn ps(params: &serde_json::Value, key: &str, ctx: &ErrorContext) -> String {
    error::get_string_param(params, key, ctx).unwrap_or("").to_string()
}

fn force_check(params: &serde_json::Value, ctx: &ErrorContext) -> Result<(), AetherError> {
    if !require_force(params) {
        Err(AetherError::ForceRequired { ctx: ctx.clone() })
    } else {
        Ok(())
    }
}
