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
use tap_node::TapNode;
use tracing::{debug, error, info};
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
/// 1. Validating the Content-Type header
/// 2. Converting the raw bytes to a UTF-8 string
/// 3. Parsing the string as a DIDComm message
/// 4. Forwarding the message to the TAP Node for further processing
///
/// The handler returns appropriate success or error responses based on the outcome.
pub async fn handle_didcomm(
    content_type: Option<String>,
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

    // Validate content type
    if let Err(e) = validate_message_security(content_type.as_deref()) {
        error!("Content-Type validation failed: {}", e);

        let response = e.to_response();
        let duration_ms = start_time.elapsed().as_millis() as u64;

        event_bus
            .publish_response_sent(e.status_code(), 200, duration_ms)
            .await;

        return Ok(response);
    }

    // Parse to JSON
    let message_str = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to parse request body as UTF-8: {}", e);

            let response =
                json_error_response(StatusCode::BAD_REQUEST, "Invalid UTF-8 in request body");
            let duration_ms = start_time.elapsed().as_millis() as u64;

            event_bus
                .publish_response_sent(StatusCode::BAD_REQUEST, 200, duration_ms)
                .await;

            return Ok(response);
        }
    };

    let message_value: serde_json::Value = match serde_json::from_str(message_str) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to parse message as JSON: {}", e);

            let response =
                json_error_response(StatusCode::BAD_REQUEST, "Invalid JSON in request body");
            let duration_ms = start_time.elapsed().as_millis() as u64;

            event_bus
                .publish_response_sent(StatusCode::BAD_REQUEST, 200, duration_ms)
                .await;

            return Ok(response);
        }
    };

    // Let the node handle routing
    match node.receive_message(message_value).await {
        Ok(_) => {
            info!("DIDComm message processed successfully");

            // Calculate response size and duration
            let response = json_success_response();
            let response_size = 100; // Approximate size
            let duration_ms = start_time.elapsed().as_millis() as u64;

            // Log response sent event
            event_bus
                .publish_response_sent(StatusCode::ACCEPTED, response_size, duration_ms)
                .await;

            Ok(response)
        }
        Err(e) => {
            error!("Failed to process message: {}", e);

            // Log message error event
            event_bus
                .publish_message_error(
                    "node_error".to_string(),
                    e.to_string(),
                    None, // We don't have message ID in this context
                )
                .await;

            // Create error response
            let response = json_error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());

            // Calculate response size and duration (approximate)
            let response_size = 200; // Approximate size
            let duration_ms = start_time.elapsed().as_millis() as u64;

            // Log response sent event
            event_bus
                .publish_response_sent(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    response_size,
                    duration_ms,
                )
                .await;

            // Return the error response
            Ok(response)
        }
    }
}

/// Validate that a message has the correct Content-Type for signed or encrypted messages.
///
/// This function ensures that only secure messages are processed by the TAP HTTP server.
/// Plain messages are rejected for security reasons.
///
/// # Parameters
/// * `content_type` - The Content-Type header value
///
/// # Returns
/// * `Ok(())` if the message has a valid signed or encrypted content type
/// * `Err(Error)` if the message is plain or has an invalid content type
fn validate_message_security(content_type: Option<&str>) -> Result<()> {
    match content_type {
        Some(ct) => {
            // Parse the content type to handle parameters like charset
            let ct_lower = ct.to_lowercase();

            // Check for valid DIDComm content types
            if ct_lower.contains("application/didcomm-signed+json") {
                debug!("Message security validation passed: signed message");
                Ok(())
            } else if ct_lower.contains("application/didcomm-encrypted+json") {
                debug!("Message security validation passed: encrypted message");
                Ok(())
            } else if ct_lower.contains("application/didcomm-plain+json") {
                Err(Error::Validation(
                    "Plain DIDComm messages are not allowed for security reasons. Only signed or encrypted messages are accepted.".to_string()
                ))
            } else {
                Err(Error::Validation(
                    format!("Invalid Content-Type '{}'. Expected 'application/didcomm-signed+json' or 'application/didcomm-encrypted+json'.", ct)
                ))
            }
        }
        None => {
            Err(Error::Validation(
                "Missing Content-Type header. Expected 'application/didcomm-signed+json' or 'application/didcomm-encrypted+json'.".to_string()
            ))
        }
    }
}

/// Create a JSON success response.
///
/// Returns a standardized success response with a 202 Accepted status code.
fn json_success_response() -> warp::reply::Response {
    warp::reply::with_status(
        json(&json!({
            "status": "success",
            "message": "Message received and processed"
        })),
        StatusCode::ACCEPTED,
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

    #[test]
    fn test_validate_message_security() {
        // Test plain message content type (should be rejected)
        let result = validate_message_security(Some("application/didcomm-plain+json"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Plain DIDComm messages are not allowed"));

        // Test signed message content type (should pass)
        let result = validate_message_security(Some("application/didcomm-signed+json"));
        assert!(result.is_ok());

        // Test encrypted message content type (should pass)
        let result = validate_message_security(Some("application/didcomm-encrypted+json"));
        assert!(result.is_ok());

        // Test with charset parameter (should pass)
        let result =
            validate_message_security(Some("application/didcomm-signed+json; charset=utf-8"));
        assert!(result.is_ok());

        // Test invalid content type (should be rejected)
        let result = validate_message_security(Some("application/json"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Content-Type"));

        // Test missing content type (should be rejected)
        let result = validate_message_security(None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing Content-Type header"));
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
        let response = handle_didcomm(
            Some("application/didcomm-signed+json".to_string()),
            invalid_bytes,
            node.clone(),
            event_bus,
        )
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
