# Installation

AETHER_02 supports four install paths. Use **npm** for the shortest setup, **install.sh** for full editor auto-registration, or build from source for development.

## Method 1: npm global install

```bash
npm install -g @foursecondfivefour/aether-linux-mcp-server
```

The `postinstall` script downloads the latest Linux binary from GitHub Releases into the package `bin/` directory. The command exposed on PATH is:

```bash
aether-mcp-server
```

## Method 2: one-click shell install

```bash
curl -sSL https://raw.githubusercontent.com/foursecondfivefour/aether-linux-mcp-server/main/scripts/install/install.sh | bash
```

The script:

1. Downloads the latest release binary.
2. Creates `.env` with safe defaults.
3. Registers the server in detected MCP clients: Cursor, Claude Desktop, Windsurf, VS Code.

## Method 3: editor install badges

| Editor | Install |
|--------|---------|
| Cursor | [![Add AETHER_02 MCP server to Cursor](https://cursor.com/deeplink/mcp-install-dark.png)](cursor://anysphere.cursor-deeplink/mcp/install?name=aether-02&config=eyJjb21tYW5kIjoibnB4IiwiYXJncyI6WyIteSIsIkBmb3Vyc2Vjb25kZml2ZWZvdXIvYWV0aGVyLWxpbnV4LW1jcC1zZXJ2ZXIiXSwiZW52Ijp7IlJVU1RfTE9HIjoiaW5mbyJ9fQ==) |
| VS Code | [![Install in VS Code](https://img.shields.io/badge/Install_in-VS_Code-007ACC?logo=visualstudiocode&logoColor=white)](https://vscode.dev/redirect/mcp/install?name=aether-02&config=%7B%22command%22%3A%22npx%22%2C%22args%22%3A%5B%22-y%22%2C%22%40foursecondfivefour%2Faether-linux-mcp-server%22%5D%2C%22env%22%3A%7B%22RUST_LOG%22%3A%22info%22%7D%7D) |
| VS Code Insiders | [![Install in VS Code Insiders](https://img.shields.io/badge/Install_in-VS_Code_Insiders-24BFA5?logo=visualstudiocode&logoColor=white)](https://insiders.vscode.dev/redirect/mcp/install?name=aether-02&config=%7B%22command%22%3A%22npx%22%2C%22args%22%3A%5B%22-y%22%2C%22%40foursecondfivefour%2Faether-linux-mcp-server%22%5D%2C%22env%22%3A%7B%22RUST_LOG%22%3A%22info%22%7D%7D&quality=insiders) |

If your browser or GitHub client does not open the Cursor deeplink, copy this into the browser address bar:

```text
cursor://anysphere.cursor-deeplink/mcp/install?name=aether-02&config=eyJjb21tYW5kIjoibnB4IiwiYXJncyI6WyIteSIsIkBmb3Vyc2Vjb25kZml2ZWZvdXIvYWV0aGVyLWxpbnV4LW1jcC1zZXJ2ZXIiXSwiZW52Ijp7IlJVU1RfTE9HIjoiaW5mbyJ9fQ==
```

## Method 4: build from source

```bash
git clone https://github.com/foursecondfivefour/aether-linux-mcp-server
cd aether-linux-mcp-server
cp config/env/.env.example .env
cargo build --release
./scripts/install/install.sh target/release/aether-mcp-server
```

## Verify

```bash
aether-mcp-server --help
```

AETHER_02 is an MCP stdio server, so in normal operation it waits for JSON-RPC messages from an MCP client. Logs go to stderr; stdout is reserved for JSON-RPC.
