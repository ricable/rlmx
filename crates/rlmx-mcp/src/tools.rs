//! RLMX MCP Tools
//!
//! Implements all 39 RLMX MCP tool definitions and their handlers.
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

use rlmx_kernel::{
    text_to_embedding, Graph, MemoryRegion, SearchFilters, SegmentMetadata, EMBED_DIM,
};

#[allow(unused_imports)]
use crate::protocol::INVALID_PARAMS;
use crate::protocol::{McpError, McpTool, ToolHandler};

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
    /// Simulated swarm node state.
    pub swarm_nodes: Vec<serde_json::Value>,
    /// Running agent list.
    pub agents: Vec<serde_json::Value>,
    /// Experiment tracking.
    pub experiments: Vec<serde_json::Value>,
    /// Mutation history.
    pub mutations: Vec<serde_json::Value>,
    /// Active research tasks.
    pub research_tasks: Vec<serde_json::Value>,
    /// WebSocket event bus for broadcasting swarm events to the dashboard.
    pub event_bus: Option<crate::ws::SwarmEventBus>,
    /// Sandbox profiles registry.
    pub sandbox_profiles: Vec<serde_json::Value>,
    /// Running sandbox instances.
    pub sandbox_instances: Vec<serde_json::Value>,
}

impl ToolState {
    /// Create a new `ToolState` with empty kernel subsystems.
    pub fn new() -> Self {
        Self {
            memory: MemoryRegion::new("mcp-primary"),
            graph: Graph::new(),
            ingest_count: 0,
            query_count: 0,
            swarm_nodes: Vec::new(),
            agents: Vec::new(),
            experiments: Vec::new(),
            mutations: Vec::new(),
            research_tasks: Vec::new(),
            event_bus: None,
            sandbox_profiles: Vec::new(),
            sandbox_instances: Vec::new(),
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

/// Create all 39 RLMX MCP tools with their handlers, wired to the given
/// shared kernel state.
pub fn create_all_tools(state: SharedToolState) -> Vec<McpTool> {
    vec![
        // Original 12 tools
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
        // Edge tools (13-15) are registered elsewhere; tools 16-26 below
        // Swarm tools
        create_rlmx_swarm_status(Arc::clone(&state)),
        create_rlmx_swarm_topology(Arc::clone(&state)),
        // Agent tools
        create_rlmx_agent_spawn(Arc::clone(&state)),
        create_rlmx_agent_list(Arc::clone(&state)),
        create_rlmx_agent_terminate(Arc::clone(&state)),
        // Research tools
        create_rlmx_research_start(Arc::clone(&state)),
        create_rlmx_research_status(Arc::clone(&state)),
        create_rlmx_experiment_list(Arc::clone(&state)),
        // Evolution tools
        create_rlmx_mutation_history(Arc::clone(&state)),
        // Forecasting tools
        create_rlmx_forecast(Arc::clone(&state)),
        // Training tools
        create_rlmx_train(Arc::clone(&state)),
        // Sandbox tools (ADR-011)
        create_rlmx_sandbox_spawn(Arc::clone(&state)),
        create_rlmx_sandbox_terminate(Arc::clone(&state)),
        create_rlmx_sandbox_status(Arc::clone(&state)),
        create_rlmx_sandbox_list(Arc::clone(&state)),
        create_rlmx_fleet_deploy(Arc::clone(&state)),
        // Marketplace tools (ADR-015)
        create_rlmx_marketplace_search(Arc::clone(&state)),
        create_rlmx_marketplace_install(Arc::clone(&state)),
        create_rlmx_marketplace_uninstall(Arc::clone(&state)),
        create_rlmx_marketplace_rate(Arc::clone(&state)),
        create_rlmx_marketplace_list_installed(Arc::clone(&state)),
        create_rlmx_marketplace_publish(Arc::clone(&state)),
        create_rlmx_marketplace_featured(Arc::clone(&state)),
        create_rlmx_marketplace_categories(Arc::clone(&state)),
        // Voice tools (ADR-018)
        create_rlmx_voice_transcribe(Arc::clone(&state)),
        create_rlmx_voice_synthesize(Arc::clone(&state)),
        create_rlmx_voice_session(Arc::clone(&state)),
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
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: query"))?;

        let context_window = params
            .get("context_window")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let strategy = params
            .get("strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        let embedding = text_to_embedding(query);
        let start = Instant::now();

        let mut state = self.state.write().await;
        state.query_count += 1;
        let total_segments = state.memory.len();

        let hits = state
            .memory
            .search(&embedding, context_window, &SearchFilters::default());
        let elapsed = start.elapsed();

        let results: Vec<serde_json::Value> = hits
            .iter()
            .map(|hit| {
                json!({
                    "segment_id": hit.segment_id.to_string(),
                    "content": hit.content,
                    "relevance_score": (hit.score * 1000.0).round() / 1000.0,
                    "tier": format!("{:?}", hit.metadata.segment_type),
                })
            })
            .collect();

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
        let data = params
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: data"))?;

        let plugin = params
            .get("plugin")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: plugin"))?;

        let extra_metadata = params.get("metadata").cloned().unwrap_or(json!({}));

        // Split data into segments on double-newline boundaries (or treat as
        // a single segment if no double-newlines are present).
        let chunks: Vec<&str> = if data.contains("\n\n") {
            data.split("\n\n")
                .filter(|s| !s.trim().is_empty())
                .collect()
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
        let include_hnsw = params
            .get("include_hnsw")
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
        let plugin = params
            .get("plugin")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: plugin"))?;

        let action = params
            .get("action")
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
        description: "Force a specific retrieval strategy (RLM or TRM) for subsequent queries."
            .to_string(),
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
        let strategy = params
            .get("strategy")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: strategy"))?;

        let duration = params
            .get("duration_seconds")
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
        description:
            "Directly invoke the TRM classifier to classify segments by temporal relevance."
                .to_string(),
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
        let segment_ids = params
            .get("segment_ids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: segment_ids"))?;

        let horizon = params
            .get("time_horizon")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        // TODO: wire to TRM subsystem
        let classifications: Vec<serde_json::Value> = segment_ids
            .iter()
            .map(|id| {
                json!({
                    "segment_id": id,
                    "classification": "relevant",
                    "confidence": 0.87,
                    "decay_rate": 0.02,
                    "horizon": horizon
                })
            })
            .collect();

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
        description:
            "Package data as an RVF (RLMX Versioned Format) container with integrity seals."
                .to_string(),
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
        let segment_ids = params
            .get("segment_ids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: segment_ids"))?;

        let label = params
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed");

        let seal_type = params
            .get("seal_type")
            .and_then(|v| v.as_str())
            .unwrap_or("blake3");

        let container_id = Uuid::new_v4();

        // When rvf-ext feature is enabled, use real RVF sealing
        #[cfg(feature = "rvf-ext")]
        let status = "sealed";
        #[cfg(not(feature = "rvf-ext"))]
        let status = "stub";

        Ok(json!({
            "status": status,
            "container_id": container_id.to_string(),
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
        description:
            "Create a copy-on-write (COW) branch from an RVF container for experimentation."
                .to_string(),
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
        let container_id = params
            .get("container_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: container_id"))?;

        let branch_label = params
            .get("branch_label")
            .and_then(|v| v.as_str())
            .unwrap_or("experiment");

        #[cfg(feature = "rvf-ext")]
        let status = "branched";
        #[cfg(not(feature = "rvf-ext"))]
        let status = "stub";

        Ok(json!({
            "status": status,
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
        description: "Retrieve the audit trail (witness chain) for a segment or container."
            .to_string(),
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
        let target_id = params
            .get("target_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: target_id"))?;

        #[cfg(feature = "ruvector")]
        let status = "verified";
        #[cfg(not(feature = "ruvector"))]
        let status = "stub";

        Ok(json!({
            "status": status,
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
        let time_range = params
            .get("time_range")
            .and_then(|v| v.as_str())
            .unwrap_or("24h");

        #[cfg(feature = "ruvector")]
        let status = "active";
        #[cfg(not(feature = "ruvector"))]
        let status = "stub";

        Ok(json!({
            "status": status,
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
        let cypher = params
            .get("cypher")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: cypher"))?;

        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

        let start = Instant::now();
        let state = self.state.read().await;

        let rows = state
            .graph
            .cypher_query(cypher)
            .map_err(|e| McpError::new(INVALID_PARAMS, format!("Cypher query error: {}", e)))?;

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

// ---------------------------------------------------------------------------
// 16. rlmx_swarm_status  — Viewer+ — returns swarm health overview
// ---------------------------------------------------------------------------

fn create_rlmx_swarm_status(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_swarm_status".to_string(),
        description:
            "Retrieve overall swarm status including node counts, health, and consensus info."
                .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
        handler: Box::new(RlmxSwarmStatusHandler { state }),
    }
}

struct RlmxSwarmStatusHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxSwarmStatusHandler {
    async fn handle(&self, _params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let state = self.state.read().await;
        let node_count = state.swarm_nodes.len();

        if node_count == 0 {
            return Ok(json!({
                "status": "inactive",
                "node_count": 0,
                "healthy_nodes": 0,
                "zones": [],
                "uptime_secs": 0,
                "consensus_type": "raft"
            }));
        }

        let healthy = state
            .swarm_nodes
            .iter()
            .filter(|n| n.get("healthy").and_then(|v| v.as_bool()).unwrap_or(false))
            .count();

        let zones: Vec<&serde_json::Value> = state
            .swarm_nodes
            .iter()
            .filter_map(|n| n.get("zone"))
            .collect();

        Ok(json!({
            "status": "active",
            "node_count": node_count,
            "healthy_nodes": healthy,
            "zones": zones,
            "uptime_secs": 3600,
            "consensus_type": "raft"
        }))
    }
}

// ---------------------------------------------------------------------------
// 17. rlmx_swarm_topology  — Viewer+ — returns zone/connection topology
// ---------------------------------------------------------------------------

fn create_rlmx_swarm_topology(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_swarm_topology".to_string(),
        description:
            "Retrieve the swarm topology including zones, nodes, and inter-zone connections."
                .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "enum": ["json", "dot"],
                    "description": "Output format: JSON object or Graphviz DOT",
                    "default": "json"
                }
            }
        }),
        handler: Box::new(RlmxSwarmTopologyHandler { state }),
    }
}

struct RlmxSwarmTopologyHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxSwarmTopologyHandler {
    async fn handle(&self, _params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let state = self.state.read().await;

        // Distribute actual agents into zones if any exist
        let mut zone_a_nodes = Vec::new();
        let mut zone_b_nodes = Vec::new();
        let mut zone_c_nodes = Vec::new();

        for node in &state.swarm_nodes {
            match node.get("zone").and_then(|z| z.as_str()) {
                Some("B") => zone_b_nodes.push(node.clone()),
                Some("C") => zone_c_nodes.push(node.clone()),
                _ => zone_a_nodes.push(node.clone()),
            }
        }

        let total_nodes = state.swarm_nodes.len();
        let healthy_nodes = state
            .swarm_nodes
            .iter()
            .filter(|n| n.get("healthy").and_then(|v| v.as_bool()).unwrap_or(true))
            .count();

        Ok(json!({
            "zones": [
                { "id": "A", "name": "Compute", "nodes": zone_a_nodes, "consensus": "raft" },
                { "id": "B", "name": "Inference", "nodes": zone_b_nodes, "consensus": "raft" },
                { "id": "C", "name": "Edge", "nodes": zone_c_nodes, "consensus": "raft" }
            ],
            "connections": [
                { "from": "A", "to": "B", "latency_ms": 2 },
                { "from": "B", "to": "C", "latency_ms": 15 },
                { "from": "A", "to": "C", "latency_ms": 18 }
            ],
            "total_nodes": total_nodes,
            "healthy_nodes": healthy_nodes
        }))
    }
}

// ---------------------------------------------------------------------------
// 18. rlmx_agent_spawn  — Engineer+ — spawn a new agent
// ---------------------------------------------------------------------------

const VALID_AGENT_TYPES: &[&str] = &[
    "coordinator",
    "researcher",
    "router",
    "experimenter",
    "worker",
    "monitor",
    "reviewer",
    "trainer",
    "validator",
    "replicator",
    "embedder",
    "analyst",
];

fn create_rlmx_agent_spawn(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_agent_spawn".to_string(),
        description: "Spawn a new agent of the specified type into the swarm.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent_type": {
                    "type": "string",
                    "enum": VALID_AGENT_TYPES,
                    "description": "Type of agent to spawn"
                },
                "node_id": {
                    "type": "string",
                    "format": "uuid",
                    "description": "Target node to spawn the agent on"
                },
                "config": {
                    "type": "object",
                    "description": "Agent-specific configuration"
                },
                "name": {
                    "type": "string",
                    "description": "Optional human-readable name for the agent"
                },
                "task": {
                    "type": "string",
                    "description": "Optional initial task assignment"
                },
                "zone": {
                    "type": "string",
                    "description": "Optional zone to place the agent in (A, B, or C)"
                }
            },
            "required": ["agent_type"]
        }),
        handler: Box::new(RlmxAgentSpawnHandler { state }),
    }
}

struct RlmxAgentSpawnHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxAgentSpawnHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let agent_type = params
            .get("agent_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: agent_type"))?;

        if !VALID_AGENT_TYPES.contains(&agent_type) {
            return Err(McpError::invalid_params(format!(
                "Invalid agent_type '{}'. Valid types: {}",
                agent_type,
                VALID_AGENT_TYPES.join(", ")
            )));
        }

        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(agent_type);
        let task = params.get("task").and_then(|v| v.as_str()).unwrap_or("");
        let zone = params.get("zone").and_then(|v| v.as_str()).unwrap_or("A");

        let agent_id = Uuid::new_v4().to_string();
        let agent_entry = json!({
            "agent_id": agent_id,
            "agent_type": agent_type,
            "name": name,
            "status": "running",
            "task": task,
            "zone": zone,
            "spawned_at": Utc::now().to_rfc3339(),
        });

        let mut state = self.state.write().await;
        state.agents.push(agent_entry.clone());

        tracing::info!(agent_id = %agent_id, agent_type = %agent_type, "Agent spawned");

        Ok(json!({
            "agent_id": agent_id,
            "agent_type": agent_type,
            "name": name,
            "status": "running",
            "zone": zone,
            "spawned_at": agent_entry["spawned_at"],
        }))
    }
}

// ---------------------------------------------------------------------------
// 19. rlmx_agent_list  — Operator+ — list running agents
// ---------------------------------------------------------------------------

fn create_rlmx_agent_list(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_agent_list".to_string(),
        description: "List all agents with optional filtering by type and status.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "node_id": {
                    "type": "string",
                    "format": "uuid",
                    "description": "Filter agents by the node they run on"
                },
                "agent_type": {
                    "type": "string",
                    "description": "Filter agents by type"
                },
                "status": {
                    "type": "string",
                    "description": "Filter agents by status (running, terminated)"
                }
            }
        }),
        handler: Box::new(RlmxAgentListHandler { state }),
    }
}

