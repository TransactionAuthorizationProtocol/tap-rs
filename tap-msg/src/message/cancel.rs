//! Cancel message type for the Transaction Authorization Protocol.
//!
//! This module defines the Cancel message type, which is used
//! for canceling transactions in the TAP protocol.

use crate::error::{Error, Result};
use crate::{TapMessage, TapMessageBody};
use serde::{Deserialize, Serialize};

/// Cancel message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage, TapMessageBody)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#cancel")]
pub struct Cancel {
    /// ID of the transfer being cancelled.
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// Optional reason for cancellation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Cancel {
    /// Create a new Cancel message
    pub fn new(transaction_id: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            reason: None,
            note: None,
        }
    }

    /// Create a new Cancel message with a reason
    pub fn with_reason(transaction_id: &str, reason: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            reason: Some(reason.to_string()),
            note: None,
        }
    }

    /// Create a new Cancel message with a reason and note
    pub fn with_reason_and_note(transaction_id: &str, reason: &str, note: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            reason: Some(reason.to_string()),
            note: Some(note.to_string()),
        }
    }
}

impl Cancel {
    /// Custom validation for Cancel messages
    pub fn validate_cancel(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Cancel message must have a transaction_id".into(),
            ));
        }
        Ok(())
    }
}
