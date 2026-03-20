//! # rlmx-wasm
//!
//! WASM kernel subset for browser/WebView execution (Zone D phone).
//!
//! Exposes a reduced syscall surface suitable for in-browser agent operation:
//! - `vec_search` / `vec_insert` — Vector similarity over user preferences
//! - `graph_query` — Traverse the user's life graph
//! - `halt_check` — Verify agent termination conditions
//! - `state_mutate` — Record agent actions and outcomes
//! - `sona_query` — Pattern matching against the local SONA bank
//! - `validate_capability` — Client-side capability token validation
//!
//! Behind the `wasm` feature gate, actual `wasm-bindgen` bindings are emitted.
//! Without it, the crate compiles as a normal Rust library for testing.
//!
//! See ADR-021 for architecture rationale.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Import kernel types — never duplicate.
use rlmx_kernel::{
    cosine_similarity, text_to_embedding, MemoryRegion, SearchFilters, SearchHit, SegmentMetadata,
    EMBED_DIM,
};

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors produced by the WASM kernel subset.
#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    #[error("capability denied: {0}")]
    CapabilityDenied(String),

    #[error("search failed: {0}")]
    SearchFailed(String),

    #[error("insert failed: {0}")]
    InsertFailed(String),

    #[error("graph error: {0}")]
    GraphError(String),

    #[error("state error: {0}")]
    StateError(String),

    #[error("sona error: {0}")]
    SonaError(String),

    #[error("quantization error: {0}")]
    QuantError(String),

    #[error("halt check error: {0}")]
    HaltError(String),
}

pub type WasmResult<T> = Result<T, WasmError>;

// ---------------------------------------------------------------------------
// Result types returned by WASM syscalls
// ---------------------------------------------------------------------------

/// Result of a vector search operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmSearchResult {
    pub hits: Vec<SearchHit>,
    pub query_embedding_dim: usize,
    pub total_segments: usize,
}

/// Result of a vector insert operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmInsertResult {
    pub segment_id: Uuid,
    pub total_segments: usize,
}

/// Result of a graph query operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmGraphResult {
    pub rows: Vec<serde_json::Value>,
    pub node_count: usize,
    pub edge_count: usize,
}

/// A capability token for client-side validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCapability {
    pub token_id: Uuid,
    pub owner: Uuid,
    pub permissions: Vec<String>,
    pub scope: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl WasmCapability {
    /// Check whether this capability grants the named permission.
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions
            .iter()
            .any(|p| p == "All" || p == permission)
    }

    /// Check whether the capability has expired.
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    /// Validate that the capability is active, belongs to the expected owner,
    /// and grants the required permission.
    pub fn validate(&self, owner: &Uuid, required_permission: &str) -> WasmResult<()> {
        if self.is_expired() {
            return Err(WasmError::CapabilityDenied("token expired".into()));
        }
        if self.owner != *owner {
            return Err(WasmError::CapabilityDenied("owner mismatch".into()));
        }
        if !self.has_permission(required_permission) {
            return Err(WasmError::CapabilityDenied(format!(
                "missing permission: {}",
                required_permission
            )));
        }
        Ok(())
    }
}

/// Result of a halt-check evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmHaltDecision {
    pub should_halt: bool,
    pub reason: String,
    pub iterations_completed: u64,
    pub max_iterations: u64,
}

/// Result of a state mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmStateMutation {
    pub key: String,
    pub previous_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// A local SONA pattern for client-side pattern matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmSonaPattern {
    pub id: Uuid,
    pub query_text: String,
    pub actions: Vec<String>,
    pub quality: f64,
    pub usage_count: usize,
}

/// Result of a SONA query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmSonaResult {
    pub matches: Vec<WasmSonaPattern>,
    pub total_patterns: usize,
}

// ---------------------------------------------------------------------------
// 4-bit quantization helpers (feature-gated)
// ---------------------------------------------------------------------------

/// Quantize an f32 embedding to 4-bit representation.
///
/// Each pair of values is packed into a single byte. The values are linearly
/// mapped from \[min, max\] to \[0, 15\].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizedEmbedding {
    pub data: Vec<u8>,
    pub min_val: f32,
    pub max_val: f32,
    pub original_len: usize,
}

