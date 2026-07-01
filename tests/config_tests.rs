//! Unit tests for Linux FeatureGates and configuration.

use aether_linux_mcp_server::config::FeatureGates;
use aether_linux_mcp_server::error::ErrorContext;

#[test]
fn default_all_gates_disabled() {
    let gates = FeatureGates::default();
    assert!(!gates.kexec_load, "KEXEC_LOAD must be disabled by default");
    assert!(!gates.module_load, "MODULE_LOAD must be disabled by default");
    assert!(!gates.bpf_load, "BPF_LOAD must be disabled by default");
    assert!(!gates.ptrace_attach, "PTRACE_ATTACH must be disabled by default");
    assert!(!gates.namespace_create, "NAMESPACE_CREATE must be disabled by default");
    assert!(!gates.offline_mount, "OFFLINE_MOUNT must be disabled by default");
    assert!(!gates.partition_edit, "PARTITION_EDIT must be disabled by default");
    assert!(!gates.token_manipulation, "TOKEN_MANIPULATION must be disabled by default");
}

#[test]
fn clone_preserves_gate_values() {
    let gates = FeatureGates {
        module_load: true,
        ptrace_attach: true,
        ..FeatureGates::default()
    };
    let cloned = gates.clone();
    assert!(cloned.module_load);
    assert!(cloned.ptrace_attach);
    assert!(!cloned.partition_edit);
}

#[test]
fn enabled_gate_check_returns_ok() {
    let gates = FeatureGates {
        partition_edit: true,
        ..FeatureGates::default()
    };
    let ctx = ErrorContext::new("file_system", "part_create");
    assert!(gates.check(ctx, gates.partition_edit, "AETHER_PARTITION_EDIT").is_ok());
}

#[test]
fn disabled_gate_check_returns_actionable_error() {
    let gates = FeatureGates::default();
    let ctx = ErrorContext::new("process_control", "ptrace_attach");
    let err = gates
        .check(ctx, gates.ptrace_attach, "AETHER_PTRACE_ATTACH")
        .unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("Feature disabled"));
    assert!(msg.contains("AETHER_PTRACE_ATTACH"));
    assert!(msg.contains(".env"));
    assert!(msg.contains("process_control::ptrace_attach"));
}

#[test]
fn load_all_gates_default_to_false_when_env_is_absent() {
    for key in [
        "AETHER_KEXEC_LOAD",
        "AETHER_MODULE_LOAD",
        "AETHER_BPF_LOAD",
        "AETHER_PTRACE_ATTACH",
        "AETHER_NAMESPACE_CREATE",
        "AETHER_OFFLINE_MOUNT",
        "AETHER_PARTITION_EDIT",
        "AETHER_TOKEN_MANIPULATION",
    ] {
        std::env::remove_var(key);
    }

    let gates = FeatureGates::load();
    assert!(!gates.kexec_load);
    assert!(!gates.module_load);
    assert!(!gates.bpf_load);
    assert!(!gates.ptrace_attach);
    assert!(!gates.namespace_create);
    assert!(!gates.offline_mount);
    assert!(!gates.partition_edit);
    assert!(!gates.token_manipulation);
}

#[test]
fn load_respects_env_value_one() {
    std::env::set_var("AETHER_MODULE_LOAD", "1");
    std::env::set_var("AETHER_BPF_LOAD", "0");

    let gates = FeatureGates::load();
    assert!(gates.module_load);
    assert!(!gates.bpf_load);

    std::env::remove_var("AETHER_MODULE_LOAD");
    std::env::remove_var("AETHER_BPF_LOAD");
}
