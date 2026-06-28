# Contributing to AETHER_02

## Development Setup

```bash
git clone https://github.com/foursecondfivefour/aether-linux-mcp-server
cd aether-linux-mcp-server
cp .env.example .env
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## Code Style

- Follow Rust 2021 edition conventions
- `cargo fmt` with `max_width = 120`
- `cargo clippy` strict
- No `unsafe` without `// SAFETY:` comment
- All public APIs documented

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Write tests for new behavior
4. Ensure `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt` all pass
5. Submit a PR with a clear description

## Security

Do not submit security vulnerabilities as public issues. See [SECURITY.md](SECURITY.md) for the disclosure process.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
