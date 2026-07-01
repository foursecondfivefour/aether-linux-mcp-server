# Tool Modules

Each file in this directory implements one MCP tool group. Tool handlers accept an `action` string and JSON `params`, validate inputs, execute the Linux operation, audit-log the result, and return a string result or `AetherError`.

| Module | MCP tool |
|--------|----------|
| `process.rs` | `process_control` |
| `filesystem.rs` | `file_system` |
| `packages.rs` | `package_manager` |
| `sysconfig.rs` | `system_config` |
| `service.rs` | `service_manager` |
| `gui.rs` | `gui_automation` |
| `sysinfo.rs` | `system_info` |
| `network.rs` | `network_manager` |
| `user.rs` | `user_management` |
| `security.rs` | `security_audit` |
| `hardware.rs` | `hardware_control` |
| `automation.rs` | `system_automation` |
| `helpers.rs` | Shared command/read helpers |

Rules:

- No shell interpolation.
- Destructive actions require `force: true`.
- Critical actions require feature gates from `.env`.
- All tool invocations must be audit-logged.
