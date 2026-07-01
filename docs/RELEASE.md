# Release Checklist

## Before tagging

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
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

GitHub Releases should provide:

```text
aether-mcp-server
```

The npm `install.js` downloads this asset from:

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
