//! # rlmx-napi
//!
//! NAPI-RS native binding layer for the RuVix cognition kernel (ADR-020).
//!
//! This crate exposes kernel operations as a Node.js native addon when
//! compiled with the `napi` feature. Without the feature, all public types
//! and stub implementations are available for testing and integration.
//!
//! ## Entry point
//!
//! [`NapiKernel`] is the aggregate root — a DDD-style facade that wraps
//! kernel subsystems and presents a Node-friendly API surface.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, warn};
use uuid::Uuid;

use rlmx_kernel::types::{
    KernelError, ProofRequest, SearchFilters, SegmentMetadata, SyscallPermission,
};
use rlmx_kernel::{
    text_to_embedding, CapabilityManager, Graph, KernelContext, MemoryRegion, ProcessManager,
    ProofEngine, Strategy, Syscall, TinyDancerRouter,
};

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors produced by the NAPI binding layer.
#[derive(Debug, thiserror::Error)]
pub enum NapiError {
    /// A kernel-level error propagated from `rlmx-kernel`.
    #[error("kernel error: {0}")]
    Kernel(#[from] KernelError),

    /// The requested operation is not available in stub mode.
    #[error("unavailable: {reason}")]
    Unavailable { reason: String },

    /// Serialization/deserialization failure.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Invalid argument provided by the caller.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// Internal binding-layer error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Result alias for NAPI operations.
pub type NapiResult<T> = Result<T, NapiError>;

// ---------------------------------------------------------------------------
// Public binding types
// ---------------------------------------------------------------------------

/// A match from the SONA pattern bank, returned by [`NapiKernel::sona_query`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    /// Unique identifier of the matched pattern.
    pub pattern_id: String,
    /// Cosine similarity score in \[0.0, 1.0\].
    pub score: f64,
    /// The content/text of the matched pattern.
    pub content: String,
    /// Arbitrary metadata attached to the pattern.
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Result of an RVF proof-seal operation, returned by [`NapiKernel::rvf_seal`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResult {
    /// Witness identifier in the append-only chain.
    pub witness_id: String,
    /// Whether the proof passed validation.
    pub valid: bool,
    /// Confidence score of the proof.
    pub confidence: f64,
    /// Human-readable reason for the validation outcome.
    pub reason: String,
}

/// Handle to a spawned agent process, returned by [`NapiKernel::agent_spawn`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHandle {
    /// The kernel process ID assigned to this agent.
    pub process_id: String,
    /// Agent type label (e.g. "worker", "researcher").
    pub agent_type: String,
    /// Permissions granted to the agent.
    pub permissions: Vec<String>,
    /// Current status of the agent process.
    pub status: String,
}

/// Health snapshot of the swarm, returned by [`NapiKernel::swarm_status`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmHealth {
    /// Number of active nodes across all zones.
    pub active_nodes: u32,
    /// Per-zone health fractions, keyed by zone name.
    pub zone_health: HashMap<String, f64>,
    /// Current consensus protocol in use.
    pub consensus_protocol: String,
    /// Average latency across the swarm in milliseconds.
    pub avg_latency_ms: f64,
    /// Whether the swarm is considered healthy overall.
    pub healthy: bool,
}

/// Result of a TinyDancerRouter routing decision, returned by [`NapiKernel::route`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// The selected strategy name.
    pub strategy: String,
    /// Confidence of the routing decision in \[0.0, 1.0\].
    pub confidence: f64,
    /// Per-strategy scores from the softmax output.
    pub scores: HashMap<String, f64>,
    /// Forward-pass latency in nanoseconds.
    pub latency_ns: u64,
}

/// Result of a kernel syscall dispatch, returned by [`NapiKernel::dispatch`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchResult {
    /// The syscall family that was dispatched.
    pub syscall_type: String,
    /// Whether the dispatch succeeded.
    pub success: bool,
    /// JSON-serialized syscall result payload.
    pub payload: serde_json::Value,
}

// ---------------------------------------------------------------------------
// NapiKernel — aggregate root
// ---------------------------------------------------------------------------

