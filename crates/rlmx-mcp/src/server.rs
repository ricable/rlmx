//! MCP Server
//!
//! Implements an MCP-protocol-compatible server that can operate over
//! stdio or HTTP transports.

use serde_json::json;
use tracing::info;

use crate::protocol::{
    McpError, McpRequest, McpResponse, McpTool,
    METHOD_NOT_FOUND,
};

/// Transport mode for the MCP server.
#[derive(Debug, Clone)]
pub enum Transport {
    /// Communicate via stdin/stdout (line-delimited JSON-RPC).
    Stdio,
    /// Communicate via HTTP using streamable-http transport.
    StreamableHttp {
        /// Host to bind to.
        host: String,
        /// Port to bind to.
        port: u16,
    },
}

/// Configuration for the MCP server.
#[derive(Debug, Clone)]
pub struct McpConfig {
    /// Transport mode.
    pub transport: Transport,
    /// Whether authentication is enabled.
    pub auth_enabled: bool,
    /// Maximum requests per minute (0 = unlimited).
    pub rate_limit_per_minute: usize,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            transport: Transport::Stdio,
            auth_enabled: false,
            rate_limit_per_minute: 0,
        }
    }
}

/// The MCP server that handles JSON-RPC requests and dispatches to tools.
pub struct McpServer {
    /// Registered tools.
    tools: Vec<McpTool>,
    /// Transport configuration.
    transport: Transport,
    /// Server configuration.
    #[allow(dead_code)]
    config: McpConfig,
    /// Whether the server has been initialized via the MCP handshake.
    initialized: bool,
}

impl McpServer {
    /// Create a new MCP server with the given configuration.
    pub fn new(config: McpConfig) -> Self {
        let transport = config.transport.clone();
        Self {
            tools: Vec::new(),
            transport,
            config,
            initialized: false,
        }
    }

    /// Register a tool with the server.
    pub fn register_tool(&mut self, tool: McpTool) {
        info!(tool_name = %tool.name, "Registering MCP tool");
        self.tools.push(tool);
    }

    /// Register multiple tools at once.
    pub fn register_tools(&mut self, tools: Vec<McpTool>) {
        for tool in tools {
            self.register_tool(tool);
        }
    }

    /// Get the number of registered tools.
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Get the list of registered tool names.
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.iter().map(|t| t.name.as_str()).collect()
    }

    /// Start the server listening on the configured transport.
    pub async fn start(&mut self) -> Result<(), McpError> {
        info!("Starting MCP server");

        match &self.transport {
            Transport::Stdio => {
                crate::stdio::run_stdio_loop(self).await
            }
            Transport::StreamableHttp { host, port } => {
                let host = host.clone();
                let port = *port;
                crate::http::run_http_server(self, &host, port).await
            }
        }
    }

    /// Handle an incoming MCP request and produce a response.
    pub async fn handle_request(&mut self, request: &McpRequest) -> McpResponse {
        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return McpResponse::error(
                request.id.clone(),
                McpError::new(-32600, "Invalid JSON-RPC version, expected 2.0"),
            );
        }

        match request.method.as_str() {
            "initialize" => self.handle_initialize(request),
            "initialized" => {
                // Notification: no response needed, but we return success
                McpResponse::success(request.id.clone(), json!({}))
            }
            "tools/list" => self.handle_tools_list(request),
            "tools/call" => self.handle_tools_call(request).await,
            _ => McpResponse::error(
                request.id.clone(),
                McpError::method_not_found(&request.method),
            ),
        }
    }

    /// Handle the `initialize` method.
    fn handle_initialize(&mut self, request: &McpRequest) -> McpResponse {
        self.initialized = true;
        info!("MCP server initialized");

        McpResponse::success(request.id.clone(), json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "rlmx-mcp",
                "version": "0.1.0"
            }
        }))
    }

    /// Handle the `tools/list` method.
    fn handle_tools_list(&self, request: &McpRequest) -> McpResponse {
        let tools: Vec<serde_json::Value> = self.tools.iter().map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.input_schema
            })
        }).collect();

        McpResponse::success(request.id.clone(), json!({ "tools": tools }))
    }

    /// Handle the `tools/call` method.
    async fn handle_tools_call(&self, request: &McpRequest) -> McpResponse {
        let params = match &request.params {
            Some(p) => p,
            None => {
                return McpResponse::error(
                    request.id.clone(),
                    McpError::invalid_params("Missing params for tools/call"),
                );
            }
        };

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                return McpResponse::error(
                    request.id.clone(),
                    McpError::invalid_params("Missing 'name' in tools/call params"),
                );
            }
        };

        let arguments = params.get("arguments")
            .cloned()
            .unwrap_or(json!({}));

        // Find the tool
        let tool = match self.tools.iter().find(|t| t.name == tool_name) {
            Some(t) => t,
            None => {
                return McpResponse::error(
                    request.id.clone(),
                    McpError::new(METHOD_NOT_FOUND, format!("Tool not found: {}", tool_name)),
                );
            }
        };

        // Execute the tool handler
        match tool.handler.handle(arguments).await {
            Ok(result) => McpResponse::success(request.id.clone(), json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                }]
            })),
            Err(e) => McpResponse::error(request.id.clone(), e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::create_all_tools;

    fn make_server() -> McpServer {
        let config = McpConfig::default();
        let mut server = McpServer::new(config);
        server.register_tools(create_all_tools());
        server
    }

    #[test]
    fn test_tool_registration() {
        let server = make_server();
        assert_eq!(server.tool_count(), 12);
    }

    #[test]
    fn test_tool_listing_names() {
        let server = make_server();
        let names = server.tool_names();
        assert!(names.contains(&"rlmx_query"));
        assert!(names.contains(&"rlmx_graph_query"));
        assert!(names.contains(&"rlmx_sona_stats"));
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let mut server = make_server();
        let req = McpRequest::new(json!(1), "initialize", Some(json!({})));
        let resp = server.handle_request(&req).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["serverInfo"]["name"], "rlmx-mcp");
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let mut server = make_server();
        let req = McpRequest::new(json!(2), "tools/list", None);
        let resp = server.handle_request(&req).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 12);
    }

    #[tokio::test]
    async fn test_handle_tools_call() {
        let mut server = make_server();
        let req = McpRequest::new(json!(3), "tools/call", Some(json!({
            "name": "rlmx_memory_stats",
            "arguments": {}
        })));
        let resp = server.handle_request(&req).await;
        assert!(resp.error.is_none());
    }

    #[tokio::test]
    async fn test_handle_method_not_found() {
        let mut server = make_server();
        let req = McpRequest::new(json!(4), "nonexistent/method", None);
        let resp = server.handle_request(&req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, METHOD_NOT_FOUND);
    }
}
