//! Request handlers for the TAP HTTP server.
//!
//! This module provides HTTP request handlers for the Transaction Authorization Protocol (TAP)
//! server, including endpoints for DIDComm message processing and health checks.
//!
//! The handlers leverage the TAP Node for message processing, which handles message validation,
//! verification, and routing through the appropriate agent.

use crate::error::{Error, Result};
use crate::event::EventBus;
use bytes::Bytes;
use serde::Serialize;
use serde_json::json;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Instant;
use tap_agent::Agent;
use tap_node::TapNode;
use tracing::{debug, error, info, warn};
use warp::{self, hyper::StatusCode, reply::json, Reply};

/// Response structure for health checks.
#[derive(Serialize)]
struct HealthResponse {
    /// Status of the server, always "ok" when reachable
    status: String,
    /// Current version of the tap-http package
    version: String,
}

/// Handler for health check requests.
///
/// Returns a simple response with the status "ok" and the current version number.
/// This endpoint allows monitoring systems to verify that the TAP HTTP server is operational.
pub async fn handle_health_check(
    event_bus: Arc<EventBus>,
) -> std::result::Result<impl Reply, Infallible> {
    info!("Health check request received");

    // Start timing the request
    let start_time = Instant::now();

    // Log request received event
    event_bus
        .publish_request_received(
            "GET".to_string(),
            "/health".to_string(),
            None, // We don't have client IP in this simplified example
        )
        .await;

    // Build response
    let response = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    // Convert response to JSON
    let json_response = json(&response);

    // Calculate response size (approximate)
    let response_size = serde_json::to_string(&response)
        .map(|s| s.len())
        .unwrap_or(0);

    // Calculate request duration
    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Log response sent event
    event_bus
        .publish_response_sent(StatusCode::OK, response_size, duration_ms)
        .await;

    Ok(json_response)
}

/// Handler for DIDComm messages.
///
/// This function processes incoming DIDComm messages by:
/// 1. Converting the raw bytes to a UTF-8 string
/// 2. Parsing the string as a DIDComm message
/// 3. Forwarding the message to the TAP Node for further processing
///
/// The handler returns appropriate success or error responses based on the outcome.
pub async fn handle_didcomm(
    body: Bytes,
    node: Arc<TapNode>,
    event_bus: Arc<EventBus>,
) -> std::result::Result<impl Reply, Infallible> {
    // Start timing the request
    let start_time = Instant::now();

    // Log request received event
    event_bus
        .publish_request_received(
            "POST".to_string(),
            "/didcomm".to_string(),
            None, // Client IP not available in this context
        )
        .await;

    // Convert bytes to string
    let message_str = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to parse request body as UTF-8: {}", e);

            // Log message error event
            event_bus
                .publish_message_error(
                    "parse_error".to_string(),
                    format!("Failed to parse request body as UTF-8: {}", e),
                    None,
                )
                .await;

            // Calculate response size and duration
            let response =
                json_error_response(StatusCode::BAD_REQUEST, "Invalid UTF-8 in request body");
            let response_size = 200; // Approximate size
            let duration_ms = start_time.elapsed().as_millis() as u64;

            // Log response sent event
            event_bus
                .publish_response_sent(StatusCode::BAD_REQUEST, response_size, duration_ms)
                .await;

            return Ok(response);
        }
    };

    // The message could be either encrypted/signed or plain
    // Pass the raw message string to the TAP Node for processing
    debug!("Processing DIDComm message (encrypted or plain)");

    // Process the raw message using the TAP Node
    match process_tap_message_raw(message_str, node, event_bus.clone()).await {
        Ok(_) => {
            info!("DIDComm message processed successfully");

            // Calculate response size and duration
            let response = json_success_response();
            let response_size = 100; // Approximate size
            let duration_ms = start_time.elapsed().as_millis() as u64;

            // Log response sent event
            event_bus
                .publish_response_sent(StatusCode::OK, response_size, duration_ms)
                .await;

            Ok(response)
        }
        Err(e) => {
            // Log error with appropriate severity
            match e.severity() {
                crate::error::ErrorSeverity::Info => debug!("Message processing error: {}", e),
                crate::error::ErrorSeverity::Warning => warn!("Message processing warning: {}", e),
                crate::error::ErrorSeverity::Critical => {
                    error!("Critical error processing message: {}", e)
                }
            }

            // Log message error event
            let error_type = match &e {
                Error::DIDComm(_) => "didcomm_error",
                Error::Validation(_) => "validation_error",
                Error::Authentication(_) => "authentication_error",
                Error::Json(_) => "json_error",
                Error::Http(_) => "http_error",
                Error::Node(_) => "node_error",
                Error::Config(_) => "configuration_error",
                Error::Io(_) => "io_error",
                Error::RateLimit(_) => "rate_limit_error",
                Error::Tls(_) => "tls_error",
                Error::Unknown(_) => "unknown_error",
            };

            event_bus
                .publish_message_error(
                    error_type.to_string(),
                    e.to_string(),
                    None, // We don't have message ID in this context
                )
                .await;

            // Create error response
            let response = e.to_response();

            // Calculate response size and duration (approximate)
            let response_size = 200; // Approximate size
            let duration_ms = start_time.elapsed().as_millis() as u64;

            // Log response sent event
            event_bus
                .publish_response_sent(e.status_code(), response_size, duration_ms)
                .await;

            // Return the structured error response
            Ok(response)
        }
    }
}

