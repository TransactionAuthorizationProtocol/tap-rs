use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::prelude::*;
use didcomm::Message as DidcommMessage;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::did::DidResolver;
use crate::error::{Error, Result};

/// Enum to represent various security modes for message packing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SecurityMode {
    /// Plain message, no encryption
    Plain,
    /// Authenticated encryption (authcrypt)
    Authcrypt,
    /// Anonymous encryption (anoncrypt)
    Anoncrypt,
}

/// A type-erased trait for packing and unpacking messages
///
/// This trait is used by the Agent to pack and unpack messages for transmission.
#[async_trait]
pub trait MessagePacker: Send + Sync + Debug {
    /// Pack a message for transmission
    async fn pack_message(
        &self,
        message: &(dyn erased_serde::Serialize + Sync),
        to: &str,
        from: Option<&str>,
        mode: SecurityMode,
    ) -> Result<String>;

    /// Unpack a received message into a raw JSON value
    async fn unpack_message(&self, packed_msg: &str)
        -> Result<(serde_json::Value, Option<String>)>;
}

/// Default implementation of [MessagePacker]
pub struct DefaultMessagePacker {
    /// The DID resolver used for resolving DIDs
    resolver: Arc<dyn DidResolver>,
}

impl Debug for DefaultMessagePacker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultMessagePacker").finish()
    }
}

impl DefaultMessagePacker {
    /// Creates a new MessagePacker with the provided DID resolver
    pub fn new(resolver: Arc<dyn DidResolver>) -> Self {
        Self { resolver }
    }

    /// Helper method to deserialize a serde_json::Value into a specific type
    pub fn deserialize_value<T: DeserializeOwned>(&self, value: serde_json::Value) -> Result<T> {
        serde_json::from_value(value)
            .map_err(|e| Error::SerializationError(format!("Failed to deserialize value: {}", e)))
    }

    /// Gets a reference to the internal resolver
    pub fn resolver(&self) -> &Arc<dyn DidResolver> {
        &self.resolver
    }

    /// Determines the appropriate security mode based on message type
    fn determine_security_mode(&self, message_type: &str, requested_mode: SecurityMode) -> SecurityMode {
        // For Presentation messages, use Authcrypt regardless of requested mode
        // unless Plain is explicitly requested
        if message_type.contains("tap/1.0/presentation") && requested_mode != SecurityMode::Plain {
            SecurityMode::Authcrypt
        } else {
            requested_mode
        }
    }

    /// Create a DIDComm message from the serialized content
    async fn create_didcomm_message(
        &self,
        value: serde_json::Value,
        to: &str,
        from: Option<&str>,
        message_type: Option<&str>,
    ) -> Result<DidcommMessage> {
        // Extract message type if available
        let type_ = match message_type {
            Some(t) => t.to_string(),
            None => {
                // Try to extract type from value
                value
                    .get("type")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "tap/1.0/message".to_string())
            }
        };

        // Create a unique ID for the message
        let id = Uuid::new_v4().to_string();

        // Build the DIDComm message
        let mut builder = DidcommMessage::build(id, type_, value.clone());
        
        // Add to
        builder = builder.to_many(vec![to.to_string()]);
        
        // Add from if provided
        if let Some(from_did) = from {
            builder = builder.from(from_did.to_string());
        }
        
        // Add current time
        let created_time = Utc::now().timestamp() as u64;
        builder = builder.created_time(created_time);
        
        // Finalize the message
        Ok(builder.finalize())
    }
}

