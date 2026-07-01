use crate::audit;
use crate::config::FeatureGates;
use crate::error::{self, require_force, AetherError, ErrorContext};
use std::fs;
use std::process::Command;

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

pub fn handle(gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("network_manager", action);
    let result = match action {
        "adapter_list" => Ok(run("ip", &["link", "show"])),
        "adapter_up" => Ok(run("ip", &["link", "set", &ps(&params, "interface", &ctx), "up"])),
        "adapter_down" => Ok(run("ip", &["link", "set", &ps(&params, "interface", &ctx), "down"])),
        "address_list" => Ok(run("ip", &["addr", "show"])),
        "route_list" => Ok(run("ip", &["route", "show"])),
        "route_add" => force_check(&params, &ctx).map(|_| run("ip", &["route", "add", &ps(&params, "route", &ctx)])),
        "route_delete" => force_check(&params, &ctx).map(|_| run("ip", &["route", "del", &ps(&params, "route", &ctx)])),
        "connection_list" => Ok(run("nmcli", &["connection", "show"])),
        "dns_servers" => Ok(run("resolvectl", &["status"])),
        "firewall_rules_list" => Ok(run("nft", &["list", "ruleset"])),
        "socket_list" => Ok(run("ss", &["-tulnp"])),
        "wifi_list" => Ok(run("nmcli", &["device", "wifi", "list"])),
        "bluetooth_list" => Ok(run("bluetoothctl", &["devices"])),
        "hosts_read" => Ok(fs::read_to_string("/etc/hosts").unwrap_or_default()),
        "network_namespace_create" => force_check(&params, &ctx)
            .and_then(|_| gates.check(ctx.clone(), gates.namespace_create, "AETHER_NAMESPACE_CREATE"))
            .map(|_| run("ip", &["netns", "add", &ps(&params, "name", &ctx)])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("network_manager", action, "ok"),
        Err(e) => audit::log_failure("network_manager", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