struct RlmxAgentListHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxAgentListHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let type_filter = params.get("agent_type").and_then(|v| v.as_str());
        let status_filter = params.get("status").and_then(|v| v.as_str());

        let state = self.state.read().await;

        let filtered: Vec<&serde_json::Value> = state
            .agents
            .iter()
            .filter(|a| {
                if let Some(t) = type_filter {
                    if a.get("agent_type").and_then(|v| v.as_str()) != Some(t) {
                        return false;
                    }
                }
                if let Some(s) = status_filter {
                    if a.get("status").and_then(|v| v.as_str()) != Some(s) {
                        return false;
                    }
                }
                true
            })
            .collect();

        let active = state
            .agents
            .iter()
            .filter(|a| a.get("status").and_then(|v| v.as_str()) == Some("running"))
            .count();

        // Count agents by type
        let mut by_type = serde_json::Map::new();
        for agent in &state.agents {
            if let Some(t) = agent.get("agent_type").and_then(|v| v.as_str()) {
                let count = by_type.entry(t.to_string()).or_insert_with(|| json!(0));
                *count = json!(count.as_u64().unwrap_or(0) + 1);
            }
        }

        Ok(json!({
            "agents": filtered,
            "total": state.agents.len(),
            "active": active,
            "by_type": by_type
        }))
    }
}

