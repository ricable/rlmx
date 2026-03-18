//! MCP Protocol Types
//!
//! Implements the JSON-RPC based Model Context Protocol (MCP) types
//! used for communication between clients and the RLMX MCP server.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A JSON-RPC 2.0 request following the MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    /// Must be "2.0"
    pub jsonrpc: String,
    /// Request identifier
    pub id: serde_json::Value,
    /// The method to invoke
    pub method: String,
    /// Optional parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// A JSON-RPC 2.0 response following the MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// Must be "2.0"
    pub jsonrpc: String,
    /// Matching request identifier
    pub id: serde_json::Value,
    /// Result on success
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error on failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

/// A JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct McpError {
    /// Error code (follows JSON-RPC conventions)
    pub code: i32,
    /// Human-readable error message
    pub message: String,
    /// Optional additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MCP error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for McpError {}

/// An MCP tool definition with its handler.
pub struct McpTool {
    /// Tool name (e.g., "rlmx_query")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// JSON Schema describing the tool's input parameters
    pub input_schema: serde_json::Value,
    /// The handler that executes the tool
    pub handler: Box<dyn ToolHandler>,
}

impl std::fmt::Debug for McpTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("input_schema", &self.input_schema)
            .finish()
    }
}

/// Trait for implementing tool handlers.
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Handle a tool invocation with the given parameters.
    async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, McpError>;
}

// --- Standard JSON-RPC error codes ---

/// Parse error: Invalid JSON was received.
pub const PARSE_ERROR: i32 = -32700;
/// Invalid request: The JSON sent is not a valid Request object.
pub const INVALID_REQUEST: i32 = -32600;
/// Method not found: The method does not exist.
pub const METHOD_NOT_FOUND: i32 = -32601;
/// Invalid params: Invalid method parameter(s).
pub const INVALID_PARAMS: i32 = -32602;
/// Internal error: Internal JSON-RPC error.
pub const INTERNAL_ERROR: i32 = -32603;

impl McpRequest {
    /// Create a new MCP request.
    pub fn new(
        id: serde_json::Value,
        method: impl Into<String>,
        params: Option<serde_json::Value>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }

    /// Parse an MCP request from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, McpError> {
        serde_json::from_str(json).map_err(|e| McpError {
            code: PARSE_ERROR,
            message: format!("Failed to parse request: {}", e),
            data: None,
        })
    }
}

impl McpResponse {
    /// Create a success response.
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: serde_json::Value, error: McpError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }

    /// Serialize the response to a JSON string.
    pub fn to_json(&self) -> Result<String, McpError> {
        serde_json::to_string(self).map_err(|e| McpError {
            code: INTERNAL_ERROR,
            message: format!("Failed to serialize response: {}", e),
            data: None,
        })
    }
}

impl McpError {
    /// Create a new MCP error.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create a method-not-found error.
    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: METHOD_NOT_FOUND,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }

    /// Create an invalid-params error.
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: INVALID_PARAMS,
            message: message.into(),
            data: None,
        }
    }

    /// Create an internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: INTERNAL_ERROR,
            message: message.into(),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_request() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":null}"#;
        let req = McpRequest::from_json(json).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.id, serde_json::json!(1));
        assert_eq!(req.method, "tools/list");
    }

    #[test]
    fn test_parse_request_with_params() {
        let json = r#"{"jsonrpc":"2.0","id":"abc-123","method":"tools/call","params":{"name":"rlmx_query","arguments":{"query":"test"}}}"#;
        let req = McpRequest::from_json(json).unwrap();
        assert_eq!(req.method, "tools/call");
        let params = req.params.unwrap();
        assert_eq!(params["name"], "rlmx_query");
    }

    #[test]
    fn test_response_success_formatting() {
        let resp = McpResponse::success(serde_json::json!(1), serde_json::json!({"status": "ok"}));
        let json = resp.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 1);
        assert_eq!(parsed["result"]["status"], "ok");
        assert!(parsed.get("error").map_or(true, |v| v.is_null()));
    }

    #[test]
    fn test_error_response_formatting() {
        let resp = McpResponse::error(
            serde_json::json!(2),
            McpError::method_not_found("nonexistent"),
        );
        let json = resp.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["error"]["code"], METHOD_NOT_FOUND);
        assert!(parsed["error"]["message"]
            .as_str()
            .unwrap()
            .contains("nonexistent"));
    }
}
