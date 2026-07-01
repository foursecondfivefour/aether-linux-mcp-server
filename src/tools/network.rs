use crate::audit;
use crate::config::FeatureGates;
use crate::error::{self, AetherError, ErrorContext};
use std::fs;

fn run(params: &serde_json::Value, cmd: &str, args: &[&str]) -> String {
    crate::tools::helpers::cmd_params(params, cmd, args)
}

fn ps(params: &serde_json::Value, key: &str, ctx: &ErrorContext) -> String {
    error::get_string_param(params, key, ctx).unwrap_or("").to_string()
}

fn force_check(params: &serde_json::Value, ctx: &ErrorContext) -> Result<(), AetherError> {
    crate::tools::helpers::require_force_or_dry_run(params, ctx)
}

pub fn handle(gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("network_manager", action);
    let result = match action {
        "adapter_list" => Ok(run(&params, "ip", &["link", "show"])),
        "adapter_up" => force_check(&params, &ctx)
            .map(|_| run(&params, "ip", &["link", "set", &ps(&params, "interface", &ctx), "up"])),
        "adapter_down" => force_check(&params, &ctx)
            .map(|_| run(&params, "ip", &["link", "set", &ps(&params, "interface", &ctx), "down"])),
        "address_list" => Ok(run(&params, "ip", &["addr", "show"])),
        "route_list" => Ok(run(&params, "ip", &["route", "show"])),
        "route_add" => {
            force_check(&params, &ctx).map(|_| run(&params, "ip", &["route", "add", &ps(&params, "route", &ctx)]))
        }
        "route_delete" => {
            force_check(&params, &ctx).map(|_| run(&params, "ip", &["route", "del", &ps(&params, "route", &ctx)]))
        }
        "connection_list" => Ok(run(&params, "nmcli", &["connection", "show"])),
        "dns_servers" => Ok(run(&params, "resolvectl", &["status"])),
        "firewall_rules_list" => Ok(run(&params, "nft", &["list", "ruleset"])),
        "socket_list" => Ok(run(&params, "ss", &["-tulnp"])),
        "wifi_list" => Ok(run(&params, "nmcli", &["device", "wifi", "list"])),
        "bluetooth_list" => Ok(run(&params, "bluetoothctl", &["devices"])),
        "hosts_read" => Ok(fs::read_to_string("/etc/hosts").unwrap_or_default()),
        "network_namespace_create" => force_check(&params, &ctx)
            .and_then(|_| gates.check(ctx.clone(), gates.namespace_create, "AETHER_NAMESPACE_CREATE"))
            .map(|_| run(&params, "ip", &["netns", "add", &ps(&params, "name", &ctx)])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("network_manager", action, "ok"),
        Err(e) => audit::log_failure("network_manager", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
