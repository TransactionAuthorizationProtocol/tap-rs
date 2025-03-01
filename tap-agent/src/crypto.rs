//! Cryptographic operations for TAP Agent
//!
//! This module provides cryptographic functionality for the TAP Agent,
//! including message packing and unpacking using DIDComm.

use crate::did::DidResolver;
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tap_core::message::TapMessage;

/// Handler for packing and unpacking TAP messages
#[async_trait]
pub trait MessagePacker: Send + Sync {
    /// Packs a TAP message for the given recipient
    ///
    /// # Arguments
    /// * `message` - The TAP message to pack
    /// * `recipient` - The DID of the recipient
    ///
    /// # Returns
    /// * `Ok(String)` - The packed message as a JSON string
    /// * `Err` - If packing fails
    async fn pack_message(&self, message: &TapMessage, recipient: &str) -> Result<String>;

    /// Unpacks a TAP message
    ///
    /// # Arguments
    /// * `packed_message` - The packed message as a JSON string
    ///
    /// # Returns
    /// * `Ok(TapMessage)` - The unpacked message
    /// * `Err` - If unpacking fails
    async fn unpack_message(&self, packed_message: &str) -> Result<TapMessage>;
}

/// Default implementation of the MessagePacker trait
pub struct DefaultMessagePacker {
    /// Agent's DID for signing messages
    #[allow(dead_code)]
    agent_did: String,
    /// DID resolver for verifying messages
    resolver: Arc<dyn DidResolver>,
}

impl DefaultMessagePacker {
    /// Creates a new DefaultMessagePacker
    pub fn new(agent_did: String, resolver: Arc<dyn DidResolver>) -> Self {
        Self {
            agent_did,
            resolver,
        }
    }
}

#[async_trait]
impl MessagePacker for DefaultMessagePacker {
    async fn pack_message(&self, message: &TapMessage, recipient: &str) -> Result<String> {
        // In a real implementation, this would use DIDComm to pack and encrypt the message
        // For now, we'll just serialize to JSON

        // First, resolve the recipient's DID to get their keys
        let _recipient_did_doc = self
            .resolver
            .resolve(recipient)
            .await
            .map_err(|e| Error::Other(format!("Failed to resolve recipient DID: {}", e)))?;

        // Serialize the message to JSON
        let json = serde_json::to_string(message)
            .map_err(|e| Error::Other(format!("Failed to serialize message: {}", e)))?;

        // In a real implementation, we would:
        // 1. Sign the message with the agent's private key
        // 2. Encrypt the message with the recipient's public key
        // 3. Format according to DIDComm specs

        Ok(json)
    }

    async fn unpack_message(&self, packed_message: &str) -> Result<TapMessage> {
        // In a real implementation, this would:
        // 1. Decrypt the message with the agent's private key
        // 2. Verify the signature using the sender's public key
        // 3. Validate the DIDComm envelope

        // For now, just parse JSON
        let message: TapMessage = serde_json::from_str(packed_message)
            .map_err(|e| Error::Other(format!("Failed to parse packed message: {}", e)))?;

        Ok(message)
    }
}
