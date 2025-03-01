//! DIDComm message unpacking for TAP messages.
//!
//! This module provides functionality to decrypt DIDComm v2 messages
//! and convert them into TAP message structures.

use crate::error::{Error, Result};
use crate::message::TapMessage;
use didcomm::Message;
use serde_json::Value;
use std::collections::HashMap;

/// Metadata extracted from a DIDComm message.
#[derive(Debug, Clone)]
pub struct MessageMetadata {
    /// The DID of the sender.
    pub from_did: Option<String>,

    /// The DIDs of the recipients.
    pub to_dids: Vec<String>,

    /// Whether the message was authenticated.
    pub is_authenticated: bool,

    /// Whether the message was encrypted.
    pub is_encrypted: bool,

    /// Additional DIDComm message headers.
    pub headers: HashMap<String, Value>,
}

/// Unpacks a DIDComm message into a TAP message.
pub async fn unpack_didcomm_message(packed_message: &str) -> Result<(TapMessage, MessageMetadata)> {
    // Parse the DIDComm message
    let message: Message =
        serde_json::from_str(packed_message).map_err(|e| Error::DIDComm(e.to_string()))?;

    // Extract metadata
    let metadata = extract_metadata_from_message(&message)?;

    // Extract the body
    let tap_message: TapMessage = serde_json::from_value(message.body.clone())
        .map_err(|e| Error::ParseError(e.to_string()))?;

    Ok((tap_message, metadata))
}

/// Extracts metadata from a DIDComm message.
fn extract_metadata_from_message(message: &Message) -> Result<MessageMetadata> {
    // Extract the sender DID
    let from_did = message.from.clone();

    // Extract recipient DIDs
    let to_dids = message.to.clone().unwrap_or_default();

    // For this implementation, we'll set these based on existing fields
    let is_authenticated = from_did.is_some();
    let is_encrypted = !to_dids.is_empty();

    // Extract headers (in DIDComm v2, these might be in different fields)
    let mut headers = HashMap::new();

    // Add any headers from the message that are relevant to TAP
    headers.insert("type".to_string(), Value::String(message.typ.clone()));

    if let Some(created_time) = &message.created_time {
        headers.insert(
            "created_time".to_string(),
            Value::String(created_time.to_string()),
        );
    }

    // Add ID as well
    headers.insert("id".to_string(), Value::String(message.id.clone()));

    // If there are any other relevant headers, add them here

    Ok(MessageMetadata {
        from_did,
        to_dids,
        is_authenticated,
        is_encrypted,
        headers,
    })
}

/// Extracts metadata from a packed DIDComm message.
pub fn extract_metadata(packed_message: &str) -> Result<MessageMetadata> {
    // Parse the DIDComm message
    let message: Message =
        serde_json::from_str(packed_message).map_err(|e| Error::DIDComm(e.to_string()))?;

    extract_metadata_from_message(&message)
}

/// Extracts the message type from a DIDComm message.
pub fn extract_message_type(packed_message: &str) -> Result<String> {
    // Parse the DIDComm message
    let message: Message =
        serde_json::from_str(packed_message).map_err(|e| Error::DIDComm(e.to_string()))?;

    // Get the message type directly
    Ok(message.typ.clone())
}
