//! file_system - File system operations.
use crate::audit;
use crate::config::FeatureGates;
use crate::error::{self, AetherError, ErrorContext};
use crate::tools::helpers::{self, RiskLevel};
use std::fs;
use std::io::Write;

pub fn handle(gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("file_system", action);
    let result = match action {
        "read" => read_file(&params, &ctx),
        "write" => write_file(&params, &ctx),
        "append" => append_file(&params, &ctx),
        "delete" => delete_path(&params, &ctx),
        "copy" => copy_path(&params, &ctx),
        "move" | "rename" => move_path(&params, &ctx),
        "list_dir" => Ok(run(&params, "ls", &["-la", &ps(&params, "path", &ctx)])),
        "stat" => stat_path(&params, &ctx),
        "statvfs" => Ok(run(&params, "df", &["-T", &ps(&params, "path", &ctx)])),
        "mkdir" => mkdir(&params, &ctx),
        "exists" => Ok(format!("{}", std::path::Path::new(&ps(&params, "path", &ctx)).exists())),
        "chmod" => force_check(&params, &ctx)
            .map(|_| run(&params, "chmod", &[&ps(&params, "mode", &ctx), &ps(&params, "path", &ctx)])),
        "chown" => force_check(&params, &ctx)
            .map(|_| run(&params, "chown", &[&ps(&params, "owner", &ctx), &ps(&params, "path", &ctx)])),
        "truncate" => force_check(&params, &ctx)
            .map(|_| run(&params, "truncate", &["-s", &ps(&params, "size", &ctx), &ps(&params, "path", &ctx)])),
        "symlink_read" => symlink_read(&params, &ctx),
        "symlink_create" => force_check(&params, &ctx)
            .map(|_| run(&params, "ln", &["-s", &ps(&params, "target", &ctx), &ps(&params, "link", &ctx)])),
        "symlink_delete" => delete_path(&params, &ctx),
        "acl_get" => Ok(run(&params, "getfacl", &[&ps(&params, "path", &ctx)])),
        "acl_set" => force_check(&params, &ctx)
            .map(|_| run(&params, "setfacl", &["-m", &ps(&params, "acl", &ctx), &ps(&params, "path", &ctx)])),
        "xattr_list" => xattr_list(&params, &ctx),
        "xattr_get" => xattr_get(&params, &ctx),
        "xattr_set" => xattr_set(&params, &ctx),
        "xattr_delete" => xattr_delete(&params, &ctx),
        "mount_list" => Ok(run(&params, "mount", &[])),
        "mount" => force_check(&params, &ctx)
            .map(|_| run(&params, "mount", &[&ps(&params, "device", &ctx), &ps(&params, "target", &ctx)])),
        "umount" => force_check(&params, &ctx).map(|_| run(&params, "umount", &[&ps(&params, "path", &ctx)])),
        "disk_list" => Ok(run(&params, "lsblk", &["-o", "NAME,SIZE,TYPE,MOUNTPOINT,FSTYPE,UUID"])),
        "disk_info" => Ok(run(&params, "lsblk", &["-f", &ps(&params, "device", &ctx)])),
        "part_list" => Ok(run(&params, "fdisk", &["-l"])),
        "part_create" => force_check(&params, &ctx)
            .and_then(|_| gates.check(ctx.clone(), gates.partition_edit, "AETHER_PARTITION_EDIT"))
            .map(|_| {
                run(
                    &params,
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
        "lvm_pvs" => Ok(run(&params, "pvs", &[])),
        "lvm_vgs" => Ok(run(&params, "vgs", &[])),
        "lvm_lvs" => Ok(run(&params, "lvs", &[])),
        "swap_list" => Ok(run(&params, "swapon", &["--show"])),
        "swap_on" => force_check(&params, &ctx).map(|_| run(&params, "swapon", &[&ps(&params, "device", &ctx)])),
        "zfs_list" => Ok(run(&params, "zfs", &["list"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("file_system", action, "ok"),
        Err(e) => audit::log_failure("file_system", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
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

fn read_file(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    let path = ps(params, "path", ctx);
    fs::read_to_string(&path).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
}

fn write_file(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let path = ps(params, "path", ctx);
    let content = ps(params, "content", ctx);
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Dangerous, format!("write {} bytes to {}", content.len(), path)));
    }
    fs::write(&path, &content)
        .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
        .map(|_| format!("Wrote {} bytes to {}", content.len(), path))
}

fn append_file(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let path = ps(params, "path", ctx);
    let content = ps(params, "content", ctx);
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(
            RiskLevel::Dangerous,
            format!("append {} bytes to {}", content.len(), path),
        ));
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })?;
    file.write_all(content.as_bytes()).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })?;
    Ok(format!("Appended {} bytes to {}", content.len(), path))
}

fn delete_path(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let path = ps(params, "path", ctx);
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Dangerous, format!("delete {}", path)));
    }
    fs::remove_file(&path)
        .or_else(|_| fs::remove_dir_all(&path))
        .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
        .map(|_| format!("Deleted {}", path))
}

fn copy_path(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let source = ps(params, "source", ctx);
    let dest = ps(params, "dest", ctx);
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Moderate, format!("copy {} to {}", source, dest)));
    }
    fs::copy(&source, &dest)
        .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
        .map(|bytes| format!("Copied {} bytes from {} to {}", bytes, source, dest))
}

fn move_path(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let source = ps(params, "source", ctx);
    let dest = ps(params, "dest", ctx);
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Moderate, format!("move {} to {}", source, dest)));
    }
    fs::rename(&source, &dest)
        .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
        .map(|_| format!("Moved {} to {}", source, dest))
}

fn mkdir(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    let path = ps(params, "path", ctx);
    if helpers::dry_run(params) {
        Ok(helpers::preview_action(RiskLevel::Moderate, format!("create directory {}", path)))
    } else {
        fs::create_dir_all(&path)
            .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
            .map(|_| format!("Created {}", path))
    }
}

fn stat_path(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    let path = ps(params, "path", ctx);
    fs::metadata(&path).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e }).map(|meta| {
        format!(
            "path: {}\ntype: {}\nsize: {} bytes\nreadonly: {}",
            path,
            if meta.is_dir() { "dir" } else { "file" },
            meta.len(),
            meta.permissions().readonly()
        )
    })
}

fn symlink_read(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    let path = ps(params, "path", ctx);
    fs::read_link(&path).map(|p| p.display().to_string()).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
}

fn xattr_list(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    let path = ps(params, "path", ctx);
    xattr::list(&path)
        .map(|items| items.map(|s| s.to_string_lossy().into_owned()).collect::<Vec<_>>().join("\n"))
        .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
}

fn xattr_get(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    let path = ps(params, "path", ctx);
    let name = ps(params, "name", ctx);
    xattr::get(&path, &name)
        .map(|value| value.map(|v| String::from_utf8_lossy(&v).into_owned()).unwrap_or_default())
        .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
}

fn xattr_set(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let path = ps(params, "path", ctx);
    let name = ps(params, "name", ctx);
    let value = ps(params, "value", ctx);
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Moderate, format!("set xattr {} on {}", name, path)));
    }
    xattr::set(&path, &name, value.as_bytes())
        .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
        .map(|_| format!("Set xattr {} on {}", name, path))
}

fn xattr_delete(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let path = ps(params, "path", ctx);
    let name = ps(params, "name", ctx);
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Moderate, format!("delete xattr {} on {}", name, path)));
    }
    xattr::remove(&path, &name)
        .map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })
        .map(|_| format!("Deleted xattr {} on {}", name, path))
}
