//! Trust Ping Protocol Processor
//!
//! Handles automatic Trust Ping responses according to DIDComm 2.0 specification

use crate::error::{Error, Result};
use crate::event::EventBus;
use crate::message::processor::PlainMessageProcessor;
use async_trait::async_trait;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tap_msg::didcomm::PlainMessage;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{TrustPing, TrustPingResponse};

/// Processor that automatically handles Trust Ping messages
pub struct TrustPingProcessor {
    /// Optional event bus for publishing response events
    event_bus: Option<Arc<EventBus>>,
}

// Manual Debug implementation
impl std::fmt::Debug for TrustPingProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrustPingProcessor")
            .field("event_bus", &self.event_bus.is_some())
            .finish()
    }
}

// Manual Clone implementation
impl Clone for TrustPingProcessor {
    fn clone(&self) -> Self {
        Self {
            event_bus: self.event_bus.clone(),
        }
    }
}

impl TrustPingProcessor {
    /// Create a new Trust Ping processor
    pub fn new() -> Self {
        Self { event_bus: None }
    }

    /// Create a new Trust Ping processor with an event bus for publishing responses
    pub fn with_event_bus(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus: Some(event_bus),
        }
    }

    /// Generate a Trust Ping response for a given Trust Ping message
    fn generate_ping_response(ping_message: &PlainMessage) -> Result<PlainMessage> {
        // Parse the ping from the message body to validate it
        let _ping: TrustPing = serde_json::from_value(ping_message.body.clone())
            .map_err(|e| Error::Serialization(format!("Failed to parse TrustPing: {}", e)))?;

        // Create a response with the same thread ID
        let response =
            TrustPingResponse::with_comment(ping_message.id.clone(), "Pong!".to_string());

        // Create the response PlainMessage
        let response_message = PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: TrustPingResponse::message_type().to_string(),
            body: serde_json::to_value(&response).map_err(|e| {
                Error::Serialization(format!("Failed to serialize response: {}", e))
            })?,
            from: ping_message.to[0].clone(), // Respond from the first recipient
            to: vec![ping_message.from.clone()], // Send back to original sender
            thid: Some(ping_message.id.clone()), // Set thread ID to original message ID
            pthid: None,
            extra_headers: HashMap::new(),
            attachments: None,
            created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            expires_time: None,
            from_prior: None,
        };

        Ok(response_message)
    }

    /// Check if a message is a Trust Ping that requests a response
    fn should_respond_to_ping(message: &PlainMessage) -> bool {
        if message.type_ != TrustPing::message_type() {
            return false;
        }

        // Parse the message body to check if response is requested
        if let Ok(ping) = serde_json::from_value::<TrustPing>(message.body.clone()) {
            ping.response_requested
        } else {
            false
        }
    }
}

#[async_trait]
impl PlainMessageProcessor for TrustPingProcessor {
    async fn process_incoming(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
        // Check if this is a Trust Ping that needs a response
        if Self::should_respond_to_ping(&message) {
            log::debug!(
                "Received Trust Ping from {}, generating response",
                message.from
            );

            // Generate and publish the response
            match Self::generate_ping_response(&message) {
                Ok(response) => {
                    log::debug!("Generated Trust Ping response for message {}", message.id);

                    // Publish the response as a message sent event if event bus is available
                    if let Some(ref event_bus) = self.event_bus {
                        // Extract the sender and recipient
                        let from = response.from.clone();
                        let to = response.to.first().cloned().unwrap_or_default();

                        // Publish the event
                        event_bus.publish_message_sent(response, from, to).await;
                        log::info!("Successfully published Trust Ping response via event bus");
                    } else {
                        // No event bus configured, just log the response
                        log::info!("Trust Ping response generated (no event bus configured): id={}, to={:?}", response.id, response.to);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to generate Trust Ping response: {}", e);
                }
            }
        }

        // Always pass the message through unchanged
        Ok(Some(message))
    }

    async fn process_outgoing(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
        // No special processing for outgoing messages
        Ok(Some(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tap_msg::message::TrustPing;

    #[test]
    fn test_should_respond_to_ping() {
        // Create a Trust Ping message that requests response
        let ping = TrustPing::new().response_requested(true);
        let ping_message = PlainMessage {
            id: "ping-123".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: TrustPing::message_type().to_string(),
            body: serde_json::to_value(&ping).unwrap(),
            from: "did:example:sender".to_string(),
            to: vec!["did:example:recipient".to_string()],
            thid: None,
            pthid: None,
            extra_headers: HashMap::new(),
            attachments: None,
            created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            expires_time: None,
            from_prior: None,
        };

        assert!(TrustPingProcessor::should_respond_to_ping(&ping_message));

        // Test with response_requested = false
        let ping_no_response = TrustPing::new().response_requested(false);
        let ping_message_no_response = PlainMessage {
            id: "ping-456".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: TrustPing::message_type().to_string(),
            body: serde_json::to_value(&ping_no_response).unwrap(),
            from: "did:example:sender".to_string(),
            to: vec!["did:example:recipient".to_string()],
            thid: None,
            pthid: None,
            extra_headers: HashMap::new(),
            attachments: None,
            created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            expires_time: None,
            from_prior: None,
        };

        assert!(!TrustPingProcessor::should_respond_to_ping(
            &ping_message_no_response
        ));
    }

    #[test]
    fn test_generate_ping_response() {
        let ping = TrustPing::with_comment("Hello!".to_string()).response_requested(true);

        let ping_message = PlainMessage {
            id: "ping-123".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: TrustPing::message_type().to_string(),
            body: serde_json::to_value(&ping).unwrap(),
            from: "did:example:sender".to_string(),
            to: vec!["did:example:recipient".to_string()],
            thid: None,
            pthid: None,
            extra_headers: HashMap::new(),
            attachments: None,
            created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            expires_time: None,
            from_prior: None,
        };

        let response = TrustPingProcessor::generate_ping_response(&ping_message).unwrap();

        assert_eq!(response.type_, TrustPingResponse::message_type());
        assert_eq!(response.from, "did:example:recipient");
        assert_eq!(response.to, vec!["did:example:sender"]);
        assert_eq!(response.thid, Some("ping-123".to_string()));

        // Verify the response body
        let response_body: TrustPingResponse = serde_json::from_value(response.body).unwrap();
        assert_eq!(response_body.thread_id, "ping-123");
        assert_eq!(response_body.comment, Some("Pong!".to_string()));
    }
}
