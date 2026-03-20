# Token Savings with qmd + Serena

**Date**: 2026-03-20
**Status**: Validated with benchmarks

## Overview

RLMX uses two complementary tools to reduce Claude Code token consumption by **98.9%** during code exploration:

- **qmd** — local RAG with BM25 + vector search across 23K indexed files
- **Serena** — LSP-powered symbol navigation (definitions, references, types)

Together they replace expensive full-file reads with targeted, chunk-level results.

## Lookup Priority (Mandatory)

```
1. qmd search "query"              ← 80% of lookups (BM25, <400ms)
2. qmd query "query"               ← complex questions (hybrid + reranking, ~9s)
3. qmd vsearch "query"             ← conceptual/exploratory (vector, ~4s)
4. serena find_symbol <name>       ← struct/enum/trait definitions + cross-refs
5. serena find_referencing_symbols ← who uses this symbol?
6. Read / Glob / Grep              ← LAST RESORT only
```

**Rule**: Never `Read` a file >100 lines without trying qmd/serena first.

## Benchmark Results (2026-03-20)

### qmd: Code Search (5 Scenarios)

| Test | Query Type | Traditional | qmd | Savings |
|------|-----------|-------------|-----|---------|
| BudgetLedger struct | `search` | 40,477 c (5 files) | 1,883 c | **95.3%** |
| ApprovalGate system | `query` | 687,339 c (32 files) | 4,196 c | **99.4%** |
| MCP tool dispatch | `search` | 607,445 c (21 files) | 3,333 c | **99.5%** |
| Domain event chain | `vsearch` | 754,992 c (58 files) | 3,872 c | **99.5%** |
| EmbeddingProvider trait | `query` | 26,808 c (2 files) | 4,059 c | **84.9%** |
| **TOTAL** | | **2,117,061 c (~529K tk)** | **17,343 c (~4.3K tk)** | **99.2%** |

### Serena: Symbol Lookup (5 Symbols)

| Symbol | Traditional | Serena | Savings |
|--------|-------------|--------|---------|
| BudgetLedger (Rust+TS, 4 fields, 10 methods) | 10,000 c | 1,480 c | **85.2%** |
| ApprovalGate (3 fields + docstring) | 5,400 c | 620 c | **88.5%** |
| EmbeddingProvider (trait, 3 methods) | 12,000 c | 520 c | **95.7%** |
| SyscallPermission (18-variant enum, 3 locations) | 13,700 c | 3,200 c | **76.6%** |
| PermissionRegistry (Rust+TS, 5 methods) | 26,000 c | 1,400 c | **94.6%** |
| **TOTAL** | **67,100 c (~16.8K tk)** | **7,220 c (~1.8K tk)** | **89.2%** |

### Combined Impact

| | Traditional | With qmd+Serena | Reduction |
|---|------------|-----------------|-----------|
| **Characters** | 2,184,161 | 24,563 | **98.9%** |
| **Tokens (approx)** | ~546,040 | ~6,141 | **98.9%** |
| **Cost per 10 investigations** | ~$1.64 | ~$0.02 | **$16.20 saved** |

## When to Use Each Tool

| Situation | Tool | Why |
|-----------|------|-----|
| "Where is X defined?" | `qmd search "X"` | Fast keyword match |
| "How does X work?" | `qmd query "how does X work"` | Hybrid search with reranking |
| "What's conceptually similar to X?" | `qmd vsearch "X concept"` | Semantic vector search |
| "Show me the struct/enum/trait for X" | `serena find_symbol X` | Exact definition + type info |
| "Who calls/uses X?" | `serena find_referencing_symbols X` | Cross-reference graph |
| "What's in this file?" | `serena get_symbols_overview file.rs` | File structure without full read |
| Reading a specific known file | `Read` | Direct access (last resort) |

## RAG Sync (CRD Controller Pattern)

The qmd index stays current via an automated reconciler inspired by Kubernetes GitKnowledgeSource CRD:

```
scripts/rag-sync.yml     ← Declarative spec (what to watch)
scripts/rag-sync.sh      ← Controller (git diff → qmd update → qmd embed)
.rag-sync-state          ← Last synced commit (state tracking)
```

**Modes**:
- `rag-sync.sh` — one-shot reconcile
- `rag-sync.sh --watch 5m` — continuous (like a K8s controller)
- `rag-sync.sh --hook` — triggered by git post-commit hook
- `rag-sync.sh --status` — check sync state
- `rag-sync.sh --reset` — force full reindex

**Git hook**: `scripts/hooks/post-commit-rag-sync.sh` auto-triggers on every commit.

## Troubleshooting

### qmd: `ERR_DLOPEN_FAILED` / `NODE_MODULE_VERSION` mismatch

```bash
cd ~/.local/share/mise/installs/npm-tobilu-qmd/*/lib/node_modules/@tobilu/qmd
npm rebuild better-sqlite3
```

This happens when Node.js is upgraded but qmd's native module wasn't rebuilt.

### Serena: tools not available

```bash
pgrep -f "serena start-mcp-server"  # Check if running
# If not running, Claude Code restarts it automatically on next tool call
```

### RAG index out of date

```bash
scripts/rag-sync.sh --status   # Check
scripts/rag-sync.sh --reset    # Force full reindex
scripts/rag-sync.sh            # Run reconciler
```

## Architecture Reference

- **ADR-040**: [Unified RAG Architecture](ADR/ADR-040-unified-rag-architecture.md)
- **@aix/rag**: `packages/rag/` — MCP server with RRF merge, backend registry
- **Kernel embeddings**: `crates/rlmx-kernel/src/memory.rs` — `EmbeddingProvider` trait
- **RAG sync config**: `scripts/rag-sync.yml` + `scripts/rag-sync.sh`
