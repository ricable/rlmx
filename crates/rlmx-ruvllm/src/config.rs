//! Configuration types for edge inference.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Hardware backend for local inference.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum HardwareBackend {
    /// CPU-only (uses NEON on ARM64).
    Cpu,
    /// Apple Metal GPU acceleration.
    Metal,
    /// NVIDIA CUDA GPU acceleration.
    Cuda,
    /// WebGPU (for WASM targets).
    WebGpu,
    /// Auto-detect the best available backend.
    #[default]
    Auto,
}

impl std::fmt::Display for HardwareBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HardwareBackend::Cpu => write!(f, "cpu"),
            HardwareBackend::Metal => write!(f, "metal"),
            HardwareBackend::Cuda => write!(f, "cuda"),
            HardwareBackend::WebGpu => write!(f, "webgpu"),
            HardwareBackend::Auto => write!(f, "auto"),
        }
    }
}

impl std::str::FromStr for HardwareBackend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cpu" => Ok(HardwareBackend::Cpu),
            "metal" => Ok(HardwareBackend::Metal),
            "cuda" => Ok(HardwareBackend::Cuda),
            "webgpu" => Ok(HardwareBackend::WebGpu),
            "auto" => Ok(HardwareBackend::Auto),
            other => Err(format!("unknown backend: {}", other)),
        }
    }
}

/// Specification for a GGUF model file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    /// Human-readable model name (e.g., "qwen2.5-0.5b-q4").
    pub name: String,
    /// Path to the GGUF model file on disk.
    pub path: PathBuf,
    /// Quantization format (e.g., "Q4_K_M").
    pub quantization: String,
    /// Maximum context length in tokens.
    pub context_length: usize,
}

/// Configuration for the edge inference engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeConfig {
    /// The model to load.
    pub model: ModelSpec,
    /// Hardware backend to use.
    pub backend: HardwareBackend,
    /// Maximum memory budget in megabytes.
    pub max_memory_mb: usize,
    /// Default maximum tokens to generate.
    pub max_tokens: u32,
    /// Sampling temperature.
    pub temperature: f64,
    /// Number of CPU threads to use for inference.
    pub threads: usize,
}

/// Model tier for tiered inference — from smallest/fastest to largest/smartest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelTier {
    /// Small local model (e.g., 0.5B Q4).
    Small,
    /// Claude Code routed model.
    ClaudeCode,
    /// Medium local model (e.g., 1-3B Q4).
    Medium,
    /// Custom tier with explicit backend and parameters.
    Custom {
        model_name: String,
        backend: InferenceBackendType,
        max_tokens: usize,
    },
}

impl std::fmt::Display for ModelTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelTier::Small => write!(f, "small"),
            ModelTier::ClaudeCode => write!(f, "claude-code"),
            ModelTier::Medium => write!(f, "medium"),
            ModelTier::Custom { model_name, .. } => write!(f, "custom({})", model_name),
        }
    }
}

/// Inference backend type for tiered routing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InferenceBackendType {
    /// Local Candle-based inference.
    Candle,
    /// Apple MLX-based inference.
    Mlx,
    /// Remote inference endpoint.
    Remote(String),
}

/// Specification for a single tier in a tiered inference setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierSpec {
    /// Which tier this spec belongs to.
    pub tier: ModelTier,
    /// The model to use for this tier.
    pub model: ModelSpec,
    /// Priority (lower = tried first).
    pub priority: u32,
}

/// Configuration for the tiered inference engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TieredConfig {
    /// Ordered list of tier specifications.
    pub models: Vec<TierSpec>,
    /// Confidence threshold below which to escalate to next tier.
    pub escalation_threshold: f32,
    /// Maximum number of escalations per request.
    pub max_escalations: u32,
    /// Tier escalation chain order (default: [Small, Medium]).
    pub chain: Vec<ModelTier>,
    /// Timeout per tier in milliseconds before escalating (default: 5000).
    pub tier_timeout_ms: u64,
}

impl Default for TieredConfig {
    fn default() -> Self {
        Self {
            models: Vec::new(),
            escalation_threshold: 0.4,
            max_escalations: 2,
            chain: vec![ModelTier::Small, ModelTier::Medium],
            tier_timeout_ms: 5000,
        }
    }
}

impl Default for EdgeConfig {
    fn default() -> Self {
        Self {
            model: ModelSpec {
                name: "none".into(),
                path: PathBuf::new(),
                quantization: "Q4_K_M".into(),
                context_length: 2048,
            },
            backend: HardwareBackend::Auto,
            max_memory_mb: 600,
            max_tokens: 256,
            temperature: 0.7,
            threads: 4,
        }
    }
}