#[async_trait]
impl MessagePacker for DefaultMessagePacker {
    async fn pack_message(
        &self,
        message: &(dyn erased_serde::Serialize + Sync),
        to: &str,
        from: Option<&str>,
        mode: SecurityMode,
    ) -> Result<String> {
        // Convert the erased_serde Serialize to a serde_json::Value
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::new(&mut buf);
        let mut serializer = <dyn erased_serde::Serializer>::erase(&mut ser);
        message.erased_serialize(&mut serializer).map_err(|e| {
            Error::SerializationError(format!("Failed to serialize message: {}", e))
        })?;

        let value: serde_json::Value = serde_json::from_slice(&buf).map_err(|e| {
            Error::SerializationError(format!("Failed to deserialize message to Value: {}", e))
        })?;

        // Extract message type for security mode determination
        let message_type = value.get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("tap/1.0/basic-message");

        // Determine the appropriate security mode
        let actual_mode = self.determine_security_mode(message_type, mode);

        // Create the DIDComm message
        let didcomm_message = self.create_didcomm_message(
            value.clone(),
            to,
            from,
            Some(message_type),
        ).await?;

        // Pack the message according to the security mode
        let packed = match actual_mode {
            SecurityMode::Plain => {
                // Just serialize the DIDComm message
                serde_json::to_string(&didcomm_message).map_err(|e| {
                    Error::SerializationError(format!("Failed to serialize DIDComm message: {}", e))
                })?
            }
            SecurityMode::Authcrypt => {
                // Authenticated encryption requires a sender
                let from_did = from.ok_or_else(|| {
                    Error::Crypto("Sender DID required for authenticated encryption".to_string())
                })?;

                // In a real implementation, we'd resolve DIDs to get keys
                // and use the didcomm library's encryption functions
                // For now, just add a header to show it's authcrypt
                format!(
                    "AUTHCRYPT:{}:{}",
                    from_did,
                    serde_json::to_string(&didcomm_message).map_err(|e| {
                        Error::SerializationError(format!("Failed to serialize DIDComm message: {}", e))
                    })?
                )
            }
            SecurityMode::Anoncrypt => {
                // In a real implementation, we'd resolve DIDs to get keys
                // and use the didcomm library's anonymous encryption
                // For now, just add a header to show it's anoncrypt
                format!(
                    "ANONCRYPT:{}",
                    serde_json::to_string(&didcomm_message).map_err(|e| {
                        Error::SerializationError(format!("Failed to serialize DIDComm message: {}", e))
                    })?
                )
            }
        };

        Ok(packed)
    }

    async fn unpack_message(
        &self,
        packed_msg: &str,
    ) -> Result<(serde_json::Value, Option<String>)> {
        // Detect the message format
        if packed_msg.starts_with("AUTHCRYPT:") {
            // Handle authcrypt messages
            let parts: Vec<&str> = packed_msg.splitn(3, ':').collect();
            if parts.len() != 3 {
                return Err(Error::SerializationError(
                    "Invalid authcrypt message format".to_string(),
                ));
            }

            let sender = parts[1];
            let json_str = parts[2];

            // Parse the JSON payload
            let didcomm_message: DidcommMessage = serde_json::from_str(json_str).map_err(|e| {
                Error::SerializationError(format!("Failed to deserialize DIDComm message: {}", e))
            })?;

            // Return the body and sender
            Ok((didcomm_message.body, Some(sender.to_string())))
        } else if let Some(json_str) = packed_msg.strip_prefix("ANONCRYPT:") {
            // Handle anoncrypt messages
            // Parse the JSON payload
            let didcomm_message: DidcommMessage = serde_json::from_str(json_str).map_err(|e| {
                Error::SerializationError(format!("Failed to deserialize DIDComm message: {}", e))
            })?;

            // For anoncrypt, we don't know the sender
            Ok((didcomm_message.body, didcomm_message.from))
        } else {
            // Try to parse as a JWE (for future use)
            if let Ok(jwe) = serde_json::from_str::<serde_json::Value>(packed_msg) {
                if jwe.get("ciphertext").is_some() {
                    // This is a JWE, but we're not fully implementing it yet
                    // For future implementation
                    return Err(Error::Crypto("JWE support not yet implemented".to_string()));
                }
            }
            
            // Assume it's a plain message
            let didcomm_message: DidcommMessage = serde_json::from_str(packed_msg).map_err(|e| {
                Error::SerializationError(format!("Failed to deserialize DIDComm message: {}", e))
            })?;

            // Return the body value and the sender DID
            Ok((didcomm_message.body, didcomm_message.from))
        }
    }
}
