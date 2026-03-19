//! Attention Mechanism Selection
//!
//! Represents the 39 attention mechanisms available to the RuVix kernel and
//! provides auto-selection based on operation type and context size.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Category of attention mechanism.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttentionCategory {
    /// Multi-head, cross-attention.
    Standard,
    /// Flash, sliding window.
    Efficient,
    /// Graph attention, Poincare.
    Topological,
    /// Causal cone, critical path.
    Temporal,
    /// MinCut gated, community.
    Structural,
}

/// A single attention mechanism descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionMechanism {
    pub name: String,
    pub category: AttentionCategory,
    pub description: String,
    pub best_for: Vec<String>,
}

/// Selects the appropriate attention mechanism for a given task.
#[derive(Debug, Clone)]
pub struct AttentionSelector {
    pub mechanisms: Vec<AttentionMechanism>,
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl AttentionSelector {
    /// Initialize with all 39 attention mechanisms.
    pub fn new() -> Self {
        Self {
            mechanisms: build_all_mechanisms(),
        }
    }

    /// Select the best mechanism for the given operation type and context size.
    pub fn select(&self, operation_type: &str, context_size: usize) -> &AttentionMechanism {
        let op = operation_type.to_lowercase();

        // For very large contexts, Flash is always preferred regardless of op.
        if context_size > 50_000 {
            if let Some(m) = self.mechanisms.iter().find(|m| m.name == "Flash") {
                return m;
            }
        }

        // First pass: find mechanisms whose best_for matches the operation.
        let mut candidates: Vec<(usize, &AttentionMechanism)> = self
            .mechanisms
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                m.best_for
                    .iter()
                    .any(|bf| op.contains(&bf.to_lowercase()) || bf.to_lowercase().contains(&op))
            })
            .collect();

        // For large contexts, prefer Efficient mechanisms among candidates.
        if context_size > 10_000 && !candidates.is_empty() {
            let efficient: Vec<_> = candidates
                .iter()
                .filter(|(_, m)| m.category == AttentionCategory::Efficient)
                .cloned()
                .collect();
            if !efficient.is_empty() {
                candidates = efficient;
            }
        }

        // If no direct match, use heuristics based on context size.
        if candidates.is_empty() {
            candidates = self.mechanisms.iter().enumerate().collect();

            if context_size > 10_000 {
                let efficient: Vec<_> = candidates
                    .iter()
                    .filter(|(_, m)| m.category == AttentionCategory::Efficient)
                    .cloned()
                    .collect();
                if !efficient.is_empty() {
                    candidates = efficient;
                }
            }
        }

        // Return the first candidate (stable ordering from initialization).
        candidates
            .first()
            .map(|(_, m)| *m)
            .unwrap_or(&self.mechanisms[0])
    }

    /// List all available mechanisms.
    pub fn list(&self) -> &[AttentionMechanism] {
        &self.mechanisms
    }
}

