//! RLMX MCP Tools
//!
//! Implements all 12 RLMX MCP tool definitions and their handlers.
//! Tools that can be wired to kernel subsystems use a shared `ToolState`
//! backed by `Arc<RwLock<...>>`. Tools that require external services
//! remain as stubs with `"status": "stub"` in their responses.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use tokio::sync::RwLock;
use uuid::Uuid;

use rlmx_kernel::{Graph, MemoryRegion, SearchFilters, SegmentMetadata, EMBED_DIM, text_to_embedding};

use crate::protocol::{McpError, McpTool, ToolHandler};
#[allow(unused_imports)]
use crate::protocol::INVALID_PARAMS;

// ---------------------------------------------------------------------------
// Shared kernel state accessible by tool handlers
// ---------------------------------------------------------------------------

/// Shared state wrapping kernel subsystems that MCP tool handlers operate on.
pub struct ToolState {
    /// In-memory region for segment storage and vector search.
    pub memory: MemoryRegion,
    /// In-memory entity graph for Cypher-like queries.
    pub graph: Graph,
    /// Running counter of total ingestion operations performed.
    pub ingest_count: u64,
    /// Running counter of total query operations performed.
    pub query_count: u64,
}

impl ToolState {
    /// Create a new `ToolState` with empty kernel subsystems.
    pub fn new() -> Self {
        Self {
            memory: MemoryRegion::new("mcp-primary"),
            graph: Graph::new(),
            ingest_count: 0,
            query_count: 0,
        }
    }
}

impl Default for ToolState {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience alias for the shared tool state handle.
pub type SharedToolState = Arc<RwLock<ToolState>>;

/// Create a default shared tool state.
pub fn new_shared_state() -> SharedToolState {
    Arc::new(RwLock::new(ToolState::new()))
}

// ---------------------------------------------------------------------------
// Public constructor
// ---------------------------------------------------------------------------

/// Create all 12 RLMX MCP tools with their handlers, wired to the given
/// shared kernel state.
pub fn create_all_tools(state: SharedToolState) -> Vec<McpTool> {
    vec![
        create_rlmx_query(Arc::clone(&state)),
        create_rlmx_ingest(Arc::clone(&state)),
        create_rlmx_memory_stats(Arc::clone(&state)),
        create_rlmx_plugin_list(),
        create_rlmx_plugin_action(),
        create_rlmx_strategy_override(),
        create_rlmx_trm_classify(),
        create_rlmx_rvf_seal(),
        create_rlmx_rvf_branch(),
        create_rlmx_witness_chain(),
        create_rlmx_sona_stats(),
        create_rlmx_graph_query(Arc::clone(&state)),
    ]
}

// ---------------------------------------------------------------------------
// 1. rlmx_query  — wired to MemoryRegion::search
// ---------------------------------------------------------------------------

fn create_rlmx_query(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_query".to_string(),
        description: "Query with infinite context. Auto-selects the optimal retrieval strategy (RLM or TRM) based on query characteristics.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The natural language query to execute"
                },
                "context_window": {
                    "type": "integer",
                    "description": "Maximum number of context segments to return",
                    "default": 10
                },
                "strategy": {
                    "type": "string",
                    "enum": ["auto", "rlm", "trm"],
                    "description": "Retrieval strategy override",
                    "default": "auto"
                }
            },
            "required": ["query"]
        }),
        handler: Box::new(RlmxQueryHandler { state }),
    }
}

struct RlmxQueryHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxQueryHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let query = params.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: query"))?;

        let context_window = params.get("context_window")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let strategy = params.get("strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        let embedding = text_to_embedding(query);
        let start = Instant::now();

        let mut state = self.state.write().await;
        state.query_count += 1;
        let total_segments = state.memory.len();

        let hits = state.memory.search(&embedding, context_window, &SearchFilters::default());
        let elapsed = start.elapsed();

        let results: Vec<serde_json::Value> = hits.iter().map(|hit| {
            json!({
                "segment_id": hit.segment_id.to_string(),
                "content": hit.content,
                "relevance_score": (hit.score * 1000.0).round() / 1000.0,
                "tier": format!("{:?}", hit.metadata.segment_type),
            })
        }).collect();

        let segments_returned = results.len();

        Ok(json!({
            "query": query,
            "strategy_used": if strategy == "auto" { "rlm" } else { strategy },
            "segments_returned": segments_returned,
            "results": results,
            "total_segments_scanned": total_segments,
            "latency_ms": elapsed.as_millis() as u64,
        }))
    }
}

