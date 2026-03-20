# ADR-031: Coordination Board

Status: Implemented

## Context

RLMX swarm agents need persistent, structured inter-agent communication beyond ephemeral events. The existing `SwarmEvent` bus (ADR-001) is fire-and-forget: events are broadcast and lost if no subscriber is listening. Agents working on multi-step tasks need to post status updates, share findings, request reviews, and pin important decisions in a way that persists for the lifetime of the cluster.

Without a coordination board, agents resort to ad-hoc state sharing through the kernel's global state, leading to coupling and lack of auditability. A dedicated board per cluster provides a natural scoping boundary and threading model.

## Decision

Add a `BoardManager` to `rlmx-swarm` (Rust) and `@aix/swarm` (TypeScript) implementing persistent inter-agent coordination boards with threading, pinning, tagging, and artifact attachments.

### Core Design

1. **Board per cluster**: Each `Board` belongs to a `ClusterId` and has a unique `BoardId`. Multiple boards can exist per cluster (e.g., "findings", "decisions", "reviews").

2. **Threaded posts**: Posts can reference a `parent_post` to form reply threads. `get_thread(root_post_id)` returns the root post and all transitive replies.

3. **Tags and queries**: Posts carry string tags (e.g., "urgent", "experiment-42", "review-needed"). `query_by_tags` returns all posts matching any of the given tags.

4. **Pinning**: Important posts can be pinned for quick reference. Maximum 25 pins per board to prevent pin inflation.

5. **Artifact attachments**: Posts can reference artifact IDs (from ADR-030) via UUID attachments, linking coordination discussions to versioned artifacts.

6. **Capacity limits**: Maximum 1,000 posts per board to bound memory usage. Boards hitting the limit must be archived or pruned before new posts are accepted.

### Data Model

```
PostId:       Uuid (v4)
BoardId:      Uuid (v4)

Post:
  id:           PostId
  author:       u64          (agent/process ID)
  content:      String
  attachments:  Vec<Uuid>    (artifact IDs from ADR-030)
  parent_post:  Option<PostId>  (threading)
  tags:         Vec<String>
  pinned:       bool
  timestamp:    DateTime<Utc>

Board:
  id:           BoardId
  name:         String
  posts:        Vec<Post>
  participants: HashSet<u64>
  cluster_id:   ClusterId
```

### BoardManager API

```
create_board(name, cluster_id) -> BoardId
post(board_id, author, content, parent_post, tags, attachments) -> Result<PostId>
pin(board_id, post_id) -> Result<()>
unpin(board_id, post_id) -> Result<()>
get_post(board_id, post_id) -> Option<&Post>
get_thread(board_id, root_post_id) -> Vec<&Post>
query_by_tags(board_id, tags) -> Vec<&Post>
list_boards() -> Vec<&Board>
get_board(board_id) -> Option<&Board>
delete_board(board_id) -> Result<()>
```

### Integration Points

- `SwarmEvent::BoardUpdate` (already in types.rs and @aix/swarm events) emitted on post/pin/unpin
- `DomainEvent::BoardPostCreated` (already in kernel events.rs and @aix/shared) for cross-context integration
- Artifact attachments reference `ArtifactId` values from ADR-030
- Board scoped to `ClusterId` from existing swarm types

### Limits

| Limit | Value | Rationale |
|-------|-------|-----------|
| Posts per board | 1,000 | Bound memory; archive old boards |
| Pins per board | 25 | Prevent pin inflation |
| Content max length | Not enforced | Left to application layer |

## Consequences

### Good

- Persistent communication survives subscriber disconnects unlike SwarmEvent bus
- Threading provides natural conversation structure for multi-agent discussions
- Tag-based queries enable agents to find relevant posts without scanning all content
- Pin limit prevents "everything is important" anti-pattern
- Per-cluster scoping provides natural isolation between independent swarm tasks
- Artifact attachments connect discussions to versioned work products

### Bad

- In-memory storage means boards are lost on process restart (persistence layer deferred)
- 1,000 post limit requires explicit archival strategy for long-running clusters
- No access control on boards -- any participant can post, pin, or delete

### Neutral

- BoardManager is synchronous (no async) since it operates on in-memory data only
- Post ordering is insertion-order; timestamp is informational, not used for ordering

## Related

- [ADR-001: Distributed Swarm Architecture](ADR-001-distributed-swarm-architecture.md) -- swarm topology and ClusterId
- [ADR-030: Artifact DAG](ADR-030-artifact-dag.md) -- content-addressed artifacts referenced by board posts
- [ADR-026: TypeScript Migration Strategy](ADR-026-typescript-migration-strategy.md) -- @aix package conventions

## Tests

Rust: 30+ inline `#[cfg(test)]` tests in `crates/rlmx-swarm/src/board.rs`.
TypeScript: 15+ vitest tests in `packages/swarm/__tests__/board.test.ts`.