/// The NAPI binding aggregate root.
///
/// Wraps kernel subsystems and exposes a Node-friendly API. When compiled
/// without the `napi` feature, all methods return typed stub responses.
pub struct NapiKernel {
    memory: Arc<Mutex<MemoryRegion>>,
    graph: Arc<Mutex<Graph>>,
    process_manager: Arc<Mutex<ProcessManager>>,
    proof_engine: Arc<Mutex<ProofEngine>>,
    capability_manager: Arc<Mutex<CapabilityManager>>,
    router: Arc<Mutex<TinyDancerRouter>>,
}

impl NapiKernel {
    /// Create a new `NapiKernel` with default subsystems.
    pub fn new() -> Self {
        debug!("NapiKernel: initializing kernel subsystems");
        Self {
            memory: Arc::new(Mutex::new(MemoryRegion::new("napi"))),
            graph: Arc::new(Mutex::new(Graph::new())),
            process_manager: Arc::new(Mutex::new(ProcessManager::new())),
            proof_engine: Arc::new(Mutex::new(ProofEngine::new())),
            capability_manager: Arc::new(Mutex::new(CapabilityManager::new())),
            router: Arc::new(Mutex::new(TinyDancerRouter::new())),
        }
    }

    /// Dispatch a kernel syscall by name with JSON parameters.
    ///
    /// Builds the appropriate [`Syscall`] variant from the provided type
    /// string and params, then delegates to `rlmx_kernel::dispatch`.
    pub async fn dispatch(
        &self,
        syscall_type: &str,
        params: serde_json::Value,
    ) -> NapiResult<DispatchResult> {
        let syscall = Self::build_syscall(syscall_type, &params)?;

        let ctx = KernelContext {
            memory: Arc::clone(&self.memory),
            graph: Arc::clone(&self.graph),
            process_manager: Arc::clone(&self.process_manager),
            proof_engine: Arc::clone(&self.proof_engine),
            capability_manager: Arc::clone(&self.capability_manager),
            caller_pid: None,
            event_bus: None,
        };

        let result = rlmx_kernel::dispatch(&syscall, &ctx).await?;

        let payload = serde_json::to_value(&result).map_err(|e| {
            NapiError::Serialization(format!("failed to serialize syscall result: {e}"))
        })?;

        debug!(syscall_type, "NapiKernel: dispatch completed");

        Ok(DispatchResult {
            syscall_type: syscall_type.to_string(),
            success: true,
            payload,
        })
    }

    /// Query the SONA pattern bank for similar patterns.
    ///
    /// In stub mode (no SONA engine connected), returns synthetic matches
    /// derived from the kernel memory region via cosine similarity.
    pub async fn sona_query(&self, query: &str, top_k: usize) -> NapiResult<Vec<PatternMatch>> {
        if query.is_empty() {
            return Err(NapiError::InvalidArgument("query must not be empty".into()));
        }

        let k = if top_k == 0 { 5 } else { top_k };

        debug!(query, top_k = k, "NapiKernel: sona_query");

        let embedding = text_to_embedding(query);
        let memory = self.memory.lock().await;
        let hits = memory.search(&embedding, k, &SearchFilters::default());

        let matches: Vec<PatternMatch> = hits
            .into_iter()
            .map(|hit| PatternMatch {
                pattern_id: hit.segment_id.to_string(),
                score: hit.score,
                content: hit.content,
                metadata: hit.metadata.extra,
            })
            .collect();

        Ok(matches)
    }

    /// Create a proof seal for a state mutation via the kernel proof engine.
    pub async fn rvf_seal(
        &self,
        action: &str,
        reasoning: &str,
        confidence: f64,
    ) -> NapiResult<ProofResult> {
        if action.is_empty() {
            return Err(NapiError::InvalidArgument(
                "action must not be empty".into(),
            ));
        }

        debug!(action, confidence, "NapiKernel: rvf_seal");

        let request = ProofRequest {
            confidence_threshold: 0.7,
            require_evidence: false,
        };

        let mut engine = self.proof_engine.lock().await;
        let proof = engine.validate(action, reasoning, vec![], confidence, &request)?;

        Ok(ProofResult {
            witness_id: proof.witness_id.to_string(),
            valid: proof.valid,
            confidence: proof.confidence,
            reason: proof.reason,
        })
    }

