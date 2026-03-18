//! RLMX MCP Tools
//!
//! Implements all 12 RLMX MCP tool definitions and their handlers.
//! Each tool provides a JSON Schema for input validation and a handler
//! that delegates to the kernel (stub/mock implementations for now).

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::protocol::{McpError, McpTool, ToolHandler};
#[allow(unused_imports)]
use crate::protocol::INVALID_PARAMS;

/// Create all 12 RLMX MCP tools with their handlers.
pub fn create_all_tools() -> Vec<McpTool> {
    vec![
        create_rlmx_query(),
        create_rlmx_ingest(),
        create_rlmx_memory_stats(),
        create_rlmx_plugin_list(),
        create_rlmx_plugin_action(),
        create_rlmx_strategy_override(),
        create_rlmx_trm_classify(),
        create_rlmx_rvf_seal(),
        create_rlmx_rvf_branch(),
        create_rlmx_witness_chain(),
        create_rlmx_sona_stats(),
        create_rlmx_graph_query(),
    ]
}

// ---------------------------------------------------------------------------
// 1. rlmx_query
// ---------------------------------------------------------------------------

fn create_rlmx_query() -> McpTool {
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
        handler: Box::new(RlmxQueryHandler),
    }
}

struct RlmxQueryHandler;

#[async_trait]
impl ToolHandler for RlmxQueryHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let query = params.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: query"))?;

        let context_window = params.get("context_window")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        let strategy = params.get("strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        // Stub: return mock query results
        Ok(json!({
            "query": query,
            "strategy_used": if strategy == "auto" { "rlm" } else { strategy },
            "segments_returned": context_window.min(3),
            "results": [
                {
                    "segment_id": Uuid::new_v4().to_string(),
                    "content": format!("Mock result for query: {}", query),
                    "relevance_score": 0.95,
                    "tier": "hot"
                },
                {
                    "segment_id": Uuid::new_v4().to_string(),
                    "content": "Additional context segment",
                    "relevance_score": 0.82,
                    "tier": "warm"
                }
            ],
            "total_segments_scanned": 1024,
            "latency_ms": 42
        }))
    }
}

// ---------------------------------------------------------------------------
// 2. rlmx_ingest
// ---------------------------------------------------------------------------

fn create_rlmx_ingest() -> McpTool {
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
        handler: Box::new(RlmxIngestHandler),
    }
}

struct RlmxIngestHandler;

#[async_trait]
impl ToolHandler for RlmxIngestHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let _data = params.get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: data"))?;

        let plugin = params.get("plugin")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: plugin"))?;

        Ok(json!({
            "status": "ingested",
            "plugin_used": plugin,
            "segments_created": 3,
            "segment_ids": [
                Uuid::new_v4().to_string(),
                Uuid::new_v4().to_string(),
                Uuid::new_v4().to_string()
            ],
            "timestamp": Utc::now().to_rfc3339()
        }))
    }
}

// ---------------------------------------------------------------------------
// 3. rlmx_memory_stats
// ---------------------------------------------------------------------------

fn create_rlmx_memory_stats() -> McpTool {
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
        handler: Box::new(RlmxMemoryStatsHandler),
    }
}

struct RlmxMemoryStatsHandler;

#[async_trait]
impl ToolHandler for RlmxMemoryStatsHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let include_hnsw = params.get("include_hnsw")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let mut stats = json!({
            "total_segments": 15342,
            "tier_distribution": {
                "hot": 1024,
                "warm": 5120,
                "cold": 9198
            },
            "total_memory_bytes": 268435456,
            "eviction_count": 42
        });

        if include_hnsw {
            stats["hnsw_health"] = json!({
                "index_size": 15342,
                "dimensions": 768,
                "max_layers": 6,
                "ef_construction": 200,
                "fragmentation_ratio": 0.03
            });
        }

        Ok(stats)
    }
}

