//! # rlmx-kernel
//!
//! The RuVix cognition kernel: core syscalls, capability tokens, process model,
//! region memory with vector search, proof engine, in-memory graph, and scheduler.

pub mod capability;
pub mod graph;
pub mod memory;
pub mod process;
pub mod proof;
pub mod scheduler;
pub mod syscall;
pub mod types;

// Re-export primary types for convenient access.
pub use capability::{CapabilityManager, CapabilityToken};
pub use graph::Graph;
pub use memory::{cosine_similarity, text_to_embedding, ContextSegment, MemoryRegion, EMBED_DIM};
pub use process::{Process, ProcessManager, ProcessStatus};
pub use proof::{Proof, ProofEngine, Witness, WitnessChain};
pub use scheduler::{Scheduler, SchedulerConfig, Strategy};
pub use syscall::{dispatch, KernelContext, Syscall};
pub use types::*;
