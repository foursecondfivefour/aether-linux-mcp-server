## Summary

- 

## Type of change

- [ ] Bug fix
- [ ] New tool/action
- [ ] Documentation
- [ ] Refactor
- [ ] Security hardening

## Safety checklist

- [ ] stdout remains MCP JSON-RPC only
- [ ] Destructive actions require `force: true`
- [ ] Dangerous operations are behind `.env` feature gates
- [ ] No shell interpolation or user-controlled shell strings
- [ ] Paths/parameters are validated before use
- [ ] Unsafe blocks include `// SAFETY:` comments
- [ ] Audit logging added/updated

## Validation

- [ ] `cargo fmt --check`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo test`
- [ ] `cargo check`

## Notes