/// Process a raw TAP message using the TAP Node.
///
/// This function handles both encrypted/signed and plain DIDComm messages.
/// For encrypted messages, it will be forwarded to the appropriate agent for decryption.
///
/// # Parameters
/// * `message_str` - The raw message string (could be encrypted or plain)
/// * `node` - The TAP Node instance that will process the message
/// * `event_bus` - The event bus for logging
///
/// # Returns
/// * `Ok(())` if processing succeeded
/// * `Err(Error)` if validation fails or message processing failed
async fn process_tap_message_raw(
    message_str: &str,
    node: Arc<TapNode>,
    event_bus: Arc<EventBus>,
) -> Result<()> {
    // Try to get the first agent to process the raw message
    // In the future, we might want to be smarter about which agent to use
    let agent_dids = node.agents().get_all_dids();
    if agent_dids.is_empty() {
        return Err(Error::Node("No agents registered".to_string()));
    }

    // Try each agent until one can process the message
    for did in &agent_dids {
        match node.agents().get_agent(did).await {
            Ok(agent) => {
                // Use the new receive_raw_message method
                match agent.receive_raw_message(message_str).await {
                    Ok(plain_message) => {
                        // Successfully unpacked the message, now process it through the node
                        debug!("Successfully unpacked message with agent {}", did);

                        // Log the plain message
                        event_bus
                            .publish_message_received(
                                plain_message.id.clone(),
                                plain_message.type_.clone(),
                                Some(plain_message.from.clone()),
                                Some(plain_message.to.join(", ")),
                            )
                            .await;

                        // Process the plain message through the node
                        return node
                            .receive_message(plain_message)
                            .await
                            .map_err(|e| Error::Node(e.to_string()));
                    }
                    Err(e) => {
                        debug!("Agent {} couldn't process message: {}", did, e);
                        continue;
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get agent {}: {}", did, e);
                continue;
            }
        }
    }

    Err(Error::Node(
        "No agent could process the message".to_string(),
    ))
}

/// Create a JSON success response.
///
/// Returns a standardized success response with a 200 status code.
fn json_success_response() -> warp::reply::Response {
    warp::reply::with_status(
        json(&json!({
            "status": "success",
            "message": "Message received and processed"
        })),
        StatusCode::OK,
    )
    .into_response()
}

/// Create a JSON error response.
///
/// Returns a standardized error response with the specified status code and error message.
///
/// # Parameters
/// * `status` - The HTTP status code to return
/// * `message` - The error message to include in the response
fn json_error_response(status: StatusCode, message: &str) -> warp::reply::Response {
    warp::reply::with_status(
        json(&json!({
            "status": "error",
            "message": message
        })),
        status,
    )
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tap_node::NodeConfig;
    use warp::hyper::body::to_bytes;

    #[tokio::test]
    async fn test_health_check() {
        // Create a dummy event bus
        let event_bus = Arc::new(crate::event::EventBus::new());

        // Call the health check handler
        let response = handle_health_check(event_bus).await.unwrap();

        // Convert the response to bytes and parse as JSON
        let response_bytes = to_bytes(response.into_response().into_body())
            .await
            .unwrap();
        let response_json: Value = serde_json::from_slice(&response_bytes).unwrap();

        // Validate the response
        assert_eq!(response_json["status"], "ok");
        assert!(response_json["version"].is_string());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_invalid_didcomm() {
        // Create a TAP Node for testing without storage
        let mut config = NodeConfig::default();
        config.storage_path = None; // Disable storage for tests
        let node = Arc::new(TapNode::new(config));

        // Create a dummy event bus
        let event_bus = Arc::new(crate::event::EventBus::new());

        // Test with invalid UTF-8 data
        let invalid_bytes = Bytes::from(vec![0xFF, 0xFF]);
        let response = handle_didcomm(invalid_bytes, node.clone(), event_bus)
            .await
            .unwrap();

        // Convert the response to bytes and parse as JSON
        let response_bytes = to_bytes(response.into_response().into_body())
            .await
            .unwrap();
        let response_json: Value = serde_json::from_slice(&response_bytes).unwrap();

        // Validate the error response
        assert_eq!(response_json["status"], "error");
        assert!(response_json["message"]
            .as_str()
            .unwrap()
            .contains("Invalid UTF-8"));
    }

    #[tokio::test]
    async fn test_json_error_response() {
        // Generate an error response
        let response = json_error_response(StatusCode::BAD_REQUEST, "Test error message");

        // Convert the response to bytes and parse as JSON
        let response_bytes = to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&response_bytes).unwrap();

        // Validate the response
        assert_eq!(response_json["status"], "error");
        assert_eq!(response_json["message"], "Test error message");
    }

    #[tokio::test]
    async fn test_json_success_response() {
        // Generate a success response
        let response = json_success_response();

        // Convert the response to bytes and parse as JSON
        let response_bytes = to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&response_bytes).unwrap();

        // Validate the response
        assert_eq!(response_json["status"], "success");
        assert_eq!(response_json["message"], "Message received and processed");
    }
}
