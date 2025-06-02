//! Reject message type for the Transaction Authorization Protocol.
//!
//! This module defines the Reject message type, which is used
//! for rejecting transactions in the TAP protocol.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::TapMessage;

/// Reject message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#Reject")]
pub struct Reject {
    /// ID of the transaction being rejected.
    #[tap(thread_id)]
    pub transaction_id: String,

    /// Reason for rejection.
    pub reason: String,
}

impl Reject {
    /// Create a new Reject message
    pub fn new(transaction_id: &str, reason: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            reason: reason.to_string(),
        }
    }
}

impl Reject {
    /// Custom validation for Reject messages
    pub fn validate_reject(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transaction ID is required in Reject".to_string(),
            ));
        }

        if self.reason.is_empty() {
            return Err(Error::Validation(
                "Reason is required in Reject".to_string(),
            ));
        }

        Ok(())
    }
}
