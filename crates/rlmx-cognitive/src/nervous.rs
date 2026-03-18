//! Nervous System Integration
//!
//! Bio-inspired cognitive architecture components:
//! - BTSP: Behavioral Time-Scale Plasticity (one-shot learning)
//! - HDC: Hyperdimensional Computing with 10K-bit hypervectors
//! - WTA: Winner-Take-All competition network
//! - Circadian Controller: compute/learn/consolidate scheduling
//! - Global Workspace: 4-7 item attention focus

use chrono::{DateTime, Timelike, Utc};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ===========================================================================
// BTSP: Behavioral Time-Scale Plasticity
// ===========================================================================

/// One-shot learning memory system inspired by hippocampal BTSP.
#[derive(Debug, Clone)]
pub struct BtspLearner {
    pub memory: Vec<BtspMemory>,
    pub capacity: usize,
}

/// A single BTSP memory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtspMemory {
    pub pattern: Vec<f64>,
    pub label: String,
    pub timestamp: DateTime<Utc>,
    pub access_count: usize,
}

impl BtspLearner {
    /// Create a new BTSP learner with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            memory: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Store a pattern immediately (one-shot). If at capacity, evict the
    /// least-recently-accessed entry.
    pub fn learn_one_shot(&mut self, pattern: Vec<f64>, label: String) {
        if self.memory.len() >= self.capacity {
            // Evict the entry with the lowest access count (ties broken by age).
            if let Some(idx) = self
                .memory
                .iter()
                .enumerate()
                .min_by_key(|(_, m)| (m.access_count, m.timestamp))
                .map(|(i, _)| i)
            {
                self.memory.remove(idx);
            }
        }
        self.memory.push(BtspMemory {
            pattern,
            label,
            timestamp: Utc::now(),
            access_count: 0,
        });
    }

    /// Find the best-matching memory for a query pattern (by dot product).
    pub fn recall(&mut self, query_pattern: &[f64]) -> Option<&BtspMemory> {
        if self.memory.is_empty() {
            return None;
        }

        let mut best_idx = 0;
        let mut best_score = f64::NEG_INFINITY;

        for (i, mem) in self.memory.iter().enumerate() {
            let score = dot_product(query_pattern, &mem.pattern);
            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }

        self.memory[best_idx].access_count += 1;
        Some(&self.memory[best_idx])
    }
}

fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

// ===========================================================================
// HDC: Hyperdimensional Computing
// ===========================================================================

/// Hyperdimensional computing engine using binary hypervectors.
#[derive(Debug, Clone)]
pub struct HdcComputer {
    pub dimension: usize, // Default: 10_000
}

/// A binary hypervector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypervector {
    pub bits: Vec<bool>,
}

impl HdcComputer {
    /// Create a new HDC computer with the specified dimension (default 10_000).
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// Deterministic projection encoding of continuous data into a binary hypervector.
    /// The same input always produces the same hypervector.
    pub fn encode(&self, data: &[f64]) -> Hypervector {
        // Derive a deterministic seed from the input data by hashing its bytes.
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        for &v in data {
            v.to_bits().hash(&mut hasher);
        }
        let seed = hasher.finish();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let mut bits = Vec::with_capacity(self.dimension);

        for d in 0..self.dimension {
            // Each bit = sign of a random projection of the data.
            let projection: f64 = data
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    // Deterministic-ish seed per (dimension_index, feature_index).
                    let seed = ((d * 31 + i * 17) % 1000) as f64 / 500.0 - 1.0;
                    v * seed
                })
                .sum();
            let noise: f64 = rng.gen_range(-0.01..0.01);
            bits.push((projection + noise) >= 0.0);
        }

        Hypervector { bits }
    }

    /// XOR binding of two hypervectors.
    pub fn bind(&self, a: &Hypervector, b: &Hypervector) -> Hypervector {
        let bits = a
            .bits
            .iter()
            .zip(b.bits.iter())
            .map(|(&x, &y)| x ^ y)
            .collect();
        Hypervector { bits }
    }

    /// Hamming similarity: fraction of bits that are equal (0.0 to 1.0).
    pub fn similarity(&self, a: &Hypervector, b: &Hypervector) -> f64 {
        let len = a.bits.len().min(b.bits.len());
        if len == 0 {
            return 0.0;
        }
        let matching = a.bits[..len]
            .iter()
            .zip(b.bits[..len].iter())
            .filter(|(&x, &y)| x == y)
            .count();
        matching as f64 / len as f64
    }
}

impl Default for HdcComputer {
    fn default() -> Self {
        Self::new(10_000)
    }
}

// ===========================================================================
// WTA: Winner-Take-All
// ===========================================================================

/// Winner-Take-All competition network.
#[derive(Debug, Clone)]
pub struct WtaNetwork {
    pub neuron_count: usize,
    pub inhibition_radius: usize,
}

impl WtaNetwork {
    /// Create a new WTA network.
    pub fn new(neuron_count: usize, inhibition_radius: usize) -> Self {
        Self {
            neuron_count,
            inhibition_radius,
        }
    }

