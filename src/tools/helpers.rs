use crate::config::FeatureGates;
use crate::error::{AetherError, ErrorContext};
use std::fs;
use std::process::Command;

pub fn cmd(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd).args(args).output()
        .map(|o| format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
        .unwrap_or_else(|_| format!("'{}' not available", cmd))
}

pub fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| format!("Cannot read {}", path))
}

pub fn p(params: &serde_json::Value, key: &str) -> String {
    crate::error::get_string_param(params, key, &ErrorContext::new("helpers", "p")).unwrap_or("").to_string()
}

pub fn check_force(_params: &serde_json::Value) {
    // Simplified noop
}

pub fn check_gate(_gates: &FeatureGates, ctx: &ErrorContext, enabled: bool, gate: &str) -> Result<(), AetherError> {
    if !enabled {
        Err(AetherError::feature_disabled(ctx.clone(), gate))
    } else {
        Ok(())
    }
}
