//! Message processors for TAP Node
//!
//! This module provides various message processors for handling messages.

use async_trait::async_trait;
use log::{debug, info};
use tap_core::message::TapMessage;

use crate::error::Result;
use crate::message::MessageProcessor;

/// A message processor that logs messages
pub struct LoggingMessageProcessor {
    /// Whether to log full message content
    log_content: bool,
}

impl LoggingMessageProcessor {
    /// Create a new logging message processor
    pub fn new(log_content: bool) -> Self {
        Self { log_content }
    }
}

#[async_trait]
impl MessageProcessor for LoggingMessageProcessor {
    async fn process_incoming(&self, message: TapMessage) -> Result<Option<TapMessage>> {
        let msg_id = message.id.clone();
        let msg_type = message.message_type.to_string();
        let from = message.from_did.clone().unwrap_or_else(|| "unknown".to_string());
        let to = message.to_did.clone().unwrap_or_else(|| "unknown".to_string());

        if self.log_content {
            info!("Incoming message: ID={}, Type={}, From={}, To={}, Content={:?}", 
                  msg_id, msg_type, from, to, message);
        } else {
            info!("Incoming message: ID={}, Type={}, From={}, To={}", 
                  msg_id, msg_type, from, to);
        }

        Ok(Some(message))
    }

    async fn process_outgoing(&self, message: TapMessage) -> Result<Option<TapMessage>> {
        let msg_id = message.id.clone();
        let msg_type = message.message_type.to_string();
        let from = message.from_did.clone().unwrap_or_else(|| "unknown".to_string());
        let to = message.to_did.clone().unwrap_or_else(|| "unknown".to_string());

        if self.log_content {
            info!("Outgoing message: ID={}, Type={}, From={}, To={}, Content={:?}", 
                  msg_id, msg_type, from, to, message);
        } else {
            info!("Outgoing message: ID={}, Type={}, From={}, To={}", 
                  msg_id, msg_type, from, to);
        }

        Ok(Some(message))
    }
}

/// Validates incoming messages to ensure they have the required fields
#[derive(Debug, Clone)]
pub struct ValidationMessageProcessor;

impl Default for ValidationMessageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationMessageProcessor {
    /// Create a new validation message processor
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MessageProcessor for ValidationMessageProcessor {
    async fn process_incoming(&self, message: TapMessage) -> Result<Option<TapMessage>> {
        // Validate required fields
        if message.id.is_empty() {
            debug!("Dropping message without ID");
            return Ok(None);
        }

        if message.to_did.is_none() {
            debug!("Dropping message without recipient (to_did field)");
            return Ok(None);
        }

        Ok(Some(message))
    }

    async fn process_outgoing(&self, message: TapMessage) -> Result<Option<TapMessage>> {
        // Validate required fields
        if message.id.is_empty() {
            debug!("Dropping outgoing message without ID");
            return Ok(None);
        }

        if message.to_did.is_none() {
            debug!("Dropping outgoing message without recipient (to_did field)");
            return Ok(None);
        }

        if message.from_did.is_none() {
            debug!("Dropping outgoing message without sender (from_did field)");
            return Ok(None);
        }

        Ok(Some(message))
    }
}