    /// Spawn an agent process with the given type and permissions.
    pub async fn agent_spawn(
        &self,
        agent_type: &str,
        permissions: Vec<String>,
    ) -> NapiResult<AgentHandle> {
        if agent_type.is_empty() {
            return Err(NapiError::InvalidArgument(
                "agent_type must not be empty".into(),
            ));
        }

        debug!(agent_type, "NapiKernel: agent_spawn");

        let syscall_perms: Vec<SyscallPermission> = permissions
            .iter()
            .filter_map(|p| parse_permission(p))
            .collect();

        let owner = Uuid::new_v4();
        let mut cm = self.capability_manager.lock().await;
        let token = cm.create_token(
            owner,
            syscall_perms,
            "napi".to_string(),
            chrono::Duration::hours(1),
        );
        drop(cm);

        let mut pm = self.process_manager.lock().await;
        let pid = pm.fork(
            None,
            token,
            "napi".to_string(),
            format!("napi-agent-{agent_type}"),
        );

        Ok(AgentHandle {
            process_id: pid.to_string(),
            agent_type: agent_type.to_string(),
            permissions,
            status: "running".to_string(),
        })
    }

    /// Return a health snapshot of the swarm.
    ///
    /// In stub mode, returns a synthetic health report with default zone
    /// health values since no real swarm is connected.
    pub async fn swarm_status(&self) -> NapiResult<SwarmHealth> {
        debug!("NapiKernel: swarm_status");

        let mut zone_health = HashMap::new();
        zone_health.insert("zone-a-mobile".to_string(), 1.0);
        zone_health.insert("zone-a-desktop".to_string(), 1.0);
        zone_health.insert("zone-b-cloud".to_string(), 0.95);
        zone_health.insert("zone-c-edge".to_string(), 0.90);
        zone_health.insert("zone-d-browser".to_string(), 0.85);

        Ok(SwarmHealth {
            active_nodes: 5,
            zone_health,
            consensus_protocol: "raft".to_string(),
            avg_latency_ms: 12.5,
            healthy: true,
        })
    }

    /// Route a query through the TinyDancerRouter.
    pub async fn route(&self, query: &str) -> NapiResult<RoutingDecision> {
        if query.is_empty() {
            return Err(NapiError::InvalidArgument("query must not be empty".into()));
        }

        debug!(query, "NapiKernel: route");

        let input = TinyDancerRouter::extract_features(query, false, 0.3);
        let router = self.router.lock().await;
        let output = router.route(&input);

        let strategy_name = match &output.strategy {
            Strategy::Rlm => "Rlm".to_string(),
            Strategy::Trm(model) => format!("Trm({model})"),
            Strategy::Edge => "Edge".to_string(),
            Strategy::Hybrid { triage, .. } => format!("Hybrid({triage})"),
            Strategy::Swarm { .. } => "Swarm".to_string(),
            Strategy::Auto => "Auto".to_string(),
        };

        Ok(RoutingDecision {
            strategy: strategy_name,
            confidence: output.confidence,
            scores: output.scores,
            latency_ns: output.latency_ns,
        })
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Parse a syscall type string and JSON params into a [`Syscall`] variant.
    fn build_syscall(syscall_type: &str, params: &serde_json::Value) -> NapiResult<Syscall> {
        match syscall_type {
            "HaltCheck" => {
                let confidence = params
                    .get("confidence")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5);
                let iteration = params
                    .get("iteration")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                let max_iterations = params
                    .get("max_iterations")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;
                Ok(Syscall::HaltCheck {
                    confidence,
                    iteration,
                    max_iterations,
                })
            }
            "VecInsert" => {
                let content = params
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let embedding = text_to_embedding(&content);
                let metadata = SegmentMetadata {
                    source: params
                        .get("source")
                        .and_then(|v| v.as_str())
                        .unwrap_or("napi")
                        .to_string(),
                    ..Default::default()
                };
                Ok(Syscall::VecInsert {
                    embedding,
                    content,
                    metadata,
                })
            }
            "VecSearch" => {
                let query_text = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let k = params.get("k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
                let query = text_to_embedding(query_text);
                Ok(Syscall::VecSearch {
                    query,
                    k,
                    filters: SearchFilters::default(),
                })
            }
            "VecDelete" => {
                let id_str = params
                    .get("segment_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let segment_id = Uuid::parse_str(id_str)
                    .map_err(|e| NapiError::InvalidArgument(format!("invalid segment_id: {e}")))?;
                Ok(Syscall::VecDelete { segment_id })
            }
            "StateMutate" => {
                let action = params
                    .get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("napi-mutation")
                    .to_string();
                let proof = ProofRequest::default();
                Ok(Syscall::StateMutate {
                    action,
                    params: params.clone(),
                    proof,
                })
            }
            "AttentionSelect" => {
                let operation_type = params
                    .get("operation_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dense")
                    .to_string();
                let context_size = params
                    .get("context_size")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(512) as usize;
                Ok(Syscall::AttentionSelect {
                    operation_type,
                    context_size,
                })
            }
            "VoiceTranscribe" => {
                let audio_len = params
                    .get("audio_len")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                let language_hint = params
                    .get("language_hint")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                Ok(Syscall::VoiceTranscribe {
                    audio_len,
                    language_hint,
                    tier_override: None,
                })
            }
            "VoiceSynthesize" => {
                let text = params
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let persona = params
                    .get("persona")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Finance")
                    .to_string();
                let streaming = params
                    .get("streaming")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(Syscall::VoiceSynthesize {
                    text,
                    persona,
                    streaming,
                })
            }
            "IntentRoute" => {
                let transcript = params
                    .get("transcript")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let decompose = params
                    .get("decompose")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let max_intents = params
                    .get("max_intents")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);
                Ok(Syscall::IntentRoute {
                    transcript,
                    decompose,
                    max_intents,
                })
            }
            other => Err(NapiError::InvalidArgument(format!(
                "unknown syscall type: {other}"
            ))),
        }
    }
}

impl Default for NapiKernel {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Feature-gated NAPI-RS bindings
// ---------------------------------------------------------------------------

/// When the `napi` feature is enabled, expose `NapiKernel` methods as
/// N-API functions callable from Node.js. Each function wraps the async
/// Rust method and bridges through napi-rs's task scheduling.
#[cfg(feature = "napi")]
mod napi_bindings {
    use super::*;
    use napi_derive::napi;
    use std::sync::OnceLock;

