//! # rlmx-kernel
//!
//! The RuVix cognition kernel: core syscalls, capability tokens, process model,
//! region memory with vector search, proof engine, in-memory graph, and scheduler.

pub mod a2a;
pub mod approval;
pub mod capability;
pub mod events;
pub mod graph;
pub mod memory;
pub mod process;
pub mod proof;
pub mod router;
pub mod scheduler;
pub mod syscall;
pub mod trigger;
pub mod types;

// Re-export primary types for convenient access.
pub use capability::{CapabilityManager, CapabilityToken};
pub use events::{create_event_bus, DomainEvent, DomainEventBus};
#[cfg(feature = "ruvnet-phase1")]
pub use graph::EnhancedGraph;
pub use graph::Graph;
#[cfg(feature = "ruvector")]
pub use memory::HnswMemoryRegion;
pub use memory::{cosine_similarity, text_to_embedding, ContextSegment, MemoryRegion, EMBED_DIM};
#[cfg(feature = "ruvnet-phase1")]
pub use memory::{BloomScreenedRegion, NamespacedMemoryStore};
pub use process::{Process, ProcessManager, ProcessStatus};
pub use proof::{Proof, ProofEngine, Witness, WitnessChain};
pub use router::{RouterInput, RouterOutput, TinyDancerRouter};
pub use scheduler::{GatherStrategy, Scheduler, SchedulerConfig, Strategy};
pub use syscall::{dispatch, KernelContext, Syscall};
pub use types::{
    Capability, Intent, KernelError, KernelMessage, KernelResult, LifeDomain, MinCutAlgorithm,
    ProcessId, ProofRequest, ResponseMode, SearchFilters, SearchHit, SegmentMetadata, SegmentTier,
    SyscallPermission, SyscallResult, VoicePersona,
};

// ADR-037: Approval tiers
pub use approval::{ApprovalDecision, ApprovalGate, ApprovalPolicy, ApprovalRequest, ApprovalTier};
// ADR-038: Trigger primitives
pub use trigger::{TriggerBinding, TriggerRegistry, TriggerType};