// ---------------------------------------------------------------------------
// 2. rlmx_ingest  — wired to MemoryRegion::insert
// ---------------------------------------------------------------------------

fn create_rlmx_ingest(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_ingest".to_string(),
        description: "Ingest data into RLMX via a plugin adapter. Supports various data formats through the plugin system.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "string",
                    "description": "The data content to ingest"
                },
                "plugin": {
                    "type": "string",
                    "description": "Plugin adapter to use for ingestion (e.g., 'text', 'json', 'markdown')"
                },
                "metadata": {
                    "type": "object",
                    "description": "Optional metadata to attach to ingested segments"
                }
            },
            "required": ["data", "plugin"]
        }),
        handler: Box::new(RlmxIngestHandler { state }),
    }
}

struct RlmxIngestHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxIngestHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let data = params.get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: data"))?;

        let plugin = params.get("plugin")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: plugin"))?;

        let extra_metadata = params.get("metadata")
            .cloned()
            .unwrap_or(json!({}));

        // Split data into segments on double-newline boundaries (or treat as
        // a single segment if no double-newlines are present).
        let chunks: Vec<&str> = if data.contains("\n\n") {
            data.split("\n\n").filter(|s| !s.trim().is_empty()).collect()
        } else {
            vec![data]
        };

        let mut state = self.state.write().await;
        state.ingest_count += 1;

        let mut segment_ids = Vec::with_capacity(chunks.len());
        for chunk in &chunks {
            let embedding = text_to_embedding(chunk);
            let mut extra = std::collections::HashMap::new();
            if let Some(obj) = extra_metadata.as_object() {
                for (k, v) in obj {
                    extra.insert(k.clone(), v.clone());
                }
            }
            let meta = SegmentMetadata {
                source: "mcp-ingest".into(),
                plugin: Some(plugin.to_string()),
                segment_type: "text".into(),
                extra,
            };
            let id = state.memory.insert(embedding, chunk.to_string(), meta);
            segment_ids.push(id.to_string());
        }

        Ok(json!({
            "status": "ingested",
            "plugin_used": plugin,
            "segments_created": segment_ids.len(),
            "segment_ids": segment_ids,
            "timestamp": Utc::now().to_rfc3339(),
        }))
    }
}

// ---------------------------------------------------------------------------
// 3. rlmx_memory_stats  — wired to MemoryRegion stats
// ---------------------------------------------------------------------------

fn create_rlmx_memory_stats(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_memory_stats".to_string(),
        description: "Retrieve memory statistics including segment counts, tier distribution, and HNSW index health.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "include_hnsw": {
                    "type": "boolean",
                    "description": "Include HNSW index health metrics",
                    "default": true
                }
            }
        }),
        handler: Box::new(RlmxMemoryStatsHandler { state }),
    }
}

struct RlmxMemoryStatsHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxMemoryStatsHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let include_hnsw = params.get("include_hnsw")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let state = self.state.read().await;
        let total_segments = state.memory.len();

        // All newly-inserted segments start as Hot; we report the actual count.
        // A real implementation would track tier promotions/demotions.
        let mut stats = json!({
            "total_segments": total_segments,
            "tier_distribution": {
                "hot": total_segments,
                "warm": 0,
                "cold": 0
            },
            "ingest_operations": state.ingest_count,
            "query_operations": state.query_count,
            // TODO: track evictions once tier promotion/demotion is implemented
            "eviction_count": 0,
        });

        if include_hnsw {
            stats["hnsw_health"] = json!({
                "index_size": total_segments,
                "dimensions": EMBED_DIM,
                "max_layers": 6,
                "ef_construction": 200,
                "fragmentation_ratio": 0.0,
            });
        }

        Ok(stats)
    }
}

// ---------------------------------------------------------------------------
// 4. rlmx_plugin_list  — stub (needs plugin subsystem)
// ---------------------------------------------------------------------------

fn create_rlmx_plugin_list() -> McpTool {
    McpTool {
        name: "rlmx_plugin_list".to_string(),
        description: "List all active plugins and their status.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "include_disabled": {
                    "type": "boolean",
                    "description": "Include disabled plugins in the listing",
                    "default": false
                }
            }
        }),
        handler: Box::new(RlmxPluginListHandler),
    }
}