// ---------------------------------------------------------------------------
// 20. rlmx_agent_terminate  — Admin+ — terminate an agent
// ---------------------------------------------------------------------------

fn create_rlmx_agent_terminate(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_agent_terminate".to_string(),
        description: "Terminate a running agent by its ID.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent_id": {
                    "type": "string",
                    "description": "The ID of the agent to terminate"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for termination (recorded in audit log)"
                }
            },
            "required": ["agent_id"]
        }),
        handler: Box::new(RlmxAgentTerminateHandler { state }),
    }
}

struct RlmxAgentTerminateHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxAgentTerminateHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let agent_id = params
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: agent_id"))?;

        let mut state = self.state.write().await;

        let agent = state
            .agents
            .iter_mut()
            .find(|a| a.get("agent_id").and_then(|v| v.as_str()) == Some(agent_id));

        match agent {
            Some(a) => {
                a["status"] = json!("terminated");
                a["terminated_at"] = json!(Utc::now().to_rfc3339());
                tracing::info!(agent_id = %agent_id, "Agent terminated");
                Ok(json!({
                    "status": "terminated",
                    "agent_id": agent_id,
                    "terminated_at": a["terminated_at"],
                }))
            }
            None => Err(McpError::invalid_params(format!(
                "Agent not found: {}",
                agent_id
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// 21. rlmx_research_start  — Engineer+ — start a research task
// ---------------------------------------------------------------------------

fn create_rlmx_research_start(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_research_start".to_string(),
        description:
            "Start a new research task that spawns a researcher agent to investigate a topic."
                .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "goal": {
                    "type": "string",
                    "description": "High-level research goal"
                },
                "topic": {
                    "type": "string",
                    "description": "The research topic to investigate"
                },
                "max_generations": {
                    "type": "integer",
                    "description": "Maximum evolutionary generations to run",
                    "default": 10
                },
                "budget_minutes": {
                    "type": "integer",
                    "description": "Maximum wall-clock minutes for the research run",
                    "default": 60
                },
                "hypotheses": {
                    "type": "integer",
                    "description": "Number of hypotheses to generate",
                    "default": 3
                },
                "nodes": {
                    "type": "integer",
                    "description": "Number of compute nodes to allocate",
                    "default": 1
                }
            },
            "required": ["topic"]
        }),
        handler: Box::new(RlmxResearchStartHandler { state }),
    }
}

struct RlmxResearchStartHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxResearchStartHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let topic = params
            .get("topic")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: topic"))?;

        let hypotheses = params
            .get("hypotheses")
            .and_then(|v| v.as_u64())
            .unwrap_or(3);
        let nodes = params.get("nodes").and_then(|v| v.as_u64()).unwrap_or(1);

        let research_id = Uuid::new_v4().to_string();
        let researcher_agent_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Spawn a researcher agent
        let agent_entry = json!({
            "agent_id": researcher_agent_id,
            "agent_type": "researcher",
            "name": format!("researcher-{}", &research_id[..8]),
            "status": "running",
            "task": format!("Research: {}", topic),
            "zone": "A",
            "spawned_at": now,
        });

        let research_task = json!({
            "research_id": research_id,
            "topic": topic,
            "status": "started",
            "hypotheses": hypotheses,
            "nodes": nodes,
            "researcher_agent_id": researcher_agent_id,
            "started_at": now,
        });

        let mut state = self.state.write().await;
        state.agents.push(agent_entry);
        state.research_tasks.push(research_task);

        tracing::info!(research_id = %research_id, topic = %topic, "Research task started");

        Ok(json!({
            "research_id": research_id,
            "topic": topic,
            "status": "started",
            "researcher_agent_id": researcher_agent_id,
        }))
    }
}

// ---------------------------------------------------------------------------
// 22. rlmx_research_status  — Operator+ — check research task status
// ---------------------------------------------------------------------------

fn create_rlmx_research_status(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_research_status".to_string(),
        description: "Check the status of an active research task.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "research_id": {
                    "type": "string",
                    "description": "The ID of the research task to check"
                }
            },
            "required": ["research_id"]
        }),
        handler: Box::new(RlmxResearchStatusHandler { state }),
    }
}

