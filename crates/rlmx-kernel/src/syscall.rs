use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::capability::CapabilityManager;
use crate::graph::Graph;
use crate::memory::MemoryRegion;
use crate::process::ProcessManager;
use crate::proof::ProofEngine;
use crate::types::{
    Capability, KernelMessage, KernelResult, MinCutAlgorithm, ProofRequest,
    SearchFilters, SegmentMetadata, SyscallResult, ProcessId,
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

/// Kernel context holding mutable references to all subsystems.
pub struct KernelContext {
    pub memory: Arc<Mutex<MemoryRegion>>,
    pub graph: Arc<Mutex<Graph>>,
    pub process_manager: Arc<Mutex<ProcessManager>>,
    pub proof_engine: Arc<Mutex<ProofEngine>>,
    pub capability_manager: Arc<Mutex<CapabilityManager>>,
    pub caller_pid: Option<ProcessId>,
}

/// Dispatch a syscall to the appropriate kernel subsystem.
///
/// Routes each syscall variant to the correct handler using the provided
/// kernel context.
pub async fn dispatch(syscall: &Syscall, ctx: &KernelContext) -> KernelResult<SyscallResult> {
    match syscall {
        Syscall::VecInsert {
            embedding,
            content,
            metadata,
        } => {
            let mut memory = ctx.memory.lock().await;
            let segment_id = memory.insert(embedding.clone(), content.clone(), metadata.clone());
            Ok(SyscallResult::VecInserted { segment_id })
        }
        Syscall::VecSearch { query, k, filters } => {
            let memory = ctx.memory.lock().await;
            let results = memory.search(query, *k, filters);
            Ok(SyscallResult::VecSearchResults { results })
        }
        Syscall::VecDelete { segment_id } => {
            let mut memory = ctx.memory.lock().await;
            let success = memory.delete(segment_id).unwrap_or(false);
            Ok(SyscallResult::VecDeleted { success })
        }
        Syscall::GraphQuery { cypher } => {
            let graph = ctx.graph.lock().await;
            let rows = graph.cypher_query(cypher)?;
            Ok(SyscallResult::GraphQueryResult { rows })
        }
        Syscall::GraphCut { algorithm, .. } => {
            let graph = ctx.graph.lock().await;
            let (cut_weight, partitions) = graph.min_cut(algorithm)?;
            Ok(SyscallResult::GraphCutResult {
                cut_weight,
                partitions,
            })
        }
        Syscall::GraphDiffuse { signal, steps, .. } => {
            let graph = ctx.graph.lock().await;
            let signal_f64: Vec<f64> = signal.iter().map(|&v| v as f64).collect();
            let output_signal = graph.diffuse(&signal_f64, *steps)?;
            Ok(SyscallResult::GraphDiffused { output_signal })
        }
        Syscall::ProcessFork {
            capabilities,
            memory_scope,
            task,
        } => {
            // Collect all SyscallPermission variants from the requested capabilities.
            let permissions: Vec<crate::types::SyscallPermission> = capabilities
                .iter()
                .flat_map(|cap| cap.permissions.clone())
                .collect();

            // Use the CapabilityManager to create a properly signed token
            // with only the requested permissions (1-hour TTL).
            let child_owner = Uuid::new_v4();
            let mut cm = ctx.capability_manager.lock().await;
            let child_token = cm.create_token(
                child_owner,
                permissions,
                memory_scope.clone(),
                chrono::Duration::hours(1),
            );
            drop(cm);

            let mut pm = ctx.process_manager.lock().await;
            let child_id = pm.fork(None, child_token, memory_scope.clone(), task.clone());
            Ok(SyscallResult::ProcessForked { child_id })
        }
        Syscall::ProcessSend { target, message } => {
            let pm = ctx.process_manager.lock().await;
            pm.send(*target, message.clone()).await?;
            Ok(SyscallResult::MessageSent { delivered: true })
        }
        Syscall::ProcessRecv { timeout } => {
            let pid = ctx.caller_pid.ok_or_else(||
                crate::types::KernelError::Internal("ProcessRecv requires a caller process id".into())
            )?;
            let mut pm = ctx.process_manager.lock().await;
            let msg = pm.recv(pid, timeout.clone()).await?;
            Ok(SyscallResult::MessageReceived { message: msg })
        }
        Syscall::StateMutate {
            action,
            params,
            proof,
        } => {
            let mut engine = ctx.proof_engine.lock().await;
            let evidence: Vec<String> = if let Some(arr) = params.get("evidence") {
                arr.as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                vec![]
            };
            let confidence = params
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.8);
            let reasoning = params
                .get("reasoning")
                .and_then(|v| v.as_str())
                .unwrap_or("syscall state mutation");
            let proof_result = engine.validate(action, reasoning, evidence, confidence, proof)?;
            Ok(SyscallResult::StateMutated {
                witness_id: proof_result.witness_id,
                success: proof_result.valid,
            })
        }
        Syscall::AttentionSelect { operation_type, context_size } => {
            // Choose an attention mechanism and window cap based on the
            // requested operation type.  This avoids a dependency on
            // rlmx-cognitive while still making the syscall operation-aware.
            const DEFAULT_CAP: usize = 1024;

            let (mechanism, cap): (&str, usize) = match operation_type.as_str() {
                "sparse" => ("sparse_topk", DEFAULT_CAP.min(*context_size)),
                "local" | "sliding_window" => {
                    // Local/sliding-window attention: keep a contiguous
                    // window of the most recent positions.
                    ("sliding_window", 512.min(*context_size))
                }
                "global" => {
                    // Global attention still caps to avoid runaway allocs.
                    ("global_full", 4096.min(*context_size))
                }
                "linear" => ("linear_approx", DEFAULT_CAP.min(*context_size)),
                _ => ("dense_capped", DEFAULT_CAP.min(*context_size)),
            };

            let selected: Vec<usize> = if mechanism == "sliding_window" {
                // Select the *last* `cap` positions (most recent context).
                let start = context_size.saturating_sub(cap);
                (start..*context_size).collect()
            } else {
                // Select the first `cap` positions.
                (0..cap).collect()
            };

            Ok(SyscallResult::AttentionSelected {
                selected_indices: selected,
                mechanism: mechanism.to_string(),
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
