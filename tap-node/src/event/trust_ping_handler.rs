//! Trust Ping Response Event Handler
//!
//! This module provides an event handler that listens for Trust Ping response events
//! and sends them through the appropriate channels.

use crate::event::{EventSubscriber, NodeEvent};
use crate::message::sender::PlainMessageSender;
use async_trait::async_trait;
use std::sync::Arc;
use tap_msg::didcomm::PlainMessage;

/// Event handler that processes Trust Ping response events and sends them
#[derive(Debug)]
pub struct TrustPingResponseHandler {
    /// Message sender for dispatching Trust Ping responses
    sender: Arc<dyn PlainMessageSender>,
}

impl TrustPingResponseHandler {
    /// Create a new Trust Ping response handler
    pub fn new(sender: Arc<dyn PlainMessageSender>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl EventSubscriber for TrustPingResponseHandler {
    async fn handle_event(&self, event: NodeEvent) {
        if let NodeEvent::PlainMessageSent { message, from, to } = event {
            // Check if this is a Trust Ping response by examining the message type
            if let Ok(plain_message) = serde_json::from_value::<PlainMessage>(message) {
                if plain_message.type_ == "https://didcomm.org/trust-ping/2.0/ping-response" {
                    log::debug!(
                        "Processing Trust Ping response event: from={}, to={}",
                        from,
                        to
                    );

                    // Serialize the message and send it
                    match serde_json::to_string(&plain_message) {
                        Ok(serialized_message) => {
                            if let Err(e) =
                                self.sender.send(serialized_message, vec![to.clone()]).await
                            {
                                log::warn!("Failed to send Trust Ping response to {}: {}", to, e);
                            } else {
                                log::info!("Successfully sent Trust Ping response to {}", to);
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to serialize Trust Ping response: {}", e);
                        }
                    }
                }
            }
        }
    }
}