impl Default for AttentionSelector {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Mechanism catalogue
// ---------------------------------------------------------------------------

fn mech(name: &str, cat: AttentionCategory, desc: &str, best_for: &[&str]) -> AttentionMechanism {
    AttentionMechanism {
        name: name.to_string(),
        category: cat,
        description: desc.to_string(),
        best_for: best_for.iter().map(|s| s.to_string()).collect(),
    }
}

fn build_all_mechanisms() -> Vec<AttentionMechanism> {
    vec![
        // --- Standard (5) ---
        mech(
            "MultiHead",
            AttentionCategory::Standard,
            "Standard multi-head self-attention",
            &["general", "classification"],
        ),
        mech(
            "Cross",
            AttentionCategory::Standard,
            "Cross-attention between encoder and decoder",
            &["translation", "cross-modal"],
        ),
        mech(
            "GroupedQuery",
            AttentionCategory::Standard,
            "Grouped-query attention for efficiency",
            &["general", "inference"],
        ),
        mech(
            "MultiQuery",
            AttentionCategory::Standard,
            "Multi-query attention with shared KV",
            &["inference", "serving"],
        ),
        mech(
            "RotaryPositional",
            AttentionCategory::Standard,
            "RoPE-enhanced positional attention",
            &["long-context", "positional"],
        ),
        // --- Efficient (8) ---
        mech(
            "Flash",
            AttentionCategory::Efficient,
            "FlashAttention: IO-aware exact attention",
            &["large-context", "training", "inference"],
        ),
        mech(
            "SlidingWindow",
            AttentionCategory::Efficient,
            "Local sliding window attention",
            &["streaming", "local-context"],
        ),
        mech(
            "LinearAttention",
            AttentionCategory::Efficient,
            "Linear complexity attention via kernel trick",
            &["long-sequence", "efficient"],
        ),
        mech(
            "SparseAttention",
            AttentionCategory::Efficient,
            "Block-sparse attention patterns",
            &["long-document", "sparse"],
        ),
        mech(
            "PagedAttention",
            AttentionCategory::Efficient,
            "Paged KV-cache for serving",
            &["serving", "batched-inference"],
        ),
        mech(
            "RingAttention",
            AttentionCategory::Efficient,
            "Distributed ring attention across devices",
            &["distributed", "very-long-context"],
        ),
        mech(
            "ChunkedPrefill",
            AttentionCategory::Efficient,
            "Chunked prefill for memory efficiency",
            &["prefill", "memory-constrained"],
        ),
        mech(
            "SpeculativePrefill",
            AttentionCategory::Efficient,
            "Speculative decoding attention",
            &["speculative", "fast-decode"],
        ),
        // --- Topological (8) ---
        mech(
            "Topological",
            AttentionCategory::Topological,
            "Topology-aware attention over graph structures",
            &["graph", "topology"],
        ),
        mech(
            "GraphAttention",
            AttentionCategory::Topological,
            "GAT-style attention on graph nodes",
            &["graph", "node-classification"],
        ),
        mech(
            "Poincare",
            AttentionCategory::Topological,
            "Hyperbolic Poincare-ball attention",
            &["hierarchical", "taxonomy"],
        ),
        mech(
            "HyperbolicNeighborhood",
            AttentionCategory::Topological,
            "Neighborhood attention in hyperbolic space",
            &["hierarchical", "embedding"],
        ),
        mech(
            "SimplicialAttention",
            AttentionCategory::Topological,
            "Attention over simplicial complexes",
            &["higher-order", "topology"],
        ),
        mech(
            "PersistenceWeighted",
            AttentionCategory::Topological,
            "Persistence-diagram weighted attention",
            &["topological-features", "stability"],
        ),
        mech(
            "SpectralAttention",
            AttentionCategory::Topological,
            "Spectral-domain graph attention",
            &["frequency", "graph-signal"],
        ),
        mech(
            "CellularSheaf",
            AttentionCategory::Topological,
            "Sheaf-theoretic cellular attention",
            &["heterogeneous-graph", "sheaf"],
        ),
        // --- Temporal (9) ---
        mech(
            "CausalCone",
            AttentionCategory::Temporal,
            "Causal-cone attention respecting time ordering",
            &["causal", "temporal"],
        ),
        mech(
            "CriticalPath",
            AttentionCategory::Temporal,
            "Attention along critical execution paths",
            &["scheduling", "critical-path"],
        ),
        mech(
            "TemporalConvolution",
            AttentionCategory::Temporal,
            "Temporal convolution attention",
            &["time-series", "sequence"],
        ),
        mech(
            "RecurrentAttention",
            AttentionCategory::Temporal,
            "Recurrent gated attention",
            &["recurrent", "stateful"],
        ),
        mech(
            "DeltaTime",
            AttentionCategory::Temporal,
            "Delta-time weighted attention",
            &["irregular-time", "event-stream"],
        ),
        mech(
            "HawkesProcess",
            AttentionCategory::Temporal,
            "Hawkes-process intensity attention",
            &["event", "self-exciting"],
        ),
        mech(
            "CausalMask",
            AttentionCategory::Temporal,
            "Standard causal masking attention",
            &["autoregressive", "generation"],
        ),
        mech(
            "RetentiveAttention",
            AttentionCategory::Temporal,
            "Retention-based recurrent attention",
            &["long-context", "recurrent"],
        ),
        mech(
            "TemporalDifference",
            AttentionCategory::Temporal,
            "TD-learning inspired attention",
            &["reinforcement", "value-estimation"],
        ),
        // --- Structural (9) ---
        mech(
            "MinCutGated",
            AttentionCategory::Structural,
            "MinCut-gated structural attention",
            &["partitioning", "structural"],
        ),
        mech(
            "CommunityAttention",
            AttentionCategory::Structural,
            "Community-detection guided attention",
            &["community", "clustering"],
        ),
        mech(
            "TreeAttention",
            AttentionCategory::Structural,
            "Tree-structured hierarchical attention",
            &["tree", "ast", "hierarchy"],
        ),
        mech(
            "AxialAttention",
            AttentionCategory::Structural,
            "Axial decomposition attention",
            &["2d-structure", "image"],
        ),
        mech(
            "BlockDiagonal",
            AttentionCategory::Structural,
            "Block-diagonal sparse attention",
            &["block-structure", "modular"],
        ),
        mech(
            "CapsuleAttention",
            AttentionCategory::Structural,
            "Capsule-network routing attention",
            &["part-whole", "compositional"],
        ),
        mech(
            "CrossScale",
            AttentionCategory::Structural,
            "Multi-scale cross-resolution attention",
            &["multi-scale", "resolution"],
        ),
        mech(
            "MemoryAugmented",
            AttentionCategory::Structural,
            "External memory augmented attention",
            &["memory", "retrieval"],
        ),
        mech(
            "MixtureOfExperts",
            AttentionCategory::Structural,
            "MoE-routed sparse attention",
            &["routing", "expert-selection"],
        ),
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_has_39_mechanisms() {
        let sel = AttentionSelector::new();
        assert_eq!(sel.list().len(), 39, "Should have exactly 39 mechanisms");
    }

    #[test]
    fn test_select_appropriate_mechanism() {
        let sel = AttentionSelector::new();

        // Large context should prefer Flash.
        let m = sel.select("general", 100_000);
        assert_eq!(m.name, "Flash", "Very large context should select Flash");

        // Graph operation should pick a topological mechanism.
        let m = sel.select("graph", 512);
        assert_eq!(
            m.category,
            AttentionCategory::Topological,
            "Graph operation should select topological attention, got {}",
            m.name
        );

        // Causal / temporal.
        let m = sel.select("causal", 1024);
        assert_eq!(
            m.category,
            AttentionCategory::Temporal,
            "Causal operation should select temporal attention, got {}",
            m.name
        );
    }
}