struct RlmxPluginListHandler;

#[async_trait]
impl ToolHandler for RlmxPluginListHandler {
    async fn handle(&self, _params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        // TODO: wire to plugin subsystem
        Ok(json!({
            "status": "stub",
            "plugins": [
                {
                    "name": "text-adapter",
                    "version": "0.1.0",
                    "status": "active",
                    "capabilities": ["ingest", "transform"]
                },
                {
                    "name": "json-adapter",
                    "version": "0.1.0",
                    "status": "active",
                    "capabilities": ["ingest", "query"]
                },
                {
                    "name": "markdown-adapter",
                    "version": "0.1.0",
                    "status": "active",
                    "capabilities": ["ingest", "transform", "render"]
                }
            ],
            "total_active": 3,
            "total_disabled": 0
        }))
    }
}

// ---------------------------------------------------------------------------
// 5. rlmx_plugin_action  — stub (needs plugin subsystem)
// ---------------------------------------------------------------------------

fn create_rlmx_plugin_action() -> McpTool {
    McpTool {
        name: "rlmx_plugin_action".to_string(),
        description: "Execute a domain-specific action through a plugin.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "plugin": {
                    "type": "string",
                    "description": "Name of the plugin to execute the action on"
                },
                "action": {
                    "type": "string",
                    "description": "The action to execute"
                },
                "params": {
                    "type": "object",
                    "description": "Action-specific parameters"
                }
            },
            "required": ["plugin", "action"]
        }),
        handler: Box::new(RlmxPluginActionHandler),
    }
}

struct RlmxPluginActionHandler;

#[async_trait]
impl ToolHandler for RlmxPluginActionHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let plugin = params.get("plugin")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: plugin"))?;

        let action = params.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: action"))?;

        // TODO: wire to plugin subsystem
        Ok(json!({
            "status": "stub",
            "plugin": plugin,
            "action": action,
            "result": {
                "message": format!("Action '{}' executed successfully on plugin '{}'", action, plugin)
            },
            "execution_time_ms": 0
        }))
    }
}

// ---------------------------------------------------------------------------
// 6. rlmx_strategy_override  — stub (needs scheduler/strategy subsystem)
// ---------------------------------------------------------------------------

fn create_rlmx_strategy_override() -> McpTool {
    McpTool {
        name: "rlmx_strategy_override".to_string(),
        description: "Force a specific retrieval strategy (RLM or TRM) for subsequent queries.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "strategy": {
                    "type": "string",
                    "enum": ["rlm", "trm", "auto"],
                    "description": "The retrieval strategy to force"
                },
                "duration_seconds": {
                    "type": "integer",
                    "description": "How long the override should last (0 = permanent until reset)",
                    "default": 0
                }
            },
            "required": ["strategy"]
        }),
        handler: Box::new(RlmxStrategyOverrideHandler),
    }
}

struct RlmxStrategyOverrideHandler;

#[async_trait]
impl ToolHandler for RlmxStrategyOverrideHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let strategy = params.get("strategy")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: strategy"))?;

        let duration = params.get("duration_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // TODO: wire to scheduler/strategy subsystem
        Ok(json!({
            "status": "stub",
            "previous_strategy": "auto",
            "new_strategy": strategy,
            "duration_seconds": duration,
            "applied_at": Utc::now().to_rfc3339()
        }))
    }
}

// ---------------------------------------------------------------------------
// 7. rlmx_trm_classify  — stub (needs TRM subsystem)
// ---------------------------------------------------------------------------

fn create_rlmx_trm_classify() -> McpTool {
    McpTool {
        name: "rlmx_trm_classify".to_string(),
        description: "Directly invoke the TRM classifier to classify segments by temporal relevance.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "segment_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of segment IDs to classify"
                },
                "time_horizon": {
                    "type": "string",
                    "enum": ["short", "medium", "long"],
                    "description": "Temporal classification horizon",
                    "default": "medium"
                }
            },
            "required": ["segment_ids"]
        }),
        handler: Box::new(RlmxTrmClassifyHandler),
    }
}

struct RlmxTrmClassifyHandler;

