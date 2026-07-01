//! Mocked tool-action tests.
//!
//! These tests set `AETHER_MOCK_COMMANDS=1`, so command-backed handlers return
//! a preview instead of touching the host. Destructive paths use `dry_run` to
//! verify safety gates without mutating the system.

use aether_linux_mcp_server::config::FeatureGates;
use aether_linux_mcp_server::tools;
use serde_json::json;
use std::sync::{Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_mock_commands<T>(f: impl FnOnce() -> T) -> T {
    let _guard = env_lock().lock().unwrap();
    std::env::set_var("AETHER_MOCK_COMMANDS", "1");
    let out = f();
    std::env::remove_var("AETHER_MOCK_COMMANDS");
    out
}

#[test]
fn safe_command_actions_are_mocked_not_executed() {
    with_mock_commands(|| {
        let gates = FeatureGates::default();

        let cases = [
            tools::packages::handle("search", json!({ "package": "bash" })),
            tools::service::handle("status", json!({ "service": "ssh" })),
            tools::filesystem::handle(&gates, "disk_list", json!({})),
            tools::network::handle(&gates, "route_list", json!({})),
            tools::gui::handle(&gates, "display_list", json!({})),
            tools::hardware::handle(&gates, "usb_devices", json!({})),
        ];

        for output in cases {
            assert!(output.starts_with("MOCK:"), "expected mocked command output, got: {output}");
        }
    });
}

#[test]
fn destructive_actions_without_force_are_blocked() {
    with_mock_commands(|| {
        let gates = FeatureGates::default();

        let cases = [
            tools::packages::handle("install", json!({ "package": "curl" })),
            tools::service::handle("stop", json!({ "service": "ssh" })),
            tools::filesystem::handle(&gates, "delete", json!({ "path": "/tmp/aether-nope" })),
            tools::network::handle(&gates, "adapter_down", json!({ "interface": "eth0" })),
        ];

        for output in cases {
            assert!(output.contains("Force required"), "expected force guard, got: {output}");
        }
    });
}

#[test]
fn dry_run_allows_preview_without_force_or_mutation() {
    with_mock_commands(|| {
        let gates = FeatureGates::default();

        let cases = [
            tools::packages::handle("install", json!({ "package": "curl", "dry_run": true })),
            tools::service::handle("restart", json!({ "service": "ssh", "dry_run": true })),
            tools::filesystem::handle(&gates, "delete", json!({ "path": "/tmp/aether-nope", "dry_run": true })),
            tools::network::handle(&gates, "adapter_down", json!({ "interface": "eth0", "dry_run": true })),
        ];

        for output in cases {
            assert!(output.contains("DRY-RUN"), "expected dry-run preview, got: {output}");
        }
    });
}

#[test]
fn critical_gated_actions_still_require_enabled_gate_even_with_force() {
    with_mock_commands(|| {
        let gates = FeatureGates::default();

        let cases = [
            tools::filesystem::handle(
                &gates,
                "part_create",
                json!({ "device": "/dev/sdz", "start": "1MiB", "end": "2MiB", "force": true }),
            ),
            tools::sysconfig::handle(&gates, "modprobe_load", json!({ "module": "dummy", "force": true })),
            tools::user::handle(&gates, "user_create", json!({ "username": "aether-test", "force": true })),
        ];

        for output in cases {
            assert!(output.contains("Feature disabled"), "expected feature gate block, got: {output}");
        }
    });
}

#[tokio::test]
async fn process_destructive_actions_are_force_guarded_and_dry_runnable() {
    let _guard = env_lock().lock().unwrap();
    std::env::set_var("AETHER_MOCK_COMMANDS", "1");

    let gates = FeatureGates::default();
    let blocked = tools::process::handle(&gates, "kill", json!({ "pid": "1" })).await;
    assert!(blocked.contains("Force required"), "expected force guard, got: {blocked}");

    let dry = tools::process::handle(&gates, "kill", json!({ "pid": "1", "dry_run": true })).await;
    assert!(dry.contains("DRY-RUN"), "expected dry-run preview, got: {dry}");

    std::env::remove_var("AETHER_MOCK_COMMANDS");
}