/// Quantize an f32 slice to 4-bit packed bytes.
pub fn quantize_4bit(values: &[f32]) -> QuantizedEmbedding {
    if values.is_empty() {
        return QuantizedEmbedding {
            data: Vec::new(),
            min_val: 0.0,
            max_val: 0.0,
            original_len: 0,
        };
    }

    let min_val = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max_val = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let range = max_val - min_val;

    let quantized_nibbles: Vec<u8> = values
        .iter()
        .map(|&v| {
            if range == 0.0 {
                0u8
            } else {
                (((v - min_val) / range * 15.0).round() as u8).min(15)
            }
        })
        .collect();

    // Pack pairs of nibbles into bytes.
    let byte_len = quantized_nibbles.len().div_ceil(2);
    let mut data = Vec::with_capacity(byte_len);
    for chunk in quantized_nibbles.chunks(2) {
        let high = chunk[0];
        let low = if chunk.len() > 1 { chunk[1] } else { 0 };
        data.push((high << 4) | low);
    }

    QuantizedEmbedding {
        data,
        min_val,
        max_val,
        original_len: values.len(),
    }
}

/// Dequantize a 4-bit packed embedding back to f32 values.
pub fn dequantize_4bit(quantized: &QuantizedEmbedding) -> Vec<f32> {
    if quantized.original_len == 0 {
        return Vec::new();
    }

    let range = quantized.max_val - quantized.min_val;
    let mut values = Vec::with_capacity(quantized.original_len);

    for (i, &byte) in quantized.data.iter().enumerate() {
        let high = (byte >> 4) & 0x0F;
        let low = byte & 0x0F;

        let high_val = if range == 0.0 {
            quantized.min_val
        } else {
            quantized.min_val + (high as f32 / 15.0) * range
        };
        values.push(high_val);

        // Only emit the low nibble if we haven't exceeded original_len.
        if i * 2 + 1 < quantized.original_len {
            let low_val = if range == 0.0 {
                quantized.min_val
            } else {
                quantized.min_val + (low as f32 / 15.0) * range
            };
            values.push(low_val);
        }
    }

    values
}

// ---------------------------------------------------------------------------
// WasmKernel — the main struct
// ---------------------------------------------------------------------------

/// A lightweight kernel subset for browser/WebView execution.
///
/// Wraps a `MemoryRegion` for vector operations, an in-memory graph, a local
/// SONA pattern bank, a state store, and capability validation.
pub struct WasmKernel {
    /// Vector memory region for preference/memory embeddings.
    memory: MemoryRegion,
    /// In-memory graph for life-graph traversal.
    graph: rlmx_kernel::Graph,
    /// Local SONA pattern bank (query text, embedding, actions, quality).
    sona_patterns: Vec<SonaEntry>,
    /// Agent state store (key-value).
    state: HashMap<String, serde_json::Value>,
    /// Registered capability tokens.
    capabilities: HashMap<Uuid, WasmCapability>,
    /// Iteration counter for halt checks.
    iteration_count: u64,
    /// Maximum iterations before forced halt.
    max_iterations: u64,
}

/// Internal SONA entry with embedding for similarity search.
#[derive(Debug, Clone)]
struct SonaEntry {
    pub id: Uuid,
    pub query_text: String,
    pub embedding: Vec<f32>,
    pub actions: Vec<String>,
    pub quality: f64,
    pub usage_count: usize,
}

impl WasmKernel {
    /// Create a new WASM kernel with default configuration.
    pub fn new() -> Self {
        Self {
            memory: MemoryRegion::new("wasm-zone-d"),
            graph: rlmx_kernel::Graph::new(),
            sona_patterns: Vec::new(),
            state: HashMap::new(),
            capabilities: HashMap::new(),
            iteration_count: 0,
            max_iterations: 10_000,
        }
    }

    /// Create a WASM kernel with a custom iteration limit.
    pub fn with_max_iterations(max_iterations: u64) -> Self {
        Self {
            max_iterations,
            ..Self::new()
        }
    }

    // -----------------------------------------------------------------------
    // Capability validation
    // -----------------------------------------------------------------------

