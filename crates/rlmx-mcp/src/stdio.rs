//! Stdio Transport
//!
//! Reads JSON-RPC messages from stdin (line-delimited) and writes
//! JSON-RPC responses to stdout. Handles the initialize/initialized handshake.

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info};

use crate::protocol::{McpError, McpRequest, McpResponse};
use crate::server::McpServer;

/// Run the stdio transport loop.
///
/// Reads line-delimited JSON-RPC messages from stdin, dispatches them
/// to the server, and writes responses to stdout.
pub async fn run_stdio_loop(server: &mut McpServer) -> Result<(), McpError> {
    info!("Starting MCP stdio transport");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        debug!(raw_input = %line, "Received stdin message");

        // Parse the request
        let request = match McpRequest::from_json(&line) {
            Ok(req) => req,
            Err(e) => {
                let error_resp = McpResponse::error(serde_json::Value::Null, e);
                if let Ok(json) = error_resp.to_json() {
                    let _ = stdout.write_all(json.as_bytes()).await;
                    let _ = stdout.write_all(b"\n").await;
                    let _ = stdout.flush().await;
                }
                continue;
            }
        };

        // Handle the request
        let response = server.handle_request(&request).await;

        // Write the response
        match response.to_json() {
            Ok(json) => {
                debug!(response = %json, "Sending response");
                if let Err(e) = stdout.write_all(json.as_bytes()).await {
                    error!(error = %e, "Failed to write response to stdout");
                    break;
                }
                if let Err(e) = stdout.write_all(b"\n").await {
                    error!(error = %e, "Failed to write newline to stdout");
                    break;
                }
                if let Err(e) = stdout.flush().await {
                    error!(error = %e, "Failed to flush stdout");
                    break;
                }
            }
            Err(e) => {
                error!(error = ?e, "Failed to serialize response");
            }
        }
    }

    info!("Stdio transport loop ended");
    Ok(())
}