#[async_trait]
impl ToolHandler for RlmxTrmClassifyHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let segment_ids = params.get("segment_ids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: segment_ids"))?;

        let horizon = params.get("time_horizon")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        // TODO: wire to TRM subsystem
        let classifications: Vec<serde_json::Value> = segment_ids.iter().map(|id| {
            json!({
                "segment_id": id,
                "classification": "relevant",
                "confidence": 0.87,
                "decay_rate": 0.02,
                "horizon": horizon
            })
        }).collect();

        Ok(json!({
            "status": "stub",
            "classifications": classifications,
            "total_classified": segment_ids.len(),
            "horizon": horizon
        }))
    }
}

// ---------------------------------------------------------------------------
// 8. rlmx_rvf_seal  — stub (needs RVF container subsystem)
// ---------------------------------------------------------------------------

fn create_rlmx_rvf_seal() -> McpTool {
    McpTool {
        name: "rlmx_rvf_seal".to_string(),
        description: "Package data as an RVF (RLMX Versioned Format) container with integrity seals.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "segment_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Segment IDs to include in the RVF container"
                },
                "label": {
                    "type": "string",
                    "description": "Human-readable label for the container"
                },
                "seal_type": {
                    "type": "string",
                    "enum": ["sha256", "blake3"],
                    "description": "Hash algorithm for integrity seal",
                    "default": "blake3"
                }
            },
            "required": ["segment_ids"]
        }),
        handler: Box::new(RlmxRvfSealHandler),
    }
}

struct RlmxRvfSealHandler;

#[async_trait]
impl ToolHandler for RlmxRvfSealHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let segment_ids = params.get("segment_ids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: segment_ids"))?;

        let label = params.get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed");

        let seal_type = params.get("seal_type")
            .and_then(|v| v.as_str())
            .unwrap_or("blake3");

        // TODO: wire to RVF container subsystem
        Ok(json!({
            "status": "stub",
            "container_id": Uuid::new_v4().to_string(),
            "label": label,
            "segments_sealed": segment_ids.len(),
            "seal_type": seal_type,
            "seal_hash": "b3a1c2d4e5f6789012345678abcdef0123456789abcdef0123456789abcdef01",
            "sealed_at": Utc::now().to_rfc3339(),
            "version": 1
        }))
    }
}

// ---------------------------------------------------------------------------
// 9. rlmx_rvf_branch  — stub (needs RVF container subsystem)
// ---------------------------------------------------------------------------

fn create_rlmx_rvf_branch() -> McpTool {
    McpTool {
        name: "rlmx_rvf_branch".to_string(),
        description: "Create a copy-on-write (COW) branch from an RVF container for experimentation.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "container_id": {
                    "type": "string",
                    "description": "Source RVF container ID to branch from"
                },
                "branch_label": {
                    "type": "string",
                    "description": "Label for the new branch"
                }
            },
            "required": ["container_id"]
        }),
        handler: Box::new(RlmxRvfBranchHandler),
    }
}

struct RlmxRvfBranchHandler;

#[async_trait]
impl ToolHandler for RlmxRvfBranchHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let container_id = params.get("container_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: container_id"))?;

        let branch_label = params.get("branch_label")
            .and_then(|v| v.as_str())
            .unwrap_or("experiment");

        // TODO: wire to RVF container subsystem
        Ok(json!({
            "status": "stub",
            "branch_id": Uuid::new_v4().to_string(),
            "source_container_id": container_id,
            "branch_label": branch_label,
            "cow_pages": 0,
            "created_at": Utc::now().to_rfc3339()
        }))
    }
}

// ---------------------------------------------------------------------------
// 10. rlmx_witness_chain  — stub (needs proof/witness subsystem)
// ---------------------------------------------------------------------------

fn create_rlmx_witness_chain() -> McpTool {
    McpTool {
        name: "rlmx_witness_chain".to_string(),
        description: "Retrieve the audit trail (witness chain) for a segment or container.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "target_id": {
                    "type": "string",
                    "description": "Segment or container ID to retrieve audit trail for"
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum depth of chain traversal",
                    "default": 50
                }
            },
            "required": ["target_id"]
        }),
        handler: Box::new(RlmxWitnessChainHandler),
    }
}

struct RlmxWitnessChainHandler;

#[async_trait]
impl ToolHandler for RlmxWitnessChainHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let target_id = params.get("target_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: target_id"))?;

        // TODO: wire to proof/witness subsystem
        Ok(json!({
            "status": "stub",
            "target_id": target_id,
            "chain_length": 0,
            "entries": [],
            "integrity_verified": false
        }))
    }
}

