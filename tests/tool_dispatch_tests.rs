//! Unit tests for Linux tool-dispatch invariants without invoking system-changing actions.

use aether_linux_mcp_server::config::FeatureGates;
use aether_linux_mcp_server::error::{AetherError, ErrorContext};

#[test]
fn all_twelve_tools_are_represented_in_error_contexts() {
    let tool_names = [
        "process_control",
        "file_system",
        "package_manager",
        "system_config",
        "service_manager",
        "gui_automation",
        "system_info",
        "network_manager",
        "user_management",
        "security_audit",
        "hardware_control",
        "system_automation",
    ];

    for tool in tool_names {
        let ctx = ErrorContext::new(tool, "unknown_action");
        let err = AetherError::invalid_param(ctx, "action", "unknown action");
        let msg = format!("{err}");
        assert!(msg.contains(tool), "{tool} must be present in: {msg}");
        assert!(msg.contains("unknown_action"), "action must be present in: {msg}");
    }
}

#[test]
fn dangerous_linux_gates_are_blocked_by_default() {
    let gates = FeatureGates::default();
    let cases = [
        ("system_config", "kexec_load", gates.kexec_load, "AETHER_KEXEC_LOAD"),
        ("system_config", "modprobe_load", gates.module_load, "AETHER_MODULE_LOAD"),
        ("security_audit", "bpf_load", gates.bpf_load, "AETHER_BPF_LOAD"),
        ("process_control", "ptrace_attach", gates.ptrace_attach, "AETHER_PTRACE_ATTACH"),
        ("process_control", "namespace_enter", gates.namespace_create, "AETHER_NAMESPACE_CREATE"),
        ("file_system", "mount", gates.offline_mount, "AETHER_OFFLINE_MOUNT"),
        ("file_system", "part_create", gates.partition_edit, "AETHER_PARTITION_EDIT"),
        ("user_management", "capability_set", gates.token_manipulation, "AETHER_TOKEN_MANIPULATION"),
    ];

    for (tool, action, enabled, gate) in cases {
        assert!(!enabled, "{gate} must be disabled by default");
        let err = gates.check(ErrorContext::new(tool, action), enabled, gate).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains(gate), "{gate} must be named in: {msg}");
        assert!(msg.contains(tool), "{tool} must be named in: {msg}");
    }
}

#[test]
fn destructive_actions_should_use_force_required_error_shape() {
    let examples = [
        ("process_control", "kill"),
        ("file_system", "delete"),
        ("package_manager", "remove"),
        ("service_manager", "stop"),
        ("network_manager", "route_delete"),
        ("user_management", "user_delete"),
        ("system_automation", "timer_delete"),
    ];

    for (tool, action) in examples {
        let err = AetherError::force_required(ErrorContext::new(tool, action));
        let msg = format!("{err}");
        assert!(msg.contains("force: true"));
        assert!(msg.contains(tool));
        assert!(msg.contains(action));
    }
}

#[test]
fn display_errors_do_not_leak_debug_structs() {
    let ctx = ErrorContext::new("test", "test");
    let cases = [
        format!("{}", AetherError::invalid_param(ctx.clone(), "param", "missing")),
        format!("{}", AetherError::permission_denied(ctx.clone(), "reason")),
        format!("{}", AetherError::not_found(ctx.clone(), "thing")),
        format!("{}", AetherError::feature_disabled(ctx.clone(), "GATE")),
        format!("{}", AetherError::operation_failed(ctx.clone(), "failed")),
        format!("{}", AetherError::system_error(ctx.clone(), "errno")),
        format!("{}", AetherError::not_implemented(ctx, "future feature")),
    ];

    for msg in cases {
        assert!(!msg.contains("AetherError::"), "enum debug leaked: {msg}");
        assert!(!msg.contains("ErrorContext {"), "struct debug leaked: {msg}");
        assert!(!msg.contains("panicked"), "panic text leaked: {msg}");
    }
}
