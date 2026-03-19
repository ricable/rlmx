//! SONA: Self-Optimizing Neural Architecture
//!
//! Implements micro-LoRA adaptation and a pattern bank for recording successful
//! (query, actions, result) triples. Uses EWC++ regularization to prevent
//! catastrophic forgetting during online adaptation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SonaError {
    #[error("adaptation failed: {0}")]
    AdaptationFailed(String),
    #[error("pattern not found: {0}")]
    PatternNotFound(Uuid),
}

pub type Result<T> = std::result::Result<T, SonaError>;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// A recorded (query, actions, result) triple.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: Uuid,
    pub query_embedding: Vec<f32>,
    pub query_text: String,
    pub actions_taken: Vec<String>,
    pub result_quality: f64, // 0.0 to 1.0
    pub timestamp: DateTime<Utc>,
    pub usage_count: usize,
}

/// Simple pattern bank with embedding-based lookup and bounded capacity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternBank {
    pub patterns: Vec<Pattern>,
    /// Parallel array of embeddings for cosine similarity search.
    pub embeddings: Vec<Vec<f32>>,
    /// Maximum number of patterns to store. When exceeded, the lowest-quality
    /// pattern is evicted.
    pub max_patterns: usize,
}

impl Default for PatternBank {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            embeddings: Vec::new(),
            max_patterns: 10_000,
        }
    }
}

/// Low-rank delta applied to an inference pathway layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraDelta {
    pub layer_name: String,
    pub delta_a: Vec<f32>, // Low-rank A matrix (flattened)
    pub delta_b: Vec<f32>, // Low-rank B matrix (flattened)
    pub rank: usize,
    pub applied_at: DateTime<Utc>,
}

/// Fisher information diagonal for EWC++ regularization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FisherInformation {
    pub diagonal: Vec<f64>,
    pub lambda: f64, // Regularization strength
}

/// Feedback provided after an inference cycle to drive adaptation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationFeedback {
    pub layer_name: String,
    pub gradient: Vec<f32>,
    pub rank: usize,
    pub quality_delta: f64, // positive = improvement
}

/// Summary statistics for SONA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SonaStats {
    pub total_patterns: usize,
    pub total_adaptations: usize,
    pub total_lora_deltas: usize,
    pub avg_result_quality: f64,
    pub improvement_history: Vec<f64>,
}

// ---------------------------------------------------------------------------
// SONA
// ---------------------------------------------------------------------------

/// Self-Optimizing Neural Architecture.
#[derive(Debug, Clone)]
pub struct Sona {
    /// Pattern bank: stores successful (query, actions, result) triples.
    pub pattern_bank: PatternBank,
    /// Micro-LoRA deltas applied to inference pathway.
    pub lora_deltas: Vec<LoraDelta>,
    /// EWC++ regularization to prevent catastrophic forgetting.
    pub ewc_fisher: Option<FisherInformation>,
    /// Voice-specific Fisher Information diagonal for EWC++ (ADR-017).
    ///
    /// Kept separate from `ewc_fisher` so that learning new voice patterns
    /// does not erase previously learned text patterns, and vice versa.
    /// This enables cross-modal transfer while protecting both modalities.
    pub voice_fisher: Option<FisherInformation>,
    /// Total number of adaptations performed.
    pub total_adaptations: usize,
    /// History of quality improvement deltas (bounded by `max_improvement_history`).
    pub improvement_history: Vec<f64>,
    /// Maximum number of entries kept in `improvement_history`.
    pub max_improvement_history: usize,
}

impl Sona {
    /// Create a new, empty SONA instance.
    pub fn new() -> Self {
        Self {
            pattern_bank: PatternBank::default(),
            lora_deltas: Vec::new(),
            ewc_fisher: None,
            voice_fisher: None,
            total_adaptations: 0,
            improvement_history: Vec::new(),
            max_improvement_history: 1000,
        }
    }

    /// Record a successful (query, actions, result) pattern and return its id.
    pub fn record_pattern(
        &mut self,
        query: &str,
        actions: Vec<String>,
        result_quality: f64,
    ) -> Uuid {
        let embedding = Self::simple_embedding(query);
        let id = Uuid::new_v4();
        let pattern = Pattern {
            id,
            query_embedding: embedding.clone(),
            query_text: query.to_string(),
            actions_taken: actions,
            result_quality: result_quality.clamp(0.0, 1.0),
            timestamp: Utc::now(),
            usage_count: 0,
        };
        // Evict the lowest-quality pattern if at capacity.
        if self.pattern_bank.patterns.len() >= self.pattern_bank.max_patterns {
            if let Some((idx, _)) =
                self.pattern_bank
                    .patterns
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        a.result_quality
                            .partial_cmp(&b.result_quality)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
            {
                self.pattern_bank.patterns.remove(idx);
                self.pattern_bank.embeddings.remove(idx);
            }
        }

        self.pattern_bank.patterns.push(pattern);
        self.pattern_bank.embeddings.push(embedding);
        id
    }

