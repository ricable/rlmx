//! HTTP Transport
//!
//! Simple HTTP server using tokio TcpListener that accepts JSON-RPC
//! requests on the POST /mcp endpoint. Includes basic request parsing,
//! response formatting, authentication, and concurrent connection handling.

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info, warn};

use crate::protocol::{McpError, McpRequest, McpResponse};
use crate::server::McpServer;

/// Maximum HTTP body size (10 MB) to prevent OOM from untrusted Content-Length.
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Maximum number of concurrent HTTP connections to prevent resource exhaustion.
const MAX_CONCURRENT_CONNECTIONS: usize = 256;

/// Run the HTTP transport server.
///
/// Takes ownership of the `McpServer` and wraps it in `Arc<RwLock<McpServer>>`
/// so it can be safely shared across spawned connection tasks.
///
/// Binds to the given host and port, accepting JSON-RPC requests via
/// POST /mcp and returning JSON-RPC responses. Each connection is
/// handled concurrently via `tokio::spawn`.
pub async fn run_http_server(
    server: McpServer,
    host: &str,
    port: u16,
) -> Result<(), McpError> {
    let addr = format!("{}:{}", host, port);
    info!(address = %addr, "Starting MCP HTTP transport");

    let listener = TcpListener::bind(&addr).await.map_err(|e| {
        McpError::internal(format!("Failed to bind to {}: {}", addr, e))
    })?;

    info!(address = %addr, "MCP HTTP server listening");

    let auth_enabled = server.auth_enabled();
    // Pre-hash the expected token once at startup so we only hash
    // the incoming token per-request (not both).
    let auth_token_hash = server
        .auth_token()
        .map(|s| rlmx_rvf::hash_sha256(s.as_bytes()));

    // Wrap the owned server in Arc<Mutex> for safe sharing across tasks.
    // Mutex suffices because handle_request takes &mut self, so every
    // request needs exclusive access anyway.
    let shared_server: Arc<Mutex<McpServer>> = Arc::new(Mutex::new(server));

    // Limit concurrent connections to prevent resource exhaustion (DoS).
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS));

    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                error!(error = %e, "Failed to accept connection");
                continue;
            }
        };

        debug!(peer = %peer_addr, "Accepted connection");

        // Acquire permit before spawning to bound task count under load.
        let permit = match semaphore.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                warn!(peer = %peer_addr, "Connection rejected: max concurrent connections reached");
                drop(stream);
                continue;
            }
        };

        let server_handle = Arc::clone(&shared_server);
        let token_hash = auth_token_hash.clone();

        tokio::spawn(async move {
            let _permit = permit; // released when task completes
            if let Err(e) = handle_connection(
                stream,
                server_handle,
                auth_enabled,
                token_hash.as_deref(),
            )
            .await
            {
                error!(peer = %peer_addr, error = ?e, "Connection handler error");
            }
        });
    }
}

/// Handle a single HTTP connection.
async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    server: Arc<Mutex<McpServer>>,
    auth_enabled: bool,
    auth_token: Option<&str>,
) -> Result<(), McpError> {
    // Read the full HTTP request using Content-Length to know when the
    // body is complete.
    let raw = match read_full_http_request(&mut stream).await {
        Ok(Some(data)) => data,
        Ok(None) => return Ok(()), // connection closed
        Err(e) => {
            let response = build_http_response(400, "Bad Request");
            let _ = stream.write_all(response.as_bytes()).await;
            return Err(e);
        }
    };

    // Parse HTTP request (minimal parser)
    let (method, path, headers, body) = match parse_http_request(&raw) {
        Some(parsed) => parsed,
        None => {
            let response = build_http_response(400, "Bad Request");
            let _ = stream.write_all(response.as_bytes()).await;
            return Ok(());
        }
    };

    // Extract the caller's bearer token (if present).
    let caller_bearer = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
        .and_then(|(_, v)| v.strip_prefix("Bearer "))
        .map(|t| t.to_string());

    // Authentication check: compare SHA-256 hash of incoming token against
    // the pre-hashed expected token to avoid timing side-channels.
    if auth_enabled {
        if let Some(expected_hash) = auth_token {
            let authorized = caller_bearer
                .as_deref()
                .map(|t| rlmx_rvf::hash_sha256(t.as_bytes()) == expected_hash)
                .unwrap_or(false);

            if !authorized {
                warn!("Rejected request: invalid or missing Bearer token");
                let response = build_http_response(401, "Unauthorized");
                let _ = stream.write_all(response.as_bytes()).await;
                return Ok(());
            }
        }
    }

    // Only accept POST /mcp
    if method != "POST" || (path != "/mcp" && path != "/mcp/") {
        let response = if method == "GET" && path == "/mcp/sse" {
            // SSE placeholder
            build_http_response(501, r#"{"error": "SSE not yet implemented"}"#)
        } else {
            build_http_response(404, "Not Found")
        };
        let _ = stream.write_all(response.as_bytes()).await;
        return Ok(());
    }

    // Parse the JSON-RPC request from the body
    let mcp_request = match McpRequest::from_json(&body) {
        Ok(req) => req,
        Err(e) => {
            let error_resp = McpResponse::error(serde_json::Value::Null, e);
            let json = error_resp.to_json().unwrap_or_else(|_| {
                r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}"#
                    .to_string()
            });
            let response = build_http_json_response(200, &json);
            let _ = stream.write_all(response.as_bytes()).await;
            return Ok(());
        }
    };

    // Handle the request — acquire mutex since handle_request takes &mut self.
    // Pass the caller's bearer token so RBAC can resolve per-caller roles.
    let mcp_response = {
        let mut srv = server.lock().await;
        srv.handle_request(&mcp_request, caller_bearer.as_deref()).await
    };

    let json = mcp_response.to_json().unwrap_or_else(|_| {
        r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Internal error"}}"#
            .to_string()
    });

    let response = build_http_json_response(200, &json);
    let _ = stream.write_all(response.as_bytes()).await;
    Ok(())
}

