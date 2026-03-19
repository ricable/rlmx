//! MCP Server
//!
//! Implements an MCP-protocol-compatible server that can operate over
//! stdio or HTTP transports.

use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};

use rlmx_rvf::rbac::{AccessControl, Operation, Role};

use crate::protocol::{McpError, McpRequest, McpResponse, McpTool, METHOD_NOT_FOUND};

/// Error code for "server not initialized" per MCP protocol.
const SERVER_NOT_INITIALIZED: i32 = -32002;

/// Error code for access denied.
const ACCESS_DENIED: i32 = -32001;

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
    /// Bearer token required when `auth_enabled` is true.
    pub auth_token: Option<String>,
    /// Maximum requests per minute (0 = unlimited).
    pub rate_limit_per_minute: usize,
    /// Server-side mapping of bearer tokens to RBAC roles.
    ///
    /// Privileged roles (`Admin`, `System`) can only be assigned via this
    /// mapping — clients cannot self-escalate to these roles through
    /// request parameters.
    pub token_roles: HashMap<String, Role>,
    /// WebSocket event server port (default: 3001).
    pub ws_port: u16,
}

impl McpConfig {
    /// Look up the RBAC role assigned to a given bearer token.
    ///
    /// Returns `None` if the token has no explicit role mapping.
    pub fn role_for_token(&self, token: &str) -> Option<&Role> {
        self.token_roles.get(token)
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            transport: Transport::Stdio,
            auth_enabled: false,
            auth_token: None,
            rate_limit_per_minute: 0,
            token_roles: HashMap::new(),
            ws_port: 3001,
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
    config: McpConfig,
    /// Whether the server has been initialized via the MCP handshake.
    initialized: bool,
    /// RBAC access control for enforcing permissions on tool calls.
    access_control: AccessControl,
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
            access_control: AccessControl::new(),
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

    /// Returns whether authentication is enabled.
    pub fn auth_enabled(&self) -> bool {
        self.config.auth_enabled
    }

    /// Returns a reference to the server configuration.
    pub fn config(&self) -> &McpConfig {
        &self.config
    }

    /// Returns the configured auth token, if any.
    pub fn auth_token(&self) -> Option<&str> {
        self.config.auth_token.as_deref()
    }

    /// Start the server listening on the configured transport.
    pub async fn start(mut self) -> Result<(), McpError> {
        info!("Starting MCP server");

        match &self.transport {
            Transport::Stdio => crate::stdio::run_stdio_loop(&mut self).await,
            Transport::StreamableHttp { host, port } => {
                let host = host.clone();
                let port = *port;
                crate::http::run_http_server(self, &host, port).await
            }
        }
    }

    /// Start the server with a shared tool state for event bus wiring.
    ///
    /// The WebSocket event bus will be stored in the `ToolState` so that
    /// research tools and the CLI research loop can broadcast events to
    /// connected dashboard clients.
    pub async fn start_with_state(
        mut self,
        state: crate::tools::SharedToolState,
    ) -> Result<(), McpError> {
        info!("Starting MCP server with shared state");

        match &self.transport {
            Transport::Stdio => crate::stdio::run_stdio_loop(&mut self).await,
            Transport::StreamableHttp { host, port } => {
                let host = host.clone();
                let port = *port;
                crate::http::run_http_server_with_state(self, &host, port, Some(state)).await
            }
        }
    }

    /// Handle an incoming MCP request and produce a response.
    ///
    /// `caller_token` is the bearer token presented by the caller (if any).
    /// It is used to look up the caller's RBAC role via `token_roles`.
    /// Pass `None` for unauthenticated transports (e.g., stdio).
    pub async fn handle_request(
        &mut self,
        request: &McpRequest,
        caller_token: Option<&str>,
    ) -> McpResponse {
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
            "tools/list" | "tools/call" => {
                // Issue 14: Reject tools requests before initialization
                if !self.initialized {
                    warn!(method = %request.method, "Rejected request: server not initialized");
                    return McpResponse::error(
                        request.id.clone(),
                        McpError::new(SERVER_NOT_INITIALIZED, "Server not initialized"),
                    );
                }
                if request.method == "tools/list" {
                    self.handle_tools_list(request)
                } else {
                    self.handle_tools_call(request, caller_token).await
                }
            }
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

        McpResponse::success(
            request.id.clone(),
            json!({
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
            }),
        )
    }

    /// Handle the `tools/list` method.
    fn handle_tools_list(&self, request: &McpRequest) -> McpResponse {
        let tools: Vec<serde_json::Value> = self
            .tools
            .iter()
            .map(|tool| {
                json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool.input_schema
                })
            })
            .collect();