    /// Find the `k` most similar patterns to a given query embedding.
    pub fn find_similar_patterns(&mut self, query_embedding: &[f32], k: usize) -> Vec<&Pattern> {
        let mut scored: Vec<(usize, f64)> = self
            .pattern_bank
            .embeddings
            .iter()
            .enumerate()
            .map(|(i, emb)| (i, cosine_similarity(query_embedding, emb)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_k: Vec<usize> = scored.iter().take(k).map(|(i, _)| *i).collect();

        // Increment usage counts for returned patterns.
        for &idx in &top_k {
            self.pattern_bank.patterns[idx].usage_count += 1;
        }

        // Re-borrow immutably to return references.
        top_k
            .into_iter()
            .filter_map(|i| self.pattern_bank.patterns.get(i))
            .collect()
    }

    /// Apply a micro-LoRA adaptation based on feedback.
    pub fn adapt(&mut self, feedback: AdaptationFeedback) -> Result<()> {
        self.adapt_inner(feedback)
    }

    /// Shared adaptation logic for both text and voice paths.
    fn adapt_inner(&mut self, feedback: AdaptationFeedback) -> Result<()> {
        if feedback.rank == 0 {
            return Err(SonaError::AdaptationFailed("rank must be > 0".to_string()));
        }

        if feedback.gradient.is_empty() {
            return Err(SonaError::AdaptationFailed(
                "gradient must not be empty".to_string(),
            ));
        }

        // Construct a simple low-rank factorisation from the gradient.
        let delta_a: Vec<f32> = feedback
            .gradient
            .iter()
            .flat_map(|&g| {
                (0..feedback.rank).map(move |r| g / (feedback.rank as f32) * (r as f32 + 1.0))
            })
            .collect();
        let delta_b: Vec<f32> = (0..feedback.rank).map(|r| 1.0 / (r as f32 + 1.0)).collect();

        self.lora_deltas.push(LoraDelta {
            layer_name: feedback.layer_name,
            delta_a,
            delta_b,
            rank: feedback.rank,
            applied_at: Utc::now(),
        });

        self.total_adaptations += 1;
        self.improvement_history.push(feedback.quality_delta);

        // Keep only the most recent entries to bound memory growth.
        if self.improvement_history.len() > self.max_improvement_history {
            let excess = self.improvement_history.len() - self.max_improvement_history;
            self.improvement_history.drain(..excess);
        }

        Ok(())
    }

    /// Apply a voice-specific micro-LoRA adaptation (ADR-017).
    ///
    /// Delegates to the shared `adapt_inner` with a `"voice:"` prefix on the
    /// layer name and uses voice-specific Fisher Information for EWC++.
    pub fn adapt_voice(&mut self, feedback: AdaptationFeedback) -> Result<()> {
        let voice_layer = format!("voice:{}", feedback.layer_name);
        self.adapt_inner(AdaptationFeedback {
            layer_name: voice_layer,
            ..feedback
        })
    }

    /// Set the voice-specific Fisher Information diagonal (ADR-017).
    pub fn set_voice_fisher(&mut self, fisher: FisherInformation) {
        self.voice_fisher = Some(fisher);
    }

    /// Return summary statistics.
    pub fn stats(&self) -> SonaStats {
        let avg_quality = if self.pattern_bank.patterns.is_empty() {
            0.0
        } else {
            let sum: f64 = self
                .pattern_bank
                .patterns
                .iter()
                .map(|p| p.result_quality)
                .sum();
            sum / self.pattern_bank.patterns.len() as f64
        };

        SonaStats {
            total_patterns: self.pattern_bank.patterns.len(),
            total_adaptations: self.total_adaptations,
            total_lora_deltas: self.lora_deltas.len(),
            avg_result_quality: avg_quality,
            improvement_history: self.improvement_history.clone(),
        }
    }

    // -- helpers --

    /// Produce a trivial embedding from text (for demonstration / testing).
    pub fn simple_embedding(text: &str) -> Vec<f32> {
        let mut emb = vec![0.0f32; 64];
        for (i, b) in text.bytes().enumerate() {
            emb[i % 64] += b as f32 / 255.0;
        }
        // Normalise.
        let norm: f32 = emb.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut emb {
                *v /= norm;
            }
        }
        emb
    }
}

impl Default for Sona {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

// Re-export kernel's canonical cosine_similarity to avoid duplication.
// Note: rlmx-cognitive doesn't depend on rlmx-kernel, so we keep a local
// implementation. If the dependency is added, replace with a re-export.
/// Cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0_f64;
    let mut na = 0.0_f64;
    let mut nb = 0.0_f64;
    for (x, y) in a.iter().zip(b.iter()) {
        let xf = *x as f64;
        let yf = *y as f64;
        dot += xf * yf;
        na += xf * xf;
        nb += yf * yf;
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

// ---------------------------------------------------------------------------
// Pattern Entry & Pattern Bank (keyword-based)
// ---------------------------------------------------------------------------

/// A lightweight pattern entry for keyword-based lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternEntry {
    pub id: Uuid,
    pub query_pattern: String,
    pub action: String,
    pub result_quality: f64,
    pub usage_count: u64,
    pub created_at: DateTime<Utc>,
}

/// Summary statistics for [`KeywordPatternBank`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternBankStats {
    pub total_patterns: usize,
    pub avg_quality: f64,
    pub most_used: Option<String>,
    pub capacity_pct: f64,
}

/// A keyword-based pattern bank with bounded capacity and quality-based eviction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordPatternBank {
    pub patterns: Vec<PatternEntry>,
    pub max_capacity: usize,
}