    /// Run competition: return the index of the winning neuron (highest
    /// activation after lateral inhibition).
    pub fn compete(&self, activations: &[f64]) -> usize {
        if activations.is_empty() {
            return 0;
        }

        let n = activations.len().min(self.neuron_count);
        let mut effective = activations[..n].to_vec();

        // Apply lateral inhibition: each neuron's activation is reduced by the
        // sum of its neighbours within the inhibition radius.
        let original = effective.clone();
        for i in 0..n {
            let start = i.saturating_sub(self.inhibition_radius);
            let end = (i + self.inhibition_radius + 1).min(n);
            let inhibition: f64 = (start..end)
                .filter(|&j| j != i)
                .map(|j| original[j].max(0.0) * 0.1)
                .sum();
            effective[i] -= inhibition;
        }

        // Return index of the maximum activation.
        effective
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
}

// ===========================================================================
// Circadian Controller
// ===========================================================================

/// Circadian phase of the cognitive system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircadianPhase {
    /// High activity: full inference, max responsiveness.
    Compute,
    /// Medium: SONA patterns consolidated, BTSP learning.
    Learn,
    /// Low: HNSW compaction, cold-tier compression, archival.
    Consolidate,
}

/// A scheduled phase period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseSchedule {
    pub phase: CircadianPhase,
    pub start_hour: u32, // 0-23
    pub duration_hours: u32,
}

/// Compute/learn/consolidate scheduler inspired by circadian rhythms.
#[derive(Debug, Clone)]
pub struct CircadianController {
    pub current_phase: CircadianPhase,
    pub phase_schedule: Vec<PhaseSchedule>,
    pub last_transition: DateTime<Utc>,
}

impl CircadianController {
    /// Create with default schedule:
    /// - Compute: 08:00-18:00
    /// - Learn: 18:00-22:00
    /// - Consolidate: 22:00-08:00
    pub fn new() -> Self {
        Self {
            current_phase: CircadianPhase::Compute,
            phase_schedule: vec![
                PhaseSchedule {
                    phase: CircadianPhase::Compute,
                    start_hour: 8,
                    duration_hours: 10,
                },
                PhaseSchedule {
                    phase: CircadianPhase::Learn,
                    start_hour: 18,
                    duration_hours: 4,
                },
                PhaseSchedule {
                    phase: CircadianPhase::Consolidate,
                    start_hour: 22,
                    duration_hours: 10,
                },
            ],
            last_transition: Utc::now(),
        }
    }

    /// Determine the current phase based on the current UTC hour.
    pub fn current_phase(&self) -> CircadianPhase {
        let hour = Utc::now().hour();
        self.phase_for_hour(hour)
    }

    /// Determine phase for a given hour (useful for testing).
    pub fn phase_for_hour(&self, hour: u32) -> CircadianPhase {
        for sched in &self.phase_schedule {
            let end = sched.start_hour + sched.duration_hours;
            if end <= 24 {
                if hour >= sched.start_hour && hour < end {
                    return sched.phase;
                }
            } else {
                // Wraps past midnight.
                let end_wrapped = end % 24;
                if hour >= sched.start_hour || hour < end_wrapped {
                    return sched.phase;
                }
            }
        }
        // Fallback.
        CircadianPhase::Compute
    }

    /// Whether the system should currently be in consolidation phase.
    pub fn should_consolidate(&self) -> bool {
        self.current_phase() == CircadianPhase::Consolidate
    }
}

impl Default for CircadianController {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Global Workspace
// ===========================================================================

/// Global workspace with limited attention slots (4-7 items).
#[derive(Debug, Clone)]
pub struct GlobalWorkspace {
    pub items: Vec<WorkspaceItem>,
    pub max_items: usize, // Default: 7
}

/// An item in the global workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceItem {
    pub id: Uuid,
    pub content: String,
    pub salience: f64, // 0.0-1.0, higher = more important
    pub added: DateTime<Utc>,
}

impl GlobalWorkspace {
    /// Create a new workspace with the given capacity (default 7).
    pub fn new(max_items: usize) -> Self {
        Self {
            items: Vec::new(),
            max_items,
        }
    }

    /// Broadcast an item into the workspace. If full, evict the item with the
    /// lowest salience.
    pub fn broadcast(&mut self, content: String, salience: f64) {
        if self.items.len() >= self.max_items {
            // Evict lowest-salience item.
            if let Some(idx) = self
                .items
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.salience
                        .partial_cmp(&b.salience)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
            {
                self.items.remove(idx);
            }
        }
        self.items.push(WorkspaceItem {
            id: Uuid::new_v4(),
            content,
            salience: salience.clamp(0.0, 1.0),
            added: Utc::now(),
        });
    }

