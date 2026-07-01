# MCP Client Configuration

AETHER_02 runs over stdio. Configure clients to launch the `aether-mcp-server` binary or use npm/npx.

## Recommended portable config

This works when the npm package is available:

```json
{
  "mcpServers": {
    "aether-02": {
      "command": "npx",
      "args": ["-y", "@foursecondfivefour/aether-linux-mcp-server"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

## Local binary config

Use this when you installed with `install.sh` or built from source:

```json
{
  "mcpServers": {
    "aether-02": {
      "command": "/absolute/path/to/aether-mcp-server",
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

## Common locations

| Client | Linux config path |
|--------|-------------------|
| Cursor | `~/.cursor/mcp.json` |
| Claude Desktop | `~/.config/Claude/claude_desktop_config.json` |
| Windsurf | `~/.codeium/windsurf/mcp_config.json` |
| VS Code Claude MCP | `~/.config/Code/User/globalStorage/anthropic.claude-mcp/mcp.json` |
| VS Code Insiders Claude MCP | `~/.config/Code - Insiders/User/globalStorage/anthropic.claude-mcp/mcp.json` |

## Notes

- Restart your editor after editing MCP config.
- Keep stdout clean. AETHER_02 already logs to stderr by design.
- Destructive actions need `force: true`.
- Critically dangerous operations additionally need feature gates in `.env`.
