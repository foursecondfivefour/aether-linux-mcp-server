//! AETHER_02 — Full-spectrum Linux MCP Server
//!
//! 12 tools covering 99.9% of Linux system administration.
//! Maximum speed (opt-level=3, LTO, native CPU), maximum security (PIE, RELRO, NX, CET/BTI).
//! Separate repository from AETHER (Windows). Same philosophy, different OS.

use aether_linux_mcp_server::config::FeatureGates;
use aether_linux_mcp_server::server::AetherServer;
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    // CRITICAL: MCP uses stdout exclusively for JSON-RPC.
    // All tracing/logging output MUST go to stderr and MUST be stripped of ANSI codes
    // to avoid corrupting the MCP protocol stream.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .with_thread_ids(false)
        .without_time()
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();

    let gates = FeatureGates::load();

    tracing::info!("AETHER_02 starting");
    tracing::info!(
        "Feature gates: KEXEC={} MODULE={} BPF={} PTRACE={} NAMESPACE={} OFFLINE_MOUNT={} PARTITION={} TOKEN={}",
        gates.kexec_load,
        gates.module_load,
        gates.bpf_load,
        gates.ptrace_attach,
        gates.namespace_create,
        gates.offline_mount,
        gates.partition_edit,
        gates.token_manipulation,
    );

    let server = AetherServer::new(gates);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let service = server.serve((stdin, stdout)).await?;

    tracing::info!("AETHER_02 ready on stdio");

    service.waiting().await?;

    tracing::info!("AETHER_02 shutting down");
    Ok(())
}
