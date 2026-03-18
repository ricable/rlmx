//! vLLM Client - OpenAI-compatible API client for vLLM inference.
//!
//! Provides HTTP client for communicating with vLLM servers using the
//! OpenAI-compatible chat completions API. Supports structured JSON output,
//! batch completions for parallel sub-agents, and retry logic with
//! exponential backoff.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

/// Errors from the vLLM client.
#[derive(Debug, Error)]
pub enum VllmError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("API returned error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("failed to parse API response: {0}")]
    ParseError(String),

    #[error("no completion choices returned")]
    EmptyResponse,

    #[error("max retries ({0}) exceeded")]
    MaxRetriesExceeded(u32),

    #[error("health check failed: {0}")]
    HealthCheckFailed(String),
}

/// A message in the chat completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// Response format specification for structured output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

impl ResponseFormat {
    pub fn json() -> Self {
        Self {
            format_type: "json_object".to_string(),
        }
    }

    pub fn text() -> Self {
        Self {
            format_type: "text".to_string(),
        }
    }
}

/// Request body for the chat completions endpoint.
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

/// A single choice in the chat completion response.
#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

/// The message content in a completion choice.
#[derive(Debug, Deserialize)]
struct ChatCompletionMessage {
    #[allow(dead_code)]
    role: String,
    content: Option<String>,
}

/// The full chat completion response from vLLM.
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
    #[allow(dead_code)]
    usage: Option<UsageInfo>,
}

/// Token usage information.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct UsageInfo {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// Configuration for the vLLM client.
#[derive(Debug, Clone)]
pub struct VllmConfig {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub model: String,
    pub max_retries: u32,
    pub base_retry_delay_ms: u64,
}

impl Default for VllmConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8000".to_string(),
            api_key: None,
            model: "default".to_string(),
            max_retries: 3,
            base_retry_delay_ms: 500,
        }
    }
}

/// OpenAI-compatible API client for vLLM inference servers.
#[derive(Debug, Clone)]
pub struct VllmClient {
    client: Client,
    config: VllmConfig,
}

impl VllmClient {
    /// Create a new vLLM client with the given configuration.
    pub fn new(config: VllmConfig) -> Self {
        let client = Client::new();
        Self { client, config }
    }

    /// Create a client with just an endpoint URL and model name.
    pub fn with_endpoint(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new(VllmConfig {
            endpoint: endpoint.into(),
            model: model.into(),
            ..Default::default()
        })
    }

    /// Get the configured endpoint URL.
    pub fn endpoint(&self) -> &str {
        &self.config.endpoint
    }

    /// Get the configured model name.
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// Build the full URL for a given API path.
    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.config.endpoint.trim_end_matches('/'), path)
    }

    /// Build the request with optional auth headers.
    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut req = self.client.post(url);
        if let Some(ref api_key) = self.config.api_key {
            req = req.bearer_auth(api_key);
        }
        req.header("Content-Type", "application/json")
    }

    /// Send a chat completion request to the vLLM server.
    ///
    /// Supports structured JSON output via the `response_format` parameter.
    /// Implements retry logic with exponential backoff on transient failures.
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        temperature: Option<f64>,
        max_tokens: Option<u32>,
        response_format: Option<ResponseFormat>,
    ) -> Result<String, VllmError> {
        let request_body = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            temperature,
            max_tokens,
            response_format,
        };

        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let delay_ms =
                    self.config.base_retry_delay_ms * 2u64.pow(attempt.saturating_sub(1));
                debug!(attempt, delay_ms, "retrying vLLM request");
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            }

            let url = self.api_url("/v1/chat/completions");
            let result = self.build_request(&url).json(&request_body).send().await;

            match result {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        let body: ChatCompletionResponse = response
                            .json()
                            .await
                            .map_err(|e| VllmError::ParseError(e.to_string()))?;

                        let content = body
                            .choices
                            .into_iter()
                            .next()
                            .ok_or(VllmError::EmptyResponse)?
                            .message
                            .content
                            .ok_or(VllmError::EmptyResponse)?;

                        return Ok(content);
                    }

                    let status_code = status.as_u16();
                    let error_body = response.text().await.unwrap_or_default();

                    // Don't retry client errors (4xx) except 429 (rate limit)
                    if status.is_client_error() && status_code != 429 {
                        return Err(VllmError::ApiError {
                            status: status_code,
                            message: error_body,
                        });
                    }

                    warn!(
                        status = status_code,
                        attempt, "transient vLLM error, will retry"
                    );
                    last_error = Some(VllmError::ApiError {
                        status: status_code,
                        message: error_body,
                    });
                }
                Err(e) => {
                    warn!(error = %e, attempt, "vLLM request failed, will retry");
                    last_error = Some(VllmError::RequestError(e));
                }
            }
        }

        match last_error {
            Some(e) => Err(e),
            None => Err(VllmError::MaxRetriesExceeded(self.config.max_retries)),
        }
    }

    /// Send multiple chat completion requests in parallel for sub-agent execution.
    ///
    /// Returns results in the same order as the input requests.
    pub async fn batch_completion(
        &self,
        requests: Vec<(Vec<ChatMessage>, Option<f64>, Option<u32>)>,
    ) -> Result<Vec<String>, VllmError> {
        let futures: Vec<_> = requests
            .into_iter()
            .map(|(messages, temperature, max_tokens)| {
                self.chat_completion(messages, temperature, max_tokens, Some(ResponseFormat::json()))
            })
            .collect();

        let results = futures::future::join_all(futures).await;
        results.into_iter().collect()
    }

    /// Check if the vLLM server is healthy and responding.
    pub async fn health_check(&self) -> Result<(), VllmError> {
        let url = self.api_url("/health");
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| VllmError::HealthCheckFailed(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(VllmError::HealthCheckFailed(format!(
                "server returned status {}",
                response.status()
            )))
        }
    }

    /// Build a messages list from system prompt and user query.
    pub fn build_messages(system_prompt: &str, user_query: &str) -> Vec<ChatMessage> {
        vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_query),
        ]
    }

    /// Build a messages list with context segments included.
    pub fn build_messages_with_context(
        system_prompt: &str,
        context: &str,
        user_query: &str,
    ) -> Vec<ChatMessage> {
        vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(format!(
                "Context:\n{}\n\nQuery: {}",
                context, user_query
            )),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vllm_client_construction() {
        let client = VllmClient::with_endpoint("http://localhost:8000", "llama-3-70b");
        assert_eq!(client.endpoint(), "http://localhost:8000");
        assert_eq!(client.model(), "llama-3-70b");
    }

    #[test]
    fn test_message_formatting() {
        let messages =
            VllmClient::build_messages("You are a helpful assistant.", "What is Rust?");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, "You are a helpful assistant.");
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[1].content, "What is Rust?");

        let messages_with_ctx = VllmClient::build_messages_with_context(
            "System prompt",
            "Some relevant context here",
            "Analyze this",
        );
        assert_eq!(messages_with_ctx.len(), 2);
        assert!(messages_with_ctx[1].content.contains("Context:"));
        assert!(messages_with_ctx[1].content.contains("Analyze this"));
    }

    #[test]
    fn test_api_url_construction() {
        let client = VllmClient::with_endpoint("http://localhost:8000", "model");
        assert_eq!(
            client.api_url("/v1/chat/completions"),
            "http://localhost:8000/v1/chat/completions"
        );

        // Trailing slash should be handled
        let client2 = VllmClient::with_endpoint("http://localhost:8000/", "model");
        assert_eq!(
            client2.api_url("/v1/chat/completions"),
            "http://localhost:8000/v1/chat/completions"
        );
    }
}
