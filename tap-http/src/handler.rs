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
use tap_agent::did::{DIDGenerationOptions, KeyType, Service};
use tap_agent::key_manager::KeyManager;
use tap_agent::{AgentConfig, AgentKeyManager, TapAgent};
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

/// Maximum allowed length for a domain name (per RFC 1035)
const MAX_DOMAIN_LENGTH: usize = 253;

/// Sanitize and validate a domain name from an HTTP Host header.
///
/// Accepts valid DNS hostnames with optional port numbers. Rejects domains
/// containing path traversal characters, whitespace, or other unsafe characters.
///
/// Returns the sanitized, lowercased domain or an error if invalid.
pub fn sanitize_domain(host: &str) -> Result<String> {
    let trimmed = host.trim();

    if trimmed.is_empty() {
        return Err(Error::Validation("Empty domain name".to_string()));
    }

    // Split off port if present
    let (domain, port) = if let Some(colon_idx) = trimmed.rfind(':') {
        let potential_port = &trimmed[colon_idx + 1..];
        // Verify it's actually a port number (all digits)
        if potential_port.chars().all(|c| c.is_ascii_digit()) && !potential_port.is_empty() {
            (&trimmed[..colon_idx], Some(potential_port))
        } else {
            (trimmed, None)
        }
    } else {
        (trimmed, None)
    };

    if domain.is_empty() {
        return Err(Error::Validation("Empty domain name".to_string()));
    }

    if domain.len() > MAX_DOMAIN_LENGTH {
        return Err(Error::Validation(format!(
            "Domain name exceeds maximum length of {} characters",
            MAX_DOMAIN_LENGTH
        )));
    }

    // Validate each character: only ASCII alphanumeric, hyphens, and dots
    for ch in domain.chars() {
        if !ch.is_ascii_alphanumeric() && ch != '-' && ch != '.' {
            return Err(Error::Validation(format!(
                "Invalid character '{}' in domain name",
                ch
            )));
        }
    }

    // No leading or trailing dots or hyphens
    if domain.starts_with('.') || domain.ends_with('.') {
        return Err(Error::Validation(
            "Domain name must not start or end with a dot".to_string(),
        ));
    }

    if domain.starts_with('-') || domain.ends_with('-') {
        return Err(Error::Validation(
            "Domain name must not start or end with a hyphen".to_string(),
        ));
    }

    // No consecutive dots
    if domain.contains("..") {
        return Err(Error::Validation(
            "Domain name must not contain consecutive dots".to_string(),
        ));
    }

    // Validate each label (part between dots)
    for label in domain.split('.') {
        if label.is_empty() {
            return Err(Error::Validation(
                "Domain name contains an empty label".to_string(),
            ));
        }
        if label.len() > 63 {
            return Err(Error::Validation(
                "Domain label exceeds maximum length of 63 characters".to_string(),
            ));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(Error::Validation(
                "Domain label must not start or end with a hyphen".to_string(),
            ));
        }
    }

    // Validate port if present
    if let Some(p) = port {
        let port_num: u16 = p
            .parse()
            .map_err(|_| Error::Validation(format!("Invalid port number: {}", p)))?;
        if port_num == 0 {
            return Err(Error::Validation(
                "Port number must not be zero".to_string(),
            ));
        }
    }

    // Build the sanitized result (lowercased)
    let sanitized_domain = domain.to_lowercase();
    match port {
        Some(p) => Ok(format!("{}:{}", sanitized_domain, p)),
        None => Ok(sanitized_domain),
    }
}

/// Convert a sanitized domain (with optional port) to a did:web DID.
///
/// Per the did:web spec, colons in the domain (from port numbers) are
/// percent-encoded as `%3A`.
pub fn domain_to_did_web(domain: &str) -> String {
    // Replace colons (from port) with %3A per did:web spec
    let encoded = domain.replace(':', "%3A");
    format!("did:web:{}", encoded)
}

