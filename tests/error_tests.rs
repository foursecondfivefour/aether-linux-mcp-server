//! Unit tests for Linux error handling — pure Rust, no system mutation.

use aether_linux_mcp_server::error::{
    get_opt_string_param, get_string_param, require_force, AetherError, ErrorContext,
};
use serde_json::json;

#[test]
fn error_context_display_uses_tool_and_action() {
    let ctx = ErrorContext::new("file_system", "read");
    assert_eq!(ctx.to_string(), "file_system::read");
}

#[test]
fn invalid_param_mentions_context_param_and_message() {
    let ctx = ErrorContext::new("process_control", "kill");
    let err = AetherError::invalid_param(ctx, "pid", "required parameter is missing");
    let msg = format!("{err}");
    assert!(msg.contains("Invalid parameter"));
    assert!(msg.contains("process_control::kill"));
    assert!(msg.contains("pid"));
    assert!(msg.contains("required parameter is missing"));
}

#[test]
fn force_required_message_is_clear() {
    let err = AetherError::force_required(ErrorContext::new("service_manager", "stop"));
    let msg = format!("{err}");
    assert!(msg.contains("Force required"));
    assert!(msg.contains("force: true"));
    assert!(msg.contains("service_manager::stop"));
}

#[test]
fn feature_disabled_message_names_gate() {
    let err = AetherError::feature_disabled(ErrorContext::new("hardware_control", "pci_rescan"), "AETHER_MODULE_LOAD");
    let msg = format!("{err}");
    assert!(msg.contains("Feature disabled"));
    assert!(msg.contains("AETHER_MODULE_LOAD"));
    assert!(msg.contains(".env"));
}

#[test]
fn error_ctx_accessor_returns_original_context() {
    let err = AetherError::not_found(ErrorContext::new("file_system", "stat"), "/missing");
    assert_eq!(err.ctx().tool, "file_system");
    assert_eq!(err.ctx().action, "stat");
}

#[test]
fn require_force_reads_boolean_force_field() {
    assert!(require_force(&json!({ "force": true })));
    assert!(!require_force(&json!({ "force": false })));
    assert!(!require_force(&json!({})));
    assert!(!require_force(&json!({ "force": "true" })));
}

#[test]
fn get_string_param_returns_value_when_present() {
    let ctx = ErrorContext::new("network_manager", "address_add");
    let params = json!({ "iface": "eth0" });
    assert_eq!(get_string_param(&params, "iface", &ctx).unwrap(), "eth0");
}

#[test]
fn get_string_param_errors_when_missing_or_wrong_type() {
    let ctx = ErrorContext::new("network_manager", "address_add");

    let missing = get_string_param(&json!({}), "iface", &ctx).unwrap_err();
    assert!(format!("{missing}").contains("required parameter is missing"));

    let wrong_type = get_string_param(&json!({ "iface": 1 }), "iface", &ctx).unwrap_err();
    assert!(format!("{wrong_type}").contains("must be a string"));
}

#[test]
fn get_opt_string_param_returns_optional_string_only() {
    let params = json!({ "name": "sshd", "count": 2 });
    assert_eq!(get_opt_string_param(&params, "name"), Some("sshd"));
    assert_eq!(get_opt_string_param(&params, "count"), None);
    assert_eq!(get_opt_string_param(&params, "missing"), None);
}
