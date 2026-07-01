# Scripts

Utility scripts for installation, local validation, and release work.

| Script | Purpose |
|--------|---------|
| `install/install.sh` | Linux shell installer and MCP client auto-registration |
| `install/install.js` | npm postinstall downloader for the release binary |
| `smoke_stdio.py` | Starts the MCP server over stdio, sends `initialize`, `tools/list`, and a safe `tools/call` request |
| `mock_matrix.py` | Starts the MCP server with mocked commands and checks representative safe, force-guarded, dry-run, and feature-gated actions across all tool groups |

## Smoke test

Run from repository root:

```bash
cargo build
python3 scripts/smoke_stdio.py
```

Override binary path:

```bash
AETHER_BIN=/path/to/aether-mcp-server python3 scripts/smoke_stdio.py
```