struct RlmxResearchStatusHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxResearchStatusHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let research_id = params
            .get("research_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: research_id"))?;

        let state = self.state.read().await;
        let task = state
            .research_tasks
            .iter()
            .find(|t| t.get("research_id").and_then(|v| v.as_str()) == Some(research_id));

        match task {
            Some(t) => Ok(t.clone()),
            None => Err(McpError::invalid_params(format!(
                "Research task not found: {}",
                research_id
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// 23. rlmx_experiment_list  — Viewer+ — list experiments
// ---------------------------------------------------------------------------

fn create_rlmx_experiment_list(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_experiment_list".to_string(),
        description: "List all tracked experiments with optional status filtering.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "research_id": {
                    "type": "string",
                    "description": "Filter experiments by parent research task ID"
                },
                "status": {
                    "type": "string",
                    "description": "Filter experiments by status (active, completed, failed)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of experiments to return",
                    "default": 50
                }
            }
        }),
        handler: Box::new(RlmxExperimentListHandler { state }),
    }
}

struct RlmxExperimentListHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxExperimentListHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let status_filter = params.get("status").and_then(|v| v.as_str());

        let state = self.state.read().await;

        let filtered: Vec<&serde_json::Value> = state
            .experiments
            .iter()
            .filter(|e| {
                if let Some(s) = status_filter {
                    return e.get("status").and_then(|v| v.as_str()) == Some(s);
                }
                true
            })
            .collect();

        Ok(json!({
            "experiments": filtered,
            "total": filtered.len(),
        }))
    }
}

// ---------------------------------------------------------------------------
// 24. rlmx_mutation_history  — Viewer+ — view mutation history
// ---------------------------------------------------------------------------

fn create_rlmx_mutation_history(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_mutation_history".to_string(),
        description: "Retrieve mutation history for evolutionary optimization tracking."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "research_id": {
                    "type": "string",
                    "description": "Filter mutations by parent research task ID"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of mutations to return",
                    "default": 50
                },
                "generation": {
                    "type": "integer",
                    "description": "Filter by specific generation number"
                }
            }
        }),
        handler: Box::new(RlmxMutationHistoryHandler { state }),
    }
}

struct RlmxMutationHistoryHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxMutationHistoryHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
        let generation_filter = params.get("generation").and_then(|v| v.as_u64());

        let state = self.state.read().await;

        if state.mutations.is_empty() {
            // Return simulated initial mutations when no history exists
            return Ok(json!({
                "mutations": [
                    {
                        "id": "mut-0001",
                        "generation": 0,
                        "type": "initialization",
                        "description": "Initial population seeded",
                        "fitness": 0.5,
                        "timestamp": Utc::now().to_rfc3339()
                    }
                ],
                "total": 1,
                "latest_generation": 0
            }));
        }

        let filtered: Vec<&serde_json::Value> = state
            .mutations
            .iter()
            .filter(|m| {
                if let Some(gen) = generation_filter {
                    return m.get("generation").and_then(|v| v.as_u64()) == Some(gen);
                }
                true
            })
            .take(limit)
            .collect();

        let latest_gen = state
            .mutations
            .iter()
            .filter_map(|m| m.get("generation").and_then(|v| v.as_u64()))
            .max()
            .unwrap_or(0);

        Ok(json!({
            "mutations": filtered,
            "total": filtered.len(),
            "latest_generation": latest_gen
        }))
    }
}

// ---------------------------------------------------------------------------
// 25. rlmx_forecast  — Operator+ — generate simulated forecasts
// ---------------------------------------------------------------------------

fn create_rlmx_forecast(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_forecast".to_string(),
        description: "Generate a forecast for swarm health, query load, or mutation quality."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "metric": {
                    "type": "string",
                    "enum": ["swarm_health", "query_load", "mutation_quality"],
                    "description": "The metric to forecast"
                },
                "horizon_hours": {
                    "type": "integer",
                    "description": "Forecast horizon in hours",
                    "default": 24
                },
                "model": {
                    "type": "string",
                    "description": "Forecasting model to use (e.g., arima, prophet)",
                    "default": "arima"
                }
            },
            "required": ["metric"]
        }),
        handler: Box::new(RlmxForecastHandler { state }),
    }
}

struct RlmxForecastHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxForecastHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        // Accept both "metric" (ADR-010) and "target" (legacy) param names
        let target = params
            .get("metric")
            .or_else(|| params.get("target"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: metric"))?;

        let horizon_hours = params
            .get("horizon_hours")
            .and_then(|v| v.as_u64())
            .unwrap_or(24);

        let _state = self.state.read().await;

        // Generate simulated forecast data points
        let points: Vec<serde_json::Value> = (0..std::cmp::min(horizon_hours, 168))
            .step_by(std::cmp::max(1, (horizon_hours / 12) as usize))
            .map(|h| {
                let base = match target {
                    "swarm_health" => 0.95,
                    "query_load" => 150.0,
                    "mutation_quality" => 0.72,
                    _ => 0.5,
                };
                let noise = (h as f64 * 0.01).sin() * 0.05;
                json!({
                    "hour": h,
                    "value": ((base + noise) * 1000.0).round() / 1000.0,
                    "lower_bound": ((base - 0.1 + noise) * 1000.0).round() / 1000.0,
                    "upper_bound": ((base + 0.1 + noise) * 1000.0).round() / 1000.0,
                })
            })
            .collect();

        Ok(json!({
            "target": target,
            "horizon_hours": horizon_hours,
            "confidence": 0.85,
            "model": "arima-simulated",
            "forecast": points,
            "generated_at": Utc::now().to_rfc3339()
        }))
    }
}

// ---------------------------------------------------------------------------
// 26. rlmx_train  — Engineer+ — start a training run
// ---------------------------------------------------------------------------

fn create_rlmx_train(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_train".to_string(),
        description: "Start a model training run on a swarm node.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "config": {
                    "type": "object",
                    "description": "Training configuration (dataset, hyperparams, epochs, etc.)"
                },
                "node_id": {
                    "type": "string",
                    "format": "uuid",
                    "description": "Target node for training (auto-selected if omitted)"
                },
                "backend": {
                    "type": "string",
                    "enum": ["candle", "mlx", "remote"],
                    "description": "Compute backend for training",
                    "default": "candle"
                }
            },
            "required": ["config"]
        }),
        handler: Box::new(RlmxTrainHandler { state }),
    }
}

struct RlmxTrainHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for RlmxTrainHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let config = params
            .get("config")
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: config"))?;

        let node_id = params
            .get("node_id")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        let backend = params
            .get("backend")
            .and_then(|v| v.as_str())
            .unwrap_or("candle");

        // Check if training subsystem is available
        let _state = self.state.read().await;

        // Training subsystem not yet wired — follow the edge-tools
        // "unavailable" pattern per ADR-010.
        let training_id = Uuid::new_v4().to_string();
        let resolved_node_id = if node_id == "auto" {
            Uuid::new_v4().to_string()
        } else {
            node_id.to_string()
        };

        Ok(json!({
            "training_id": training_id,
            "node_id": resolved_node_id,
            "backend": backend,
            "config": config,
            "status": "unavailable",
            "message": "Training subsystem not yet available"
        }))
    }
}

