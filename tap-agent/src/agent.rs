//! TAP Agent implementation.
//!
//! This module provides the core Agent functionality for the TAP Protocol:
//! - The `Agent` trait defining core capabilities
//! - The `DefaultAgent` implementation of the trait
//! - Functions for sending and receiving TAP messages with DIDComm

use crate::config::AgentConfig;
use crate::crypto::MessagePacker;
use crate::error::{Error, Result};
use crate::message::SecurityMode;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;
use tap_msg::message::tap_message_trait::TapMessageBody;

/// A trait for agents that can send and receive TAP messages
///
/// This trait defines the core capabilities of a TAP Agent, including
/// managing identity, sending messages, and receiving messages.
#[async_trait]
pub trait Agent: Debug + Sync + Send {
    /// Get the agent's DID
    ///
    /// Returns the Decentralized Identifier (DID) that identifies this agent
    fn get_agent_did(&self) -> &str;

    /// Send a TAP message to a recipient
    ///
    /// This method handles:
    /// 1. Serializing the message
    /// 2. Determining appropriate security mode
    /// 3. Packing the message with DIDComm
    ///
    /// # Parameters
    /// * `message` - The message to send, implementing TapMessageBody
    /// * `to` - The DID of the recipient
    ///
    /// # Returns
    /// The packed message as a string, ready for transmission
    async fn send_message<T: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &T,
        to: &str,
    ) -> Result<String>;

    /// Receive and unpack a TAP message
    ///
    /// This method handles:
    /// 1. Unpacking the DIDComm message
    /// 2. Validating the message type
    /// 3. Deserializing to the requested type
    ///
    /// # Parameters
    /// * `packed_message` - The packed message as received
    ///
    /// # Returns
    /// The unpacked message deserialized to type T
    async fn receive_message<T: TapMessageBody + DeserializeOwned + Send>(
        &self,
        packed_message: &str,
    ) -> Result<T>;
}

/// Default implementation of the Agent trait
///
/// This implementation provides the standard TAP Agent functionality
/// using DIDComm for secure message exchange.
#[derive(Debug)]
pub struct DefaultAgent {
    /// Configuration for the agent
    config: AgentConfig,
    /// Message packer for handling DIDComm message packing/unpacking
    message_packer: Arc<dyn MessagePacker>,
}

impl DefaultAgent {
    /// Create a new DefaultAgent with the given configuration and message packer
    ///
    /// # Parameters
    /// * `config` - The agent configuration
    /// * `message_packer` - The message packer for DIDComm operations
    ///
    /// # Returns
    /// A new DefaultAgent instance
    pub fn new(config: AgentConfig, message_packer: Arc<dyn MessagePacker>) -> Self {
        Self {
            config,
            message_packer,
        }
    }

    /// Determine the appropriate security mode for a message type
    ///
    /// This method implements TAP protocol rules for which security modes
    /// should be used with different message types:
    /// - Presentation messages use authenticated encryption (AuthCrypt)
    /// - All other messages use digital signatures (Signed)
    ///
    /// # Parameters
    /// * `message_type` - The type of the message
    ///
    /// # Returns
    /// The appropriate SecurityMode for the message type
    fn determine_security_mode<T: TapMessageBody>(&self) -> SecurityMode {
        let message_type = T::message_type();
        if message_type == crate::message::PRESENTATION_MESSAGE_TYPE {
            SecurityMode::AuthCrypt
        } else {
            SecurityMode::Signed
        }
    }
}

#[async_trait]
impl Agent for DefaultAgent {
    fn get_agent_did(&self) -> &str {
        &self.config.agent_did
    }

    async fn send_message<T: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &T,
        to: &str,
    ) -> Result<String> {
        // Add type to message if needed
        let mut message_obj = serde_json::to_value(message)
            .map_err(|e| Error::Serialization(format!("Failed to serialize message: {}", e)))?;

        // Ensure message has a type field
        if message_obj.get("type").is_none() {
            if let serde_json::Value::Object(ref mut obj) = message_obj {
                obj.insert(
                    "type".to_string(),
                    serde_json::Value::String(T::message_type().to_string()),
                );
            }
        }

        // Validate the message
        message.validate().map_err(|e| {
            Error::Validation(format!(
                "Message validation failed for type {}: {}",
                T::message_type(),
                e
            ))
        })?;

        // Determine the appropriate security mode
        let security_mode = self.determine_security_mode::<T>();

        // Use message packer to pack the message
        let packed = self
            .message_packer
            .pack_message(&message_obj, to, Some(self.get_agent_did()), security_mode)
            .await?;

        Ok(packed)
    }

    async fn receive_message<T: TapMessageBody + DeserializeOwned + Send>(
        &self,
        packed_message: &str,
    ) -> Result<T> {
        // Unpack the message
        let message_value: Value = self
            .message_packer
            .unpack_message_value(packed_message)
            .await?;

        // Get the message type from the unpacked message
        let message_type = message_value
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| Error::Validation("Message missing 'type' field".to_string()))?;

        // Validate the message type
        if message_type != T::message_type() {
            return Err(Error::Validation(format!(
                "Expected message type {} but got {}",
                T::message_type(),
                message_type
            )));
        }

        // Check if we need to convert to a DIDComm message first
        if let Some(id) = message_value.get("id") {
            // This appears to be a proper DIDComm message already
            // Create a DIDComm message with the required fields
            let didcomm_message = didcomm::Message {
                id: id.as_str().unwrap_or("").to_string(),
                typ: "application/didcomm-plain+json".to_string(),
                type_: message_type.to_string(),
                // Use the entire message as the body, not just the "body" field
                body: message_value.clone(),
                from: message_value
                    .get("from")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                to: message_value
                    .get("to")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                thid: None,
                pthid: None,
                extra_headers: Default::default(),
                created_time: None,
                expires_time: None,
                from_prior: None,
                attachments: None,
            };

            // Convert to the requested type using the TapMessageBody trait
            let message = T::from_didcomm(&didcomm_message)
                .map_err(|e| Error::Validation(format!("Failed to convert message: {}", e)))?;

            // Validate the message
            message.validate().map_err(|e| {
                Error::Validation(format!(
                    "Message validation failed for type {}: {}",
                    T::message_type(),
                    e
                ))
            })?;

            Ok(message)
        } else {
            // This might be just the message body directly, try to deserialize
            let message = serde_json::from_value::<T>(message_value).map_err(|e| {
                Error::Serialization(format!("Failed to deserialize message: {}", e))
            })?;

            // Validate the message
            message.validate().map_err(|e| {
                Error::Validation(format!(
                    "Message validation failed for type {}: {}",
                    T::message_type(),
                    e
                ))
            })?;

            Ok(message)
        }
    }
}
