//! rlmx-ruvllm: Local LLM inference for RLMX edge deployment
//!
//! This crate provides a local inference engine backed by ruvllm for
//! running GGUF models on CPU (NEON/ARM64), Metal, CUDA, or WebGPU.
//!
//! ## Feature Flags
//!
//! - `ruvllm`: Enable the actual ruvllm inference engine. Without this
//!   feature, `LocalEngine` compiles as a stub returning `NotAvailable`.
//!
//! ## Architecture
//!
//! - **config**: `EdgeConfig`, `HardwareBackend`, `ModelSpec`
//! - **engine**: `LocalEngine` — the core inference API
//! - **model**: `ModelManager` — GGUF file discovery and caching
//! - **error**: `RuvllmError` via thiserror

pub mod config;
pub mod engine;
pub mod error;
pub mod mlx_bridge;
pub mod model;
pub mod tiered;

// Re-export primary types for convenience
pub use config::{
    EdgeConfig, HardwareBackend, InferenceBackendType, ModelSpec, ModelTier, TieredConfig,
};
pub use engine::{ChatMessage, GenerateResult, LocalEngine};
pub use error::RuvllmError;
pub use mlx_bridge::{MlxResponse, MlxSubprocess};
pub use model::ModelManager;
pub use tiered::{TieredEngine, TieredResult, TieredStats};
