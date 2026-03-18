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

pub mod protocol;
pub mod server;
pub mod tools;
pub mod stdio;
pub mod http;

// Re-export primary types for convenience
pub use protocol::{McpRequest, McpResponse, McpError, McpTool, ToolHandler};
pub use server::{McpServer, McpConfig, Transport};
pub use tools::create_all_tools;
