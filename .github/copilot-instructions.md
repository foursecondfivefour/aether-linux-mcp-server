# GitHub Copilot Instructions — AETHER_02

AETHER_02 is a Linux MCP server written in Rust. It exposes 12 administrative tool groups over stdio.

## Non-negotiable rules

- stdout is MCP JSON-RPC only; never print logs to stdout.
- Prefer direct Linux APIs (`nix`, `libc`, `/proc`, `/sys`, DBus) over shell commands.
- If a CLI is unavoidable, call it with `Command::new(program).args([...])` and no shell interpolation.
- Destructive actions require `force: true`.
- Kernel/system-dangerous actions require disabled-by-default feature gates from `.env`.
- Add `// SAFETY:` comments to every unsafe block.
- Return `Result<String, AetherError>` from tool handlers and include `ErrorContext`.
- Log all tool invocations via `audit`/`tracing` to stderr.

## Checks

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo check
```