    /// Return items sorted by salience (highest first).
    pub fn focus(&self) -> Vec<&WorkspaceItem> {
        let mut sorted: Vec<&WorkspaceItem> = self.items.iter().collect();
        sorted.sort_by(|a, b| {
            b.salience
                .partial_cmp(&a.salience)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    /// Clear all items.
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl Default for GlobalWorkspace {
    fn default() -> Self {
        Self::new(7)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- BTSP tests --

    #[test]
    fn test_btsp_learn_one_shot() {
        let mut btsp = BtspLearner::new(10);
        btsp.learn_one_shot(vec![1.0, 0.0, 0.0], "cat".into());
        btsp.learn_one_shot(vec![0.0, 1.0, 0.0], "dog".into());
        assert_eq!(btsp.memory.len(), 2);
        assert_eq!(btsp.memory[0].label, "cat");
    }

    #[test]
    fn test_btsp_recall() {
        let mut btsp = BtspLearner::new(10);
        btsp.learn_one_shot(vec![1.0, 0.0, 0.0], "cat".into());
        btsp.learn_one_shot(vec![0.0, 1.0, 0.0], "dog".into());
        btsp.learn_one_shot(vec![0.0, 0.0, 1.0], "bird".into());

        let result = btsp.recall(&[0.9, 0.1, 0.0]);
        assert!(result.is_some());
        assert_eq!(result.unwrap().label, "cat");
    }

    // -- HDC tests --

    #[test]
    fn test_hdc_encode_and_similarity() {
        let hdc = HdcComputer::new(1000);
        let a = hdc.encode(&[1.0, 0.0, 0.0]);
        let b = hdc.encode(&[1.0, 0.0, 0.0]);

        // Same input must produce identical hypervectors (deterministic encoding).
        let sim = hdc.similarity(&a, &b);
        assert!(
            (sim - 1.0).abs() < f64::EPSILON,
            "Expected identical hypervectors for same input, got similarity {}",
            sim
        );
        assert_eq!(a.bits, b.bits);
    }

    #[test]
    fn test_hdc_bind() {
        let hdc = HdcComputer::new(1000);
        let a = hdc.encode(&[1.0, 0.0]);
        let b = hdc.encode(&[0.0, 1.0]);
        let bound = hdc.bind(&a, &b);

        // Binding should produce a vector dissimilar to both inputs.
        let sim_a = hdc.similarity(&bound, &a);
        let sim_b = hdc.similarity(&bound, &b);
        // XOR of two random-ish vectors should be near 0.5 similarity.
        assert!(
            sim_a < 0.75,
            "Expected low similarity after bind, got {}",
            sim_a
        );
        assert!(
            sim_b < 0.75,
            "Expected low similarity after bind, got {}",
            sim_b
        );
    }

    // -- WTA test --

    #[test]
    fn test_wta_competition() {
        let wta = WtaNetwork::new(5, 1);
        let activations = vec![0.1, 0.3, 0.9, 0.2, 0.4];
        let winner = wta.compete(&activations);
        assert_eq!(winner, 2, "Neuron 2 (0.9) should win");
    }

    // -- Circadian tests --

    #[test]
    fn test_circadian_compute_phase() {
        let controller = CircadianController::new();
        // Hour 10 should be Compute phase.
        assert_eq!(controller.phase_for_hour(10), CircadianPhase::Compute);
        // Hour 12 should be Compute.
        assert_eq!(controller.phase_for_hour(12), CircadianPhase::Compute);
    }

    #[test]
    fn test_circadian_learn_and_consolidate() {
        let controller = CircadianController::new();
        // Hour 19 should be Learn.
        assert_eq!(controller.phase_for_hour(19), CircadianPhase::Learn);
        // Hour 23 should be Consolidate.
        assert_eq!(controller.phase_for_hour(23), CircadianPhase::Consolidate);
        // Hour 3 should be Consolidate (wraps past midnight).
        assert_eq!(controller.phase_for_hour(3), CircadianPhase::Consolidate);
    }

    // -- Global Workspace tests --

    #[test]
    fn test_global_workspace_broadcast() {
        let mut gw = GlobalWorkspace::new(3);
        gw.broadcast("item_a".into(), 0.5);
        gw.broadcast("item_b".into(), 0.9);
        gw.broadcast("item_c".into(), 0.7);
        assert_eq!(gw.items.len(), 3);

        let focused = gw.focus();
        assert_eq!(focused[0].content, "item_b"); // highest salience
    }

    #[test]
    fn test_global_workspace_eviction() {
        let mut gw = GlobalWorkspace::new(2);
        gw.broadcast("low".into(), 0.1);
        gw.broadcast("high".into(), 0.9);
        assert_eq!(gw.items.len(), 2);

        // Adding a third should evict the lowest salience.
        gw.broadcast("medium".into(), 0.5);
        assert_eq!(gw.items.len(), 2);

        let contents: Vec<&str> = gw.items.iter().map(|i| i.content.as_str()).collect();
        assert!(
            !contents.contains(&"low"),
            "Low-salience item should have been evicted"
        );
        assert!(contents.contains(&"high"));
        assert!(contents.contains(&"medium"));
    }
}
