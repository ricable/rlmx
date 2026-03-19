//! Model management — GGUF file discovery and caching.

use std::path::PathBuf;

use crate::config::ModelSpec;

/// Manages GGUF model files in a local cache directory.
pub struct ModelManager {
    /// Directory where GGUF model files are stored.
    pub cache_dir: PathBuf,
}

impl ModelManager {
    /// Create a new model manager pointing at the given cache directory.
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// List all GGUF model files found in the cache directory.
    pub fn list_models(&self) -> Vec<ModelSpec> {
        let mut models = Vec::new();

        let entries = match std::fs::read_dir(&self.cache_dir) {
            Ok(entries) => entries,
            Err(_) => return models,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("gguf") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                models.push(ModelSpec {
                    name,
                    path,
                    quantization: "unknown".into(),
                    context_length: 2048,
                });
            }
        }

        models
    }

    /// Resolve a model name to its GGUF file path in the cache directory.
    pub fn resolve(&self, name: &str) -> Option<PathBuf> {
        // Try exact filename first.
        let exact = self.cache_dir.join(format!("{}.gguf", name));
        if exact.exists() {
            return Some(exact);
        }

        // Try partial match.
        let entries = std::fs::read_dir(&self.cache_dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("gguf") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem.contains(name) {
                        return Some(path);
                    }
                }
            }
        }

        None
    }

    /// Check whether a model is already cached.
    pub fn is_cached(&self, name: &str) -> bool {
        self.resolve(name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_models_empty_dir() {
        let dir = std::env::temp_dir().join("rlmx-test-models-empty");
        let _ = std::fs::create_dir_all(&dir);
        let mgr = ModelManager::new(dir.clone());
        let models = mgr.list_models();
        assert!(models.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_nonexistent() {
        let dir = std::env::temp_dir().join("rlmx-test-models-resolve");
        let _ = std::fs::create_dir_all(&dir);
        let mgr = ModelManager::new(dir.clone());
        assert!(mgr.resolve("nonexistent").is_none());
        assert!(!mgr.is_cached("nonexistent"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
