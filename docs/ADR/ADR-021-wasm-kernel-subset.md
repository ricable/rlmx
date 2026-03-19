# ADR-021: WASM Kernel Subset for Phone Zone D

Status: Implemented

## Context
RuVix Mesh runs agents on the user's phone via Progressive Web App or native WebView. The phone (Zone D) needs a subset of the kernel compiled to WebAssembly, complementing the existing `@ruvector/ruvllm-wasm` (which provides inference primitives). This new `rlmx-wasm` crate provides kernel syscalls in the browser.

## Decision
Create a new `rlmx-wasm` crate using `wasm-bindgen` that exposes a kernel syscall subset optimized for browser/WebView execution.

### Exposed Syscall Subset
- `VecSearch` — Vector similarity search over user preference embeddings
- `VecInsert` — Store new preference/memory vectors
- `GraphQuery` — Traverse user's life graph (people, places, products)
- `HaltCheck` — Verify agent termination conditions
- `StateMutate` — Record agent actions and outcomes
- SONA query — Pattern matching against user's local SONA bank
- Capability token validation — Enforce agent permissions client-side

### Excluded from WASM (too heavy / security-sensitive)
- `ProcessFork` / `ProcessSend` — Agent lifecycle managed by laptop coordinator
- `WitnessAppend` / `ProofSeal` — Cryptographic operations delegated to home hub
- Full TinyDancerRouter — Routing decisions made by Zone A coordinator
- `SwarmScatter` / `SwarmGather` — Cross-zone coordination via WebSocket to Zone A

### Architecture
- `wasm-bindgen` generates JS bindings from `#[wasm_bindgen]` annotated functions
- `wasm-pack build --target web` produces ES module consumable by PWA
- Memory: Uses `ruvector-core` HNSW with `rvf-quant` 4-bit quantization (50K vectors in ~25MB)
- Storage: IndexedDB backing for persistent vectors and SONA patterns
- Compute: WebGPU for embedding generation, WASM SIMD for vector operations
- Communication: WebSocket to Zone A coordinator for cross-zone operations

### Relationship to @ruvector/ruvllm-wasm
- `@ruvector/ruvllm-wasm` = inference primitives (matmul, attention, token generation)
- `rlmx-wasm` = kernel syscalls (vector search, graph query, state management, capabilities)
- Together they form the complete phone agent runtime
- Shared WebGPU context managed by `BrowserComputePool`

### Build
```
wasm-pack build -p rlmx-wasm --target web
```
Produces: `pkg/rlmx_wasm.js` + `pkg/rlmx_wasm_bg.wasm`

## Consequences

### Positive
- Agents run directly on phone with no server round-trip for common operations
- 4-bit quantized vectors make 50K user preferences fit in phone memory
- WebGPU acceleration for embedding-heavy operations
- Works offline — critical for "5 always-on agents with zero network" invariant
- PWA distribution — no app store approval needed

### Negative
- WASM binary size (~2-5MB) adds to initial PWA load
- WebGPU availability varies across mobile browsers (fallback to WASM SIMD)
- IndexedDB has storage limits on some browsers
- Reduced syscall surface means some operations require network to Zone A

### Risks
- Browser WASM memory limits (mitigated: streaming vector access, not full load)
- WebGPU spec changes (mitigated: abstraction layer in BrowserComputePool)

## References
- PRD: "RuVix Mesh — The Personal Agent Cloud", Step 2
- ADR-009: Browser WASM Compute Pool
- ADR-018: Multimodal Response Protocol
