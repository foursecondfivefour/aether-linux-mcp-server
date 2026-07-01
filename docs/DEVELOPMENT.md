# Development

## Requirements

- Rust nightly, because `.cargo/config.toml` uses nightly hardening flags.
- Node.js 18+ for npm package checks and `install.js`.
- Linux x86-64 or arm64.

Install Rust with:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
  -y --profile minimal --default-toolchain nightly \
  --component rustfmt --component clippy
```

Then:

```bash
. "$HOME/.cargo/env"
```

## Common commands

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
node --check scripts/install/install.js
python3 -m json.tool package.json >/dev/null
python3 -m json.tool integrations/lobehub/lhm.plugin.json >/dev/null
```

## MCP smoke test

Build first:

```bash
cargo build
```

Then run:

```bash
python3 scripts/smoke_stdio.py
```

The script sends newline-delimited JSON-RPC messages to `target/debug/aether-linux-mcp-server`. The server should respond to:

- `initialize`
- `tools/list`
- `tools/call` with a safe read-only action such as `system_info` / `uptime`

## Conventions

- stdout must remain JSON-RPC only.
- Use stderr for tracing and audit logs.
- Do not use `bash -c`, `sh -c`, or interpolated command strings for tool actions.
- Prefer direct Linux APIs: `/proc`, `/sys`, `nix`, `libc`, DBus.
- If a CLI is unavoidable, use `Command::new(program).args([...])`.
- Destructive actions require `force: true`.
- Kernel/system-dangerous actions require `.env` feature gates.
- Every unsafe block must have a `// SAFETY:` comment.

## Test layout

| File | Scope |
|------|-------|
| `tests/config_tests.rs` | Feature gate defaults and env parsing |
| `tests/error_tests.rs` | Error formatting and helper behavior |
| `tests/integration_tests.rs` | Public API smoke tests |
| `tests/tool_dispatch_tests.rs` | Tool names, destructive action error shape, gate checks |
