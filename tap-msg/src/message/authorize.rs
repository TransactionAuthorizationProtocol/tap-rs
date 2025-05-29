//! Authorize message type for the Transaction Authorization Protocol.
//!
//! This module defines the Authorize message type, which is used
//! for authorizing transactions in the TAP protocol.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::{TapMessage, TapMessageBody};

/// Authorize message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage, TapMessageBody)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#authorize")]
pub struct Authorize {
    /// ID of the transaction being authorized.
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Authorize {
    /// Create a new Authorize message
    pub fn new(transaction_id: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            note: None,
        }
    }

    /// Create a new Authorize message with a note
    pub fn with_note(transaction_id: &str, note: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            note: Some(note.to_string()),
        }
    }
}

impl Authorize {
    /// Custom validation for Authorize messages
    pub fn validate_authorize(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transaction ID is required in Authorize".to_string(),
            ));
        }

        Ok(())
    }
}