/// Convert a DIDDoc to a W3C DID Core spec compliant JSON-LD document.
///
/// The internal DIDDoc uses snake_case field names, but the DID spec requires camelCase.
/// This function produces a properly formatted document suitable for serving at
/// `/.well-known/did.json`.
fn did_doc_to_json_ld(doc: &tap_agent::did::DIDDoc) -> serde_json::Value {
    use tap_agent::did::{VerificationMaterial, VerificationMethodType};

    let verification_methods: Vec<serde_json::Value> = doc
        .verification_method
        .iter()
        .map(|vm| {
            let mut obj = json!({
                "id": vm.id,
                "type": match &vm.type_ {
                    VerificationMethodType::Ed25519VerificationKey2018 => "Ed25519VerificationKey2018",
                    VerificationMethodType::X25519KeyAgreementKey2019 => "X25519KeyAgreementKey2019",
                    VerificationMethodType::EcdsaSecp256k1VerificationKey2019 => "EcdsaSecp256k1VerificationKey2019",
                    VerificationMethodType::JsonWebKey2020 => "JsonWebKey2020",
                },
                "controller": vm.controller,
            });

            match &vm.verification_material {
                VerificationMaterial::Base58 { public_key_base58 } => {
                    obj["publicKeyBase58"] = json!(public_key_base58);
                }
                VerificationMaterial::Multibase {
                    public_key_multibase,
                } => {
                    obj["publicKeyMultibase"] = json!(public_key_multibase);
                }
                VerificationMaterial::JWK { public_key_jwk } => {
                    obj["publicKeyJwk"] = public_key_jwk.clone();
                }
            }

            obj
        })
        .collect();

    let services: Vec<serde_json::Value> = doc
        .service
        .iter()
        .map(|svc| {
            let mut obj = json!({
                "id": svc.id,
                "type": svc.type_,
                "serviceEndpoint": svc.service_endpoint,
            });
            // Add additional properties
            for (k, v) in &svc.properties {
                obj[k] = v.clone();
            }
            obj
        })
        .collect();

    let mut result = json!({
        "@context": [
            "https://www.w3.org/ns/did/v1",
            "https://w3id.org/security/suites/ed25519-2018/v1"
        ],
        "id": doc.id,
        "verificationMethod": verification_methods,
        "authentication": doc.authentication,
    });

    if !doc.key_agreement.is_empty() {
        result["keyAgreement"] = json!(doc.key_agreement);
    }
    if !doc.assertion_method.is_empty() {
        result["assertionMethod"] = json!(doc.assertion_method);
    }
    if !doc.capability_invocation.is_empty() {
        result["capabilityInvocation"] = json!(doc.capability_invocation);
    }
    if !doc.capability_delegation.is_empty() {
        result["capabilityDelegation"] = json!(doc.capability_delegation);
    }
    if !services.is_empty() {
        result["service"] = json!(services);
    }

    result
}