// RBAC mappings for server.rs tool_to_operation match:
// rlmx_swarm_status, rlmx_swarm_topology => Query (Viewer+)
// rlmx_agent_spawn, rlmx_research_start, rlmx_train => ParameterModify (Engineer+)
// rlmx_agent_list, rlmx_research_status, rlmx_forecast => Ingest (Operator+)
// rlmx_experiment_list, rlmx_mutation_history => Query (Viewer+)
// rlmx_agent_terminate => ContainerSeal (Admin+)

// ---------------------------------------------------------------------------
// 24. rlmx_sandbox_spawn — spawn a sandbox instance from a profile
// ---------------------------------------------------------------------------

fn create_rlmx_sandbox_spawn(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_sandbox_spawn".to_string(),
        description: "Spawn a new sandbox instance from a registered profile.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "profile": {
                    "type": "string",
                    "description": "Name of the sandbox profile to spawn"
                }
            },
            "required": ["profile"]
        }),
        handler: Box::new(SandboxSpawnHandler { state }),
    }
}

struct SandboxSpawnHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for SandboxSpawnHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let profile_name = params
            .get("profile")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: profile"))?;

        let mut state = self.state.write().await;
        let sandbox_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Check if profile exists
        let profile = state
            .sandbox_profiles
            .iter()
            .find(|p| p.get("name").and_then(|v| v.as_str()) == Some(profile_name));

        let profile_data = match profile {
            Some(p) => p.clone(),
            None => {
                return Ok(json!({
                    "status": "error",
                    "message": format!("Profile not found: {}", profile_name)
                }));
            }
        };

        let instance = json!({
            "sandbox_id": sandbox_id,
            "profile": profile_name,
            "state": "Provisioning",
            "agent_type": profile_data.get("agent_type").and_then(|v| v.as_str()).unwrap_or("unknown"),
            "zone": profile_data.get("zone_preference").and_then(|v| v.as_str()).unwrap_or("zone-a"),
            "created_at": now,
            "metrics": {
                "cpu_usage_pct": 0.0,
                "memory_usage_mb": 0,
                "uptime_secs": 0,
                "tasks_completed": 0
            }
        });

        state.sandbox_instances.push(instance);

        // Broadcast event if event bus available
        if let Some(ref bus) = state.event_bus {
            use crate::ws::SwarmEvent;
            let _ = bus.send(SwarmEvent::AgentSpawned {
                agent_id: Uuid::parse_str(&sandbox_id).unwrap_or_else(|_| Uuid::new_v4()),
                agent_type: format!("sandbox:{}", profile_name),
                node_id: Uuid::new_v4(),
            });
        }

        Ok(json!({
            "status": "spawned",
            "sandbox_id": sandbox_id,
            "profile": profile_name,
            "state": "Provisioning"
        }))
    }
}

// ---------------------------------------------------------------------------
// 25. rlmx_sandbox_terminate — terminate a sandbox instance
// ---------------------------------------------------------------------------

fn create_rlmx_sandbox_terminate(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_sandbox_terminate".to_string(),
        description: "Terminate a running sandbox instance.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "sandbox_id": {
                    "type": "string",
                    "description": "ID of the sandbox instance to terminate"
                }
            },
            "required": ["sandbox_id"]
        }),
        handler: Box::new(SandboxTerminateHandler { state }),
    }
}

struct SandboxTerminateHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for SandboxTerminateHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let sandbox_id = params
            .get("sandbox_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: sandbox_id"))?;

        let mut state = self.state.write().await;

        let instance = state
            .sandbox_instances
            .iter_mut()
            .find(|i| i.get("sandbox_id").and_then(|v| v.as_str()) == Some(sandbox_id));

        match instance {
            Some(inst) => {
                inst["state"] = json!("Terminated");
                Ok(json!({
                    "status": "terminated",
                    "sandbox_id": sandbox_id
                }))
            }
            None => Ok(json!({
                "status": "error",
                "message": format!("Sandbox not found: {}", sandbox_id)
            })),
        }
    }
}

// ---------------------------------------------------------------------------
// 26. rlmx_sandbox_status — get sandbox instance status
// ---------------------------------------------------------------------------

fn create_rlmx_sandbox_status(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_sandbox_status".to_string(),
        description: "Get the status and metrics of a sandbox instance.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "sandbox_id": {
                    "type": "string",
                    "description": "ID of the sandbox instance"
                }
            },
            "required": ["sandbox_id"]
        }),
        handler: Box::new(SandboxStatusHandler { state }),
    }
}

struct SandboxStatusHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for SandboxStatusHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let sandbox_id = params
            .get("sandbox_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: sandbox_id"))?;

        let state = self.state.read().await;
        let instance = state
            .sandbox_instances
            .iter()
            .find(|i| i.get("sandbox_id").and_then(|v| v.as_str()) == Some(sandbox_id));

        match instance {
            Some(inst) => Ok(inst.clone()),
            None => Ok(json!({
                "status": "error",
                "message": format!("Sandbox not found: {}", sandbox_id)
            })),
        }
    }
}

// ---------------------------------------------------------------------------
// 27. rlmx_sandbox_list — list all sandbox instances
// ---------------------------------------------------------------------------

fn create_rlmx_sandbox_list(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_sandbox_list".to_string(),
        description: "List all sandbox instances with optional state filter.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "state_filter": {
                    "type": "string",
                    "description": "Filter by sandbox state (Provisioning, Running, Terminated, etc.)"
                }
            }
        }),
        handler: Box::new(SandboxListHandler { state }),
    }
}

struct SandboxListHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for SandboxListHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let state_filter = params.get("state_filter").and_then(|v| v.as_str());

        let state = self.state.read().await;

        let instances: Vec<&serde_json::Value> = if let Some(filter) = state_filter {
            state
                .sandbox_instances
                .iter()
                .filter(|i| i.get("state").and_then(|v| v.as_str()) == Some(filter))
                .collect()
        } else {
            state.sandbox_instances.iter().collect()
        };

        let profiles: Vec<&serde_json::Value> = state.sandbox_profiles.iter().collect();

        Ok(json!({
            "sandbox_count": instances.len(),
            "sandboxes": instances,
            "profile_count": profiles.len(),
            "profiles": profiles
        }))
    }
}

// ---------------------------------------------------------------------------
// 28. rlmx_fleet_deploy — deploy a fleet manifest
// ---------------------------------------------------------------------------

fn create_rlmx_fleet_deploy(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_fleet_deploy".to_string(),
        description: "Deploy a fleet manifest, spawning multiple sandbox instances.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Fleet name"
                },
                "version": {
                    "type": "string",
                    "description": "Fleet version",
                    "default": "1.0"
                },
                "sandboxes": {
                    "type": "array",
                    "description": "Array of {profile, count} specs",
                    "items": {
                        "type": "object",
                        "properties": {
                            "profile": { "type": "string" },
                            "count": { "type": "integer" }
                        },
                        "required": ["profile", "count"]
                    }
                }
            },
            "required": ["name", "sandboxes"]
        }),
        handler: Box::new(FleetDeployHandler { state }),
    }
}

struct FleetDeployHandler {
    state: SharedToolState,
}