    /// Register a capability token for client-side validation.
    pub fn register_capability(&mut self, cap: WasmCapability) {
        self.capabilities.insert(cap.token_id, cap);
    }

    /// Validate a capability token against an owner and required permission.
    pub fn validate_capability(
        &self,
        token_id: &Uuid,
        owner: &Uuid,
        permission: &str,
    ) -> WasmResult<()> {
        let cap = self
            .capabilities
            .get(token_id)
            .ok_or_else(|| WasmError::CapabilityDenied("unknown token".into()))?;
        cap.validate(owner, permission)
    }

    // -----------------------------------------------------------------------
    // VecSearch syscall
    // -----------------------------------------------------------------------

    /// Search for the top-k most similar vectors to the query text.
    pub fn vec_search(
        &self,
        query_text: &str,
        k: usize,
        filters: &SearchFilters,
    ) -> WasmResult<WasmSearchResult> {
        let embedding = text_to_embedding(query_text);
        let hits = self.memory.search(&embedding, k, filters);
        Ok(WasmSearchResult {
            hits,
            query_embedding_dim: EMBED_DIM,
            total_segments: self.memory.len(),
        })
    }

    /// Search using a pre-computed embedding vector.
    pub fn vec_search_by_embedding(
        &self,
        embedding: &[f32],
        k: usize,
        filters: &SearchFilters,
    ) -> WasmResult<WasmSearchResult> {
        let hits = self.memory.search(embedding, k, filters);
        Ok(WasmSearchResult {
            hits,
            query_embedding_dim: embedding.len(),
            total_segments: self.memory.len(),
        })
    }

    // -----------------------------------------------------------------------
    // VecInsert syscall
    // -----------------------------------------------------------------------

    /// Insert a text segment into the vector memory.
    pub fn vec_insert(
        &mut self,
        content: &str,
        metadata: SegmentMetadata,
    ) -> WasmResult<WasmInsertResult> {
        let embedding = text_to_embedding(content);
        let segment_id = self.memory.insert(embedding, content.to_string(), metadata)
            .map_err(|e| WasmError::InsertFailed(e.to_string()))?;
        Ok(WasmInsertResult {
            segment_id,
            total_segments: self.memory.len(),
        })
    }

    /// Insert with a pre-computed embedding vector.
    pub fn vec_insert_with_embedding(
        &mut self,
        embedding: Vec<f32>,
        content: &str,
        metadata: SegmentMetadata,
    ) -> WasmResult<WasmInsertResult> {
        let segment_id = self.memory.insert(embedding, content.to_string(), metadata)
            .map_err(|e| WasmError::InsertFailed(e.to_string()))?;
        Ok(WasmInsertResult {
            segment_id,
            total_segments: self.memory.len(),
        })
    }

    // -----------------------------------------------------------------------
    // GraphQuery syscall
    // -----------------------------------------------------------------------

    /// Execute a Cypher-like query on the in-memory graph.
    pub fn graph_query(&self, cypher: &str) -> WasmResult<WasmGraphResult> {
        let rows = self
            .graph
            .cypher_query(cypher)
            .map_err(|e| WasmError::GraphError(e.to_string()))?;
        Ok(WasmGraphResult {
            rows,
            node_count: self.graph.node_count(),
            edge_count: self.graph.edge_count(),
        })
    }

    /// Insert a node into the graph and return its id.
    pub fn graph_insert_node(&mut self, node_type: &str) -> Uuid {
        self.graph.insert_node(node_type)
    }

    /// Insert a node with properties.
    pub fn graph_insert_node_with_props(
        &mut self,
        node_type: &str,
        properties: HashMap<String, serde_json::Value>,
    ) -> Uuid {
        self.graph.insert_node_with_props(node_type, properties)
    }

    /// Insert an edge between two nodes.
    pub fn graph_insert_edge(
        &mut self,
        source: Uuid,
        target: Uuid,
        edge_type: &str,
        weight: f64,
    ) -> WasmResult<()> {
        self.graph
            .insert_edge(source, target, edge_type, weight)
            .map_err(|e| WasmError::GraphError(e.to_string()))
    }

    // -----------------------------------------------------------------------
    // HaltCheck syscall
    // -----------------------------------------------------------------------