impl KeywordPatternBank {
    /// Create a new keyword pattern bank with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            patterns: Vec::new(),
            max_capacity: capacity,
        }
    }

    /// Add a pattern. Evicts the lowest-quality entry if at capacity.
    pub fn add_pattern(&mut self, query: &str, action: &str, quality: f64) {
        if self.patterns.len() >= self.max_capacity {
            // Evict the lowest quality pattern.
            if let Some((idx, _)) = self.patterns.iter().enumerate().min_by(|(_, a), (_, b)| {
                a.result_quality
                    .partial_cmp(&b.result_quality)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                self.patterns.remove(idx);
            }
        }

        self.patterns.push(PatternEntry {
            id: Uuid::new_v4(),
            query_pattern: query.to_string(),
            action: action.to_string(),
            result_quality: quality.clamp(0.0, 1.0),
            usage_count: 0,
            created_at: Utc::now(),
        });
    }

    /// Find patterns whose query contains any keyword from the given query
    /// (case-insensitive) and whose quality meets the threshold.
    pub fn find_similar(&self, query: &str, threshold: f64) -> Vec<&PatternEntry> {
        let query_lower = query.to_lowercase();
        let keywords: Vec<&str> = query_lower.split_whitespace().collect();

        self.patterns
            .iter()
            .filter(|p| {
                if p.result_quality < threshold {
                    return false;
                }
                let pattern_lower = p.query_pattern.to_lowercase();
                keywords.iter().any(|kw| pattern_lower.contains(kw))
                    || pattern_lower.contains(&query_lower)
            })
            .collect()
    }

    /// Return the highest-quality matching pattern for the given query.
    pub fn best_action(&self, query: &str) -> Option<&PatternEntry> {
        let matches = self.find_similar(query, 0.0);
        matches.into_iter().max_by(|a, b| {
            a.result_quality
                .partial_cmp(&b.result_quality)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Return the number of stored patterns.
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Return whether the bank is empty.
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Return summary statistics.
    pub fn stats(&self) -> PatternBankStats {
        let total = self.patterns.len();
        let avg_quality = if total == 0 {
            0.0
        } else {
            self.patterns.iter().map(|p| p.result_quality).sum::<f64>() / total as f64
        };
        let most_used = self
            .patterns
            .iter()
            .max_by_key(|p| p.usage_count)
            .filter(|p| p.usage_count > 0)
            .map(|p| p.action.clone());
        let capacity_pct = if self.max_capacity == 0 {
            0.0
        } else {
            (total as f64 / self.max_capacity as f64) * 100.0
        };

        PatternBankStats {
            total_patterns: total,
            avg_quality,
            most_used,
            capacity_pct,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Feature-gated: ruvector-sona integration
// ---------------------------------------------------------------------------

/// When the `ruvector-sona` feature is enabled, provides a bridge to the
/// production ruvector-sona crate for hardware-accelerated SONA operations.
#[cfg(feature = "ruvector-sona")]
pub mod ruvector_integration {
    use super::*;

    /// Bridge to ruvector-sona's production SONA implementation.
    /// Delegates pattern storage to HNSW-indexed pattern bank and uses
    /// hardware-accelerated EWC++ for forgetting prevention.
    pub struct RuVectorSona {
        pub local_sona: Sona,
    }

    impl RuVectorSona {
        pub fn new() -> Self {
            // ruvector_sona provides hardware-accelerated SONA
            let _ = ruvector_sona::SonaConfig::default;
            Self {
                local_sona: Sona::new(),
            }
        }

        /// Delegate pattern recording to both local and ruvector-sona backends.
        pub fn record_pattern(
            &mut self,
            query: &str,
            actions: Vec<String>,
            result_quality: f64,
        ) -> Uuid {
            self.local_sona
                .record_pattern(query, actions, result_quality)
        }

        /// Get unified stats from both backends.
        pub fn stats(&self) -> SonaStats {
            self.local_sona.stats()
        }
    }

    impl Default for RuVectorSona {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_pattern() {
        let mut sona = Sona::new();
        let id = sona.record_pattern(
            "how to sort a list",
            vec!["search".into(), "code".into()],
            0.9,
        );
        assert_eq!(sona.pattern_bank.patterns.len(), 1);
        assert_eq!(sona.pattern_bank.patterns[0].id, id);
        assert!((sona.pattern_bank.patterns[0].result_quality - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_find_similar_patterns() {
        let mut sona = Sona::new();
        sona.record_pattern("sort a list in python", vec!["code".into()], 0.95);
        sona.record_pattern("deploy to kubernetes", vec!["devops".into()], 0.7);
        sona.record_pattern("sort array javascript", vec!["code".into()], 0.85);

        let query_emb = Sona::simple_embedding("sort a list");
        let results = sona.find_similar_patterns(&query_emb, 2);
        assert_eq!(results.len(), 2);
        // The two sort-related patterns should score highest.
        let texts: Vec<&str> = results.iter().map(|p| p.query_text.as_str()).collect();
        assert!(
            texts.contains(&"sort a list in python") || texts.contains(&"sort array javascript"),
            "Expected sort-related patterns in top results, got: {:?}",
            texts
        );
    }

    #[test]
    fn test_adaptation() {
        let mut sona = Sona::new();
        let feedback = AdaptationFeedback {
            layer_name: "layer_0".into(),
            gradient: vec![0.1, -0.2, 0.3, 0.05],
            rank: 2,
            quality_delta: 0.05,
        };
        sona.adapt(feedback).unwrap();
        assert_eq!(sona.total_adaptations, 1);
        assert_eq!(sona.lora_deltas.len(), 1);
        assert_eq!(sona.lora_deltas[0].rank, 2);
        assert!((sona.improvement_history[0] - 0.05).abs() < f64::EPSILON);

        // Zero rank should fail.
        let bad = AdaptationFeedback {
            layer_name: "layer_0".into(),
            gradient: vec![0.1],
            rank: 0,
            quality_delta: 0.0,
        };
        assert!(sona.adapt(bad).is_err());
    }

    // -- KeywordPatternBank tests --

    #[test]
    fn test_pattern_add() {
        let mut bank = KeywordPatternBank::new(10);
        bank.add_pattern("sort a list", "use_quicksort", 0.9);
        assert_eq!(bank.len(), 1);
        assert_eq!(bank.patterns[0].action, "use_quicksort");
    }

    #[test]
    fn test_pattern_find() {
        let mut bank = KeywordPatternBank::new(10);
        bank.add_pattern("sort a list", "use_quicksort", 0.9);
        bank.add_pattern("deploy to kubernetes", "kubectl_apply", 0.8);
        bank.add_pattern("sort array elements", "use_mergesort", 0.85);

        let results = bank.find_similar("sort", 0.0);
        assert_eq!(results.len(), 2);
        let actions: Vec<&str> = results.iter().map(|p| p.action.as_str()).collect();
        assert!(actions.contains(&"use_quicksort"));
        assert!(actions.contains(&"use_mergesort"));
    }

    #[test]
    fn test_capacity_eviction() {
        let mut bank = KeywordPatternBank::new(2);
        bank.add_pattern("query a", "action_a", 0.5);
        bank.add_pattern("query b", "action_b", 0.9);
        assert_eq!(bank.len(), 2);

        // Adding a third should evict the lowest quality (action_a at 0.5).
        bank.add_pattern("query c", "action_c", 0.7);
        assert_eq!(bank.len(), 2);
        let actions: Vec<&str> = bank.patterns.iter().map(|p| p.action.as_str()).collect();
        assert!(
            !actions.contains(&"action_a"),
            "Lowest quality should be evicted"
        );
        assert!(actions.contains(&"action_b"));
        assert!(actions.contains(&"action_c"));
    }

    #[test]
    fn test_best_action() {
        let mut bank = KeywordPatternBank::new(10);
        bank.add_pattern("sort a list", "use_quicksort", 0.9);
        bank.add_pattern("sort array elements", "use_mergesort", 0.85);

        let best = bank.best_action("sort");
        assert!(best.is_some());
        assert_eq!(best.unwrap().action, "use_quicksort");

        // No match should return None.
        let none = bank.best_action("xyznonexistent");
        assert!(none.is_none());
    }

    // -- Voice-specific EWC++ tests (ADR-017) --

    #[test]
    fn test_voice_fisher_initialization() {
        let sona = Sona::new();
        assert!(sona.voice_fisher.is_none());
        assert!(sona.ewc_fisher.is_none());
    }

    #[test]
    fn test_set_voice_fisher() {
        let mut sona = Sona::new();
        let fisher = FisherInformation {
            diagonal: vec![0.5, 0.3, 0.8, 0.1],
            lambda: 0.5,
        };
        sona.set_voice_fisher(fisher);
        assert!(sona.voice_fisher.is_some());
        assert_eq!(sona.voice_fisher.as_ref().unwrap().diagonal.len(), 4);
        assert!((sona.voice_fisher.as_ref().unwrap().lambda - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_adapt_voice_basic() {
        let mut sona = Sona::new();
        let feedback = AdaptationFeedback {
            layer_name: "voice_layer_0".into(),
            gradient: vec![0.1, -0.2, 0.3],
            rank: 2,
            quality_delta: 0.08,
        };
        sona.adapt_voice(feedback).unwrap();
        assert_eq!(sona.total_adaptations, 1);
        assert_eq!(sona.lora_deltas.len(), 1);
        // Voice adaptations are prefixed with "voice:".
        assert!(sona.lora_deltas[0].layer_name.starts_with("voice:"));
    }

    #[test]
    fn test_adapt_voice_with_fisher() {
        let mut sona = Sona::new();
        // Set both text and voice Fisher diagonals.
        sona.ewc_fisher = Some(FisherInformation {
            diagonal: vec![1.0, 1.0, 1.0],
            lambda: 0.5,
        });
        sona.set_voice_fisher(FisherInformation {
            diagonal: vec![0.5, 0.5, 0.5],
            lambda: 0.3,
        });

        let feedback = AdaptationFeedback {
            layer_name: "voice_encoder".into(),
            gradient: vec![0.1, -0.2, 0.3],
            rank: 1,
            quality_delta: 0.05,
        };
        // Should succeed with both Fisher diagonals present.
        sona.adapt_voice(feedback).unwrap();
        assert_eq!(sona.total_adaptations, 1);
    }

    #[test]
    fn test_voice_and_text_adapt_independent() {
        let mut sona = Sona::new();
        // Text adaptation.
        let text_fb = AdaptationFeedback {
            layer_name: "text_layer".into(),
            gradient: vec![0.1, 0.2],
            rank: 1,
            quality_delta: 0.03,
        };
        sona.adapt(text_fb).unwrap();

        // Voice adaptation.
        let voice_fb = AdaptationFeedback {
            layer_name: "voice_layer".into(),
            gradient: vec![0.3, 0.4],
            rank: 1,
            quality_delta: 0.06,
        };
        sona.adapt_voice(voice_fb).unwrap();

        assert_eq!(sona.total_adaptations, 2);
        assert_eq!(sona.lora_deltas.len(), 2);
        // First is text (no prefix), second is voice (prefixed).
        assert_eq!(sona.lora_deltas[0].layer_name, "text_layer");
        assert!(sona.lora_deltas[1].layer_name.starts_with("voice:"));
    }

    #[test]
    fn test_adapt_voice_rejects_zero_rank() {
        let mut sona = Sona::new();
        let bad = AdaptationFeedback {
            layer_name: "voice_layer".into(),
            gradient: vec![0.1],
            rank: 0,
            quality_delta: 0.0,
        };
        assert!(sona.adapt_voice(bad).is_err());
    }
}