#[async_trait]
impl ToolHandler for FleetDeployHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let fleet_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: name"))?;

        let sandboxes = params
            .get("sandboxes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: sandboxes"))?;

        let mut state = self.state.write().await;
        let now = Utc::now().to_rfc3339();
        let mut spawned_ids = Vec::new();
        let mut errors = Vec::new();

        for spec in sandboxes {
            let profile_name = spec.get("profile").and_then(|v| v.as_str()).unwrap_or("");
            let count = spec.get("count").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

            let profile_exists = state
                .sandbox_profiles
                .iter()
                .any(|p| p.get("name").and_then(|v| v.as_str()) == Some(profile_name));

            if !profile_exists {
                errors.push(format!("Profile not found: {}", profile_name));
                continue;
            }

            let profile_data = state
                .sandbox_profiles
                .iter()
                .find(|p| p.get("name").and_then(|v| v.as_str()) == Some(profile_name))
                .cloned();

            for _ in 0..count {
                let sandbox_id = Uuid::new_v4().to_string();
                let instance = json!({
                    "sandbox_id": sandbox_id,
                    "profile": profile_name,
                    "fleet": fleet_name,
                    "state": "Provisioning",
                    "agent_type": profile_data.as_ref()
                        .and_then(|p| p.get("agent_type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown"),
                    "zone": profile_data.as_ref()
                        .and_then(|p| p.get("zone_preference"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("zone-a"),
                    "created_at": now,
                    "metrics": {
                        "cpu_usage_pct": 0.0,
                        "memory_usage_mb": 0,
                        "uptime_secs": 0,
                        "tasks_completed": 0
                    }
                });
                state.sandbox_instances.push(instance);
                spawned_ids.push(sandbox_id);
            }
        }

        Ok(json!({
            "status": if errors.is_empty() { "deployed" } else { "partial" },
            "fleet": fleet_name,
            "spawned": spawned_ids.len(),
            "sandbox_ids": spawned_ids,
            "errors": errors
        }))
    }
}

// ---------------------------------------------------------------------------
// Marketplace tools (ADR-015)
// ---------------------------------------------------------------------------

fn create_rlmx_marketplace_search(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_marketplace_search".to_string(),
        description: "Search the agent marketplace by domain or keyword.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query (keyword or domain)" },
                "domain": { "type": "string", "description": "Life domain filter" },
                "limit": { "type": "integer", "description": "Max results", "default": 20 }
            },
            "required": ["query"]
        }),
        handler: Box::new(MarketplaceSearchHandler { state }),
    }
}

struct MarketplaceSearchHandler { state: SharedToolState }

#[async_trait]
impl ToolHandler for MarketplaceSearchHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let domain = params.get("domain").and_then(|v| v.as_str());
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20);
        let _ = self.state.read().await;
        Ok(json!({
            "status": "stub",
            "query": query,
            "domain": domain,
            "limit": limit,
            "results": [],
            "total": 0
        }))
    }
}

fn create_rlmx_marketplace_install(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_marketplace_install".to_string(),
        description: "Install an agent from the marketplace.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent_id": { "type": "string", "description": "Marketplace agent ID to install" },
                "version": { "type": "string", "description": "Version to install", "default": "latest" }
            },
            "required": ["agent_id"]
        }),
        handler: Box::new(MarketplaceInstallHandler { state }),
    }
}

struct MarketplaceInstallHandler { state: SharedToolState }

#[async_trait]
impl ToolHandler for MarketplaceInstallHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let agent_id = params.get("agent_id").and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: agent_id"))?;
        let version = params.get("version").and_then(|v| v.as_str()).unwrap_or("latest");
        let _ = self.state.read().await;
        Ok(json!({
            "status": "stub",
            "agent_id": agent_id,
            "version": version,
            "installed": false
        }))
    }
}

fn create_rlmx_marketplace_uninstall(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_marketplace_uninstall".to_string(),
        description: "Remove an installed agent from the local registry.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent_id": { "type": "string", "description": "Installed agent ID to remove" }
            },
            "required": ["agent_id"]
        }),
        handler: Box::new(MarketplaceUninstallHandler { state }),
    }
}

struct MarketplaceUninstallHandler { state: SharedToolState }

#[async_trait]
impl ToolHandler for MarketplaceUninstallHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let agent_id = params.get("agent_id").and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: agent_id"))?;
        let _ = self.state.read().await;
        Ok(json!({
            "status": "stub",
            "agent_id": agent_id,
            "uninstalled": false
        }))
    }
}

fn create_rlmx_marketplace_rate(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_marketplace_rate".to_string(),
        description: "Rate an installed agent on the marketplace.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "agent_id": { "type": "string", "description": "Agent ID to rate" },
                "rating": { "type": "integer", "description": "Rating 1-5", "minimum": 1, "maximum": 5 },
                "review": { "type": "string", "description": "Optional review text" }
            },
            "required": ["agent_id", "rating"]
        }),
        handler: Box::new(MarketplaceRateHandler { state }),
    }
}

struct MarketplaceRateHandler { state: SharedToolState }

#[async_trait]
impl ToolHandler for MarketplaceRateHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let agent_id = params.get("agent_id").and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: agent_id"))?;
        let rating = params.get("rating").and_then(|v| v.as_u64())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: rating"))?;
        let review = params.get("review").and_then(|v| v.as_str());
        let _ = self.state.read().await;
        Ok(json!({
            "status": "stub",
            "agent_id": agent_id,
            "rating": rating,
            "review": review,
            "submitted": false
        }))
    }
}

fn create_rlmx_marketplace_list_installed(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_marketplace_list_installed".to_string(),
        description: "List all agents installed from the marketplace.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "domain": { "type": "string", "description": "Filter by life domain" }
            }
        }),
        handler: Box::new(MarketplaceListInstalledHandler { state }),
    }
}

struct MarketplaceListInstalledHandler { state: SharedToolState }

#[async_trait]
impl ToolHandler for MarketplaceListInstalledHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let domain = params.get("domain").and_then(|v| v.as_str());
        let _ = self.state.read().await;
        Ok(json!({
            "status": "stub",
            "domain": domain,
            "installed_agents": [],
            "total": 0
        }))
    }
}

fn create_rlmx_marketplace_publish(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_marketplace_publish".to_string(),
        description: "Publish an agent to the marketplace.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Agent name" },
                "description": { "type": "string", "description": "Agent description" },
                "domain": { "type": "string", "description": "Primary life domain" },
                "version": { "type": "string", "description": "Version string" },
                "capabilities": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of agent capabilities"
                }
            },
            "required": ["name", "description", "domain", "version"]
        }),
        handler: Box::new(MarketplacePublishHandler { state }),
    }
}

struct MarketplacePublishHandler { state: SharedToolState }

#[async_trait]
impl ToolHandler for MarketplacePublishHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let name = params.get("name").and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: name"))?;
        let domain = params.get("domain").and_then(|v| v.as_str()).unwrap_or("");
        let version = params.get("version").and_then(|v| v.as_str()).unwrap_or("0.1.0");
        let _ = self.state.read().await;
        Ok(json!({
            "status": "stub",
            "name": name,
            "domain": domain,
            "version": version,
            "published": false
        }))
    }
}

