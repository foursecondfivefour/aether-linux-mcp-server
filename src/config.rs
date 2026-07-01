//! Feature gates loaded from `.env` at startup.
//! All dangerous capabilities default to **disabled** (false).
//! The administrator enables them explicitly via the `.env` file.

use std::env;

use crate::error::{AetherError, ErrorContext};

/// Central feature gate configuration loaded from environment variables.
/// Each gate controls a dangerous or sensitive system operation.
#[derive(Debug, Clone, Default)]
pub struct FeatureGates {
    /// Load new kernel via kexec
    pub kexec_load: bool,
    /// Load/unload kernel modules
    pub module_load: bool,
    /// Load BPF programs
    pub bpf_load: bool,
    /// ptrace attach to non-child processes
    pub ptrace_attach: bool,
    /// Create new namespaces, namespace_enter
    pub namespace_create: bool,
    /// Mount filesystems from raw block devices
    pub offline_mount: bool,
    /// Create/delete/resize disk partitions
    pub partition_edit: bool,
    /// Capability changes, setuid/setgid manipulation
    pub token_manipulation: bool,
}

impl FeatureGates {
    /// Load all feature gates from environment variables.
    ///
    /// `dotenvy::dotenv()` must be called before this in `main.rs`.
    /// All gates default to `false` (disabled) when the env var is unset or != "1".
    #[must_use]
    pub fn load() -> Self {
        Self {
            kexec_load: env_bool("AETHER_KEXEC_LOAD"),
            module_load: env_bool("AETHER_MODULE_LOAD"),
            bpf_load: env_bool("AETHER_BPF_LOAD"),
            ptrace_attach: env_bool("AETHER_PTRACE_ATTACH"),
            namespace_create: env_bool("AETHER_NAMESPACE_CREATE"),
            offline_mount: env_bool("AETHER_OFFLINE_MOUNT"),
            partition_edit: env_bool("AETHER_PARTITION_EDIT"),
            token_manipulation: env_bool("AETHER_TOKEN_MANIPULATION"),
        }
    }

    /// Verify that a specific gate is enabled, returning an `AetherError::FeatureDisabled` if not.
    pub fn check(&self, ctx: ErrorContext, enabled: bool, gate_name: &str) -> Result<(), AetherError> {
        if !enabled {
            return Err(AetherError::feature_disabled(ctx, gate_name));
        }
        Ok(())
    }
}

fn env_bool(key: &str) -> bool {
    env::var(key).unwrap_or_default().trim() == "1"
}