    /// Shared singleton: Tokio runtime + NapiKernel, initialized once.
    static SHARED: OnceLock<(tokio::runtime::Runtime, NapiKernel)> = OnceLock::new();

    fn shared() -> napi::Result<&'static (tokio::runtime::Runtime, NapiKernel)> {
        Ok(SHARED.get_or_init(|| {
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            let kernel = NapiKernel::new();
            (rt, kernel)
        }))
    }

    #[napi]
    pub fn napi_dispatch(
        syscall_type: String,
        params: serde_json::Value,
    ) -> napi::Result<serde_json::Value> {
        let (rt, kernel) = shared()?;
        let result = rt
            .block_on(kernel.dispatch(&syscall_type, params))
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        serde_json::to_value(&result)
            .map_err(|e| napi::Error::from_reason(format!("serialization: {e}")))
    }

    #[napi]
    pub fn napi_sona_query(query: String, top_k: u32) -> napi::Result<serde_json::Value> {
        let (rt, kernel) = shared()?;
        let result = rt
            .block_on(kernel.sona_query(&query, top_k as usize))
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        serde_json::to_value(&result)
            .map_err(|e| napi::Error::from_reason(format!("serialization: {e}")))
    }

    #[napi]
    pub fn napi_rvf_seal(
        action: String,
        reasoning: String,
        confidence: f64,
    ) -> napi::Result<serde_json::Value> {
        let (rt, kernel) = shared()?;
        let result = rt
            .block_on(kernel.rvf_seal(&action, &reasoning, confidence))
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        serde_json::to_value(&result)
            .map_err(|e| napi::Error::from_reason(format!("serialization: {e}")))
    }

    #[napi]
    pub fn napi_agent_spawn(
        agent_type: String,
        permissions: Vec<String>,
    ) -> napi::Result<serde_json::Value> {
        let (rt, kernel) = shared()?;
        let result = rt
            .block_on(kernel.agent_spawn(&agent_type, permissions))
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        serde_json::to_value(&result)
            .map_err(|e| napi::Error::from_reason(format!("serialization: {e}")))
    }

