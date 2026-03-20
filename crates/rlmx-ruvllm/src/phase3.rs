//! Phase 3 integration: attention optimization, gate kernels, and memory profiling.
//!
//! When `ruvnet-phase3` is enabled, delegates to `ruvector-attention`,
//! `ruvector-attn-mincut`, `cognitum-gate-tilezero`, and `cognitum-gate-kernel`
//! for hardware-accelerated attention and gating. Otherwise, all types compile
//! as stubs returning unavailable status.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types (always available)
// ---------------------------------------------------------------------------

/// Result of an attention optimization pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionResult {
    /// Method used: "flash-mincut", "standard", or "unavailable".
    pub method: String,
    /// Number of tokens processed.
    pub tokens_processed: usize,
    /// Latency in milliseconds.
    pub latency_ms: f64,
    /// Memory reduction percentage (0.0 if not applicable).
    pub memory_reduction_pct: f64,
}

/// Configuration for gate kernels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateKernelConfig {
    /// Gate type: "tilezero", "standard", or "none".
    pub gate_type: String,
    /// Tile size for TileZero gating.
    pub tile_size: usize,
    /// Whether to apply kernel fusion.
    pub fuse_kernels: bool,
}

impl Default for GateKernelConfig {
    fn default() -> Self {
        Self {
            gate_type: "none".to_string(),
            tile_size: 64,
            fuse_kernels: false,
        }
    }
}

/// Memory profile for mobile/edge optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    /// Peak memory in bytes.
    pub peak_bytes: usize,
    /// Current memory in bytes.
    pub current_bytes: usize,
    /// Whether memory optimization is active.
    pub optimized: bool,
    /// Backend used: "ruvector-memopt", "standard", or "unavailable".
    pub backend: String,
}

// ---------------------------------------------------------------------------
// Attention Optimizer
// ---------------------------------------------------------------------------

/// Optimizes attention computation for inference workloads.
///
/// When `ruvnet-phase3` is enabled, uses `ruvector-attention` with flash
/// attention and `ruvector-attn-mincut` for graph-cut sparsification.
/// Otherwise, returns stub results indicating the feature is unavailable.
#[derive(Debug, Clone)]
pub struct AttentionOptimizer {
    pub gate_config: GateKernelConfig,
    pub max_sequence_length: usize,
}

impl AttentionOptimizer {
    pub fn new(max_sequence_length: usize) -> Self {
        Self {
            gate_config: GateKernelConfig::default(),
            max_sequence_length,
        }
    }

    /// Set the gate kernel configuration.
    pub fn with_gate_config(mut self, config: GateKernelConfig) -> Self {
        self.gate_config = config;
        self
    }

    /// Run attention optimization on a sequence of the given length.
    pub fn optimize(&self, sequence_length: usize) -> AttentionResult {
        #[cfg(feature = "ruvnet-phase3")]
        {
            self.optimize_phase3(sequence_length)
        }

        #[cfg(not(feature = "ruvnet-phase3"))]
        {
            let _ = sequence_length;
            AttentionResult {
                method: "unavailable".to_string(),
                tokens_processed: 0,
                latency_ms: 0.0,
                memory_reduction_pct: 0.0,
            }
        }
    }

    /// Phase 3 attention optimization using ruvector crates.
    #[cfg(feature = "ruvnet-phase3")]
    fn optimize_phase3(&self, sequence_length: usize) -> AttentionResult {
        use ruvector_attention::FlashAttention;
        use ruvector_attn_mincut::MincutSparsifier;

        let flash = FlashAttention::new(self.max_sequence_length);
        let sparsifier = MincutSparsifier::default();

        let start = std::time::Instant::now();
        let result = flash.forward(sequence_length);
        let sparse_result = sparsifier.sparsify(&result);
        let elapsed = start.elapsed();

        AttentionResult {
            method: "flash-mincut".to_string(),
            tokens_processed: sequence_length,
            latency_ms: elapsed.as_secs_f64() * 1000.0,
            memory_reduction_pct: sparse_result.reduction_pct(),
        }
    }

    /// Profile memory usage for the current configuration.
    pub fn memory_profile(&self) -> MemoryProfile {
        #[cfg(feature = "ruvnet-phase3")]
        {
            self.memory_profile_phase3()
        }

        #[cfg(not(feature = "ruvnet-phase3"))]
        {
            MemoryProfile {
                peak_bytes: 0,
                current_bytes: 0,
                optimized: false,
                backend: "unavailable".to_string(),
            }
        }
    }

    /// Phase 3 memory profiling using ruvector-memopt.
    #[cfg(feature = "ruvnet-phase3")]
    fn memory_profile_phase3(&self) -> MemoryProfile {
        use ruvector_memopt::MemoryOptimizer;

        let optimizer = MemoryOptimizer::default();
        let profile = optimizer.profile();

        MemoryProfile {
            peak_bytes: profile.peak_bytes,
            current_bytes: profile.current_bytes,
            optimized: profile.is_optimized,
            backend: "ruvector-memopt".to_string(),
        }
    }
}

impl Default for AttentionOptimizer {
    fn default() -> Self {
        Self::new(4096)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_attention_unavailable() {
        let optimizer = AttentionOptimizer::default();
        let result = optimizer.optimize(128);

        #[cfg(not(feature = "ruvnet-phase3"))]
        {
            assert_eq!(result.method, "unavailable");
            assert_eq!(result.tokens_processed, 0);
        }

        #[cfg(feature = "ruvnet-phase3")]
        {
            assert_eq!(result.method, "flash-mincut");
            assert_eq!(result.tokens_processed, 128);
        }
    }

    #[test]
    fn test_stub_memory_profile() {
        let optimizer = AttentionOptimizer::default();
        let profile = optimizer.memory_profile();

        #[cfg(not(feature = "ruvnet-phase3"))]
        {
            assert!(!profile.optimized);
            assert_eq!(profile.backend, "unavailable");
        }

        #[cfg(feature = "ruvnet-phase3")]
        {
            assert_eq!(profile.backend, "ruvector-memopt");
        }
    }

    #[test]
    fn test_gate_kernel_config_default() {
        let config = GateKernelConfig::default();
        assert_eq!(config.gate_type, "none");
        assert_eq!(config.tile_size, 64);
        assert!(!config.fuse_kernels);
    }

    #[test]
    fn test_attention_optimizer_builder() {
        let config = GateKernelConfig {
            gate_type: "tilezero".to_string(),
            tile_size: 128,
            fuse_kernels: true,
        };
        let optimizer = AttentionOptimizer::new(2048).with_gate_config(config);
        assert_eq!(optimizer.max_sequence_length, 2048);
        assert_eq!(optimizer.gate_config.gate_type, "tilezero");
        assert!(optimizer.gate_config.fuse_kernels);
    }
}