        McpResponse::success(request.id.clone(), json!({ "tools": tools }))
    }

    /// Handle the `tools/call` method.
    async fn handle_tools_call(
        &self,
        request: &McpRequest,
        caller_token: Option<&str>,
    ) -> McpResponse {
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

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

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

        // RBAC enforcement: determine the caller's role.
        //
        // SECURITY: Privileged roles (Admin, System) CANNOT be assigned via
        // client-supplied `_role` parameter. They can only be granted
        // server-side through the `token_roles` mapping in McpConfig.
        // This prevents privilege escalation attacks where a client
        // self-assigns Admin or System roles.

        // First, reject any attempt to claim a privileged role via params.
        if let Some(role_str) = params.get("_role").and_then(|v| v.as_str()) {
            match role_str.to_lowercase().as_str() {
                "admin" | "system" => {
                    warn!(
                        requested_role = role_str,
                        "Rejected attempt to self-assign privileged role via _role parameter"
                    );
                    return McpResponse::error(
                        request.id.clone(),
                        McpError::new(
                            ACCESS_DENIED,
                            format!(
                                "Access denied: role '{}' cannot be assigned via request parameters; \
                                 privileged roles must be configured server-side",
                                role_str
                            ),
                        ),
                    );
                }
                _ => {} // non-privileged roles checked below
            }
        }

        let caller_role = if self.config.auth_enabled {
            // When auth is enabled, check if the caller's token has a
            // server-side role mapping (which may include Admin/System).
            let token_role =
                caller_token.and_then(|token| self.config.role_for_token(token).cloned());

            if let Some(role) = token_role {
                role
            } else {
                // Token is valid but has no explicit role mapping.
                // Allow client to request a non-privileged role, default to Viewer.
                Self::parse_unprivileged_role(params).unwrap_or(Role::Viewer)
            }
        } else {
            // Auth disabled — allow non-privileged role from params,
            // defaulting to Operator for backward compatibility.
            Self::parse_unprivileged_role(params).unwrap_or(Role::Operator)
        };

        // Map tool name to the required RBAC operation.
        let required_operation = Self::tool_to_operation(tool_name);

        if !self.access_control.check(&caller_role, &required_operation) {
            warn!(
                tool = tool_name,
                role = ?caller_role,
                operation = ?required_operation,
                "RBAC denied tool call"
            );
            return McpResponse::error(
                request.id.clone(),
                McpError::new(
                    ACCESS_DENIED,
                    format!(
                        "Access denied: role {:?} cannot perform {:?}",
                        caller_role, required_operation
                    ),
                ),
            );
        }

        // Execute the tool handler
        match tool.handler.handle(arguments).await {
            Ok(result) => McpResponse::success(
                request.id.clone(),
                json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                    }]
                }),
            ),
            Err(e) => McpResponse::error(request.id.clone(), e),
        }
    }

    /// Parse a non-privileged role from the `_role` request parameter.
    ///
    /// Only allows `Viewer`, `Operator`, `Engineer`, and `Auditor`.
    /// Returns `None` if `_role` is not present. Returns an error-like
    /// `None` for unknown values (callers should default appropriately).
    fn parse_unprivileged_role(params: &serde_json::Value) -> Option<Role> {
        let role_str = params.get("_role").and_then(|v| v.as_str())?;
        match role_str.to_lowercase().as_str() {
            "viewer" => Some(Role::Viewer),
            "operator" => Some(Role::Operator),
            "engineer" => Some(Role::Engineer),
            "auditor" => Some(Role::Auditor),
            // Admin and System are handled before this is called (rejected).
            // Unknown roles fall through to the caller's default.
            _ => None,
        }
    }

    /// Map a tool name to the RBAC operation it requires.
    fn tool_to_operation(tool_name: &str) -> Operation {
        match tool_name {
            "rlmx_query" | "rlmx_graph_query" => Operation::Query,
            "rlmx_ingest" => Operation::Ingest,
            "rlmx_plugin_list" | "rlmx_plugin_action" => Operation::PluginManage,
            "rlmx_strategy_override" => Operation::ParameterModify,
            "rlmx_witness_chain" => Operation::WitnessView,
            "rlmx_rvf_seal" => Operation::ContainerSeal,
            "rlmx_rvf_branch" => Operation::ContainerBranch,
            // Sandbox tools (ADR-011)
            "rlmx_sandbox_spawn" | "rlmx_sandbox_terminate" | "rlmx_fleet_deploy" => {
                Operation::ParameterModify
            }
            "rlmx_sandbox_status" | "rlmx_sandbox_list" => Operation::Query,
            // Default to Query for informational/stats tools.
            _ => Operation::Query,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{create_all_tools, new_shared_state};

    fn make_server() -> McpServer {
        let config = McpConfig::default();
        let mut server = McpServer::new(config);
        let state = new_shared_state();
        server.register_tools(create_all_tools(state));
        server
    }

    #[test]
    fn test_tool_registration() {
        let server = make_server();
        assert_eq!(server.tool_count(), 39);
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
        let resp = server.handle_request(&req, None).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["serverInfo"]["name"], "rlmx-mcp");
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let mut server = make_server();
        // Initialize first (Issue 14)
        let init_req = McpRequest::new(json!(0), "initialize", Some(json!({})));
        server.handle_request(&init_req, None).await;

        let req = McpRequest::new(json!(2), "tools/list", None);
        let resp = server.handle_request(&req, None).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 39);
    }

    #[tokio::test]
    async fn test_handle_tools_call() {
        let mut server = make_server();
        // Initialize first (Issue 14)
        let init_req = McpRequest::new(json!(0), "initialize", Some(json!({})));
        server.handle_request(&init_req, None).await;

        let req = McpRequest::new(
            json!(3),
            "tools/call",
            Some(json!({
                "name": "rlmx_memory_stats",
                "arguments": {}
            })),
        );
        let resp = server.handle_request(&req, None).await;
        assert!(resp.error.is_none());
    }

    #[tokio::test]
    async fn test_tools_rejected_before_initialize() {
        let mut server = make_server();
        let req = McpRequest::new(json!(1), "tools/list", None);
        let resp = server.handle_request(&req, None).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, SERVER_NOT_INITIALIZED);
    }

    #[tokio::test]
    async fn test_rbac_viewer_cannot_ingest() {
        let mut server = make_server();
        let init_req = McpRequest::new(json!(0), "initialize", Some(json!({})));
        server.handle_request(&init_req, None).await;

        let req = McpRequest::new(
            json!(5),
            "tools/call",
            Some(json!({
                "name": "rlmx_ingest",
                "arguments": {"data": "test"},
                "_role": "viewer"
            })),
        );
        let resp = server.handle_request(&req, None).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, ACCESS_DENIED);
    }

    #[tokio::test]
    async fn test_handle_method_not_found() {
        let mut server = make_server();
        let req = McpRequest::new(json!(4), "nonexistent/method", None);
        let resp = server.handle_request(&req, None).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, METHOD_NOT_FOUND);
    }

    #[tokio::test]
    async fn test_rbac_client_cannot_self_assign_admin() {
        let mut server = make_server();
        let init_req = McpRequest::new(json!(0), "initialize", Some(json!({})));
        server.handle_request(&init_req, None).await;

        let req = McpRequest::new(
            json!(6),
            "tools/call",
            Some(json!({
                "name": "rlmx_query",
                "arguments": {"query": "test"},
                "_role": "admin"
            })),
        );
        let resp = server.handle_request(&req, None).await;
        assert!(
            resp.error.is_some(),
            "Client should not be able to self-assign Admin role"
        );
        let err = resp.error.unwrap();
        assert_eq!(err.code, ACCESS_DENIED);
        assert!(err
            .message
            .contains("privileged roles must be configured server-side"));
    }

    #[tokio::test]
    async fn test_rbac_client_cannot_self_assign_system() {
        let mut server = make_server();
        let init_req = McpRequest::new(json!(0), "initialize", Some(json!({})));
        server.handle_request(&init_req, None).await;

        let req = McpRequest::new(
            json!(7),
            "tools/call",
            Some(json!({
                "name": "rlmx_query",
                "arguments": {"query": "test"},
                "_role": "system"
            })),
        );
        let resp = server.handle_request(&req, None).await;
        assert!(
            resp.error.is_some(),
            "Client should not be able to self-assign System role"
        );
        let err = resp.error.unwrap();
        assert_eq!(err.code, ACCESS_DENIED);
        assert!(err
            .message
            .contains("privileged roles must be configured server-side"));
    }

    #[tokio::test]
    async fn test_rbac_server_side_admin_role_via_token() {
        let mut token_roles = HashMap::new();
        token_roles.insert("admin-token-123".to_string(), Role::Admin);

        let config = McpConfig {
            auth_enabled: true,
            auth_token: Some("admin-token-123".to_string()),
            token_roles,
            ..Default::default()
        };
        let mut server = McpServer::new(config);
        server.register_tools(create_all_tools(new_shared_state()));

        let init_req = McpRequest::new(json!(0), "initialize", Some(json!({})));
        server.handle_request(&init_req, None).await;

        let req = McpRequest::new(
            json!(8),
            "tools/call",
            Some(json!({
                "name": "rlmx_query",
                "arguments": {"query": "test"}
            })),
        );
        // Pass the caller's token so RBAC resolves the Admin role.
        let resp = server.handle_request(&req, Some("admin-token-123")).await;
        assert!(
            resp.error.is_none(),
            "Server-side Admin token should be authorized"
        );
    }

    #[tokio::test]
    async fn test_rbac_operator_role_allowed_via_param() {
        let mut server = make_server();
        let init_req = McpRequest::new(json!(0), "initialize", Some(json!({})));
        server.handle_request(&init_req, None).await;

        let req = McpRequest::new(
            json!(9),
            "tools/call",
            Some(json!({
                "name": "rlmx_query",
                "arguments": {"query": "test"},
                "_role": "operator"
            })),
        );
        let resp = server.handle_request(&req, None).await;
        assert!(
            resp.error.is_none(),
            "Operator role should be allowed via _role param"
        );
    }

    #[test]
    fn test_config_role_for_token() {
        let mut token_roles = HashMap::new();
        token_roles.insert("token-abc".to_string(), Role::Admin);
        token_roles.insert("token-xyz".to_string(), Role::System);

        let config = McpConfig {
            token_roles,
            ..Default::default()
        };

        assert_eq!(config.role_for_token("token-abc"), Some(&Role::Admin));
        assert_eq!(config.role_for_token("token-xyz"), Some(&Role::System));
        assert_eq!(config.role_for_token("unknown-token"), None);
    }
}
