//! Validation logic for TAP messages.
//!
//! This module provides functionality to validate TAP messages according to the specification.

use crate::error::{Error, Result};
use crate::message::tap_message_trait::TapMessageBody;
use crate::message::types::{
    AddAgents, Authorize, DIDCommPresentation, ErrorBody, Presentation, Reject, Settle, Transfer,
};
use didcomm::Message;
use serde_json::Value;

/// Validate a didcomm message's contents based on its type
pub fn validate_message(message: &Message) -> Result<()> {
    // Basic validation for DIDComm message
    if message.id.is_empty() {
        return Err(Error::Validation("Message ID is required".to_string()));
    }

    // Use the type field to determine which validation to apply
    let message_type = &message.type_;

    // If there's a body, validate it based on the message type
    validate_message_body(message_type, &message.body)?;

    Ok(())
}

/// Validate a message body based on its type
/// Used for validation without constructing a full Message object
pub fn validate_message_body(message_type: &str, body: &Value) -> Result<()> {
    match message_type {
        "https://tap.rsvp/schema/1.0#transfer" => {
            let transfer: Transfer = serde_json::from_value(body.clone())
                .map_err(|e| Error::SerializationError(e.to_string()))?;
            transfer.validate()
        }
        "https://tap.rsvp/schema/1.0#presentation" => {
            let presentation: Presentation = serde_json::from_value(body.clone())
                .map_err(|e| Error::SerializationError(e.to_string()))?;
            presentation.validate()
        }
        "https://didcomm.org/present-proof/3.0/presentation" => {
            // Handle the standard DIDComm present-proof protocol format
            let didcomm_presentation: DIDCommPresentation = serde_json::from_value(body.clone())
                .map_err(|e| Error::SerializationError(e.to_string()))?;
            didcomm_presentation.validate()
        }
        "https://tap.rsvp/schema/1.0#authorize" => {
            let authorize: Authorize = serde_json::from_value(body.clone())
                .map_err(|e| Error::SerializationError(e.to_string()))?;
            authorize.validate()
        }
        "https://tap.rsvp/schema/1.0#reject" => {
            let reject: Reject = serde_json::from_value(body.clone())
                .map_err(|e| Error::SerializationError(e.to_string()))?;
            reject.validate()
        }
        "https://tap.rsvp/schema/1.0#settle" => {
            let settle: Settle = serde_json::from_value(body.clone())
                .map_err(|e| Error::SerializationError(e.to_string()))?;
            settle.validate()
        }
        "https://tap.rsvp/schema/1.0#addagents" => {
            let add_agents: AddAgents = serde_json::from_value(body.clone())
                .map_err(|e| Error::SerializationError(e.to_string()))?;
            add_agents.validate()
        }
        "https://tap.rsvp/schema/1.0#error" => {
            let error: ErrorBody = serde_json::from_value(body.clone())
                .map_err(|e| Error::SerializationError(e.to_string()))?;
            error.validate()
        }
        _ => {
            // For custom types, we don't have specific validation
            Ok(())
        }
    }
}