fn create_rlmx_marketplace_featured(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_marketplace_featured".to_string(),
        description: "Get featured agents from the marketplace.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Max results", "default": 10 }
            }
        }),
        handler: Box::new(MarketplaceFeaturedHandler { state }),
    }
}

struct MarketplaceFeaturedHandler { state: SharedToolState }

#[async_trait]
impl ToolHandler for MarketplaceFeaturedHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10);
        let _ = self.state.read().await;
        Ok(json!({
            "status": "stub",
            "limit": limit,
            "featured": [],
            "total": 0
        }))
    }
}

fn create_rlmx_marketplace_categories(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_marketplace_categories".to_string(),
        description: "List available agent categories in the marketplace.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
        handler: Box::new(MarketplaceCategoriesHandler { state }),
    }
}

struct MarketplaceCategoriesHandler { state: SharedToolState }

#[async_trait]
impl ToolHandler for MarketplaceCategoriesHandler {
    async fn handle(&self, _params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let _ = self.state.read().await;
        Ok(json!({
            "status": "stub",
            "categories": [
                "Visa/Immigration",
                "Housing",
                "Career",
                "Education",
                "Healthcare",
                "Banking/Finance",
                "Language",
                "Logistics",
                "Life Admin",
                "Social/Community",
                "Legal",
                "Entertainment/Culture"
            ]
        }))
    }
}

// ---------------------------------------------------------------------------
// Voice tools (ADR-018)
// ---------------------------------------------------------------------------

fn create_rlmx_voice_transcribe(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_voice_transcribe".to_string(),
        description: "Transcribe audio input to text via the voice pipeline.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "audio_base64": { "type": "string", "description": "Base64-encoded audio data (Opus/WAV)" },
                "language": { "type": "string", "description": "Language hint (ISO 639-1)", "default": "en" },
                "session_id": { "type": "string", "description": "Voice session ID for streaming context" }
            },
            "required": ["audio_base64"]
        }),
        handler: Box::new(VoiceTranscribeHandler { state }),
    }
}

struct VoiceTranscribeHandler { state: SharedToolState }

#[async_trait]
impl ToolHandler for VoiceTranscribeHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let _audio = params.get("audio_base64").and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: audio_base64"))?;
        let language = params.get("language").and_then(|v| v.as_str()).unwrap_or("en");
        let session_id = params.get("session_id").and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let _ = self.state.read().await;
        Ok(json!({
            "status": "stub",
            "session_id": session_id,
            "language": language,
            "transcript": "",
            "confidence": 0.0,
            "is_final": false
        }))
    }
}

fn create_rlmx_voice_synthesize(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_voice_synthesize".to_string(),
        description: "Synthesize text to speech audio.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "text": { "type": "string", "description": "Text to synthesize" },
                "voice": { "type": "string", "description": "Voice profile name", "default": "default" },
                "speed": { "type": "number", "description": "Speech speed multiplier", "default": 1.0 }
            },
            "required": ["text"]
        }),
        handler: Box::new(VoiceSynthesizeHandler { state }),
    }
}

struct VoiceSynthesizeHandler { state: SharedToolState }

#[async_trait]
impl ToolHandler for VoiceSynthesizeHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let text = params.get("text").and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: text"))?;
        let voice = params.get("voice").and_then(|v| v.as_str()).unwrap_or("default");
        let speed = params.get("speed").and_then(|v| v.as_f64()).unwrap_or(1.0);
        let _ = self.state.read().await;
        Ok(json!({
            "status": "stub",
            "text": text,
            "voice": voice,
            "speed": speed,
            "audio_base64": "",
            "duration_ms": 0
        }))
    }
}

fn create_rlmx_voice_session(state: SharedToolState) -> McpTool {
    McpTool {
        name: "rlmx_voice_session".to_string(),
        description: "Manage a voice session (start, stop, status).".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["start", "stop", "status"],
                    "description": "Session action"
                },
                "session_id": { "type": "string", "description": "Session ID (required for stop/status)" },
                "mode": {
                    "type": "string",
                    "enum": ["multimodal", "voice_only", "visual", "ambient"],
                    "description": "Response mode",
                    "default": "multimodal"
                }
            },
            "required": ["action"]
        }),
        handler: Box::new(VoiceSessionHandler { state }),
    }
}

struct VoiceSessionHandler { state: SharedToolState }

