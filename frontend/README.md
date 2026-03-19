# RLMX Frontend

## Overview

Single-page web dashboard for the RLMX cognition kernel. Vanilla JavaScript, no build tools, no framework. Communicates with the MCP server via JSON-RPC 2.0 over HTTP.

## Running

```bash
# Terminal 1: Start MCP server
cargo run -p rlmx-cli -- serve --port 3000

# Terminal 2: Serve frontend
cd frontend
python3 -m http.server 8080

# Open http://127.0.0.1:8080
```

For edge inference, start the server with a model:

```bash
RLMX_EDGE_MODEL=tinyllama-1.1b-q4_k_m \
  cargo run -p rlmx-cli --features "ruvllm,metal" -- serve --port 3000
```

## Views

| View | Description |
|------|-------------|
| **Query** | Semantic search across vector memory with relevance scores |
| **Ingest** | Insert text data into memory via plugin adapters |
| **Memory Stats** | Segment counts, tier distribution (Hot/Warm/Cold), HNSW health |
| **Graph Query** | Run Cypher queries against the property graph |
| **Edge Inference** | WASM engine init, feature detection, chat formatting, benchmarks, text generation |
| **List Tools** | Browse all 15 MCP tools with descriptions |
| **Raw JSON-RPC** | Send arbitrary JSON-RPC requests for debugging |
| **Request Log** | Session history of all RPC calls |

## Edge AI View

The Edge Inference view integrates `@ruvector/ruvllm-wasm` (v2.0.2) for in-browser capabilities.

### Buttons

| Button | What it does |
|--------|-------------|
| **Init WASM Engine** | Loads ruvllm-wasm (394KB .wasm), initializes RuvLLMWasm + KV cache |
| **Feature Summary** | Reports WASM SIMD, WebGPU, SharedArrayBuffer, Web Workers, capability level |
| **Benchmark MatMul** | 512x512 matrix multiply via ParallelInference with Web Workers |
| **Server Status** | Queries MCP server's `rlmx_edge_status` tool |

### Chat Formatting

Formats messages using chat templates that run entirely in WASM (no server round-trip):
- ChatML / Qwen
- Llama 3
- Mistral
- Gemma
- Phi

### Text Generation

Sends prompts to the MCP server's `rlmx_edge_generate` tool. Requires a GGUF model loaded on the server.

## WASM Module

The `ruvllm-wasm/` directory contains `@ruvector/ruvllm-wasm` v2.0.2:

```
ruvllm-wasm/
├── package.json           # npm package metadata
├── ruvllm_wasm.js         # JavaScript bindings (128KB)
├── ruvllm_wasm.d.ts       # TypeScript type definitions (45KB)
└── ruvllm_wasm_bg.wasm    # Compiled WASM binary (394KB)
```

### WASM API Notes

- **Initialization**: Call `await mod.default()` first (loads + instantiates the .wasm binary). Do NOT call `mod.init()` after — it errors because `default()` already handles panic hooks.
- **ParallelInference**: Constructor returns a Promise — must `await new mod.ParallelInference()`.
- **Chat formatting**: `tpl.format(msgs)` consumes WASM pointers internally. Do NOT call `.free()` on messages or template after `format()`.
- **Feature detection**: Use `mod.feature_summary()`, `mod.is_simd_available()`, `mod.detect_capability_level()` etc.

### Updating WASM Module

```bash
cd frontend/ruvllm-wasm
npm pack @ruvector/ruvllm-wasm
tar xzf ruvector-ruvllm-wasm-*.tgz
mv package/* .
rm -rf package ruvector-ruvllm-wasm-*.tgz
```

## Configuration

The frontend connects to `http://127.0.0.1:3000/mcp` by default. To change, edit the `API` constant at the top of the `<script>` block in `index.html`.

## Cross-Origin Isolation

Some WASM features (SharedArrayBuffer, full SIMD) require cross-origin isolation headers. Python's `http.server` does not set these. For full capability, serve with headers:

```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

Without these headers, the feature report will show `SharedArrayBuffer: false` and `cross_origin_isolated: false`. ParallelInference still works via transferable objects fallback.
