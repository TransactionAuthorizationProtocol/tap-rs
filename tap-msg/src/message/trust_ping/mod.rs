//! Trust Ping Protocol Implementation
//!
//! Implementation of the DIDComm Trust Ping 2.0 protocol as specified at:
//! https://identity.foundation/didcomm-messaging/spec/#trust-ping-protocol-20
//!
//! The Trust Ping protocol is used to verify connectivity and test the
//! communication channel between DIDComm agents.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tap_msg_derive::TapMessage;

pub const TRUST_PING_TYPE: &str = "https://didcomm.org/trust-ping/2.0/ping";
pub const TRUST_PING_RESPONSE_TYPE: &str = "https://didcomm.org/trust-ping/2.0/ping-response";

/// Trust Ping message for testing connectivity between agents
///
/// The Trust Ping protocol allows agents to test their ability to communicate
/// and verify that the communication channel is working properly.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(
    message_type = "https://didcomm.org/trust-ping/2.0/ping",
    custom_validation
)]
pub struct TrustPing {
    /// Whether a response is requested (defaults to true)
    #[serde(default = "default_response_requested")]
    pub response_requested: bool,

    /// Optional comment or description for the ping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Trust Ping Response message
///
/// Response to a Trust Ping message, confirming that the communication
/// channel is working and the recipient is reachable.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TapMessage)]
#[tap(
    message_type = "https://didcomm.org/trust-ping/2.0/ping-response",
    custom_validation
)]
pub struct TrustPingResponse {
    /// Optional comment or description for the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Thread ID referencing the original ping message
    #[serde(default)]
    pub thread_id: String,

    /// Additional metadata
    #[serde(flatten, default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_response_requested() -> bool {
    true
}

impl Default for TrustPing {
    fn default() -> Self {
        Self {
            response_requested: true,
            comment: None,
            metadata: HashMap::new(),
        }
    }
}

impl TrustPing {
    /// Create a new Trust Ping message
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a Trust Ping with a comment
    pub fn with_comment(comment: String) -> Self {
        Self {
            comment: Some(comment),
            response_requested: true,
            metadata: HashMap::new(),
        }
    }

    /// Set whether a response is requested
    pub fn response_requested(mut self, requested: bool) -> Self {
        self.response_requested = requested;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Custom validation for Trust Ping messages
    pub fn validate_trustping(&self) -> Result<()> {
        // Trust Ping messages are very simple and don't require much validation
        // The only optional validation would be comment length if needed
        if let Some(ref comment) = self.comment {
            if comment.len() > 1000 {
                return Err(Error::Validation(
                    "Trust Ping comment exceeds maximum length of 1000 characters".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl TrustPingResponse {
    /// Create a new Trust Ping Response
    pub fn new(thread_id: String) -> Self {
        Self {
            comment: None,
            thread_id,
            metadata: HashMap::new(),
        }
    }

    /// Create a Trust Ping Response with a comment
    pub fn with_comment(thread_id: String, comment: String) -> Self {
        Self {
            comment: Some(comment),
            thread_id,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Custom validation for Trust Ping Response messages
    pub fn validate_trustpingresponse(&self) -> Result<()> {
        if self.thread_id.is_empty() {
            return Err(Error::Validation(
                "Thread ID is required in Trust Ping Response".to_string(),
            ));
        }

        // Validate comment length if present
        if let Some(ref comment) = self.comment {
            if comment.len() > 1000 {
                return Err(Error::Validation(
                    "Trust Ping Response comment exceeds maximum length of 1000 characters"
                        .to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_ping_creation() {
        let ping = TrustPing::new();
        assert!(ping.response_requested);
        assert!(ping.comment.is_none());
        assert!(ping.metadata.is_empty());
    }

    #[test]
    fn test_trust_ping_with_comment() {
        let ping = TrustPing::with_comment("Testing connectivity".to_string());
        assert_eq!(ping.comment, Some("Testing connectivity".to_string()));
        assert!(ping.response_requested);
    }

    #[test]
    fn test_trust_ping_no_response() {
        let ping = TrustPing::new().response_requested(false);
        assert!(!ping.response_requested);
    }

    #[test]
    fn test_trust_ping_response_creation() {
        let response = TrustPingResponse::new("thread-123".to_string());
        assert_eq!(response.thread_id, "thread-123");
        assert!(response.comment.is_none());
        assert!(response.metadata.is_empty());
    }

    #[test]
    fn test_trust_ping_response_with_comment() {
        let response =
            TrustPingResponse::with_comment("thread-123".to_string(), "Pong!".to_string());
        assert_eq!(response.thread_id, "thread-123");
        assert_eq!(response.comment, Some("Pong!".to_string()));
    }

    #[test]
    fn test_trust_ping_validation() {
        let ping = TrustPing::new();
        assert!(ping.validate_trustping().is_ok());

        let long_comment = "a".repeat(1001);
        let ping_with_long_comment = TrustPing::with_comment(long_comment);
        assert!(ping_with_long_comment.validate_trustping().is_err());
    }

    #[test]
    fn test_trust_ping_response_validation() {
        let response = TrustPingResponse::new("valid-thread-id".to_string());
        assert!(response.validate_trustpingresponse().is_ok());

        let response_empty_thread = TrustPingResponse::new("".to_string());
        assert!(response_empty_thread.validate_trustpingresponse().is_err());

        let long_comment = "a".repeat(1001);
        let response_with_long_comment =
            TrustPingResponse::with_comment("valid-thread-id".to_string(), long_comment);
        assert!(response_with_long_comment
            .validate_trustpingresponse()
            .is_err());
    }

    #[test]
    fn test_trust_ping_serialization() {
        let ping = TrustPing::with_comment("Test".to_string())
            .response_requested(false)
            .with_metadata("test_key".to_string(), serde_json::json!("test_value"));

        let serialized = serde_json::to_string(&ping).unwrap();
        let deserialized: TrustPing = serde_json::from_str(&serialized).unwrap();

        assert_eq!(ping.comment, deserialized.comment);
        assert_eq!(ping.response_requested, deserialized.response_requested);
        assert_eq!(ping.metadata, deserialized.metadata);
    }

    #[test]
    fn test_trust_ping_response_serialization() {
        let response =
            TrustPingResponse::with_comment("thread-123".to_string(), "Pong!".to_string())
                .with_metadata("test_key".to_string(), serde_json::json!("test_value"));

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: TrustPingResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response.comment, deserialized.comment);
        assert_eq!(response.thread_id, deserialized.thread_id);
        assert_eq!(response.metadata, deserialized.metadata);
    }

    #[test]
    fn test_message_types() {
        use crate::message::tap_message_trait::TapMessageBody;
        assert_eq!(TrustPing::message_type(), TRUST_PING_TYPE);
        assert_eq!(TrustPingResponse::message_type(), TRUST_PING_RESPONSE_TYPE);
    }
}
