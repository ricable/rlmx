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

/// Embedding dimension — 256 with real model, 64 with hash fallback.
#[cfg(feature = "real-embeddings")]
pub const EMBED_DIM: usize = 256;
#[cfg(not(feature = "real-embeddings"))]
pub const EMBED_DIM: usize = 64;

/// Trait for embedding providers — both hash-based and real model backends.
pub trait EmbeddingProvider: Send + Sync {
    /// Return the output dimension.
    fn dimension(&self) -> usize;
    /// Embed a single text.
    fn embed(&self, text: &str) -> KernelResult<Vec<f32>>;
    /// Embed a batch of texts (default: sequential).
    fn embed_batch(&self, texts: &[&str]) -> KernelResult<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
}

/// Hash-based pseudo-embedding provider (always available, used as fallback).
pub struct HashEmbeddingProvider;

impl EmbeddingProvider for HashEmbeddingProvider {
    fn dimension(&self) -> usize {
        EMBED_DIM
    }

    fn embed(&self, text: &str) -> KernelResult<Vec<f32>> {
        Ok(pseudo_text_to_embedding(text, self.dimension()))
    }
}

/// Generate a deterministic pseudo-embedding from text.
///
/// This is NOT a real embedding model — it produces a reproducible float
/// vector by hashing character unigrams and trigrams into buckets, then
/// L2-normalising. Good enough for demo / integration-test purposes.
fn pseudo_text_to_embedding(text: &str, dim: usize) -> Vec<f32> {
    let mut vec = vec![0.0_f32; dim];
    let lower = text.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();

    // Hash unigrams and trigrams into the vector.
    for ch in &chars {
        let idx = (*ch as usize) % dim;
        vec[idx] += 1.0;
    }
    for window in chars.windows(3) {
        let hash = window.iter().fold(0_usize, |acc, c| {
            acc.wrapping_mul(31).wrapping_add(*c as usize)
        });
        let idx = hash % dim;
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

// ---------------------------------------------------------------------------
// Global embedding provider (OnceLock)
// ---------------------------------------------------------------------------

static EMBEDDING_PROVIDER: std::sync::OnceLock<Box<dyn EmbeddingProvider>> =
    std::sync::OnceLock::new();

/// Initialize the global embedding provider. Call once at startup.
/// Returns Err if already initialized.
pub fn init_embedding_provider(provider: Box<dyn EmbeddingProvider>) -> KernelResult<()> {
    EMBEDDING_PROVIDER
        .set(provider)
        .map_err(|_| KernelError::Internal("embedding provider already initialized".into()))
}

/// Generate an embedding using the global provider (hash fallback if uninitialized).
pub fn text_to_embedding(text: &str) -> Vec<f32> {
    match EMBEDDING_PROVIDER.get() {
        Some(provider) => provider.embed(text).unwrap_or_else(|e| {
            tracing::warn!("embedding provider failed, using hash fallback: {e}");
            pseudo_text_to_embedding(text, EMBED_DIM)
        }),
        None => pseudo_text_to_embedding(text, EMBED_DIM),
    }
}

// ---------------------------------------------------------------------------
// Real embedding model via Candle (feature = "real-embeddings")
// ---------------------------------------------------------------------------

#[cfg(feature = "real-embeddings")]
pub mod candle_embedder {
    use super::*;
    use std::sync::Mutex;

    /// Real embedding provider using embeddinggemma-300M via Candle.
    pub struct CandleEmbedder {
        tokenizer: tokenizers::Tokenizer,
        model: Mutex<candle_transformers::models::bert::BertModel>,
        #[allow(dead_code)]
        device: candle_core::Device,
    }

    impl CandleEmbedder {
        /// Load from default model directory ($RLMX_MODEL_DIR or ~/.rlmx/models).
        pub fn load_default() -> KernelResult<Self> {
            let model_dir = std::env::var("RLMX_MODEL_DIR")
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                    format!("{home}/.rlmx/models")
                });
            Self::load(&model_dir)
        }

        /// Load model and tokenizer from a directory.
        pub fn load(model_dir: &str) -> KernelResult<Self> {
            let tokenizer_path = format!("{model_dir}/embeddinggemma-300M-tokenizer.json");
            let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
                .map_err(|e| KernelError::ModelLoadError(format!("tokenizer: {e}")))?;

            #[cfg(target_os = "macos")]
            let device = candle_core::Device::new_metal(0)
                .unwrap_or(candle_core::Device::Cpu);
            #[cfg(not(target_os = "macos"))]
            let device = candle_core::Device::Cpu;

            // Load model weights
            let model_path = format!("{model_dir}/embeddinggemma-300M.gguf");
            let vb = candle_nn::VarBuilder::from_gguf(&model_path, &device)
                .map_err(|e| KernelError::ModelLoadError(format!("model weights: {e}")))?;

            let config = candle_transformers::models::bert::Config {
                hidden_size: 256,
                num_attention_heads: 4,
                num_hidden_layers: 6,
                intermediate_size: 1024,
                vocab_size: 256000,
                max_position_embeddings: 512,
                type_vocab_size: 2,
                ..Default::default()
            };

            let model = candle_transformers::models::bert::BertModel::load(vb, &config)
                .map_err(|e| KernelError::ModelLoadError(format!("model load: {e}")))?;

            tracing::info!(model_dir, "loaded embeddinggemma-300M");

            Ok(Self {
                tokenizer,
                model: Mutex::new(model),
                device,
            })
        }
    }

    impl EmbeddingProvider for CandleEmbedder {
        fn dimension(&self) -> usize {
            256
        }

        fn embed(&self, text: &str) -> KernelResult<Vec<f32>> {
            let emb_err = |label: &'static str| {
                move |e: impl std::fmt::Display| KernelError::EmbeddingError(format!("{label}: {e}"))
            };

            let encoding = self.tokenizer.encode(text, true).map_err(emb_err("tokenize"))?;

            let ids = encoding.get_ids();
            let token_ids = candle_core::Tensor::new(ids, &self.device)
                .map_err(emb_err("tensor"))?
                .unsqueeze(0)
                .map_err(emb_err("unsqueeze"))?;

            let token_type_ids = token_ids.zeros_like().map_err(emb_err("zeros"))?;

            let model = self.model.lock().map_err(emb_err("lock"))?;

            let output = model.forward(&token_ids, &token_type_ids, None)
                .map_err(emb_err("forward"))?;

            // Mean pool over sequence length
            let (_, seq_len, _) = output.dims3().map_err(emb_err("dims"))?;
            let pooled = (output.sum(1).map_err(emb_err("sum"))?) / (seq_len as f64);
            let pooled = pooled.squeeze(0).map_err(emb_err("squeeze"))?;

            // L2-normalize
            let norm = pooled.sqr().map_err(emb_err("sqr"))?
                .sum_all().map_err(emb_err("sum_all"))?
                .sqrt().map_err(emb_err("sqrt"))?;

            let normalized = pooled.broadcast_div(&norm).map_err(emb_err("div"))?;
            let vec: Vec<f32> = normalized.to_vec1().map_err(emb_err("to_vec"))?;

            Ok(vec)
        }
    }
}

