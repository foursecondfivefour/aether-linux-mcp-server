# Tests

The tests are intentionally focused on safe, deterministic behavior that does not mutate the host system.

| File | Coverage |
|------|----------|
| `config_tests.rs` | Feature gate defaults, env parsing, gate-check errors |
| `error_tests.rs` | Error display, context propagation, JSON parameter helpers |
| `integration_tests.rs` | Public server/action parameter construction smoke tests |
| `tool_dispatch_tests.rs` | Tool inventory, dangerous gate blocking, destructive error shape |

Run:

```bash
cargo test
```

For full quality checks:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
