# Strict Rules (Always Enforced)

1. **Stub build**: `cargo build --workspace` without features MUST compile clean
2. **No println!**: use `tracing::*` in library crates
3. **RBAC**: Admin/System only via server-side `token_roles`
4. **Tests pass**: `cargo test --workspace` >= 1,286; `npm run test:ts` >= 1,135
5. **Zero clippy warnings**: `-D warnings`
6. **Edge unavailable**: tools return `"status": "unavailable"` without engine
7. **Permission matrix**: `PermissionRegistry` in `registry.rs` (18x17) is source of truth — 18 permissions x 17 agent types
8. **Domain events**: crate-local enums, not kernel `DomainEvent`
9. **Feature gates**: both `#[cfg(feature)]` and `#[cfg(not(feature))]` paths required (ADR-024)
10. **New crates**: add to Cargo.toml members, update counts, add to key files
11. **New agent types**: update AgentType enum + PermissionRegistry + spawn hierarchy + CLI + MCP
12. **Budget CAS**: BudgetLedger uses compare-and-swap versioning; reject stale concurrent writes
13. **Approval tiers**: Auto/Notify/Confirm/Escalate; Confirm/Escalate auto-deny after 5min timeout
14. **Trigger depth**: max 5 recursive trigger invocations to prevent cycles
15. **Artifact DAG**: content-addressed SHA-256; parent validation on insert; 512KB max per artifact
16. **Evolution lifecycle**: draft->staging->production->deprecated->killed; no skip transitions; dual-gate required
17. **TS types**: never re-define kernel enums, import from `@aix/shared`
18. **@aix/core degradation**: mandatory `{ status: 'unavailable' }` handling
