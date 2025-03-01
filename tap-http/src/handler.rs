//! Request handlers for the TAP HTTP server.

use crate::error::{Error, Result};
use bytes::Bytes;
use log::{debug, error, info};
use serde::Serialize;
use serde_json::json;
use std::convert::Infallible;
use std::sync::Arc;
use tap_core::{didcomm::unpack_didcomm_message, message::TapMessage};
use tap_node::TapNode;
use warp::{self, hyper::StatusCode, reply::json, Reply};

/// Response structure for health checks.
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

/// Handler for health check requests.
pub async fn handle_health_check() -> std::result::Result<impl Reply, Infallible> {
    info!("Health check request received");

    // Build response
    let response = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Ok(json(&response))
}

/// Handler for DIDComm messages.
pub async fn handle_didcomm(
    body: Bytes,
    node: Arc<TapNode>,
) -> std::result::Result<impl Reply, Infallible> {
    // Convert bytes to string
    let message_str = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to parse request body as UTF-8: {}", e);
            return Ok(json_error_response(
                StatusCode::BAD_REQUEST,
                "Invalid UTF-8 in request body",
            ));
        }
    };

    // Parse and unpack the DIDComm message
    debug!("Processing DIDComm message");
    let (tap_message, _metadata) = match unpack_didcomm_message(message_str).await {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to unpack DIDComm message: {}", e);
            return Ok(json_error_response(
                StatusCode::BAD_REQUEST,
                "Invalid DIDComm message format",
            ));
        }
    };

    // Process the message using the TAP Node
    match process_tap_message(tap_message, node).await {
        Ok(_) => {
            info!("DIDComm message processed successfully");
            Ok(json_success_response())
        }
        Err(e) => {
            error!("Error processing TAP message: {}", e);
            Ok(json_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Error processing message: {}", e),
            ))
        }
    }
}

/// Process a TAP message using the TAP Node.
async fn process_tap_message(message: TapMessage, node: Arc<TapNode>) -> Result<()> {
    // Process the message through the TAP Node
    node.process_message(message)
        .await
        .map_err(|e| Error::Node(e.to_string()))?;
    Ok(())
}

/// Create a JSON success response.
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
        // Call the health check handler
        let response = handle_health_check().await.unwrap();

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

        // Test with invalid UTF-8 data
        let invalid_bytes = Bytes::from(vec![0xFF, 0xFF]);
        let response = handle_didcomm(invalid_bytes, node.clone()).await.unwrap();

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
