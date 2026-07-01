use crate::audit;
use crate::config::FeatureGates;
use crate::error::{self, AetherError, ErrorContext};
use crate::tools::helpers::{self, RiskLevel};
use std::fs;

fn run(params: &serde_json::Value, cmd: &str, args: &[&str]) -> String {
    helpers::cmd_params(params, cmd, args)
}

fn ps(params: &serde_json::Value, key: &str, ctx: &ErrorContext) -> String {
    error::get_string_param(params, key, ctx).unwrap_or("").to_string()
}

fn force_check(params: &serde_json::Value, ctx: &ErrorContext) -> Result<(), AetherError> {
    helpers::require_force_or_dry_run(params, ctx)
}

pub fn handle(gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("network_manager", action);
    let result = match action {
        "adapter_list" => Ok(run(&params, "ip", &["link", "show"])),
        "adapter_info" => Ok(run(&params, "ip", &["-details", "link", "show", &ps(&params, "interface", &ctx)])),
        "adapter_stats" => Ok(run(&params, "ip", &["-s", "link", "show", &ps(&params, "interface", &ctx)])),
        "adapter_speed" => Ok(run(&params, "ethtool", &[&ps(&params, "interface", &ctx)])),
        "adapter_up" => force_check(&params, &ctx)
            .map(|_| run(&params, "ip", &["link", "set", &ps(&params, "interface", &ctx), "up"])),
        "adapter_down" => force_check(&params, &ctx)
            .map(|_| run(&params, "ip", &["link", "set", &ps(&params, "interface", &ctx), "down"])),
        "address_list" => Ok(run(&params, "ip", &["addr", "show"])),
        "address_add" => force_check(&params, &ctx).map(|_| {
            run(&params, "ip", &["addr", "add", &ps(&params, "address", &ctx), "dev", &ps(&params, "interface", &ctx)])
        }),
        "address_delete" => force_check(&params, &ctx).map(|_| {
            run(&params, "ip", &["addr", "del", &ps(&params, "address", &ctx), "dev", &ps(&params, "interface", &ctx)])
        }),
        "route_list" => Ok(run(&params, "ip", &["route", "show"])),
        "route_add" => {
            force_check(&params, &ctx).map(|_| run(&params, "ip", &["route", "add", &ps(&params, "route", &ctx)]))
        }
        "route_delete" => {
            force_check(&params, &ctx).map(|_| run(&params, "ip", &["route", "del", &ps(&params, "route", &ctx)]))
        }
        "connection_list" => Ok(run(&params, "nmcli", &["connection", "show"])),
        "connection_info" => Ok(run(&params, "nmcli", &["connection", "show", &ps(&params, "name", &ctx)])),
        "connection_up" => {
            force_check(&params, &ctx).map(|_| run(&params, "nmcli", &["connection", "up", &ps(&params, "name", &ctx)]))
        }
        "connection_down" => force_check(&params, &ctx)
            .map(|_| run(&params, "nmcli", &["connection", "down", &ps(&params, "name", &ctx)])),
        "dns_servers" => Ok(run(&params, "resolvectl", &["status"])),
        "dns_flush" => force_check(&params, &ctx).map(|_| run(&params, "resolvectl", &["flush-caches"])),
        "firewall_rules_list" => Ok(run(&params, "nft", &["list", "ruleset"])),
        "firewall_rule_add" => {
            force_check(&params, &ctx).map(|_| run(&params, "nft", &["add", "rule", &ps(&params, "rule", &ctx)]))
        }
        "firewall_rule_delete" => {
            force_check(&params, &ctx).map(|_| run(&params, "nft", &["delete", "rule", &ps(&params, "rule", &ctx)]))
        }
        "nftables_table_list" => Ok(run(&params, "nft", &["list", "tables"])),
        "nftables_chain_list" => Ok(run(&params, "nft", &["list", "chains"])),
        "wireguard_list" => Ok(run(&params, "wg", &["show"])),
        "bridge_list" => Ok(run(&params, "bridge", &["link", "show"])),
        "vlan_list" => Ok(run(&params, "ip", &["-d", "link", "show", "type", "vlan"])),
        "network_namespace_list" => Ok(run(&params, "ip", &["netns", "list"])),
        "network_namespace_create" => force_check(&params, &ctx)
            .and_then(|_| gates.check(ctx.clone(), gates.namespace_create, "AETHER_NAMESPACE_CREATE"))
            .map(|_| run(&params, "ip", &["netns", "add", &ps(&params, "name", &ctx)])),
        "network_namespace_delete" => force_check(&params, &ctx)
            .and_then(|_| gates.check(ctx.clone(), gates.namespace_create, "AETHER_NAMESPACE_CREATE"))
            .map(|_| run(&params, "ip", &["netns", "delete", &ps(&params, "name", &ctx)])),
        "socket_list" => Ok(run(&params, "ss", &["-tulnp"])),
        "socket_info" => Ok(run(&params, "ss", &["-pane", &ps(&params, "filter", &ctx)])),
        "wifi_list" => Ok(run(&params, "nmcli", &["device", "wifi", "list"])),
        "wifi_scan" => Ok(run(&params, "nmcli", &["device", "wifi", "rescan"])),
        "bluetooth_list" => Ok(run(&params, "bluetoothctl", &["devices"])),
        "hosts_read" => Ok(fs::read_to_string("/etc/hosts").unwrap_or_default()),
        "hosts_write" => write_hosts(&params, &ctx),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("network_manager", action, "ok"),
        Err(e) => audit::log_failure("network_manager", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}

fn write_hosts(params: &serde_json::Value, ctx: &ErrorContext) -> Result<String, AetherError> {
    force_check(params, ctx)?;
    let content = ps(params, "content", ctx);
    if helpers::dry_run(params) {
        return Ok(helpers::preview_action(RiskLevel::Critical, "overwrite /etc/hosts"));
    }
    fs::write("/etc/hosts", content).map_err(|e| AetherError::Io { ctx: ctx.clone(), source: e })?;
    Ok("Wrote /etc/hosts".to_string())
}
