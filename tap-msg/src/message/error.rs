//! Error message type for the Transaction Authorization Protocol.
//!
//! This module defines the ErrorBody message type, which is used
//! to communicate errors in the TAP protocol.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::TapMessage;

/// Error message body.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(generated_id)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#Error")]
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

impl ErrorBody {
    /// Custom validation for ErrorBody messages
    pub fn validate_error(&self) -> Result<()> {
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
}