// ---------------------------------------------------------------------------
// 4. rlmx_plugin_list
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
        Ok(json!({
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
// 5. rlmx_plugin_action
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

        Ok(json!({
            "plugin": plugin,
            "action": action,
            "status": "completed",
            "result": {
                "message": format!("Action '{}' executed successfully on plugin '{}'", action, plugin)
            },
            "execution_time_ms": 15
        }))
    }
}

// ---------------------------------------------------------------------------
// 6. rlmx_strategy_override
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

        Ok(json!({
            "previous_strategy": "auto",
            "new_strategy": strategy,
            "duration_seconds": duration,
            "applied_at": Utc::now().to_rfc3339()
        }))
    }
}

// ---------------------------------------------------------------------------
// 7. rlmx_trm_classify
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
            "classifications": classifications,
            "total_classified": segment_ids.len(),
            "horizon": horizon
        }))
    }
}

// ---------------------------------------------------------------------------
// 8. rlmx_rvf_seal
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

        Ok(json!({
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
// 9. rlmx_rvf_branch
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

        Ok(json!({
            "branch_id": Uuid::new_v4().to_string(),
            "source_container_id": container_id,
            "branch_label": branch_label,
            "cow_pages": 0,
            "created_at": Utc::now().to_rfc3339()
        }))
    }
}

// ---------------------------------------------------------------------------
// 10. rlmx_witness_chain
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

        Ok(json!({
            "target_id": target_id,
            "chain_length": 3,
            "entries": [
                {
                    "event": "created",
                    "timestamp": "2025-01-15T10:30:00Z",
                    "actor": "system",
                    "hash": "abc123"
                },
                {
                    "event": "modified",
                    "timestamp": "2025-01-15T11:00:00Z",
                    "actor": "user",
                    "hash": "def456"
                },
                {
                    "event": "sealed",
                    "timestamp": "2025-01-15T12:00:00Z",
                    "actor": "system",
                    "hash": "ghi789"
                }
            ],
            "integrity_verified": true
        }))
    }
}

// ---------------------------------------------------------------------------
// 11. rlmx_sona_stats
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

        Ok(json!({
            "time_range": time_range,
            "adaptations_count": 127,
            "learning_rate": 0.001,
            "accuracy_improvement": 0.034,
            "feedback_loops": {
                "positive": 89,
                "negative": 12,
                "neutral": 26
            },
            "model_version": "sona-v0.3.1",
            "last_adaptation": Utc::now().to_rfc3339()
        }))
    }
}

// ---------------------------------------------------------------------------
// 12. rlmx_graph_query
// ---------------------------------------------------------------------------

fn create_rlmx_graph_query() -> McpTool {
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
        handler: Box::new(RlmxGraphQueryHandler),
    }
}

struct RlmxGraphQueryHandler;

#[async_trait]
impl ToolHandler for RlmxGraphQueryHandler {
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let cypher = params.get("cypher")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing required parameter: cypher"))?;

        let limit = params.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(100);

        Ok(json!({
            "query": cypher,
            "rows_returned": 2,
            "limit": limit,
            "results": [
                {
                    "node": { "id": "n1", "label": "Entity", "name": "sample_entity_1" },
                    "relationships": [
                        { "type": "RELATES_TO", "target": "n2" }
                    ]
                },
                {
                    "node": { "id": "n2", "label": "Entity", "name": "sample_entity_2" },
                    "relationships": []
                }
            ],
            "execution_time_ms": 8
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
        let tools = create_all_tools();
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
    async fn test_tool_call_dispatch_query() {
        let tools = create_all_tools();
        let query_tool = tools.into_iter().find(|t| t.name == "rlmx_query").unwrap();

        let result = query_tool.handler.handle(json!({
            "query": "What is RLMX?"
        })).await.unwrap();

        assert_eq!(result["strategy_used"], "rlm");
        assert!(result["results"].is_array());
    }

    #[tokio::test]
    async fn test_tool_call_dispatch_missing_params() {
        let tools = create_all_tools();
        let query_tool = tools.into_iter().find(|t| t.name == "rlmx_query").unwrap();

        let result = query_tool.handler.handle(json!({})).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, INVALID_PARAMS);
    }
}
