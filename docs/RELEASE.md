# Release Checklist

## Automated GitHub release

Use **Actions → Release → Run workflow**.

Inputs:

- `version` — optional. Empty means read `Cargo.toml` and release `v<version>`.
- `prerelease` — optional GitHub prerelease flag.

The workflow:

1. Creates/pushes the tag when launched manually.
2. Runs `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`.
3. Builds `target/release/aether-mcp-server` with `cargo build --release -j 1`.
4. Uploads release assets.
5. Creates a GitHub Release with GitHub-generated release notes (`--generate-notes`).

Pushing a `v*` tag also runs the same release workflow.

## Before tagging manually

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release -j 1
node --check scripts/install/install.js
python3 -m json.tool package.json >/dev/null
python3 -m json.tool integrations/lobehub/lhm.plugin.json >/dev/null
```

## Version alignment

Keep these versions in sync when releasing:

- `Cargo.toml` package version
- `package.json` version
- `integrations/lobehub/lhm.plugin.json` version
- README badge expectations / release notes

## Binary asset

GitHub Releases should provide these assets:

```text
aether-mcp-server
aether-mcp-server-linux-x86_64.tar.gz
SHA256SUMS
```

The npm `install.js` downloads the raw binary asset from:

```text
https://github.com/foursecondfivefour/aether-linux-mcp-server/releases/latest/download/aether-mcp-server
```

## Package smoke checks

```bash
npm pack --dry-run
npm install -g ./foursecondfivefour-aether-linux-mcp-server-*.tgz
which aether-mcp-server
aether-mcp-server
```

Remember: running the binary directly waits for MCP JSON-RPC on stdin.
