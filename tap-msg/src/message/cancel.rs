//! Cancel message type for the Transaction Authorization Protocol.
//!
//! This module defines the Cancel message type, which is used
//! for canceling transactions in the TAP protocol.

use crate::error::{Error, Result};
use crate::{TapMessage, TapMessageBody};
use serde::{Deserialize, Serialize};

/// Cancel message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage, TapMessageBody)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#Cancel")]
pub struct Cancel {
    /// ID of the transfer being cancelled.
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// The party of the transaction wishing to cancel it.
    /// (In case of a Transfer [TAIP3] `originator` or `beneficiary`)
    pub by: String,

    /// Optional reason for cancellation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl Cancel {
    /// Create a new Cancel message
    pub fn new(transaction_id: &str, by: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            by: by.to_string(),
            reason: None,
        }
    }

    /// Create a new Cancel message with a reason
    pub fn with_reason(transaction_id: &str, by: &str, reason: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            by: by.to_string(),
            reason: Some(reason.to_string()),
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
        if self.by.is_empty() {
            return Err(Error::Validation(
                "Cancel message must specify 'by' field".into(),
            ));
        }
        Ok(())
    }
}
