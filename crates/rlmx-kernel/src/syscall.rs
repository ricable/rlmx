use std::time::Duration;
use uuid::Uuid;

use crate::types::{
    Capability, KernelMessage, KernelResult, MinCutAlgorithm, ProofRequest, SearchFilters,
    SegmentMetadata, SyscallResult,
    ProcessId,
};

/// The 12 RuVix kernel syscalls.
#[derive(Debug, Clone)]
pub enum Syscall {
    VecInsert {
        embedding: Vec<f32>,
        content: String,
        metadata: SegmentMetadata,
    },
    VecSearch {
        query: Vec<f32>,
        k: usize,
        filters: SearchFilters,
    },
    VecDelete {
        segment_id: Uuid,
    },
    GraphQuery {
        cypher: String,
    },
    GraphCut {
        graph_id: Uuid,
        algorithm: MinCutAlgorithm,
    },
    GraphDiffuse {
        graph_id: Uuid,
        signal: Vec<f32>,
        steps: usize,
    },
    ProcessFork {
        capabilities: Vec<Capability>,
        memory_scope: String,
        task: String,
    },
    ProcessSend {
        target: ProcessId,
        message: KernelMessage,
    },
    ProcessRecv {
        timeout: Option<Duration>,
    },
    StateMutate {
        action: String,
        params: serde_json::Value,
        proof: ProofRequest,
    },
    AttentionSelect {
        operation_type: String,
        context_size: usize,
    },
    HaltCheck {
        confidence: f64,
        iteration: usize,
        max_iterations: usize,
    },
}

impl Syscall {
    /// Return the syscall family name (for capability checking).
    pub fn family(&self) -> &'static str {
        match self {
            Syscall::VecInsert { .. } => "VecInsert",
            Syscall::VecSearch { .. } => "VecSearch",
            Syscall::VecDelete { .. } => "VecDelete",
            Syscall::GraphQuery { .. } => "GraphQuery",
            Syscall::GraphCut { .. } => "GraphCut",
            Syscall::GraphDiffuse { .. } => "GraphDiffuse",
            Syscall::ProcessFork { .. } => "ProcessFork",
            Syscall::ProcessSend { .. } => "ProcessSend",
            Syscall::ProcessRecv { .. } => "ProcessRecv",
            Syscall::StateMutate { .. } => "StateMutate",
            Syscall::AttentionSelect { .. } => "AttentionSelect",
            Syscall::HaltCheck { .. } => "HaltCheck",
        }
    }
}

/// Dispatch a syscall to the appropriate kernel subsystem.
///
/// This is the central dispatch function that routes each syscall variant
/// to the correct handler. In a full implementation, `ctx` would be a
/// kernel context holding references to all subsystems.
pub fn dispatch(syscall: &Syscall) -> KernelResult<SyscallResult> {
    match syscall {
        Syscall::VecInsert { .. } => {
            // Handled by memory subsystem via kernel context.
            Ok(SyscallResult::VecInserted {
                segment_id: Uuid::new_v4(),
            })
        }
        Syscall::VecSearch { query, k, filters: _ } => {
            // Placeholder: return empty results.
            let _ = (query, k);
            Ok(SyscallResult::VecSearchResults { results: vec![] })
        }
        Syscall::VecDelete { segment_id } => {
            let _ = segment_id;
            Ok(SyscallResult::VecDeleted { success: true })
        }
        Syscall::GraphQuery { cypher } => {
            let _ = cypher;
            Ok(SyscallResult::GraphQueryResult { rows: vec![] })
        }
        Syscall::GraphCut { .. } => Ok(SyscallResult::GraphCutResult {
            cut_weight: 0.0,
            partitions: vec![],
        }),
        Syscall::GraphDiffuse { .. } => Ok(SyscallResult::GraphDiffused {
            output_signal: vec![],
        }),
        Syscall::ProcessFork { .. } => Ok(SyscallResult::ProcessForked {
            child_id: Uuid::new_v4(),
        }),
        Syscall::ProcessSend { .. } => Ok(SyscallResult::MessageSent { delivered: true }),
        Syscall::ProcessRecv { .. } => Ok(SyscallResult::MessageReceived { message: None }),
        Syscall::StateMutate { .. } => Ok(SyscallResult::StateMutated {
            witness_id: Uuid::new_v4(),
            success: true,
        }),
        Syscall::AttentionSelect { context_size, .. } => {
            // Placeholder: select all indices.
            let selected: Vec<usize> = (0..*context_size).collect();
            Ok(SyscallResult::AttentionSelected {
                selected_indices: selected,
            })
        }
        Syscall::HaltCheck {
            confidence,
            iteration,
            max_iterations,
        } => {
            let should_halt = *confidence > 0.95 || *iteration >= *max_iterations;
            let reason = if *confidence > 0.95 {
                "confidence threshold reached".into()
            } else if *iteration >= *max_iterations {
                "max iterations reached".into()
            } else {
                "continue".into()
            };
            Ok(SyscallResult::HaltDecision {
                should_halt,
                reason,
            })
        }
    }
}
