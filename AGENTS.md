# AGENTS.md — AETHER_02 AI Agent Instructions

## Project Identity

- **Name:** AETHER_02
- **Type:** MCP Server (Model Context Protocol)
- **Platform:** Linux (any distribution)
- **Language:** Rust (edition 2021)
- **Transport:** stdio (JSON-RPC)
- **Separate repository from AETHER (Windows MCP)**

## Architecture

- `rmcp` 0.5 SDK for MCP protocol
- `tokio` async runtime
- Direct syscalls via `nix`/`libc` (no shell injection)
- CLI tools invoked with `Vec<String>` arguments (no `system()` or `bash -c`)
- Structured audit logging to stderr via `tracing`

## Security Model

1. **No network** — stdio-only transport
2. **feature gates** — 8 dangerous capabilities disabled by default in `.env`
3. **force: true** — destructive operations require explicit confirmation
4. **Parameter validation** — all inputs checked at boundary
5. **Path canonicalization** — no traversal attacks
6. **Compiler hardening** — PIE, RELRO, NX, CET/BTI, strip, LTO

## Build Commands

```bash
cargo build              # Debug build
cargo build --release    # Release (hardened)
cargo test               # Run tests
cargo clippy -- -D warnings  # Lint
cargo fmt --check        # Format check
```

## Adding a New Tool

1. Create `src/tools/newtool.rs` with `pub fn handle(...) -> String`
2. Add `pub mod newtool;` to `src/tools/mod.rs`
3. Register in `src/server.rs` with `#[tool(description = "...")]`
4. Validate params, check feature gates, log via audit
5. Return `Result<String, AetherError>` unwrapped to string

## Documentation Standards

All documentation claims must be verified via **Context7 MCP**. Use **Sequential Thinking MCP** before non-trivial design decisions.