#[cfg(feature = "real-embeddings")]
pub use candle_embedder::CandleEmbedder;

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
    ///
    /// Returns an error if `embedding.len() != EMBED_DIM`.
    pub fn insert(
        &mut self,
        embedding: Vec<f32>,
        content: String,
        metadata: SegmentMetadata,
    ) -> KernelResult<Uuid> {
        if embedding.len() != EMBED_DIM {
            return Err(KernelError::Internal(format!(
                "embedding dimension mismatch: expected {EMBED_DIM}, got {}",
                embedding.len()
            )));
        }
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
        Ok(id)
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
    ) -> KernelResult<Uuid> {
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
        Ok(id)
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

#[cfg(feature = "ruvnet-phase1")]
pub mod phase1_integration {
    use super::*;
    use ruvector_collections as rvc_col;
    use ruvector_filter as rvf;
    use tracing::debug;
    const BUCKET_COUNT: u32 = 16;
    fn embedding_to_bucket_key(embedding: &[f32]) -> String {
        let dims = embedding.len().min(8);
        let mut key = String::with_capacity(dims * 3);
        for (i, &v) in embedding.iter().take(dims).enumerate() {
            let clamped = v.clamp(-1.0, 1.0);
            let bucket = ((clamped + 1.0) / 2.0 * (BUCKET_COUNT - 1) as f32).round() as u32;
            if i > 0 {
                key.push(':');
            }
            key.push_str(&bucket.to_string());
        }
        key
    }
    pub struct BloomScreenedRegion {
        pub name: String,
        inner: HnswMemoryRegion,
        filter: rvf::BloomFilter,
    }
    impl BloomScreenedRegion {
        pub fn new(
            name: impl Into<String>,
            dim: usize,
            expected_items: usize,
            fp_rate: f64,
        ) -> Self {
            let name = name.into();
            let filter = rvf::BloomFilter::new(expected_items, fp_rate);
            debug!(region = %name, dim, expected_items, fp_rate, "created BloomScreenedRegion");
            Self {
                name: name.clone(),
                inner: HnswMemoryRegion::new(name, dim),
                filter,
            }
        }
        pub fn insert(
            &mut self,
            embedding: Vec<f32>,
            content: String,
            metadata: SegmentMetadata,
        ) -> KernelResult<Uuid> {
            let key = embedding_to_bucket_key(&embedding);
            self.filter.insert(&key);
            self.inner.insert(embedding, content, metadata)
        }
        pub fn delete(&mut self, segment_id: &Uuid) -> KernelResult<bool> {
            self.inner.delete(segment_id)
        }
        pub fn search(&self, query: &[f32], k: usize, filters: &SearchFilters) -> Vec<SearchHit> {
            let key = embedding_to_bucket_key(query);
            if !self.filter.contains(&key) {
                debug!(region = %self.name, "bloom filter rejected query");
                return Vec::new();
            }
            self.inner.search(query, k, filters)
        }
        pub fn len(&self) -> usize {
            self.inner.len()
        }
        pub fn is_empty(&self) -> bool {
            self.inner.is_empty()
        }
        pub fn bloom_fp_estimate(&self) -> f64 {
            self.filter.estimated_fpp()
        }
    }
    pub struct NamespacedMemoryStore {
        namespaces: rvc_col::TypedCollection<HnswMemoryRegion>,
        dim: usize,
    }
    impl NamespacedMemoryStore {
        pub fn new(dim: usize) -> Self {
            Self {
                namespaces: rvc_col::TypedCollection::new(),
                dim,
            }
        }
        pub fn get_or_create(&mut self, namespace: &str) -> &mut HnswMemoryRegion {
            if !self.namespaces.contains(namespace) {
                let region = HnswMemoryRegion::new(namespace, self.dim);
                self.namespaces.insert(namespace.to_string(), region);
                debug!(namespace, dim = self.dim, "created new memory namespace");
            }
            self.namespaces
                .get_mut(namespace)
                .expect("namespace just inserted")
        }
        pub fn insert(
            &mut self,
            namespace: &str,
            embedding: Vec<f32>,
            content: String,
            metadata: SegmentMetadata,
        ) -> KernelResult<Uuid> {
            self.get_or_create(namespace)
                .insert(embedding, content, metadata)
        }
        pub fn search(
            &mut self,
            namespace: &str,
            query: &[f32],
            k: usize,
            filters: &SearchFilters,
        ) -> Vec<SearchHit> {
            self.get_or_create(namespace).search(query, k, filters)
        }
        pub fn namespaces(&self) -> Vec<String> {
            self.namespaces.keys()
        }
        pub fn total_segments(&self) -> usize {
            self.namespaces
                .keys()
                .iter()
                .filter_map(|k| self.namespaces.get(k))
                .map(|r| r.len())
                .sum()
        }
    }
}
#[cfg(feature = "ruvnet-phase1")]
pub use phase1_integration::{BloomScreenedRegion, NamespacedMemoryStore};

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

    /// Create an EMBED_DIM-length vector with the given leading values, zero-padded.
    fn padded(values: &[f32]) -> Vec<f32> {
        let mut v = vec![0.0_f32; EMBED_DIM];
        for (i, &val) in values.iter().enumerate() {
            v[i] = val;
        }
        v
    }

    #[test]
    fn test_insert_and_len() {
        let mut region = MemoryRegion::new("test");
        let id = region.insert(padded(&[1.0, 0.0, 0.0]), "hello".into(), make_meta()).unwrap();
        assert_eq!(region.len(), 1);
        assert!(!id.is_nil());
    }

    #[test]
    fn test_search_returns_best_match() {
        let mut region = MemoryRegion::new("test");
        region.insert(padded(&[1.0, 0.0, 0.0]), "east".into(), make_meta()).unwrap();
        region.insert(padded(&[0.0, 1.0, 0.0]), "north".into(), make_meta()).unwrap();
        region.insert(padded(&[0.7, 0.7, 0.0]), "northeast".into(), make_meta()).unwrap();

        let results = region.search(&padded(&[1.0, 0.0, 0.0]), 2, &SearchFilters::default());
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].content, "east");
        assert!((results[0].score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_delete_segment() {
        let mut region = MemoryRegion::new("test");
        let id = region.insert(padded(&[1.0, 0.0]), "a".into(), make_meta()).unwrap();
        region.insert(padded(&[0.0, 1.0]), "b".into(), make_meta()).unwrap();
        assert_eq!(region.len(), 2);

        let deleted = region.delete(&id).unwrap();
        assert!(deleted);
        assert_eq!(region.len(), 1);

        // Deleting again should error.
        assert!(region.delete(&id).is_err());
    }

    #[test]
    fn test_insert_rejects_wrong_dimension() {
        let mut region = MemoryRegion::new("test");
        let result = region.insert(vec![1.0, 0.0, 0.0], "bad".into(), make_meta());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("embedding dimension mismatch"), "unexpected error: {err}");
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