    /// Check whether the agent should halt based on iteration limits and
    /// optional quality thresholds.
    pub fn halt_check(&mut self, quality_score: Option<f64>) -> WasmHaltDecision {
        self.iteration_count += 1;

        if self.iteration_count >= self.max_iterations {
            return WasmHaltDecision {
                should_halt: true,
                reason: format!(
                    "max iterations reached ({}/{})",
                    self.iteration_count, self.max_iterations
                ),
                iterations_completed: self.iteration_count,
                max_iterations: self.max_iterations,
            };
        }

        // If a quality score is provided and it exceeds 0.95, halt early.
        if let Some(quality) = quality_score {
            if quality >= 0.95 {
                return WasmHaltDecision {
                    should_halt: true,
                    reason: format!("quality threshold met: {:.3}", quality),
                    iterations_completed: self.iteration_count,
                    max_iterations: self.max_iterations,
                };
            }
        }

        WasmHaltDecision {
            should_halt: false,
            reason: format!(
                "continuing ({}/{})",
                self.iteration_count, self.max_iterations
            ),
            iterations_completed: self.iteration_count,
            max_iterations: self.max_iterations,
        }
    }

    /// Reset the iteration counter.
    pub fn reset_iterations(&mut self) {
        self.iteration_count = 0;
    }

    // -----------------------------------------------------------------------
    // StateMutate syscall
    // -----------------------------------------------------------------------

    /// Set a key-value pair in the agent state store.
    pub fn state_mutate(
        &mut self,
        key: &str,
        value: serde_json::Value,
    ) -> WasmResult<WasmStateMutation> {
        let previous = self.state.get(key).cloned();
        let mutation = WasmStateMutation {
            key: key.to_string(),
            previous_value: previous,
            new_value: value.clone(),
            timestamp: chrono::Utc::now(),
        };
        self.state.insert(key.to_string(), value);
        Ok(mutation)
    }

    /// Retrieve a value from the agent state store.
    pub fn state_get(&self, key: &str) -> Option<&serde_json::Value> {
        self.state.get(key)
    }

    /// List all keys in the state store.
    pub fn state_keys(&self) -> Vec<String> {
        self.state.keys().cloned().collect()
    }

    // -----------------------------------------------------------------------
    // SONA query
    // -----------------------------------------------------------------------

    /// Record a SONA pattern in the local bank.
    pub fn sona_record(&mut self, query_text: &str, actions: Vec<String>, quality: f64) -> Uuid {
        let id = Uuid::new_v4();
        let embedding = text_to_embedding(query_text);
        self.sona_patterns.push(SonaEntry {
            id,
            query_text: query_text.to_string(),
            embedding,
            actions,
            quality: quality.clamp(0.0, 1.0),
            usage_count: 0,
        });
        id
    }

