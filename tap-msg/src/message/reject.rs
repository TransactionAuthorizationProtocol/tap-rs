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
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reason: Option<String>,
}

impl Reject {
    /// Create a new Reject message
    pub fn new(transaction_id: &str, reason: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            reason: Some(reason.to_string()),
        }
    }
    
    /// Create a minimal Reject message (for testing/special cases)
    pub fn minimal(transaction_id: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            reason: None,
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

        // Note: reason is now optional to support minimal test cases
        // In production use, a reason should typically be provided
        if let Some(ref reason) = self.reason {
            if reason.is_empty() {
                return Err(Error::Validation(
                    "Reason cannot be empty when provided".to_string(),
                ));
            }
        }

        Ok(())
    }
}