/// Handler for `/.well-known/did.json` requests.
///
/// Maps the hostname from the HTTP request to a `did:web` DID. If an agent
/// with that DID exists in the node's agent registry, returns the DID document.
/// If not, creates a new agent with fresh keys and returns the document.
///
/// The DID document includes the DIDComm messaging service endpoint and
/// the agent's public keys.
pub async fn handle_well_known_did(
    host: Option<String>,
    node: Arc<TapNode>,
    event_bus: Arc<EventBus>,
) -> std::result::Result<impl Reply, Infallible> {
    let start_time = Instant::now();

    event_bus
        .publish_request_received("GET".to_string(), "/.well-known/did.json".to_string(), None)
        .await;

    // Extract and sanitize domain from Host header
    let domain = match host {
        Some(h) => match sanitize_domain(&h) {
            Ok(d) => d,
            Err(e) => {
                warn!("Invalid Host header for web DID: {}", e);
                let response = json_error_response(
                    StatusCode::BAD_REQUEST,
                    &format!("Invalid Host header: {}", e),
                );
                let duration_ms = start_time.elapsed().as_millis() as u64;
                event_bus
                    .publish_response_sent(StatusCode::BAD_REQUEST, 200, duration_ms)
                    .await;
                return Ok(response);
            }
        },
        None => {
            let response = json_error_response(StatusCode::BAD_REQUEST, "Missing Host header");
            let duration_ms = start_time.elapsed().as_millis() as u64;
            event_bus
                .publish_response_sent(StatusCode::BAD_REQUEST, 200, duration_ms)
                .await;
            return Ok(response);
        }
    };

    let did_web = domain_to_did_web(&domain);
    info!("Web DID document requested for: {}", did_web);

    // Check if an agent already exists for this DID
    if node.agents().has_agent(&did_web) {
        debug!("Found existing agent for {}", did_web);
        match node.agents().get_agent(&did_web).await {
            Ok(agent) => match agent.key_manager().get_generated_key(&did_web) {
                Ok(generated_key) => {
                    let did_doc = did_doc_to_json_ld(&generated_key.did_doc);
                    let response =
                        warp::reply::with_status(warp::reply::json(&did_doc), StatusCode::OK)
                            .into_response();
                    let duration_ms = start_time.elapsed().as_millis() as u64;
                    event_bus
                        .publish_response_sent(StatusCode::OK, 500, duration_ms)
                        .await;
                    return Ok(response);
                }
                Err(e) => {
                    error!("Failed to get DID document for {}: {}", did_web, e);
                    let response = json_error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to retrieve DID document",
                    );
                    let duration_ms = start_time.elapsed().as_millis() as u64;
                    event_bus
                        .publish_response_sent(StatusCode::INTERNAL_SERVER_ERROR, 200, duration_ms)
                        .await;
                    return Ok(response);
                }
            },
            Err(e) => {
                error!("Failed to get agent for {}: {}", did_web, e);
                let response = json_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to retrieve agent",
                );
                let duration_ms = start_time.elapsed().as_millis() as u64;
                event_bus
                    .publish_response_sent(StatusCode::INTERNAL_SERVER_ERROR, 200, duration_ms)
                    .await;
                return Ok(response);
            }
        }
    }

    // No agent exists — create a new one
    info!("Creating new agent for {}", did_web);

    let key_manager = AgentKeyManager::new();
    let options = DIDGenerationOptions {
        key_type: KeyType::Ed25519,
    };

    // Percent-encode the domain for did:web (colons in port become %3A)
    let encoded_domain = domain.replace(':', "%3A");
    let generated_key = match key_manager.generate_web_did(&encoded_domain, options) {
        Ok(k) => k,
        Err(e) => {
            error!("Failed to generate web DID for {}: {}", domain, e);
            let response =
                json_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate DID");
            let duration_ms = start_time.elapsed().as_millis() as u64;
            event_bus
                .publish_response_sent(StatusCode::INTERNAL_SERVER_ERROR, 200, duration_ms)
                .await;
            return Ok(response);
        }
    };

    // Build the DIDComm service endpoint URL
    let scheme = "https";
    let didcomm_endpoint = format!("{}://{}/didcomm", scheme, domain);

    // Add the DIDComm service endpoint to the DID document
    let mut did_doc = generated_key.did_doc.clone();
    did_doc.service.push(Service {
        id: format!("{}#didcomm", did_web),
        type_: "DIDCommMessaging".to_string(),
        service_endpoint: didcomm_endpoint,
        properties: Default::default(),
    });

    // Create and register the agent
    let config = AgentConfig::new(did_web.clone());
    let agent = TapAgent::new(config, Arc::new(key_manager));

    if let Err(e) = node.register_agent(Arc::new(agent)).await {
        // If registration fails (e.g., race condition, max agents), log but still return the doc
        warn!("Failed to register new agent for {}: {}", did_web, e);
    }

    let json_doc = did_doc_to_json_ld(&did_doc);
    let response =
        warp::reply::with_status(warp::reply::json(&json_doc), StatusCode::OK).into_response();
    let duration_ms = start_time.elapsed().as_millis() as u64;
    event_bus
        .publish_response_sent(StatusCode::OK, 500, duration_ms)
        .await;

    Ok(response)
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
        let config = NodeConfig {
            storage_path: None, // Disable storage for tests
            ..Default::default()
        };
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

    // --- Domain sanitization tests ---

    #[test]
    fn test_sanitize_domain_valid() {
        assert_eq!(sanitize_domain("example.com").unwrap(), "example.com");
        assert_eq!(
            sanitize_domain("sub.example.com").unwrap(),
            "sub.example.com"
        );
        assert_eq!(
            sanitize_domain("example.com:8080").unwrap(),
            "example.com:8080"
        );
        assert_eq!(sanitize_domain("EXAMPLE.COM").unwrap(), "example.com");
        assert_eq!(
            sanitize_domain("my-host.example.com").unwrap(),
            "my-host.example.com"
        );
    }

    #[test]
    fn test_sanitize_domain_trims_whitespace() {
        assert_eq!(sanitize_domain("  example.com  ").unwrap(), "example.com");
    }

    #[test]
    fn test_sanitize_domain_rejects_empty() {
        assert!(sanitize_domain("").is_err());
        assert!(sanitize_domain("   ").is_err());
    }

    #[test]
    fn test_sanitize_domain_rejects_invalid_chars() {
        assert!(sanitize_domain("example.com/path").is_err());
        assert!(sanitize_domain("example.com\\path").is_err());
        assert!(sanitize_domain("exam ple.com").is_err());
        assert!(sanitize_domain("example.com?query").is_err());
        assert!(sanitize_domain("<script>").is_err());
        assert!(sanitize_domain("example.com#frag").is_err());
        assert!(sanitize_domain("ex@mple.com").is_err());
    }

    #[test]
    fn test_sanitize_domain_rejects_invalid_structure() {
        assert!(sanitize_domain(".example.com").is_err());
        assert!(sanitize_domain("example.com.").is_err());
        assert!(sanitize_domain("example..com").is_err());
        assert!(sanitize_domain("-example.com").is_err());
        assert!(sanitize_domain("example.com-").is_err());
        assert!(sanitize_domain("exam.-ple.com").is_err());
    }

    #[test]
    fn test_sanitize_domain_rejects_port_zero() {
        assert!(sanitize_domain("example.com:0").is_err());
    }

    #[test]
    fn test_sanitize_domain_rejects_too_long() {
        let long_label = "a".repeat(64);
        let long_domain = format!("{}.com", long_label);
        assert!(sanitize_domain(&long_domain).is_err());
    }

    // --- domain_to_did_web tests ---

    #[test]
    fn test_domain_to_did_web_simple() {
        assert_eq!(domain_to_did_web("example.com"), "did:web:example.com");
    }

    #[test]
    fn test_domain_to_did_web_with_port() {
        assert_eq!(
            domain_to_did_web("example.com:3000"),
            "did:web:example.com%3A3000"
        );
    }

    #[test]
    fn test_domain_to_did_web_subdomain() {
        assert_eq!(
            domain_to_did_web("api.example.com"),
            "did:web:api.example.com"
        );
    }

    // --- did_doc_to_json_ld tests ---

    #[test]
    fn test_did_doc_to_json_ld_has_context() {
        use tap_agent::did::DIDDoc;
        let doc = DIDDoc {
            id: "did:web:example.com".to_string(),
            verification_method: vec![],
            authentication: vec![],
            key_agreement: vec![],
            assertion_method: vec![],
            capability_invocation: vec![],
            capability_delegation: vec![],
            service: vec![],
        };

        let json = did_doc_to_json_ld(&doc);
        assert!(json["@context"].is_array());
        assert_eq!(json["id"], "did:web:example.com");
        assert!(json["verificationMethod"].is_array());
        // Empty optional arrays should be omitted
        assert!(json.get("assertionMethod").is_none());
        assert!(json.get("capabilityInvocation").is_none());
        assert!(json.get("capabilityDelegation").is_none());
        // Service should be omitted when empty
        assert!(json.get("service").is_none());
    }

    // --- handle_well_known_did tests ---

    #[tokio::test(flavor = "multi_thread")]
    async fn test_well_known_did_missing_host() {
        let config = NodeConfig {
            storage_path: None,
            ..Default::default()
        };
        let node = Arc::new(TapNode::new(config));
        let event_bus = Arc::new(crate::event::EventBus::new());

        let response = handle_well_known_did(None, node, event_bus).await.unwrap();

        let response_bytes = to_bytes(response.into_response().into_body())
            .await
            .unwrap();
        let response_json: Value = serde_json::from_slice(&response_bytes).unwrap();
        assert_eq!(response_json["status"], "error");
        assert!(response_json["message"]
            .as_str()
            .unwrap()
            .contains("Missing Host header"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_well_known_did_invalid_host() {
        let config = NodeConfig {
            storage_path: None,
            ..Default::default()
        };
        let node = Arc::new(TapNode::new(config));
        let event_bus = Arc::new(crate::event::EventBus::new());

        let response = handle_well_known_did(
            Some("<script>alert(1)</script>".to_string()),
            node,
            event_bus,
        )
        .await
        .unwrap();

        let response_bytes = to_bytes(response.into_response().into_body())
            .await
            .unwrap();
        let response_json: Value = serde_json::from_slice(&response_bytes).unwrap();
        assert_eq!(response_json["status"], "error");
        assert!(response_json["message"]
            .as_str()
            .unwrap()
            .contains("Invalid Host header"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_well_known_did_creates_new_agent() {
        let config = NodeConfig {
            storage_path: None,
            ..Default::default()
        };
        let node = Arc::new(TapNode::new(config));
        let event_bus = Arc::new(crate::event::EventBus::new());

        let response =
            handle_well_known_did(Some("example.com".to_string()), node.clone(), event_bus)
                .await
                .unwrap();

        let response_bytes = to_bytes(response.into_response().into_body())
            .await
            .unwrap();
        let response_json: Value = serde_json::from_slice(&response_bytes).unwrap();

        // Verify the DID document structure (W3C DID Core spec uses camelCase)
        assert_eq!(response_json["id"], "did:web:example.com");
        assert!(response_json["@context"].is_array());
        assert!(response_json["verificationMethod"].is_array());
        assert!(!response_json["verificationMethod"]
            .as_array()
            .unwrap()
            .is_empty());

        // Verify the service endpoint
        let services = response_json["service"].as_array().unwrap();
        assert!(!services.is_empty());
        assert_eq!(services[0]["type"], "DIDCommMessaging");
        assert_eq!(
            services[0]["serviceEndpoint"],
            "https://example.com/didcomm"
        );

        // Verify agent was registered
        assert!(node.agents().has_agent("did:web:example.com"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_well_known_did_returns_existing_agent() {
        let config = NodeConfig {
            storage_path: None,
            ..Default::default()
        };
        let node = Arc::new(TapNode::new(config));
        let event_bus = Arc::new(crate::event::EventBus::new());

        // Create agent manually first
        let key_manager = AgentKeyManager::new();
        let options = DIDGenerationOptions {
            key_type: KeyType::Ed25519,
        };
        let _generated = key_manager
            .generate_web_did("test.example.com", options)
            .unwrap();
        let agent_config = AgentConfig::new("did:web:test.example.com".to_string());
        let agent = TapAgent::new(agent_config, Arc::new(key_manager));
        node.register_agent(Arc::new(agent)).await.unwrap();

        // Now request the DID document
        let response = handle_well_known_did(
            Some("test.example.com".to_string()),
            node.clone(),
            event_bus,
        )
        .await
        .unwrap();

        let response_bytes = to_bytes(response.into_response().into_body())
            .await
            .unwrap();
        let response_json: Value = serde_json::from_slice(&response_bytes).unwrap();

        assert_eq!(response_json["id"], "did:web:test.example.com");
        assert!(response_json["@context"].is_array());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_well_known_did_with_port() {
        let config = NodeConfig {
            storage_path: None,
            ..Default::default()
        };
        let node = Arc::new(TapNode::new(config));
        let event_bus = Arc::new(crate::event::EventBus::new());

        let response =
            handle_well_known_did(Some("localhost:3000".to_string()), node.clone(), event_bus)
                .await
                .unwrap();

        let response_bytes = to_bytes(response.into_response().into_body())
            .await
            .unwrap();
        let response_json: Value = serde_json::from_slice(&response_bytes).unwrap();

        assert_eq!(response_json["id"], "did:web:localhost%3A3000");

        // Verify service endpoint includes the port
        let services = response_json["service"].as_array().unwrap();
        assert_eq!(
            services[0]["serviceEndpoint"],
            "https://localhost:3000/didcomm"
        );

        // Verify agent was registered with percent-encoded DID
        assert!(node.agents().has_agent("did:web:localhost%3A3000"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_well_known_did_idempotent() {
        let config = NodeConfig {
            storage_path: None,
            ..Default::default()
        };
        let node = Arc::new(TapNode::new(config));
        let event_bus = Arc::new(crate::event::EventBus::new());

        // First request creates the agent
        let response1 = handle_well_known_did(
            Some("idempotent.example.com".to_string()),
            node.clone(),
            event_bus.clone(),
        )
        .await
        .unwrap();
        let bytes1 = to_bytes(response1.into_response().into_body())
            .await
            .unwrap();
        let json1: Value = serde_json::from_slice(&bytes1).unwrap();

        // Second request returns the same agent
        let response2 = handle_well_known_did(
            Some("idempotent.example.com".to_string()),
            node.clone(),
            event_bus,
        )
        .await
        .unwrap();
        let bytes2 = to_bytes(response2.into_response().into_body())
            .await
            .unwrap();
        let json2: Value = serde_json::from_slice(&bytes2).unwrap();

        // The DID documents should have the same ID and keys
        assert_eq!(json1["id"], json2["id"]);
        assert_eq!(json1["verificationMethod"], json2["verificationMethod"]);
    }
}
