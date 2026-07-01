# Security Model

AETHER_02 is a local system administration tool. It is intentionally powerful, but it is designed to make risky actions explicit and auditable.

## Boundaries

- **Transport:** stdio only. No HTTP server, no listening TCP port.
- **Caller:** local MCP client process running as the same user.
- **Logs:** stderr only. stdout is reserved for JSON-RPC.
- **Network:** AETHER_02 does not expose a remote API.

## Safety layers

1. **Parameter validation** at tool boundaries.
2. **No shell interpolation** for tool operations. Use syscalls or explicit argv vectors.
3. **`force: true`** for destructive operations.
4. **Feature gates** for critically dangerous Linux capabilities.
5. **Audit logging** for every tool invocation.
6. **Path canonicalization** where filesystem semantics require it.
7. **Compiler/linker hardening** for release builds.

## Feature gates

All gates default to disabled (`0`). Enable only what you need and only on machines where you accept the risk.

| Gate | Env var | Description |
|------|---------|-------------|
| Kexec | `AETHER_KEXEC_LOAD=0` | Load a new kernel via kexec |
| Module Load | `AETHER_MODULE_LOAD=0` | Load/unload kernel modules |
| BPF Load | `AETHER_BPF_LOAD=0` | Load BPF programs |
| Ptrace | `AETHER_PTRACE_ATTACH=0` | ptrace attach to non-child processes |
| Namespace Create | `AETHER_NAMESPACE_CREATE=0` | Create/enter Linux namespaces |
| Offline Mount | `AETHER_OFFLINE_MOUNT=0` | Mount filesystems from raw block devices |
| Partition Edit | `AETHER_PARTITION_EDIT=0` | Create/delete/resize disk partitions |
| Token Manipulation | `AETHER_TOKEN_MANIPULATION=0` | Capability, setuid, setgid, and user-token changes |

## Example `.env`

```env
AETHER_KEXEC_LOAD=0
AETHER_MODULE_LOAD=0
AETHER_BPF_LOAD=0
AETHER_PTRACE_ATTACH=0
AETHER_NAMESPACE_CREATE=0
AETHER_OFFLINE_MOUNT=0
AETHER_PARTITION_EDIT=0
AETHER_TOKEN_MANIPULATION=0
```

## Reporting vulnerabilities

Do not open public issues for vulnerabilities. See [../.github/community/SECURITY.md](../.github/community/SECURITY.md).