/// Read a full HTTP request from the stream, using Content-Length to
/// determine how many body bytes to read. Reads headers first, then
/// reads the remaining body bytes in a loop.
async fn read_full_http_request(
    stream: &mut tokio::net::TcpStream,
) -> Result<Option<String>, McpError> {
    let mut buf = Vec::with_capacity(8192);
    let mut tmp = [0u8; 4096];

    // Read until we have the full header section (terminated by \r\n\r\n).
    let header_end;
    loop {
        let n = stream.read(&mut tmp).await.map_err(|e| {
            McpError::internal(format!("Failed to read from connection: {}", e))
        })?;
        if n == 0 {
            if buf.is_empty() {
                return Ok(None);
            }
            // Connection closed before headers complete — return what we have.
            return Ok(Some(String::from_utf8_lossy(&buf).into_owned()));
        }
        buf.extend_from_slice(&tmp[..n]);

        // Check for header terminator.
        if let Some(pos) = find_header_end(&buf) {
            header_end = pos;
            break;
        }

        // Guard against absurdly large headers.
        if buf.len() > 65536 {
            return Err(McpError::internal("HTTP headers too large"));
        }
    }

    // Parse Content-Length from the headers.
    let header_section = String::from_utf8_lossy(&buf[..header_end]);
    let content_length = parse_content_length(&header_section).unwrap_or(0);

    // Reject bodies that exceed the maximum allowed size to prevent OOM.
    if content_length > MAX_BODY_SIZE {
        return Err(McpError::internal(format!(
            "Content-Length {} exceeds maximum allowed body size of {} bytes",
            content_length, MAX_BODY_SIZE
        )));
    }

    // The body starts after the header terminator (4 bytes for \r\n\r\n).
    let body_start = header_end.checked_add(4)
        .ok_or_else(|| McpError::internal("header offset overflow"))?;
    let total_needed = body_start.checked_add(content_length)
        .ok_or_else(|| McpError::internal("body size overflow"))?;

    // Read remaining body bytes if we don't have them yet.
    while buf.len() < total_needed {
        let n = stream.read(&mut tmp).await.map_err(|e| {
            McpError::internal(format!("Failed to read body: {}", e))
        })?;
        if n == 0 {
            break; // Connection closed — use what we have.
        }
        buf.extend_from_slice(&tmp[..n]);
    }

    Ok(Some(String::from_utf8_lossy(&buf[..buf.len().min(total_needed)]).into_owned()))
}

/// Find the position of the header/body separator (\r\n\r\n).
/// Returns the index of the first \r in the separator.
fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4)
        .position(|w| w == b"\r\n\r\n")
}

/// Parse the Content-Length header value from the raw header section.
fn parse_content_length(headers: &str) -> Option<usize> {
    for line in headers.lines() {
        let lower = line.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("content-length:") {
            return rest.trim().parse().ok();
        }
    }
    None
}

/// Minimal HTTP request parser.
///
/// Returns (method, path, headers, body) or None if the request is malformed.
fn parse_http_request(raw: &str) -> Option<(String, String, Vec<(String, String)>, String)> {
    let mut lines = raw.lines();

    // Parse request line
    let request_line = lines.next()?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let method = parts[0].to_string();
    let path = parts[1].to_string();

    // Parse headers
    let mut headers = Vec::new();
    for line in lines.by_ref() {
        if line.is_empty() || line == "\r" {
            break;
        }
        let line = line.trim_end_matches('\r');
        if let Some((key, value)) = line.split_once(':') {
            headers.push((key.trim().to_string(), value.trim().to_string()));
        }
    }

    // Find the body (after the empty line)
    let body = if let Some(pos) = raw.find("\r\n\r\n") {
        raw[pos + 4..].to_string()
    } else if let Some(pos) = raw.find("\n\n") {
        raw[pos + 2..].to_string()
    } else {
        String::new()
    };

    Some((method, path, headers, body))
}

/// Build a simple HTTP response with the given status code and body.
fn build_http_response(status: u16, body: &str) -> String {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
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
        let (method, path, headers, body) = parse_http_request(raw).unwrap();
        assert_eq!(method, "POST");
        assert_eq!(path, "/mcp");
        assert!(body.contains("initialize"));
        assert!(headers.iter().any(|(k, _)| k == "Host"));
    }

    #[test]
    fn test_parse_content_length() {
        let headers = "POST /mcp HTTP/1.1\r\nContent-Length: 42\r\nHost: localhost";
        assert_eq!(parse_content_length(headers), Some(42));
    }

    #[test]
    fn test_parse_content_length_case_insensitive() {
        let headers = "POST /mcp HTTP/1.1\r\ncontent-length: 100\r\nHost: localhost";
        assert_eq!(parse_content_length(headers), Some(100));
    }

    #[test]
    fn test_find_header_end() {
        let buf = b"GET / HTTP/1.1\r\nHost: x\r\n\r\nbody";
        assert_eq!(find_header_end(buf), Some(23));
    }

    #[test]
    fn test_auth_header_parsing() {
        let raw = "POST /mcp HTTP/1.1\r\nAuthorization: Bearer secret123\r\nContent-Type: application/json\r\n\r\n{}";
        let (_, _, headers, _) = parse_http_request(raw).unwrap();
        let auth = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
            .and_then(|(_, v)| v.strip_prefix("Bearer "));
        assert_eq!(auth, Some("secret123"));
    }
}