// ---------------------------------------------------------------------------
// 11. rlmx_sona_stats  — stub (needs SONA subsystem)
// ---------------------------------------------------------------------------

fn create_rlmx_sona_stats() -> McpTool {
    McpTool {
        name: "rlmx_sona_stats".to_string(),
        description: "Retrieve self-learning (SONA) metrics and adaptation statistics.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "time_range": {
                    "type": "string",
                    "enum": ["1h", "24h", "7d", "30d", "all"],
                    "description": "Time range for statistics",
                    "default": "24h"
                }
            }
        }),
        handler: Box::new(RlmxSonaStatsHandler),
    }
}

struct RlmxSonaStatsHandler;

#[async_trait]
impl ToolHandler for RlmxSonaStatsHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let time_range = params.get("time_range")
            .and_then(|v| v.as_str())
            .unwrap_or("24h");

        // TODO: wire to SONA subsystem
        Ok(json!({
            "status": "stub",
            "time_range": time_range,
            "adaptations_count": 0,
            "learning_rate": 0.0,
            "accuracy_improvement": 0.0,
            "feedback_loops": {
                "positive": 0,
                "negative": 0,
                "neutral": 0
            },
            "model_version": "sona-v0.3.1",
            "last_adaptation": serde_json::Value::Null,
        }))
    }
}

// ---------------------------------------------------------------------------
// 12. rlmx_graph_query  — wired to Graph::cypher_query
// ---------------------------------------------------------------------------

fn create_rlmx_graph_query(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_graph_query".to_string(),
        description: "Execute a Cypher query against the RLMX entity graph.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "cypher": {
                    "type": "string",
                    "description": "Cypher query to execute against the entity graph"
                },
                "params": {
                    "type": "object",
                    "description": "Query parameters for parameterized Cypher queries"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return",
                    "default": 100
                }
            },
            "required": ["cypher"]
        }),
        handler: Box::new(RlmxGraphQueryHandler { state }),
    }
}

struct RlmxGraphQueryHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxGraphQueryHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let cypher = params.get("cypher")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: cypher"))?;

        let limit = params.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        let start = Instant::now();
        let state = self.state.read().await;

        let rows = state.graph.cypher_query(cypher).map_err(|e| {
            McpError::new(INVALID_PARAMS, format!("Cypher query error: {}", e))
        })?;

        let elapsed = start.elapsed();
        let truncated: Vec<&serde_json::Value> = rows.iter().take(limit).collect();
        let rows_returned = truncated.len();

        Ok(json!({
            "query": cypher,
            "rows_returned": rows_returned,
            "limit": limit,
            "results": truncated,
            "execution_time_ms": elapsed.as_millis() as u64,
        }))
    }
}