    /// Query the local SONA bank for the top-k most similar patterns.
    pub fn sona_query(&mut self, query_text: &str, k: usize) -> WasmSonaResult {
        let query_emb = text_to_embedding(query_text);

        let mut scored: Vec<(usize, f64)> = self
            .sona_patterns
            .iter()
            .enumerate()
            .map(|(i, entry)| (i, cosine_similarity(&query_emb, &entry.embedding)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_indices: Vec<usize> = scored.iter().take(k).map(|(i, _)| *i).collect();

        // Increment usage counts.
        for &idx in &top_indices {
            self.sona_patterns[idx].usage_count += 1;
        }

        let matches: Vec<WasmSonaPattern> = top_indices
            .iter()
            .map(|&idx| {
                let entry = &self.sona_patterns[idx];
                WasmSonaPattern {
                    id: entry.id,
                    query_text: entry.query_text.clone(),
                    actions: entry.actions.clone(),
                    quality: entry.quality,
                    usage_count: entry.usage_count,
                }
            })
            .collect();

        WasmSonaResult {
            total_patterns: self.sona_patterns.len(),
            matches,
        }
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// Number of segments in the vector memory.
    pub fn segment_count(&self) -> usize {
        self.memory.len()
    }

    /// Number of SONA patterns in the local bank.
    pub fn sona_pattern_count(&self) -> usize {
        self.sona_patterns.len()
    }

    /// Number of nodes in the graph.
    pub fn graph_node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Number of edges in the graph.
    pub fn graph_edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Current iteration count.
    pub fn current_iterations(&self) -> u64 {
        self.iteration_count
    }
}

impl Default for WasmKernel {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// wasm-bindgen bindings (feature = "wasm")
// ---------------------------------------------------------------------------

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmKernelJs {
    inner: WasmKernel,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmKernelJs {
    /// Create a new WASM kernel instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: WasmKernel::new(),
        }
    }

    /// Search for similar vectors. Returns JSON string.
    #[wasm_bindgen(js_name = "vecSearch")]
    pub fn vec_search(&self, query: &str, k: usize) -> Result<String, JsValue> {
        let result = self
            .inner
            .vec_search(query, k, &SearchFilters::default())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Insert a text segment. Returns JSON string with segment id.
    #[wasm_bindgen(js_name = "vecInsert")]
    pub fn vec_insert(&mut self, content: &str, source: &str) -> Result<String, JsValue> {
        let meta = SegmentMetadata {
            source: source.to_string(),
            ..Default::default()
        };
        let result = self
            .inner
            .vec_insert(content, meta)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Execute a graph query. Returns JSON string.
    #[wasm_bindgen(js_name = "graphQuery")]
    pub fn graph_query(&self, cypher: &str) -> Result<String, JsValue> {
        let result = self
            .inner
            .graph_query(cypher)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Check halt condition. Returns JSON string.
    #[wasm_bindgen(js_name = "haltCheck")]
    pub fn halt_check(&mut self, quality_score: Option<f64>) -> String {
        let result = self.inner.halt_check(quality_score);
        serde_json::to_string(&result).unwrap_or_default()
    }

    /// Mutate agent state. Returns JSON string.
    #[wasm_bindgen(js_name = "stateMutate")]
    pub fn state_mutate(&mut self, key: &str, value_json: &str) -> Result<String, JsValue> {
        let value: serde_json::Value =
            serde_json::from_str(value_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let result = self
            .inner
            .state_mutate(key, value)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Query SONA patterns. Returns JSON string.
    #[wasm_bindgen(js_name = "sonaQuery")]
    pub fn sona_query(&mut self, query: &str, k: usize) -> String {
        let result = self.inner.sona_query(query, k);
        serde_json::to_string(&result).unwrap_or_default()
    }

    /// Number of stored segments.
    #[wasm_bindgen(js_name = "segmentCount")]
    pub fn segment_count(&self) -> usize {
        self.inner.segment_count()
    }
}

#[cfg(feature = "wasm")]
impl Default for WasmKernelJs {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_meta(source: &str) -> SegmentMetadata {
        SegmentMetadata {
            source: source.to_string(),
            plugin: None,
            segment_type: "text".into(),
            extra: Default::default(),
        }
    }

    #[test]
    fn test_vec_insert_and_search() {
        let mut kernel = WasmKernel::new();
        let result = kernel
            .vec_insert("best restaurants in Paris", make_meta("user"))
            .unwrap();
        assert!(!result.segment_id.is_nil());
        assert_eq!(result.total_segments, 1);

        let search = kernel
            .vec_search("restaurants Paris", 5, &SearchFilters::default())
            .unwrap();
        assert_eq!(search.hits.len(), 1);
        assert!(search.hits[0].score > 0.5);
    }

    #[test]
    fn test_vec_search_empty_region() {
        let kernel = WasmKernel::new();
        let search = kernel
            .vec_search("anything", 5, &SearchFilters::default())
            .unwrap();
        assert!(search.hits.is_empty());
        assert_eq!(search.total_segments, 0);
    }

    #[test]
    fn test_vec_search_with_filters() {
        let mut kernel = WasmKernel::new();
        kernel
            .vec_insert("Paris hotel booking", make_meta("travel"))
            .unwrap();
        kernel
            .vec_insert("Paris restaurant reviews", make_meta("food"))
            .unwrap();

        let filters = SearchFilters {
            source: Some("travel".into()),
            ..Default::default()
        };
        let search = kernel.vec_search("Paris", 5, &filters).unwrap();
        assert_eq!(search.hits.len(), 1);
        assert_eq!(search.hits[0].metadata.source, "travel");
    }

    #[test]
    fn test_vec_insert_with_embedding() {
        let mut kernel = WasmKernel::new();
        let mut emb = vec![0.0_f32; 64];
        emb[0] = 1.0;
        let result = kernel
            .vec_insert_with_embedding(emb, "custom vector", make_meta("test"))
            .unwrap();
        assert!(!result.segment_id.is_nil());
        assert_eq!(result.total_segments, 1);
    }

    #[test]
    fn test_graph_query() {
        let mut kernel = WasmKernel::new();
        let a = kernel.graph_insert_node("Person");
        let b = kernel.graph_insert_node("Restaurant");
        kernel.graph_insert_edge(a, b, "VISITED", 1.0).unwrap();

        let result = kernel
            .graph_query("MATCH (n:Person)-[r:VISITED]->(m:Restaurant) RETURN n,m")
            .unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.node_count, 2);
        assert_eq!(result.edge_count, 1);
    }

    #[test]
    fn test_graph_query_invalid_cypher() {
        let kernel = WasmKernel::new();
        let result = kernel.graph_query("SELECT * FROM nodes");
        assert!(result.is_err());
    }

    #[test]
    fn test_halt_check_continues() {
        let mut kernel = WasmKernel::with_max_iterations(100);
        let decision = kernel.halt_check(None);
        assert!(!decision.should_halt);
        assert_eq!(decision.iterations_completed, 1);
    }

    #[test]
    fn test_halt_check_max_iterations() {
        let mut kernel = WasmKernel::with_max_iterations(2);
        kernel.halt_check(None);
        let decision = kernel.halt_check(None);
        assert!(decision.should_halt);
        assert!(decision.reason.contains("max iterations"));
    }

    #[test]
    fn test_halt_check_quality_threshold() {
        let mut kernel = WasmKernel::with_max_iterations(1000);
        let decision = kernel.halt_check(Some(0.96));
        assert!(decision.should_halt);
        assert!(decision.reason.contains("quality threshold"));
    }

    #[test]
    fn test_state_mutate_and_get() {
        let mut kernel = WasmKernel::new();
        let mutation = kernel
            .state_mutate("agent.status", serde_json::json!("running"))
            .unwrap();
        assert!(mutation.previous_value.is_none());
        assert_eq!(mutation.new_value, serde_json::json!("running"));

        let val = kernel.state_get("agent.status").unwrap();
        assert_eq!(val, &serde_json::json!("running"));

        // Overwrite returns previous value.
        let mutation2 = kernel
            .state_mutate("agent.status", serde_json::json!("complete"))
            .unwrap();
        assert_eq!(mutation2.previous_value, Some(serde_json::json!("running")));
    }

    #[test]
    fn test_state_keys() {
        let mut kernel = WasmKernel::new();
        kernel.state_mutate("a", serde_json::json!(1)).unwrap();
        kernel.state_mutate("b", serde_json::json!(2)).unwrap();
        let mut keys = kernel.state_keys();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn test_sona_record_and_query() {
        let mut kernel = WasmKernel::new();
        kernel.sona_record(
            "book a flight to Tokyo",
            vec!["search_flights".into(), "compare_prices".into()],
            0.9,
        );
        kernel.sona_record("find a hotel in London", vec!["search_hotels".into()], 0.85);
        kernel.sona_record(
            "pay my electricity bill",
            vec!["find_provider".into(), "process_payment".into()],
            0.8,
        );

        let result = kernel.sona_query("flights to Japan", 2);
        assert_eq!(result.total_patterns, 3);
        assert_eq!(result.matches.len(), 2);
        // The flight pattern should rank highest for a flight query.
        assert!(result.matches[0].query_text.contains("flight"));
    }

    #[test]
    fn test_sona_empty_bank() {
        let mut kernel = WasmKernel::new();
        let result = kernel.sona_query("anything", 5);
        assert!(result.matches.is_empty());
        assert_eq!(result.total_patterns, 0);
    }

    #[test]
    fn test_capability_validation() {
        let mut kernel = WasmKernel::new();
        let owner = Uuid::new_v4();
        let token_id = Uuid::new_v4();

        let cap = WasmCapability {
            token_id,
            owner,
            permissions: vec!["VecSearch".into(), "VecInsert".into()],
            scope: "zone-d".into(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };
        kernel.register_capability(cap);

        // Valid permission.
        assert!(kernel
            .validate_capability(&token_id, &owner, "VecSearch")
            .is_ok());

        // Missing permission.
        assert!(kernel
            .validate_capability(&token_id, &owner, "ProcessFork")
            .is_err());

        // Wrong owner.
        let wrong_owner = Uuid::new_v4();
        assert!(kernel
            .validate_capability(&token_id, &wrong_owner, "VecSearch")
            .is_err());

        // Unknown token.
        let unknown = Uuid::new_v4();
        assert!(kernel
            .validate_capability(&unknown, &owner, "VecSearch")
            .is_err());
    }

    #[test]
    fn test_capability_all_permission() {
        let mut kernel = WasmKernel::new();
        let owner = Uuid::new_v4();
        let token_id = Uuid::new_v4();

        let cap = WasmCapability {
            token_id,
            owner,
            permissions: vec!["All".into()],
            scope: "admin".into(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };
        kernel.register_capability(cap);

        // "All" grants every permission.
        assert!(kernel
            .validate_capability(&token_id, &owner, "VecSearch")
            .is_ok());
        assert!(kernel
            .validate_capability(&token_id, &owner, "ProcessFork")
            .is_ok());
    }

    #[test]
    fn test_capability_expired() {
        let mut kernel = WasmKernel::new();
        let owner = Uuid::new_v4();
        let token_id = Uuid::new_v4();

        let cap = WasmCapability {
            token_id,
            owner,
            permissions: vec!["VecSearch".into()],
            scope: "zone-d".into(),
            // Already expired.
            expires_at: chrono::Utc::now() - chrono::Duration::hours(1),
        };
        kernel.register_capability(cap);

        assert!(kernel
            .validate_capability(&token_id, &owner, "VecSearch")
            .is_err());
    }

    #[test]
    fn test_quantize_dequantize_roundtrip() {
        let values = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        let quantized = quantize_4bit(&values);
        assert_eq!(quantized.original_len, 5);

        let restored = dequantize_4bit(&quantized);
        assert_eq!(restored.len(), 5);

        // 4-bit quantization has ~1/15 precision.
        for (orig, restored) in values.iter().zip(restored.iter()) {
            assert!(
                (orig - restored).abs() < 0.08,
                "orig={}, restored={}",
                orig,
                restored
            );
        }
    }

    #[test]
    fn test_quantize_empty() {
        let quantized = quantize_4bit(&[]);
        assert_eq!(quantized.original_len, 0);
        let restored = dequantize_4bit(&quantized);
        assert!(restored.is_empty());
    }

    #[test]
    fn test_quantize_constant_values() {
        let values = vec![0.5, 0.5, 0.5, 0.5];
        let quantized = quantize_4bit(&values);
        let restored = dequantize_4bit(&quantized);
        assert_eq!(restored.len(), 4);
        for v in &restored {
            assert!((v - 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn test_quantize_odd_length() {
        let values = vec![0.0, 0.5, 1.0];
        let quantized = quantize_4bit(&values);
        assert_eq!(quantized.data.len(), 2); // ceil(3/2) = 2 bytes
        let restored = dequantize_4bit(&quantized);
        assert_eq!(restored.len(), 3);
    }

    #[test]
    fn test_default_kernel() {
        let kernel = WasmKernel::default();
        assert_eq!(kernel.segment_count(), 0);
        assert_eq!(kernel.sona_pattern_count(), 0);
        assert_eq!(kernel.graph_node_count(), 0);
        assert_eq!(kernel.graph_edge_count(), 0);
        assert_eq!(kernel.current_iterations(), 0);
    }

    #[test]
    fn test_reset_iterations() {
        let mut kernel = WasmKernel::with_max_iterations(100);
        kernel.halt_check(None);
        kernel.halt_check(None);
        assert_eq!(kernel.current_iterations(), 2);
        kernel.reset_iterations();
        assert_eq!(kernel.current_iterations(), 0);
    }
}
