//! Error message type for the Transaction Authorization Protocol.
//!
//! This module defines the ErrorBody message type, which is used
//! to communicate errors in the TAP protocol.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::tap_message_trait::TapMessageBody;

/// Error message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Error code.
    pub code: String,

    /// Error description.
    pub description: String,

    /// Original message ID (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_message_id: Option<String>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ErrorBody {
    /// Creates a new ErrorBody message.
    pub fn new(code: &str, description: &str) -> Self {
        Self {
            code: code.to_string(),
            description: description.to_string(),
            original_message_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Creates a new ErrorBody message with a reference to the original message.
    pub fn with_original_message(code: &str, description: &str, original_message_id: &str) -> Self {
        Self {
            code: code.to_string(),
            description: description.to_string(),
            original_message_id: Some(original_message_id.to_string()),
            metadata: HashMap::new(),
        }
    }

    /// Adds metadata to the error message.
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

impl TapMessageBody for ErrorBody {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#error"
    }

    fn validate(&self) -> Result<()> {
        if self.code.is_empty() {
            return Err(Error::Validation(
                "Error code is required in ErrorBody".to_string(),
            ));
        }

        if self.description.is_empty() {
            return Err(Error::Validation(
                "Error description is required in ErrorBody".to_string(),
            ));
        }

        Ok(())
    }

    fn to_didcomm(&self, from_did: &str) -> Result<PlainMessage> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // The from field is required in our PlainMessage
        let from = from_did.to_string();

        // If we have an original message ID, use it as the thread ID
        let thid = self.original_message_id.clone();

        // Create the message
        let message = PlainMessage {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from,
            to: Vec::new(), // Empty recipients
            thid,
            pthid: None,
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
        };

        Ok(message)
    }
}

impl_tap_message!(ErrorBody, generated_id);
