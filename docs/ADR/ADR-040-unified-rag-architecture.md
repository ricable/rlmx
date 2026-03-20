# ADR-040: Unified RAG Architecture

**Status**: Accepted
**Date**: 2026-03-20
**Deciders**: Architecture team

## Context

Claude Code token usage on RLMX is dominated by exploration — 80-90% of tokens go to Glob/Grep/Read before actual coding. The project already has qmd (23K indexed files across 16 collections) and Serena MCP (LSP-based symbol navigation), but they operate independently. PDF/DOCX documents cannot be searched, and the kernel's embedding system uses a 64-dim hash function instead of real semantic embeddings.

## Decision

Implement a unified RAG system (`@aix/rag`) that:

1. **Fans out** search queries to multiple backends (qmd for code/docs, docling for converted documents, kernel MemoryRegion for runtime vectors)
2. **Merges** results using Reciprocal Rank Fusion (RRF): `score = sum(weight_i * 1/(k + rank_i))`, k=60
3. **Adapts** weights via SONA when `@aix/core` is available, falling back to uniform weights
4. **Ingests** PDF/DOCX via docling-mcp → markdown → qmd indexing pipeline
5. **Swaps** kernel embeddings from 64-dim hash to 256-dim embeddinggemma-300M via Candle (feature-gated behind `real-embeddings`)
6. **Enforces** a qmd-first lookup hierarchy in CLAUDE.md: qmd → serena → Read/Glob/Grep

### Search Hierarchy

```
User query
    ├── qmd search (BM25, fast keyword) ← 80% of lookups
    ├── qmd query (hybrid + reranking) ← complex questions
    ├── qmd vsearch (semantic vector) ← conceptual/exploratory
    ├── serena find_symbol ← symbol navigation
    └── Read / Glob / Grep ← last resort only
```

### @aix/rag MCP Server

8 tools exposed via MCP protocol:

| Tool | Purpose |
|------|---------|
| `rag_search` | Unified multi-backend search with RRF merge |
| `rag_ingest` | PDF/DOCX → docling → markdown → qmd pipeline |
| `rag_compare` | Semantic similarity between two texts |
| `rag_cluster` | Group search results by similarity |
| `rag_provenance` | Track document: source → conversion → chunk → embed |
| `rag_stats` | Backend health and document counts |
| `rag_reindex` | Trigger collection reindexing |
| `rag_export` | Export search results as JSON/markdown/CSV |

Error codes: -37001 through -37010 (RagError extends AixError).

### Backend Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  QmdBackend  │     │DoclingBackend│     │MemoryBackend │
│  (search +   │     │ (ingest only)│     │  (stub for   │
│   BM25/vec)  │     │ PDF→markdown │     │  HNSW/NAPI)  │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       └────────────────────┼────────────────────┘
                            │
                    ┌───────┴───────┐
                    │BackendRegistry│
                    └───────┬───────┘
                            │
                    ┌───────┴───────┐
                    │  RRF Merger   │
                    │ + SONA Adapt  │
                    └───────┬───────┘
                            │
                    ┌───────┴───────┐
                    │   RagServer   │
                    │  (MCP stdio)  │
                    └───────────────┘
```

### Kernel Embedding Swap

| Feature | Without `real-embeddings` | With `real-embeddings` |
|---------|--------------------------|------------------------|
| EMBED_DIM | 64 | 256 |
| Provider | HashEmbeddingProvider | CandleEmbedder |
| Model | Hash function (unigrams+trigrams) | embeddinggemma-300M via Candle |
| Quality | Same-string matching only | Real semantic similarity |
| Deps | None | candle-core, candle-nn, candle-transformers, tokenizers |

The `EmbeddingProvider` trait allows runtime selection:
- `init_embedding_provider()` sets the global provider at startup
- `text_to_embedding()` delegates to the provider, falling back to hash on error
- Thread-safe via `OnceLock` + `Mutex<Model>`

### Docling Ingestion Pipeline

```
PDF/DOCX → docling-mcp convert → markdown → docs/ingested/<hash>.md → qmd collection update → indexed
```

## Validated Benchmark (2026-03-20)

| Tool | Traditional | With Tool | Reduction |
|------|------------|-----------|-----------|
| qmd (5 code searches) | ~529K tokens | ~4.3K tokens | **99.2%** |
| Serena (5 symbol lookups) | ~16.8K tokens | ~1.8K tokens | **89.2%** |
| **Combined** | **~546K tokens** | **~6.1K tokens** | **98.9%** |

See [docs/TOKEN-SAVINGS.md](../TOKEN-SAVINGS.md) for full per-query breakdown.

### CRD-Inspired RAG Sync

Automated reconciler keeps qmd index current:
- `scripts/rag-sync.yml` — declarative spec (what to watch, what to index)
- `scripts/rag-sync.sh` — controller (git diff detection → qmd update → embed)
- `scripts/hooks/post-commit-rag-sync.sh` — git hook for auto-trigger
- `.rag-sync-state` — last synced commit (idempotency)

Pattern modeled after Kubernetes GitKnowledgeSource CRD + controller reconciliation loop.

## Consequences

### Positive
- **98.9%** reduction in exploration tokens (validated via benchmark)
- PDF/DOCX documents become searchable through the same interface
- Real semantic embeddings enable meaningful similarity search in the kernel
- Single `rag_search` tool replaces manual multi-tool orchestration
- Graceful degradation: each backend independently failover-safe

### Negative
- `real-embeddings` feature adds ~50MB compile-time dependency (candle)
- embeddinggemma-300M model requires ~300MB disk at `$RLMX_MODEL_DIR`
- docling-mcp requires Python/uvx runtime for PDF conversion

### Risks
- Candle API may change between 0.8.x releases — pin exact version in CI
- EMBED_DIM change (64→256) is a breaking change for persisted vectors — migration needed
- RRF k=60 is a heuristic; may need tuning per-collection

## Related ADRs
- ADR-024: Feature Gates (real-embeddings follows same pattern)
- ADR-029: @aix/deploy (error pattern reuse)
- ADR-011: Sandbox Orchestration (similar MCP server pattern)
