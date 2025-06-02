//! MCP transport implementations

use crate::error::{Error, Result};
use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, trace};

/// Stdio transport for MCP communication
pub struct StdioTransport {
    stdin: BufReader<tokio::io::Stdin>,
    stdout: tokio::io::Stdout,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new() -> Self {
        Self {
            stdin: BufReader::new(tokio::io::stdin()),
            stdout: tokio::io::stdout(),
        }
    }

    /// Read a JSON-RPC request from stdin
    pub async fn read_request(&mut self) -> Result<Option<JsonRpcRequest>> {
        loop {
            let mut line = String::new();

            match self.stdin.read_line(&mut line).await {
                Ok(0) => {
                    debug!("EOF reached on stdin");
                    return Ok(None);
                }
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to read from stdin: {}", e);
                    return Err(Error::Io(e));
                }
            }

            // Remove trailing newline
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }

            if line.trim().is_empty() {
                trace!("Received empty line, skipping");
                continue;
            }

            trace!("Received line: {}", line);

            // Parse JSON-RPC request
            match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(request) => {
                    debug!(
                        "Parsed request: method={}, id={:?}",
                        request.method, request.id
                    );
                    return Ok(Some(request));
                }
                Err(e) => {
                    error!("Failed to parse JSON-RPC request: {}", e);
                    return Err(Error::Json(e));
                }
            }
        }
    }

    /// Write a JSON-RPC response to stdout
    pub async fn write_response(&mut self, response: JsonRpcResponse) -> Result<()> {
        let json = serde_json::to_string(&response)?;
        trace!("Sending response: {}", json);

        self.stdout.write_all(json.as_bytes()).await?;
        self.stdout.write_all(b"\n").await?;
        self.stdout.flush().await?;

        debug!("Sent response for id={:?}", response.id);
        Ok(())
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}
