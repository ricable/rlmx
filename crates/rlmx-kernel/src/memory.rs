use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{
    KernelError, KernelResult, SearchFilters, SearchHit, SegmentMetadata, SegmentTier,
};

// When the `ruvector` feature is enabled, use ruvector-core's HNSW index
// for O(log n) approximate nearest neighbor search instead of brute-force.
#[cfg(feature = "ruvector")]
use ruvector_core as rvc;

/// A single context segment stored in memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSegment {
    pub id: Uuid,
    pub embedding: Vec<f32>,
    pub content: String,
    pub source: String,
    pub plugin: Option<String>,
    pub segment_type: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub tier: SegmentTier,
}

/// Dimension used for the hash-based pseudo-embedding.
pub const EMBED_DIM: usize = 64;

/// Generate a deterministic pseudo-embedding from text.
///
/// This is NOT a real embedding model — it produces a reproducible float
/// vector by hashing character unigrams and trigrams into buckets, then
/// L2-normalising. It is good enough for demo / integration-test purposes
/// where identical or very similar strings should have high cosine similarity.
pub fn text_to_embedding(text: &str) -> Vec<f32> {
    let mut vec = vec![0.0_f32; EMBED_DIM];
    let lower = text.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();

    // Hash unigrams and trigrams into the vector.
    for ch in &chars {
        let idx = (*ch as usize) % EMBED_DIM;
        vec[idx] += 1.0;
    }
    for window in chars.windows(3) {
        let hash = window.iter().fold(0_usize, |acc, c| {
            acc.wrapping_mul(31).wrapping_add(*c as usize)
        });
        let idx = hash % EMBED_DIM;
        vec[idx] += 0.5;
    }

    // L2-normalise.
    let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut vec {
            *v /= norm;
        }
    }
    vec
}

/// Cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0_f64;
    let mut norm_a = 0.0_f64;
    let mut norm_b = 0.0_f64;
    for (x, y) in a.iter().zip(b.iter()) {
        let xf = *x as f64;
        let yf = *y as f64;
        dot += xf * yf;
        norm_a += xf * xf;
        norm_b += yf * yf;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

/// In-memory region that stores context segments and supports vector search.
///
/// Uses a simple brute-force approach (sorted vecs) as a placeholder for a
/// real HNSW index that would be provided by ruvector-core.
pub struct MemoryRegion {
    pub name: String,
    segments: Vec<ContextSegment>,
}

impl MemoryRegion {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            segments: Vec::new(),
        }
    }

    /// Insert a new context segment and return its id.
    pub fn insert(
        &mut self,
        embedding: Vec<f32>,
        content: String,
        metadata: SegmentMetadata,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let segment = ContextSegment {
            id,
            embedding,
            content,
            source: metadata.source.clone(),
            plugin: metadata.plugin.clone(),
            segment_type: metadata.segment_type.clone(),
            timestamp: Utc::now(),
            metadata: serde_json::to_value(&metadata).unwrap_or_default(),
            tier: SegmentTier::Hot,
        };
        self.segments.push(segment);
        id
    }

    /// Delete a segment by id. Returns true if found and removed.
    pub fn delete(&mut self, segment_id: &Uuid) -> KernelResult<bool> {
        let before = self.segments.len();
        self.segments.retain(|s| s.id != *segment_id);
        if self.segments.len() < before {
            Ok(true)
        } else {
            Err(KernelError::SegmentNotFound(*segment_id))
        }
    }

    /// Search for the top-k most similar segments to the query embedding,
    /// applying optional filters.
    pub fn search(&self, query: &[f32], k: usize, filters: &SearchFilters) -> Vec<SearchHit> {
        let mut scored: Vec<(f64, &ContextSegment)> = self
            .segments
            .iter()
            .filter(|s| Self::matches_filters(s, filters))
            .map(|s| (cosine_similarity(query, &s.embedding), s))
            .collect();

        // Sort descending by score.
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(k)
            .map(|(score, seg)| {
                let meta: SegmentMetadata =
                    serde_json::from_value(seg.metadata.clone()).unwrap_or_default();
                SearchHit {
                    segment_id: seg.id,
                    content: seg.content.clone(),
                    score,
                    metadata: meta,
                }
            })
            .collect()
    }

    /// Number of stored segments.
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    fn matches_filters(seg: &ContextSegment, f: &SearchFilters) -> bool {
        if let Some(ref src) = f.source {
            if seg.source != *src {
                return false;
            }
        }
        if let Some(ref plug) = f.plugin {
            match &seg.plugin {
                Some(p) if p == plug => {}
                _ => return false,
            }
        }
        if let Some(ref st) = f.segment_type {
            if seg.segment_type != *st {
                return false;
            }
        }
        if let Some(ref tier) = f.tier {
            if seg.tier != *tier {
                return false;
            }
        }
        if let Some(ref after) = f.after {
            if seg.timestamp < *after {
                return false;
            }
        }
        if let Some(ref before) = f.before {
            if seg.timestamp > *before {
                return false;
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// HNSW-backed MemoryRegion (feature = "ruvector")
// ---------------------------------------------------------------------------

/// When the `ruvector` feature is enabled, `HnswMemoryRegion` provides an
/// HNSW-indexed backend for O(log n) approximate nearest-neighbor search.
/// It wraps `ruvector_core::VectorDb` and delegates search to HNSW while
/// keeping the same public API as the brute-force `MemoryRegion`.
#[cfg(feature = "ruvector")]
pub struct HnswMemoryRegion {
    pub name: String,
    db: rvc::VectorDb,
    segments: std::collections::HashMap<Uuid, ContextSegment>,
}

#[cfg(feature = "ruvector")]
impl HnswMemoryRegion {
    /// Create an HNSW-backed region with default parameters (ef=128, M=16).
    pub fn new(name: impl Into<String>, dim: usize) -> Self {
        let config = rvc::VectorDbConfig {
            dimensions: dim,
            ef_construction: 128,
            m: 16,
            ..Default::default()
        };
        Self {
            name: name.into(),
            db: rvc::VectorDb::new(config),
            segments: std::collections::HashMap::new(),
        }
    }

    /// Insert a segment; returns its id. The embedding is indexed in HNSW.
    pub fn insert(
        &mut self,
        embedding: Vec<f32>,
        content: String,
        metadata: SegmentMetadata,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let segment = ContextSegment {
            id,
            embedding: embedding.clone(),
            content,
            source: metadata.source.clone(),
            plugin: metadata.plugin.clone(),
            segment_type: metadata.segment_type.clone(),
            timestamp: Utc::now(),
            metadata: serde_json::to_value(&metadata).unwrap_or_default(),
            tier: SegmentTier::Hot,
        };
        // Index the vector in HNSW under the UUID as string key.
        let _ = self.db.insert(&id.to_string(), &embedding);
        self.segments.insert(id, segment);
        id
    }

    /// Delete a segment by id.
    pub fn delete(&mut self, segment_id: &Uuid) -> KernelResult<bool> {
        if self.segments.remove(segment_id).is_some() {
            let _ = self.db.delete(&segment_id.to_string());
            Ok(true)
        } else {
            Err(KernelError::SegmentNotFound(*segment_id))
        }
    }

    /// HNSW-accelerated search with post-hoc filter application.
    pub fn search(&self, query: &[f32], k: usize, filters: &SearchFilters) -> Vec<SearchHit> {
        // Over-fetch from HNSW to account for filter rejects, then trim.
        let fetch_k = k * 4;
        let results = self.db.search(query, fetch_k);
        let mut hits = Vec::with_capacity(k);
        for result in results {
            if hits.len() >= k {
                break;
            }
            let Ok(id) = result.id.parse::<Uuid>() else {
                continue;
            };
            let Some(seg) = self.segments.get(&id) else {
                continue;
            };
            if !MemoryRegion::matches_filters_static(seg, filters) {
                continue;
            }
            let meta: SegmentMetadata =
                serde_json::from_value(seg.metadata.clone()).unwrap_or_default();
            hits.push(SearchHit {
                segment_id: seg.id,
                content: seg.content.clone(),
                score: result.score as f64,
                metadata: meta,
            });
        }
        hits
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}

impl MemoryRegion {
    /// Static filter check (usable from both MemoryRegion and HnswMemoryRegion).
    #[cfg(feature = "ruvector")]
    pub fn matches_filters_static(seg: &ContextSegment, f: &SearchFilters) -> bool {
        Self::matches_filters(seg, f)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_meta() -> SegmentMetadata {
        SegmentMetadata {
            source: "test".into(),
            plugin: None,
            segment_type: "text".into(),
            extra: Default::default(),
        }
    }

    #[test]
    fn test_insert_and_len() {
        let mut region = MemoryRegion::new("test");
        let id = region.insert(vec![1.0, 0.0, 0.0], "hello".into(), make_meta());
        assert_eq!(region.len(), 1);
        assert!(!id.is_nil());
    }

    #[test]
    fn test_search_returns_best_match() {
        let mut region = MemoryRegion::new("test");
        region.insert(vec![1.0, 0.0, 0.0], "east".into(), make_meta());
        region.insert(vec![0.0, 1.0, 0.0], "north".into(), make_meta());
        region.insert(vec![0.7, 0.7, 0.0], "northeast".into(), make_meta());

        let results = region.search(&[1.0, 0.0, 0.0], 2, &SearchFilters::default());
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].content, "east");
        assert!((results[0].score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_delete_segment() {
        let mut region = MemoryRegion::new("test");
        let id = region.insert(vec![1.0, 0.0], "a".into(), make_meta());
        region.insert(vec![0.0, 1.0], "b".into(), make_meta());
        assert_eq!(region.len(), 2);

        let deleted = region.delete(&id).unwrap();
        assert!(deleted);
        assert_eq!(region.len(), 1);

        // Deleting again should error.
        assert!(region.delete(&id).is_err());
    }

    #[test]
    fn test_cosine_similarity_correctness() {
        // Identical vectors => 1.0
        assert!((cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 1e-9);

        // Orthogonal vectors => 0.0
        assert!((cosine_similarity(&[1.0, 0.0], &[0.0, 1.0])).abs() < 1e-9);

        // Opposite vectors => -1.0
        assert!((cosine_similarity(&[1.0, 0.0], &[-1.0, 0.0]) - (-1.0)).abs() < 1e-9);

        // 45 degrees => cos(45) ~= 0.7071
        let val = cosine_similarity(&[1.0, 0.0], &[1.0, 1.0]);
        assert!((val - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-6);
    }
}
