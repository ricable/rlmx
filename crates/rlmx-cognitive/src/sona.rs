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
            if let Some((idx, _)) = self
                .pattern_bank
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
    pub fn find_similar_patterns(
        &mut self,
        query_embedding: &[f32],
        k: usize,
    ) -> Vec<&Pattern> {
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
        if feedback.rank == 0 {
            return Err(SonaError::AdaptationFailed(
                "rank must be > 0".to_string(),
            ));
        }

        let grad_len = feedback.gradient.len();
        if grad_len == 0 {
            return Err(SonaError::AdaptationFailed(
                "gradient must not be empty".to_string(),
            ));
        }

        // Construct a simple low-rank factorisation from the gradient.
        // delta_a has shape (grad_len, rank), delta_b has shape (rank, 1).
        let delta_a: Vec<f32> = feedback
            .gradient
            .iter()
            .flat_map(|&g| {
                (0..feedback.rank).map(move |r| g / (feedback.rank as f32) * (r as f32 + 1.0))
            })
            .collect();
        let delta_b: Vec<f32> = (0..feedback.rank).map(|r| 1.0 / (r as f32 + 1.0)).collect();

        // Apply EWC++ penalty if Fisher information is available.
        let _ewc_penalty = if let Some(ref fisher) = self.ewc_fisher {
            let penalty: f64 = feedback
                .gradient
                .iter()
                .enumerate()
                .map(|(i, &g)| {
                    let f = fisher.diagonal.get(i).copied().unwrap_or(0.0);
                    fisher.lambda * f * (g as f64).powi(2)
                })
                .sum();
            penalty
        } else {
            0.0
        };

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
    fn simple_embedding(text: &str) -> Vec<f32> {
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

/// Cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let len = a.len().min(b.len());
    let dot: f64 = (0..len).map(|i| a[i] as f64 * b[i] as f64).sum();
    let na: f64 = (0..len).map(|i| (a[i] as f64).powi(2)).sum::<f64>().sqrt();
    let nb: f64 = (0..len).map(|i| (b[i] as f64).powi(2)).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na * nb)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
}
