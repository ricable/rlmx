# ADR-030: Artifact DAG

Status: Implemented

## Context

RLMX agents produce and exchange artifacts (code, documents, models, data) during multi-agent workflows. Without structured version tracking, agents overwrite each other's work, lose provenance, and cannot diff or branch artifacts. The swarm (ADR-001) and coordination board (ADR-031) need a content-addressed storage layer that provides deterministic IDs, branching, lineage tracing, and efficient diffing.

Existing file-system or blob-store approaches lack the DAG semantics required for multi-parent merges (e.g., two agents independently improve an artifact and a third merges them). A Git-like content-addressed DAG solves this while remaining simple enough to embed in-process.

## Decision

Create `rlmx-artifact` (Rust) and `@aix/artifact` (TypeScript) implementing a content-addressed artifact DAG with the following properties:

### Core Design

1. **Content-addressed IDs**: Every artifact is identified by the SHA-256 hash of its content bytes, producing deterministic `ArtifactId` values. Two identical payloads always produce the same ID.

2. **DAG structure**: Each artifact records zero or more `parent_ids`, forming a directed acyclic graph. Root artifacts have no parents. Multi-parent artifacts represent merges.

3. **Branching**: Named branches (strings) point to a head `ArtifactId`. Branches can be created, advanced, listed, and deleted. Advancing a branch requires the new head to exist in the store.

4. **Diffing**: The `diff(a, b)` operation compares two artifacts by content size, producing `ArtifactDiff` with byte-level addition/removal counts. Content-type-specific semantic diffs are deferred to consumers.

5. **Lineage**: `lineage(id)` traces an artifact back through its first parent chain to a root, providing a linear history view.

### Artifact Structure

```
ArtifactId:     [u8; 32]  (SHA-256 hash)
Artifact:
  id:           ArtifactId
  content:      Vec<u8>
  content_type: String     (MIME type or custom identifier)
  parent_ids:   Vec<ArtifactId>
  creator:      u64        (agent/process ID)
  created_at:   DateTime<Utc>
  metadata:     HashMap<String, String>
```

### Package Structure

**Rust (`rlmx-artifact`)**:
```
crates/rlmx-artifact/
  Cargo.toml
  src/
    lib.rs       -- module declarations + re-exports
    types.rs     -- ArtifactId, Artifact, ArtifactDiff, ArtifactError
    dag.rs       -- ArtifactDag (in-memory DAG with insert/query/lineage)
    store.rs     -- ContentAddressedStore (SHA-256 hashing + DAG wrapper)
    branch.rs    -- BranchManager (named branch heads)
```

**TypeScript (`@aix/artifact`)**:
```
packages/artifact/
  package.json
  tsconfig.json
  src/
    index.ts     -- types + ArtifactStore class (Map-based in-memory store)
  tests/
    artifact.test.ts
```

### Integration Points

- `DomainEvent::ArtifactCreated` and `DomainEvent::ArtifactBranchAdvanced` (already in kernel events.rs and @aix/shared)
- `SyscallPermission::ArtifactWrite` (already in kernel types.rs and @aix/shared)
- Coordination Board (ADR-031) references artifact IDs via `attachments: Vec<Uuid>` on board posts
- Swarm agents with `ArtifactWrite` permission (Researcher, Experimenter) can create and branch artifacts

### Dependencies

**Rust**: sha2, serde, serde_json, chrono, uuid, thiserror, tracing (all workspace deps)
**TypeScript**: @aix/shared (for generateId)

## Consequences

### Good

- Deterministic content-addressed IDs eliminate duplicate storage and enable cache-friendly lookups
- DAG structure naturally supports multi-parent merges from concurrent agent work
- Named branches provide familiar Git-like workflow for agent coordination
- Lineage tracing enables full provenance auditing
- In-memory implementation keeps latency minimal for agent-to-agent artifact exchange

### Bad

- In-memory store does not persist across process restarts (persistence layer deferred)
- Byte-level diff is coarse; semantic diffing (e.g., JSON patch, text diff) is left to consumers
- SHA-256 computation adds ~1ms overhead per MB of content

### Neutral

- ArtifactId uses raw `[u8; 32]` rather than a hex string for efficiency; Display trait provides hex formatting
- Branch operations require the store to be passed separately (BranchManager does not own the DAG)

## Related

- [ADR-001: Distributed Swarm Architecture](ADR-001-distributed-swarm-architecture.md) -- swarm topology
- [ADR-031: Coordination Board](ADR-031-coordination-board.md) -- inter-agent communication with artifact attachments
- [ADR-026: TypeScript Migration Strategy](ADR-026-typescript-migration-strategy.md) -- @aix package conventions
- [ADR-028: @aix Package Architecture](ADR-028-aix-package-architecture.md) -- workspace layout

## Tests

Rust: 40+ inline `#[cfg(test)]` tests across types, dag, store, and branch modules.
TypeScript: 15+ vitest tests covering store, branching, lineage, and diffing.
