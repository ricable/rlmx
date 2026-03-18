//! HTTP Transport
//!
//! Simple HTTP server using tokio TcpListener that accepts JSON-RPC
//! requests on the POST /mcp endpoint. Includes basic request parsing,
//! response formatting, and an SSE support placeholder for streamable-http.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{debug, error, info};

use crate::protocol::{McpError, McpRequest, McpResponse};
use crate::server::McpServer;

/// Run the HTTP transport server.
///
/// Binds to the given host and port, accepting JSON-RPC requests via
/// POST /mcp and returning JSON-RPC responses.
pub async fn run_http_server(
    server: &mut McpServer,
    host: &str,
    port: u16,
) -> Result<(), McpError> {
    let addr = format!("{}:{}", host, port);
    info!(address = %addr, "Starting MCP HTTP transport");

    let listener = TcpListener::bind(&addr).await.map_err(|e| {
        McpError::internal(format!("Failed to bind to {}: {}", addr, e))
    })?;

    info!(address = %addr, "MCP HTTP server listening");

    loop {
        let (mut stream, peer_addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                error!(error = %e, "Failed to accept connection");
                continue;
            }
        };

        debug!(peer = %peer_addr, "Accepted connection");

        // Read the full HTTP request
        let mut buf = vec![0u8; 65536];
        let n = match stream.read(&mut buf).await {
            Ok(n) if n == 0 => continue,
            Ok(n) => n,
            Err(e) => {
                error!(error = %e, "Failed to read from connection");
                continue;
            }
        };

        let raw = String::from_utf8_lossy(&buf[..n]);

        // Parse HTTP request (minimal parser)
        let (method, path, body) = match parse_http_request(&raw) {
            Some(parsed) => parsed,
            None => {
                let response = build_http_response(400, "Bad Request");
                let _ = stream.write_all(response.as_bytes()).await;
                continue;
            }
        };

        // Only accept POST /mcp
        if method != "POST" || (path != "/mcp" && path != "/mcp/") {
            let response = if method == "GET" && path == "/mcp/sse" {
                // SSE placeholder
                build_http_response(501, r#"{"error": "SSE not yet implemented"}"#)
            } else {
                build_http_response(404, "Not Found")
            };
            let _ = stream.write_all(response.as_bytes()).await;
            continue;
        }

        // Parse the JSON-RPC request from the body
        let mcp_request = match McpRequest::from_json(&body) {
            Ok(req) => req,
            Err(e) => {
                let error_resp = McpResponse::error(serde_json::Value::Null, e);
                let json = error_resp.to_json().unwrap_or_else(|_| {
                    r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}"#.to_string()
                });
                let response = build_http_json_response(200, &json);
                let _ = stream.write_all(response.as_bytes()).await;
                continue;
            }
        };

        // Handle the request
        let mcp_response = server.handle_request(&mcp_request).await;

        let json = mcp_response.to_json().unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Internal error"}}"#.to_string()
        });

        let response = build_http_json_response(200, &json);
        let _ = stream.write_all(response.as_bytes()).await;
    }
}

/// Minimal HTTP request parser.
///
/// Returns (method, path, body) or None if the request is malformed.
fn parse_http_request(raw: &str) -> Option<(String, String, String)> {
    let mut lines = raw.lines();

    // Parse request line
    let request_line = lines.next()?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let method = parts[0].to_string();
    let path = parts[1].to_string();

    // Find the body (after the empty line)
    let body = if let Some(pos) = raw.find("\r\n\r\n") {
        raw[pos + 4..].to_string()
    } else if let Some(pos) = raw.find("\n\n") {
        raw[pos + 2..].to_string()
    } else {
        String::new()
    };

    Some((method, path, body))
}

/// Build a simple HTTP response with the given status code and body.
fn build_http_response(status: u16, body: &str) -> String {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        501 => "Not Implemented",
        _ => "Unknown",
    };

    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        status_text,
        body.len(),
        body
    )
}

/// Build an HTTP response with JSON content type.
fn build_http_json_response(status: u16, json_body: &str) -> String {
    let status_text = match status {
        200 => "OK",
        _ => "Unknown",
    };

    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        status_text,
        json_body.len(),
        json_body
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_request() {
        let raw = "POST /mcp HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\"}";
        let (method, path, body) = parse_http_request(raw).unwrap();
        assert_eq!(method, "POST");
        assert_eq!(path, "/mcp");
        assert!(body.contains("initialize"));
    }
}