    #[napi]
    pub fn napi_swarm_status() -> napi::Result<serde_json::Value> {
        let (rt, kernel) = shared()?;
        let result = rt
            .block_on(kernel.swarm_status())
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        serde_json::to_value(&result)
            .map_err(|e| napi::Error::from_reason(format!("serialization: {e}")))
    }

    #[napi]
    pub fn napi_route(query: String) -> napi::Result<serde_json::Value> {
        let (rt, kernel) = shared()?;
        let result = rt
            .block_on(kernel.route(&query))
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        serde_json::to_value(&result)
            .map_err(|e| napi::Error::from_reason(format!("serialization: {e}")))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a permission string into a [`SyscallPermission`].
fn parse_permission(s: &str) -> Option<SyscallPermission> {
    match s {
        "VecInsert" => Some(SyscallPermission::VecInsert),
        "VecSearch" => Some(SyscallPermission::VecSearch),
        "VecDelete" => Some(SyscallPermission::VecDelete),
        "GraphQuery" => Some(SyscallPermission::GraphQuery),
        "GraphCut" => Some(SyscallPermission::GraphCut),
        "GraphDiffuse" => Some(SyscallPermission::GraphDiffuse),
        "ProcessFork" => Some(SyscallPermission::ProcessFork),
        "ProcessSend" => Some(SyscallPermission::ProcessSend),
        "ProcessRecv" => Some(SyscallPermission::ProcessRecv),
        "StateMutate" => Some(SyscallPermission::StateMutate),
        "AttentionSelect" => Some(SyscallPermission::AttentionSelect),
        "HaltCheck" => Some(SyscallPermission::HaltCheck),
        "VoiceTranscribe" => Some(SyscallPermission::VoiceTranscribe),
        "VoiceSynthesize" => Some(SyscallPermission::VoiceSynthesize),
        "IntentRoute" => Some(SyscallPermission::IntentRoute),
        "MeshSync" => Some(SyscallPermission::MeshSync),
        "FederationContribute" => Some(SyscallPermission::FederationContribute),
        "All" => Some(SyscallPermission::All),
        other => {
            warn!(permission = other, "NapiKernel: unknown permission string");
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn kernel() -> NapiKernel {
        NapiKernel::new()
    }

    // -- dispatch tests --

    #[tokio::test]
    async fn test_dispatch_halt_check() {
        let k = kernel();
        let params = serde_json::json!({
            "confidence": 0.99,
            "iteration": 5,
            "max_iterations": 10
        });
        let result = k.dispatch("HaltCheck", params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.syscall_type, "HaltCheck");
        assert!(result.payload.get("HaltDecision").is_some());
    }

    #[tokio::test]
    async fn test_dispatch_vec_insert_and_search() {
        let k = kernel();

        // Insert a segment.
        let insert_params = serde_json::json!({
            "content": "Rust is a systems programming language",
            "source": "test"
        });
        let insert_result = k.dispatch("VecInsert", insert_params).await.unwrap();
        assert!(insert_result.success);

        // Search for it.
        let search_params = serde_json::json!({
            "query": "systems programming",
            "k": 3
        });
        let search_result = k.dispatch("VecSearch", search_params).await.unwrap();
        assert!(search_result.success);
    }

    #[tokio::test]
    async fn test_dispatch_unknown_syscall() {
        let k = kernel();
        let result = k.dispatch("NonExistent", serde_json::json!({})).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NapiError::InvalidArgument(msg) => {
                assert!(msg.contains("unknown syscall type"));
            }
            other => panic!("expected InvalidArgument, got: {other}"),
        }
    }

    #[tokio::test]
    async fn test_dispatch_state_mutate() {
        let k = kernel();
        let params = serde_json::json!({
            "action": "test-mutation",
            "confidence": 0.9
        });
        let result = k.dispatch("StateMutate", params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.syscall_type, "StateMutate");
    }

    #[tokio::test]
    async fn test_dispatch_attention_select() {
        let k = kernel();
        let params = serde_json::json!({
            "operation_type": "sparse",
            "context_size": 256
        });
        let result = k.dispatch("AttentionSelect", params).await.unwrap();
        assert!(result.success);
    }

    // -- sona_query tests --

    #[tokio::test]
    async fn test_sona_query_empty_returns_error() {
        let k = kernel();
        let result = k.sona_query("", 5).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NapiError::InvalidArgument(msg) => {
                assert!(msg.contains("empty"));
            }
            other => panic!("expected InvalidArgument, got: {other}"),
        }
    }

    #[tokio::test]
    async fn test_sona_query_returns_vec() {
        let k = kernel();
        let matches = k.sona_query("test query", 3).await.unwrap();
        // Empty memory region, so no matches expected.
        assert!(matches.is_empty());
    }

    // -- rvf_seal tests --

    #[tokio::test]
    async fn test_rvf_seal_valid() {
        let k = kernel();
        let result = k
            .rvf_seal("update-state", "good reasoning", 0.9)
            .await
            .unwrap();
        assert!(result.valid);
        assert!(result.confidence >= 0.9 - f64::EPSILON);
        assert!(!result.witness_id.is_empty());
    }

    #[tokio::test]
    async fn test_rvf_seal_invalid_below_threshold() {
        let k = kernel();
        let result = k
            .rvf_seal("risky-action", "weak reasoning", 0.3)
            .await
            .unwrap();
        assert!(!result.valid);
    }

    #[tokio::test]
    async fn test_rvf_seal_empty_action_error() {
        let k = kernel();
        let result = k.rvf_seal("", "reasoning", 0.9).await;
        assert!(result.is_err());
    }

    // -- agent_spawn tests --

    #[tokio::test]
    async fn test_agent_spawn_success() {
        let k = kernel();
        let handle = k
            .agent_spawn("worker", vec!["VecSearch".into(), "VecInsert".into()])
            .await
            .unwrap();
        assert_eq!(handle.agent_type, "worker");
        assert_eq!(handle.status, "running");
        assert!(!handle.process_id.is_empty());
        // Validate that process_id is a valid UUID.
        Uuid::parse_str(&handle.process_id).expect("process_id should be valid UUID");
    }

    #[tokio::test]
    async fn test_agent_spawn_empty_type_error() {
        let k = kernel();
        let result = k.agent_spawn("", vec![]).await;
        assert!(result.is_err());
    }

    // -- swarm_status tests --

    #[tokio::test]
    async fn test_swarm_status_stub() {
        let k = kernel();
        let health = k.swarm_status().await.unwrap();
        assert!(health.healthy);
        assert_eq!(health.active_nodes, 5);
        assert_eq!(health.consensus_protocol, "raft");
        assert!(health.zone_health.contains_key("zone-a-mobile"));
        assert!(health.zone_health.contains_key("zone-d-browser"));
    }

    // -- route tests --

    #[tokio::test]
    async fn test_route_question() {
        let k = kernel();
        let decision = k.route("What is the meaning of life?").await.unwrap();
        assert!(decision.confidence > 0.0);
        assert!(decision.confidence <= 1.0);
        assert!(!decision.strategy.is_empty());
        assert!(!decision.scores.is_empty());
        // Scores should sum to ~1.0.
        let total: f64 = decision.scores.values().sum();
        assert!(
            (total - 1.0).abs() < 1e-6,
            "scores should sum to 1.0, got {total}"
        );
    }

    #[tokio::test]
    async fn test_route_empty_query_error() {
        let k = kernel();
        let result = k.route("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_route_code_query() {
        let k = kernel();
        let decision = k.route("```rust\nfn main() {}\n```").await.unwrap();
        assert!(decision.confidence > 0.0);
        // Code queries should have non-trivial Trm score.
        let trm_score = decision.scores.get("Trm").copied().unwrap_or(0.0);
        assert!(trm_score > 0.01, "Trm should have notable score for code");
    }

    // -- parse_permission tests --

    #[test]
    fn test_parse_permission_known() {
        assert_eq!(
            parse_permission("VecInsert"),
            Some(SyscallPermission::VecInsert)
        );
        assert_eq!(parse_permission("All"), Some(SyscallPermission::All));
        assert_eq!(
            parse_permission("VoiceTranscribe"),
            Some(SyscallPermission::VoiceTranscribe)
        );
    }

    #[test]
    fn test_parse_permission_unknown() {
        assert_eq!(parse_permission("Bogus"), None);
    }

    // -- type serialization tests --

    #[test]
    fn test_pattern_match_serialization() {
        let pm = PatternMatch {
            pattern_id: "abc-123".into(),
            score: 0.95,
            content: "test content".into(),
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&pm).unwrap();
        let deserialized: PatternMatch = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pattern_id, "abc-123");
        assert!((deserialized.score - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_proof_result_serialization() {
        let pr = ProofResult {
            witness_id: "w-001".into(),
            valid: true,
            confidence: 0.88,
            reason: "threshold met".into(),
        };
        let json = serde_json::to_string(&pr).unwrap();
        let deserialized: ProofResult = serde_json::from_str(&json).unwrap();
        assert!(deserialized.valid);
    }

    #[test]
    fn test_agent_handle_serialization() {
        let ah = AgentHandle {
            process_id: Uuid::new_v4().to_string(),
            agent_type: "researcher".into(),
            permissions: vec!["VecSearch".into()],
            status: "running".into(),
        };
        let json = serde_json::to_string(&ah).unwrap();
        let deserialized: AgentHandle = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.agent_type, "researcher");
    }

    #[test]
    fn test_swarm_health_serialization() {
        let sh = SwarmHealth {
            active_nodes: 3,
            zone_health: HashMap::new(),
            consensus_protocol: "pbft".into(),
            avg_latency_ms: 5.0,
            healthy: true,
        };
        let json = serde_json::to_string(&sh).unwrap();
        let deserialized: SwarmHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.active_nodes, 3);
    }

    #[test]
    fn test_routing_decision_serialization() {
        let rd = RoutingDecision {
            strategy: "Rlm".into(),
            confidence: 0.75,
            scores: {
                let mut s = HashMap::new();
                s.insert("Rlm".into(), 0.75);
                s.insert("Trm".into(), 0.25);
                s
            },
            latency_ns: 500,
        };
        let json = serde_json::to_string(&rd).unwrap();
        let deserialized: RoutingDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.strategy, "Rlm");
    }

    #[test]
    fn test_dispatch_result_serialization() {
        let dr = DispatchResult {
            syscall_type: "HaltCheck".into(),
            success: true,
            payload: serde_json::json!({"should_halt": true}),
        };
        let json = serde_json::to_string(&dr).unwrap();
        let deserialized: DispatchResult = serde_json::from_str(&json).unwrap();
        assert!(deserialized.success);
    }

    // -- NapiKernel default trait --

    #[test]
    fn test_napi_kernel_default() {
        let _k = NapiKernel::default();
        // Just verifies Default impl compiles and doesn't panic.
    }

    // -- dispatch voice syscalls --

    #[tokio::test]
    async fn test_dispatch_voice_transcribe() {
        let k = kernel();
        let params = serde_json::json!({
            "audio_len": 16000,
            "language_hint": "en"
        });
        let result = k.dispatch("VoiceTranscribe", params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.syscall_type, "VoiceTranscribe");
    }

    #[tokio::test]
    async fn test_dispatch_voice_synthesize() {
        let k = kernel();
        let params = serde_json::json!({
            "text": "Hello, how can I help you today?",
            "persona": "Finance",
            "streaming": false
        });
        let result = k.dispatch("VoiceSynthesize", params).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_dispatch_intent_route() {
        let k = kernel();
        let params = serde_json::json!({
            "transcript": "I want to pay my bills and schedule a doctor appointment",
            "decompose": true,
            "max_intents": 5
        });
        let result = k.dispatch("IntentRoute", params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.syscall_type, "IntentRoute");
    }

    // -- error type coverage --

    #[test]
    fn test_napi_error_display() {
        let e1 = NapiError::Unavailable {
            reason: "no engine".into(),
        };
        assert!(e1.to_string().contains("unavailable"));

        let e2 = NapiError::Serialization("bad json".into());
        assert!(e2.to_string().contains("serialization"));

        let e3 = NapiError::Internal("oops".into());
        assert!(e3.to_string().contains("internal"));

        let e4 = NapiError::InvalidArgument("bad arg".into());
        assert!(e4.to_string().contains("invalid argument"));
    }
}
