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
use chrono;
use serde::Serialize;
use serde_json::json;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Instant;
use tap_msg::didcomm::PlainMessage;
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

    // Parse the DIDComm message
    debug!("Parsing DIDComm message");
    let didcomm_message: PlainMessage = match serde_json::from_str(message_str) {
        Ok(msg) => msg,
        Err(e) => {
            error!("Failed to parse DIDComm message: {}", e);

            // Log message error event
            event_bus
                .publish_message_error(
                    "parse_error".to_string(),
                    format!("Failed to parse DIDComm message: {}", e),
                    None,
                )
                .await;

            // Calculate response size and duration
            let response =
                json_error_response(StatusCode::BAD_REQUEST, "Invalid DIDComm message format");
            let response_size = 200; // Approximate size
            let duration_ms = start_time.elapsed().as_millis() as u64;

            // Log response sent event
            event_bus
                .publish_response_sent(StatusCode::BAD_REQUEST, response_size, duration_ms)
                .await;

            return Ok(response);
        }
    };

    // Log message received event
    event_bus
        .publish_message_received(
            didcomm_message.id.clone(),
            didcomm_message.typ.clone(),
            Some(didcomm_message.from.clone()),
            if didcomm_message.to.is_empty() {
                None
            } else {
                Some(didcomm_message.to.join(", "))
            },
        )
        .await;

    // Process the message using the TAP Node
    match process_tap_message(didcomm_message, node).await {
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

/// Process a TAP message using the TAP Node.
///
/// This function performs pre-processing validation on DIDComm messages before
/// forwarding them to the TAP Node for processing. The validation includes:
///
/// 1. Checking for required fields (id, type)
/// 2. Validating the message type conforms to the TAP protocol
///    - Must contain "tap.rsvp" or "https://tap.rsvp" in the type
///
/// After validation, the TapNode.receive_message method will:
/// 1. Further validate and verify the message
/// 2. Route the message to the appropriate agent
/// 3. Process the message according to the TAP protocol
///
/// # Parameters
/// * `message` - The DIDComm message to process
/// * `node` - The TAP Node instance that will process the message
///
/// # Returns
/// * `Ok(())` if processing succeeded
/// * `Err(Error)` if validation fails or message processing failed
///
/// # Error Conditions
/// The function will return an error if:
/// * The message is missing required fields
/// * The message type is not a valid TAP protocol message
/// * The TAP Node encounters an error during processing
async fn process_tap_message(message: PlainMessage, node: Arc<TapNode>) -> Result<()> {
    // Basic validation for the message
    // Ensure message has required fields
    if message.typ.is_empty() || message.id.is_empty() {
        return Err(Error::Validation(
            "Missing required message fields: type or id".to_string(),
        ));
    }

    // Check for missing from/to fields if required
    if message.from.is_empty() || message.to.is_empty() {
        return Err(Error::Validation(
            "Message missing sender or recipient information".to_string(),
        ));
    }

    // Log the complete message for debugging (always log this for now)
    error!("RECEIVED DIDCOMM MESSAGE BEFORE PROCESSING:\n- id: {}\n- typ: {}\n- type_: {}\n- from: {}\n- to: {:?}\n- body: {}\n", 
           message.id, message.typ, message.type_, message.from, message.to,
           serde_json::to_string_pretty(&message.body).unwrap_or_default());

    // Validate the message conforms to TAP protocol
    // Check both type_ and typ fields for TAP protocol identifiers
    let valid_types = ["tap.rsvp", "https://tap.rsvp"];

    // Check if type_ field contains TAP protocol identifier
    let is_valid_type = valid_types
        .iter()
        .any(|valid_type| message.type_.contains(valid_type));

    if !is_valid_type {
        error!(
            "TYPE FIELD VALIDATION FAILED: message.type_ = '{}'",
            message.type_
        );
    }

    // Also check if typ field contains TAP protocol identifier
    let is_valid_typ = valid_types
        .iter()
        .any(|valid_type| message.typ.contains(valid_type));

    if !is_valid_typ {
        error!(
            "TYP FIELD VALIDATION FAILED: message.typ = '{}'",
            message.typ
        );
    }

    // If either field is valid, allow the message through
    let is_valid = is_valid_type || is_valid_typ;

    if !is_valid {
        error!(
            "MESSAGE TYPE VALIDATION FAILED: typ={}, type_={}",
            message.typ, message.type_
        );
        return Err(Error::Validation(format!(
            "Unsupported message type: {}, expected TAP protocol message",
            message.typ
        )));
    }

    // Validate that message has valid timestamps
    if let Some(created_time) = message.created_time {
        let current_time = chrono::Utc::now().timestamp() as u64;

        // Check for future timestamps (within a small margin)
        if created_time > current_time + 300 {
            // 5 minute margin
            return Err(Error::Validation(format!(
                "Message has future timestamp: {} (current time: {})",
                created_time, current_time
            )));
        }
    }

    // Process the message through the TAP Node using the receive_message method
    match node.receive_message(message).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let error_str = e.to_string();

            // Categorize TapNode errors
            if error_str.contains("authentication") || error_str.contains("unauthorized") {
                Err(Error::Authentication(format!(
                    "Authentication failed: {}",
                    error_str
                )))
            } else if error_str.contains("validation") || error_str.contains("invalid") {
                Err(Error::Validation(format!(
                    "Node validation failed: {}",
                    error_str
                )))
            } else {
                Err(Error::Node(error_str))
            }
        }
    }
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

    #[tokio::test]
    async fn test_handle_invalid_didcomm() {
        // Create a TAP Node for testing
        let node = Arc::new(TapNode::new(NodeConfig::default()));

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