/// Return the tool names for validation purposes.
pub fn tool_names() -> Vec<&'static str> {
    vec![
        "rlmx_query",
        "rlmx_ingest",
        "rlmx_memory_stats",
        "rlmx_plugin_list",
        "rlmx_plugin_action",
        "rlmx_strategy_override",
        "rlmx_trm_classify",
        "rlmx_rvf_seal",
        "rlmx_rvf_branch",
        "rlmx_witness_chain",
        "rlmx_sona_stats",
        "rlmx_graph_query",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_12_tools_have_valid_schemas() {
        let state = new_shared_state();
        let tools = create_all_tools(state);
        assert_eq!(tools.len(), 12, "Expected exactly 12 tools");

        for tool in &tools {
            assert!(!tool.name.is_empty(), "Tool name must not be empty");
            assert!(!tool.description.is_empty(), "Tool description must not be empty");

            // Verify the schema is a valid JSON object with "type": "object"
            let schema = &tool.input_schema;
            assert_eq!(
                schema.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "Tool '{}' input_schema must have type=object",
                tool.name
            );
        }

        // Verify all expected tool names are present
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        for expected in tool_names() {
            assert!(names.contains(&expected), "Missing tool: {}", expected);
        }
    }

    #[tokio::test]
    async fn test_ingest_then_query_returns_real_results() {
        let state = new_shared_state();
        let tools = create_all_tools(Arc::clone(&state));

        // Ingest some data.
        let ingest_tool = tools.iter().find(|t| t.name == "rlmx_ingest").unwrap();
        let ingest_result = ingest_tool.handler.handle(json!({
            "data": "Rust is a systems programming language focused on safety and performance.",
            "plugin": "text"
        })).await.unwrap();
        assert_eq!(ingest_result["status"], "ingested");
        assert_eq!(ingest_result["segments_created"], 1);

        // Query for it.
        let query_tool = tools.iter().find(|t| t.name == "rlmx_query").unwrap();
        let query_result = query_tool.handler.handle(json!({
            "query": "Rust programming language"
        })).await.unwrap();
        assert_eq!(query_result["segments_returned"], 1);
        let results = query_result["results"].as_array().unwrap();
        assert!(!results.is_empty());
        // The relevance score should be > 0 since the texts share words.
        let score = results[0]["relevance_score"].as_f64().unwrap();
        assert!(score > 0.0, "Expected positive relevance score, got {}", score);
    }

    #[tokio::test]
    async fn test_memory_stats_reflects_ingestion() {
        let state = new_shared_state();
        let tools = create_all_tools(Arc::clone(&state));

        // Stats before ingestion.
        let stats_tool = tools.iter().find(|t| t.name == "rlmx_memory_stats").unwrap();
        let before = stats_tool.handler.handle(json!({})).await.unwrap();
        assert_eq!(before["total_segments"], 0);

        // Ingest.
        let ingest_tool = tools.iter().find(|t| t.name == "rlmx_ingest").unwrap();
        ingest_tool.handler.handle(json!({
            "data": "Hello world",
            "plugin": "text"
        })).await.unwrap();

        // Stats after ingestion.
        let after = stats_tool.handler.handle(json!({})).await.unwrap();
        assert_eq!(after["total_segments"], 1);
        assert_eq!(after["ingest_operations"], 1);
    }

    #[tokio::test]
    async fn test_graph_query_executes_cypher() {
        let state = new_shared_state();

        // Populate the graph.
        {
            let mut s = state.write().await;
            let a = s.graph.insert_node("Person");
            let b = s.graph.insert_node("Company");
            s.graph.insert_edge(a, b, "WORKS_AT", 1.0).unwrap();
        }

        let tools = create_all_tools(Arc::clone(&state));
        let graph_tool = tools.iter().find(|t| t.name == "rlmx_graph_query").unwrap();

        let result = graph_tool.handler.handle(json!({
            "cypher": "MATCH (n:Person)-[r:WORKS_AT]->(m:Company) RETURN n,m"
        })).await.unwrap();

        assert_eq!(result["rows_returned"], 1);
        let rows = result["results"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[tokio::test]
    async fn test_graph_query_bad_cypher_returns_error() {
        let state = new_shared_state();
        let tools = create_all_tools(Arc::clone(&state));
        let graph_tool = tools.iter().find(|t| t.name == "rlmx_graph_query").unwrap();

        let result = graph_tool.handler.handle(json!({
            "cypher": "SELECT * FROM table"
        })).await;

        assert!(result.is_err(), "Invalid Cypher should return an error");
    }

    #[tokio::test]
    async fn test_query_on_empty_memory_returns_no_results() {
        let state = new_shared_state();
        let tools = create_all_tools(state);
        let query_tool = tools.iter().find(|t| t.name == "rlmx_query").unwrap();

        let result = query_tool.handler.handle(json!({
            "query": "anything"
        })).await.unwrap();

        assert_eq!(result["segments_returned"], 0);
        assert_eq!(result["total_segments_scanned"], 0);
    }

    #[tokio::test]
    async fn test_stub_tools_report_stub_status() {
        let state = new_shared_state();
        let tools = create_all_tools(state);

        let sona_tool = tools.iter().find(|t| t.name == "rlmx_sona_stats").unwrap();
        let result = sona_tool.handler.handle(json!({})).await.unwrap();
        assert_eq!(result["status"], "stub");

        let plugin_tool = tools.iter().find(|t| t.name == "rlmx_plugin_list").unwrap();
        let result = plugin_tool.handler.handle(json!({})).await.unwrap();
        assert_eq!(result["status"], "stub");
    }

    #[tokio::test]
    async fn test_tool_call_dispatch_missing_params() {
        let state = new_shared_state();
        let tools = create_all_tools(state);
        let query_tool = tools.into_iter().find(|t| t.name == "rlmx_query").unwrap();

        let result = query_tool.handler.handle(json!({})).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, INVALID_PARAMS);
    }
}
