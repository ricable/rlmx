use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Unique identifier for a kernel process.
pub type ProcessId = Uuid;

// ---------------------------------------------------------------------------
// Segment & Search types
// ---------------------------------------------------------------------------

/// Tier classification for context segments based on access frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentTier {
    Hot,
    Warm,
    Cold,
}

/// Metadata attached to a context segment on insertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentMetadata {
    pub source: String,
    pub plugin: Option<String>,
    pub segment_type: String,
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for SegmentMetadata {
    fn default() -> Self {
        Self {
            source: "unknown".into(),
            plugin: None,
            segment_type: "text".into(),
            extra: HashMap::new(),
        }
    }
}

/// Filters applied during vector search.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    pub source: Option<String>,
    pub plugin: Option<String>,
    pub segment_type: Option<String>,
    pub tier: Option<SegmentTier>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Graph types
// ---------------------------------------------------------------------------

/// Algorithm selection for graph min-cut operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MinCutAlgorithm {
    Karger,
    StoerWagner,
}

// ---------------------------------------------------------------------------
// Capability types
// ---------------------------------------------------------------------------

/// Permission for a specific syscall family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyscallPermission {
    VecInsert,
    VecSearch,
    VecDelete,
    GraphQuery,
    GraphCut,
    GraphDiffuse,
    ProcessFork,
    ProcessSend,
    ProcessRecv,
    StateMutate,
    AttentionSelect,
    HaltCheck,
    All,
}

/// A capability granted to a process, restricting which syscalls it may invoke.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub permissions: Vec<SyscallPermission>,
}

// ---------------------------------------------------------------------------
// Message types
// ---------------------------------------------------------------------------

/// A message sent between kernel processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelMessage {
    pub from: ProcessId,
    pub to: ProcessId,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl KernelMessage {
    pub fn new(from: ProcessId, to: ProcessId, payload: serde_json::Value) -> Self {
        Self {
            from,
            to,
            payload,
            timestamp: Utc::now(),
        }
    }
}

// ---------------------------------------------------------------------------
// Proof types
// ---------------------------------------------------------------------------

/// A request to create a proof for a state mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRequest {
    pub confidence_threshold: f64,
    pub require_evidence: bool,
}

impl Default for ProofRequest {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.7,
            require_evidence: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Syscall result
// ---------------------------------------------------------------------------

/// The result of executing a syscall.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyscallResult {
    VecInserted { segment_id: Uuid },
    VecSearchResults { results: Vec<SearchHit> },
    VecDeleted { success: bool },
    GraphQueryResult { rows: Vec<serde_json::Value> },
    GraphCutResult { cut_weight: f64, partitions: Vec<Vec<Uuid>> },
    GraphDiffused { output_signal: Vec<f64> },
    ProcessForked { child_id: ProcessId },
    MessageSent { delivered: bool },
    MessageReceived { message: Option<KernelMessage> },
    StateMutated { witness_id: Uuid, success: bool },
    AttentionSelected { selected_indices: Vec<usize>, mechanism: String },
    HaltDecision { should_halt: bool, reason: String },
}

/// A single search hit with score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub segment_id: Uuid,
    pub content: String,
    pub score: f64,
    pub metadata: SegmentMetadata,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Kernel-level errors.
#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("capability denied: {0}")]
    CapabilityDenied(String),

    #[error("process not found: {0}")]
    ProcessNotFound(ProcessId),

    #[error("segment not found: {0}")]
    SegmentNotFound(Uuid),

    #[error("graph error: {0}")]
    GraphError(String),

    #[error("proof validation failed: {0}")]
    ProofError(String),

    #[error("scheduler error: {0}")]
    SchedulerError(String),

    #[error("timeout after {0:?}")]
    Timeout(Duration),

    #[error("channel error: {0}")]
    ChannelError(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type KernelResult<T> = Result<T, KernelError>;
