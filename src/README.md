# Source Layout

`src/` contains the Rust MCP server implementation.

| File/dir | Purpose |
|----------|---------|
| `main.rs` | Process entry point: `.env`, tracing to stderr, stdio MCP service |
| `lib.rs` | Public module declarations |
| `server.rs` | MCP server metadata, tool router, prompts, and resources |
| `config.rs` | Feature gates loaded from environment variables |
| `error.rs` | Shared `AetherError`, `ErrorContext`, and JSON parameter helpers |
| `audit.rs` | Structured success/failure/security/forced-operation audit logs |
| `tools/` | 12 Linux administration tool groups |

See [../docs/PROJECT_STRUCTURE.md](../docs/PROJECT_STRUCTURE.md) for the full repository map.
