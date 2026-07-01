# Contributing to AETHER_02

Thanks for improving AETHER_02. Keep changes small, auditable, and safe by default.

## Development setup

```bash
git clone https://github.com/foursecondfivefour/aether-linux-mcp-server
cd aether-linux-mcp-server
cp config/env/.env.example .env
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Detailed development notes: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Code style

- Rust 2021 edition.
- `cargo fmt` with repository `rustfmt.toml`.
- `cargo clippy -- -D warnings` must pass.
- No `unsafe` without a `// SAFETY:` comment.
- stdout is reserved for MCP JSON-RPC; logs go to stderr.
- No shell interpolation for tool actions.
- Destructive actions require `force: true`.
- Critical Linux capabilities require disabled-by-default `.env` feature gates.

## Pull request process

1. Fork the repository.
2. Create a feature branch.
3. Add/update tests for behavior changes.
4. Run the validation commands in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).
5. Submit a PR with a clear summary and safety notes.

## Security

Do not submit security vulnerabilities as public issues. See [SECURITY.md](SECURITY.md) for the disclosure process.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
