//! rlmx-mcp: MCP (Model Context Protocol) server
//!
//! This crate implements the MCP server that exposes the RLMX kernel's
//! capabilities over the Model Context Protocol, enabling integration with
//! AI assistants and external tools.
//!
//! ## Architecture
//!
//! - **protocol**: JSON-RPC 2.0 types for MCP communication
//! - **server**: Core MCP server with tool registration and request dispatch
//! - **tools**: All 12 RLMX MCP tool definitions and handlers
//! - **stdio**: Stdio transport (line-delimited JSON-RPC via stdin/stdout)
//! - **http**: HTTP transport (POST /mcp endpoint via TCP)
//! - **ws**: WebSocket transport for real-time SwarmEvent streaming

pub mod http;
pub mod protocol;
pub mod server;
pub mod stdio;
pub mod tools;
pub mod ws;

// Re-export primary types for convenience
pub use protocol::{McpError, McpRequest, McpResponse, McpTool, ToolHandler};
pub use server::{McpConfig, McpServer, Transport};
pub use tools::{create_all_tools, new_shared_state, SharedToolState, ToolState};
pub use ws::{SwarmEvent, SwarmEventBus, WsServer};
