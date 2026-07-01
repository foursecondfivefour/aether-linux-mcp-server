//! Tests for the central action registry/spec.

use aether_linux_mcp_server::actions::{get, registry_json, ACTION_SPECS};
use std::collections::HashSet;

#[test]
fn action_specs_have_no_duplicate_tool_action_pairs() {
    let mut seen = HashSet::new();
    for spec in ACTION_SPECS {
        assert!(seen.insert((spec.tool, spec.action)), "duplicate action spec: {}::{}", spec.tool, spec.action);
    }
}

#[test]
fn action_registry_covers_all_current_implemented_actions() {
    // Keep this number honest: it is generated from implemented match arms, not the marketing table.
    assert!(ACTION_SPECS.len() >= 292, "registry shrank unexpectedly: {}", ACTION_SPECS.len());

    for (tool, action) in [
        ("process_control", "kill"),
        ("file_system", "xattr_set"),
        ("package_manager", "install"),
        ("system_config", "modprobe_load"),
        ("service_manager", "restart"),
        ("gui_automation", "display_list"),
        ("system_info", "uptime"),
        ("network_manager", "hosts_write"),
        ("user_management", "ssh_authorized_keys_add"),
        ("security_audit", "kernel_hardening"),
        ("hardware_control", "hugepages_alloc"),
        ("system_automation", "sysctl_d_apply"),
    ] {
        assert!(get(tool, action).is_some(), "missing spec for {tool}::{action}");
    }
}

#[test]
fn dangerous_and_critical_actions_have_safety_metadata() {
    for spec in ACTION_SPECS {
        if matches!(spec.risk, "dangerous" | "critical") {
            assert!(
                spec.requires_force || spec.feature_gate.is_some() || spec.supports_dry_run,
                "{}::{} is {} but has no safety metadata",
                spec.tool,
                spec.action,
                spec.risk
            );
        }
    }
}

#[test]
fn specific_feature_gates_are_declared() {
    assert_eq!(get("file_system", "part_create").unwrap().feature_gate, Some("AETHER_PARTITION_EDIT"));
    assert_eq!(get("system_config", "modprobe_load").unwrap().feature_gate, Some("AETHER_MODULE_LOAD"));
    assert_eq!(
        get("network_manager", "network_namespace_create").unwrap().feature_gate,
        Some("AETHER_NAMESPACE_CREATE")
    );
    assert_eq!(get("user_management", "user_create").unwrap().feature_gate, Some("AETHER_TOKEN_MANIPULATION"));
}

#[test]
fn registry_json_is_machine_readable() {
    let json = registry_json();
    assert_eq!(json["count"].as_u64(), Some(ACTION_SPECS.len() as u64));
    assert!(json["actions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| { v["tool"] == "service_manager" && v["action"] == "restart" && v["requires_force"] == true }));
}