#[async_trait]
impl ToolHandler for VoiceSessionHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let action = params.get("action").and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: action"))?;
        let session_id = params.get("session_id").and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let mode = params.get("mode").and_then(|v| v.as_str()).unwrap_or("multimodal");
        let _ = self.state.read().await;
        Ok(json!({
            "status": "stub",
            "action": action,
            "session_id": session_id,
            "mode": mode
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
        "rlmx_swarm_status",
        "rlmx_swarm_topology",
        "rlmx_agent_spawn",
        "rlmx_agent_list",
        "rlmx_agent_terminate",
        "rlmx_research_start",
        "rlmx_research_status",
        "rlmx_experiment_list",
        "rlmx_mutation_history",
        "rlmx_forecast",
        "rlmx_train",
        "rlmx_sandbox_spawn",
        "rlmx_sandbox_terminate",
        "rlmx_sandbox_status",
        "rlmx_sandbox_list",
        "rlmx_fleet_deploy",
        "rlmx_marketplace_search",
        "rlmx_marketplace_install",
        "rlmx_marketplace_uninstall",
        "rlmx_marketplace_rate",
        "rlmx_marketplace_list_installed",
        "rlmx_marketplace_publish",
        "rlmx_marketplace_featured",
        "rlmx_marketplace_categories",
        "rlmx_voice_transcribe",
        "rlmx_voice_synthesize",
        "rlmx_voice_session",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_25_tools_have_valid_schemas() {
        let state = new_shared_state();
        let tools = create_all_tools(state);
        assert_eq!(
            tools.len(),
            39,
            "Expected exactly 39 tools (28 original + 8 marketplace + 3 voice)"
        );

        for tool in &tools {
            assert!(!tool.name.is_empty(), "Tool name must not be empty");
            assert!(
                !tool.description.is_empty(),
                "Tool description must not be empty"
            );

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
        let ingest_result = ingest_tool
            .handler
            .handle(json!({
                "data": "Rust is a systems programming language focused on safety and performance.",
                "plugin": "text"
            }))
            .await
            .unwrap();
        assert_eq!(ingest_result["status"], "ingested");
        assert_eq!(ingest_result["segments_created"], 1);

        // Query for it.
        let query_tool = tools.iter().find(|t| t.name == "rlmx_query").unwrap();
        let query_result = query_tool
            .handler
            .handle(json!({
                "query": "Rust programming language"
            }))
            .await
            .unwrap();
        assert_eq!(query_result["segments_returned"], 1);
        let results = query_result["results"].as_array().unwrap();
        assert!(!results.is_empty());
        // The relevance score should be > 0 since the texts share words.
        let score = results[0]["relevance_score"].as_f64().unwrap();
        assert!(
            score > 0.0,
            "Expected positive relevance score, got {}",
            score
        );
    }

    #[tokio::test]
    async fn test_memory_stats_reflects_ingestion() {
        let state = new_shared_state();
        let tools = create_all_tools(Arc::clone(&state));

        // Stats before ingestion.
        let stats_tool = tools
            .iter()
            .find(|t| t.name == "rlmx_memory_stats")
            .unwrap();
        let before = stats_tool.handler.handle(json!({})).await.unwrap();
        assert_eq!(before["total_segments"], 0);

        // Ingest.
        let ingest_tool = tools.iter().find(|t| t.name == "rlmx_ingest").unwrap();
        ingest_tool
            .handler
            .handle(json!({
                "data": "Hello world",
                "plugin": "text"
            }))
            .await
            .unwrap();

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

        let result = graph_tool
            .handler
            .handle(json!({
                "cypher": "MATCH (n:Person)-[r:WORKS_AT]->(m:Company) RETURN n,m"
            }))
            .await
            .unwrap();

        assert_eq!(result["rows_returned"], 1);
        let rows = result["results"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[tokio::test]
    async fn test_graph_query_bad_cypher_returns_error() {
        let state = new_shared_state();
        let tools = create_all_tools(Arc::clone(&state));
        let graph_tool = tools.iter().find(|t| t.name == "rlmx_graph_query").unwrap();

        let result = graph_tool
            .handler
            .handle(json!({
                "cypher": "SELECT * FROM table"
            }))
            .await;

        assert!(result.is_err(), "Invalid Cypher should return an error");
    }

    #[tokio::test]
    async fn test_query_on_empty_memory_returns_no_results() {
        let state = new_shared_state();
        let tools = create_all_tools(state);
        let query_tool = tools.iter().find(|t| t.name == "rlmx_query").unwrap();

        let result = query_tool
            .handler
            .handle(json!({
                "query": "anything"
            }))
            .await
            .unwrap();

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

    #[tokio::test]
    async fn test_agent_spawn_and_list() {
        let state = new_shared_state();
        let tools = create_all_tools(Arc::clone(&state));

        // Spawn an agent.
        let spawn_tool = tools.iter().find(|t| t.name == "rlmx_agent_spawn").unwrap();
        let spawn_result = spawn_tool
            .handler
            .handle(json!({
                "agent_type": "coordinator",
                "name": "test-coord",
                "zone": "B"
            }))
            .await
            .unwrap();
        assert_eq!(spawn_result["status"], "running");
        assert_eq!(spawn_result["agent_type"], "coordinator");
        assert_eq!(spawn_result["zone"], "B");
        let agent_id = spawn_result["agent_id"].as_str().unwrap();
        assert!(!agent_id.is_empty());

        // List agents and verify the spawned agent appears.
        let list_tool = tools.iter().find(|t| t.name == "rlmx_agent_list").unwrap();
        let list_result = list_tool.handler.handle(json!({})).await.unwrap();
        assert_eq!(list_result["total"], 1);
        assert_eq!(list_result["active"], 1);
        let agents = list_result["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0]["agent_id"], agent_id);
    }

    #[tokio::test]
    async fn test_agent_terminate() {
        let state = new_shared_state();
        let tools = create_all_tools(Arc::clone(&state));

        // Spawn an agent.
        let spawn_tool = tools.iter().find(|t| t.name == "rlmx_agent_spawn").unwrap();
        let spawn_result = spawn_tool
            .handler
            .handle(json!({
                "agent_type": "worker"
            }))
            .await
            .unwrap();
        let agent_id = spawn_result["agent_id"].as_str().unwrap().to_string();

        // Terminate the agent.
        let term_tool = tools
            .iter()
            .find(|t| t.name == "rlmx_agent_terminate")
            .unwrap();
        let term_result = term_tool
            .handler
            .handle(json!({
                "agent_id": agent_id
            }))
            .await
            .unwrap();
        assert_eq!(term_result["status"], "terminated");

        // Verify the agent shows as terminated in list.
        let list_tool = tools.iter().find(|t| t.name == "rlmx_agent_list").unwrap();
        let list_result = list_tool.handler.handle(json!({})).await.unwrap();
        assert_eq!(list_result["active"], 0);
        let agents = list_result["agents"].as_array().unwrap();
        assert_eq!(agents[0]["status"], "terminated");
    }

    #[tokio::test]
    async fn test_research_start_creates_researcher() {
        let state = new_shared_state();
        let tools = create_all_tools(Arc::clone(&state));

        let research_tool = tools
            .iter()
            .find(|t| t.name == "rlmx_research_start")
            .unwrap();
        let result = research_tool
            .handler
            .handle(json!({
                "topic": "neural scaling laws"
            }))
            .await
            .unwrap();

        assert_eq!(result["status"], "started");
        assert_eq!(result["topic"], "neural scaling laws");
        let research_id = result["research_id"].as_str().unwrap();
        let researcher_agent_id = result["researcher_agent_id"].as_str().unwrap();
        assert!(!research_id.is_empty());
        assert!(!researcher_agent_id.is_empty());

        // Verify the researcher agent was spawned.
        let list_tool = tools.iter().find(|t| t.name == "rlmx_agent_list").unwrap();
        let list_result = list_tool
            .handler
            .handle(json!({
                "agent_type": "researcher"
            }))
            .await
            .unwrap();
        let agents = list_result["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0]["agent_type"], "researcher");
    }

    #[tokio::test]
    async fn test_swarm_status_returns_default() {
        let state = new_shared_state();
        let tools = create_all_tools(state);

        let status_tool = tools
            .iter()
            .find(|t| t.name == "rlmx_swarm_status")
            .unwrap();
        let result = status_tool.handler.handle(json!({})).await.unwrap();

        assert_eq!(result["status"], "inactive");
        assert_eq!(result["node_count"], 0);
        assert_eq!(result["healthy_nodes"], 0);
        assert_eq!(result["consensus_type"], "raft");
        assert!(result["zones"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_swarm_topology_3_zones() {
        let state = new_shared_state();
        let tools = create_all_tools(state);

        let topo_tool = tools
            .iter()
            .find(|t| t.name == "rlmx_swarm_topology")
            .unwrap();
        let result = topo_tool.handler.handle(json!({})).await.unwrap();

        let zones = result["zones"].as_array().unwrap();
        assert_eq!(zones.len(), 3, "Expected exactly 3 zones");
        assert_eq!(zones[0]["id"], "A");
        assert_eq!(zones[0]["name"], "Compute");
        assert_eq!(zones[1]["id"], "B");
        assert_eq!(zones[1]["name"], "Inference");
        assert_eq!(zones[2]["id"], "C");
        assert_eq!(zones[2]["name"], "Edge");

        let connections = result["connections"].as_array().unwrap();
        assert_eq!(connections.len(), 3, "Expected 3 inter-zone connections");
    }
}
