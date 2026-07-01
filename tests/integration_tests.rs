//! Lightweight integration smoke tests for public API surface.

use aether_linux_mcp_server::config::FeatureGates;
use aether_linux_mcp_server::server::{ActionParams, AetherServer};
use serde_json::json;

#[test]
fn server_can_be_constructed_with_default_gates() {
    let server = AetherServer::new(FeatureGates::default());
    assert!(!server.gates.module_load);
    assert!(!server.gates.partition_edit);
}

#[test]
fn action_params_deserializes_with_default_params() {
    let params: ActionParams = serde_json::from_value(json!({ "action": "cpu_info" })).unwrap();
    assert_eq!(params.action, "cpu_info");
    assert_eq!(params.params, serde_json::Value::Null);
}

#[test]
fn action_params_deserializes_with_object_params() {
    let params: ActionParams = serde_json::from_value(json!({
        "action": "read",
        "params": { "path": "/etc/os-release" }
    }))
    .unwrap();

    assert_eq!(params.action, "read");
    assert_eq!(params.params["path"], "/etc/os-release");
}
