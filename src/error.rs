//! AetherError — unified error type mapping POSIX errno, nix errors,
//! and protocol errors into structured MCP tool responses.
//!
//! Follows the same pattern as AETHER_01 with Linux-specific extensions.

use thiserror::Error;

/// Structured context for every error — tool name and action that failed.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub tool: String,
    pub action: String,
}

impl ErrorContext {
    #[must_use]
    pub fn new(tool: impl Into<String>, action: impl Into<String>) -> Self {
        Self { tool: tool.into(), action: action.into() }
    }
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.tool, self.action)
    }
}

/// Central error type for AETHER_02.
#[derive(Debug, Error)]
pub enum AetherError {
    #[error("Operation failed: {ctx} — {message}")]
    OperationFailed { ctx: ErrorContext, message: String },

    #[error("Feature disabled: {ctx} — gate '{gate}' must be enabled in .env")]
    FeatureDisabled { ctx: ErrorContext, gate: String },

    #[error("Permission denied: {ctx} — {reason}")]
    PermissionDenied { ctx: ErrorContext, reason: String },

    #[error("Force required: {ctx} — set `force: true` to execute this action")]
    ForceRequired { ctx: ErrorContext },

    #[error("Invalid parameter: {ctx} — parameter '{param}': {message}")]
    InvalidParameter { ctx: ErrorContext, param: String, message: String },

    #[error("Not found: {ctx} — {what}")]
    NotFound { ctx: ErrorContext, what: String },

    #[error("IO error: {ctx} — {source}")]
    Io {
        ctx: ErrorContext,
        #[source]
        source: std::io::Error,
    },

    #[error("System error: {ctx} — {message}")]
    System { ctx: ErrorContext, message: String },

    #[error("Not implemented: {ctx} — {feature}")]
    NotImplemented { ctx: ErrorContext, feature: String },
}

impl AetherError {
    #[must_use]
    pub fn feature_disabled(ctx: ErrorContext, gate: &str) -> Self {
        Self::FeatureDisabled { ctx, gate: gate.to_string() }
    }

    #[must_use]
    pub fn force_required(ctx: ErrorContext) -> Self {
        Self::ForceRequired { ctx }
    }

    #[must_use]
    pub fn permission_denied(ctx: ErrorContext, reason: impl Into<String>) -> Self {
        Self::PermissionDenied { ctx, reason: reason.into() }
    }

    #[must_use]
    pub fn invalid_param(ctx: ErrorContext, param: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidParameter { ctx, param: param.into(), message: message.into() }
    }

    #[must_use]
    pub fn not_found(ctx: ErrorContext, what: impl Into<String>) -> Self {
        Self::NotFound { ctx, what: what.into() }
    }

    #[must_use]
    pub fn operation_failed(ctx: ErrorContext, message: impl Into<String>) -> Self {
        Self::OperationFailed { ctx, message: message.into() }
    }

    #[must_use]
    pub fn system_error(ctx: ErrorContext, message: impl Into<String>) -> Self {
        Self::System { ctx, message: message.into() }
    }

    #[must_use]
    pub fn not_implemented(ctx: ErrorContext, feature: impl Into<String>) -> Self {
        Self::NotImplemented { ctx, feature: feature.into() }
    }

    #[must_use]
    pub fn ctx(&self) -> &ErrorContext {
        match self {
            Self::OperationFailed { ctx, .. }
            | Self::FeatureDisabled { ctx, .. }
            | Self::PermissionDenied { ctx, .. }
            | Self::ForceRequired { ctx }
            | Self::InvalidParameter { ctx, .. }
            | Self::NotFound { ctx, .. }
            | Self::Io { ctx, .. }
            | Self::System { ctx, .. }
            | Self::NotImplemented { ctx, .. } => ctx,
        }
    }
}

impl From<std::io::Error> for AetherError {
    fn from(source: std::io::Error) -> Self {
        Self::Io { ctx: ErrorContext::new("unknown", "unknown"), source }
    }
}

impl From<nix::errno::Errno> for AetherError {
    fn from(errno: nix::errno::Errno) -> Self {
        Self::System { ctx: ErrorContext::new("unknown", "unknown"), message: errno.desc().to_string() }
    }
}

/// Helper to extract `force` field from params JSON.
pub fn require_force(params: &serde_json::Value) -> bool {
    params.get("force").and_then(|v| v.as_bool()).unwrap_or(false)
}

/// Helper to get a string param, returning Err(InvalidParameter) if missing or wrong type.
pub fn get_string_param<'a>(
    params: &'a serde_json::Value,
    name: &str,
    ctx: &ErrorContext,
) -> Result<&'a str, AetherError> {
    params
        .get(name)
        .ok_or_else(|| AetherError::invalid_param(ctx.clone(), name, "required parameter is missing"))?
        .as_str()
        .ok_or_else(|| AetherError::invalid_param(ctx.clone(), name, "must be a string"))
}

/// Helper to get an optional string param.
pub fn get_opt_string_param<'a>(params: &'a serde_json::Value, name: &str) -> Option<&'a str> {
    params.get(name).and_then(|v| v.as_str())
}
