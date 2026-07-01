# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.x     | Yes (active) |

## Threat Model

For the full design notes, see [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md).

AETHER_02 is a **local, single-machine, stdio-only** MCP server. It does **not** open network ports, listen for incoming connections, or expose any remote API. Its entire attack surface is:

1. **stdin** — MCP JSON-RPC messages from the AI client (local, same-machine, same-user)
2. **Feature gates in `.env`** — configuration read from disk at startup
3. **Shell install script** — one-time setup from the internet

### Trust Boundaries

```
User's machine (fully trusted)
│
├── Cursor/Claude/VS Code (AI client) — same user, same machine
│   │
│   └── AETHER_02 (stdio subprocess) ← THE SERVER
│       │
│       └── Linux kernel (syscalls) — same machine, kernel
│
└── Internet ← AETHER_02 does NOT connect here
```

**AETHER_02 has NO network access.** It is a pure stdio process that only communicates with the AI client that spawned it.

## Defensive Architecture

### 1. Input Validation
Every parameter is validated before any syscall or CLI invocation. Invalid parameters return structured errors.

### 2. Principle of Least Privilege
- Read-only operations require **no confirmation**
- Destructive operations require **`force: true`**
- Critically dangerous operations are **gated behind `.env` feature flags**, all **disabled by default**

### 3. No Shell Injection
All CLI invocations pass arguments as `Vec<String>` — never through shell interpolation. Direct syscalls via `nix`/`libc` where possible.

### 4. Compiler & Binary Hardening
- **Full RELRO** (`-Wl,-z,relro,-z,now`)
- **Stack protector** (`-fstack-protector-strong`)
- **NX stack** (`-z,noexecstack`)
- **PIE** (`-Wl,-pie`)
- **Read-only code segments** (`-z,separate-code`)
- **CET/BTI** (`-C passes=cf-protection=full`)
- **Fat LTO + codegen-units=1**
- **Static binary** (musl target)
- **Panic=abort**
- **Symbol stripping**

### 5. Audit Trail
Every tool invocation is logged via `tracing` to stderr with tool, action, parameters, result.

## Reporting a Vulnerability

**Please do NOT open a public issue.**

Send vulnerability reports to: `security@foursecondfivefour.dev`

You can expect:
- Acknowledgment within 48 hours
- Status update within 7 days
- Disclosure coordinated with fix availability

## License

MIT — see [LICENSE](LICENSE)
