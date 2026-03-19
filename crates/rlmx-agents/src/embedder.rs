use crate::types::AgentId;

/// Embedder agent — generates 64-dimensional pseudo-embeddings.
pub struct EmbedderAgent {
    pub id: AgentId,
    pub embeddings_generated: u64,
    pub model: String,
}

impl EmbedderAgent {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            id: AgentId::new(),
            embeddings_generated: 0,
            model: model.into(),
        }
    }

    /// Generate a 64-dimensional pseudo-embedding from text using hash-based approach.
    /// Matches the kernel's brute-force cosine similarity with 64-dim hash-based embeddings.
    pub fn embed(&mut self, text: &str) -> Vec<f32> {
        self.embeddings_generated += 1;
        Self::hash_embed(text)
    }

    /// Generate embeddings for multiple texts.
    pub fn batch_embed(&mut self, texts: &[&str]) -> Vec<Vec<f32>> {
        texts
            .iter()
            .map(|t| {
                self.embeddings_generated += 1;
                Self::hash_embed(t)
            })
            .collect()
    }

    /// Total number of embeddings generated.
    pub fn embeddings_count(&self) -> u64 {
        self.embeddings_generated
    }

    /// Hash-based 64-dimensional pseudo-embedding generator.
    /// Uses a simple hash spreading algorithm for deterministic output.
    fn hash_embed(text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0f32; 64];

        // Use FNV-1a-like hash for each position
        for (i, dim) in embedding.iter_mut().enumerate() {
            let mut hash: u64 = 14695981039346656037_u64.wrapping_add(i as u64);
            for byte in text.bytes() {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(1099511628211);
            }
            // Map to [-1, 1] range
            *dim = ((hash as f32) / (u64::MAX as f32)) * 2.0 - 1.0;
        }

        // Normalize to unit vector
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for dim in &mut embedding {
                *dim /= norm;
            }
        }

        embedding
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_dimension() {
        let mut embedder = EmbedderAgent::new("pseudo-64d");
        let emb = embedder.embed("hello world");
        assert_eq!(emb.len(), 64);
    }

    #[test]
    fn test_embed_deterministic() {
        let mut embedder = EmbedderAgent::new("pseudo-64d");
        let emb1 = embedder.embed("hello world");
        let emb2 = embedder.embed("hello world");
        assert_eq!(emb1, emb2);
    }

    #[test]
    fn test_embed_different_inputs() {
        let mut embedder = EmbedderAgent::new("pseudo-64d");
        let emb1 = embedder.embed("hello");
        let emb2 = embedder.embed("world");
        assert_ne!(emb1, emb2);
    }

    #[test]
    fn test_embed_normalized() {
        let mut embedder = EmbedderAgent::new("pseudo-64d");
        let emb = embedder.embed("test normalization");
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 0.001,
            "Expected unit vector, got norm={norm}"
        );
    }

    #[test]
    fn test_batch_embed() {
        let mut embedder = EmbedderAgent::new("pseudo-64d");
        let texts = vec!["hello", "world", "foo"];
        let embeddings = embedder.batch_embed(&texts);
        assert_eq!(embeddings.len(), 3);
        assert!(embeddings.iter().all(|e| e.len() == 64));
    }

    #[test]
    fn test_embeddings_count() {
        let mut embedder = EmbedderAgent::new("pseudo-64d");
        assert_eq!(embedder.embeddings_count(), 0);
        embedder.embed("a");
        embedder.embed("b");
        assert_eq!(embedder.embeddings_count(), 2);
        embedder.batch_embed(&["c", "d", "e"]);
        assert_eq!(embedder.embeddings_count(), 5);
    }

    #[test]
    fn test_embed_empty_string() {
        let mut embedder = EmbedderAgent::new("pseudo-64d");
        let emb = embedder.embed("");
        assert_eq!(emb.len(), 64);
    }
}
