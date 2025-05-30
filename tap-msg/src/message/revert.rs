//! Revert message type for the Transaction Authorization Protocol.
//!
//! This module defines the Revert message type, which is used
//! for requesting reversal of settled transactions in the TAP protocol.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::{TapMessage, TapMessageBody};

/// Revert message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage, TapMessageBody)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#Revert")]
pub struct Revert {
    /// ID of the transfer being reverted.
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// Settlement address in CAIP-10 format to return the funds to.
    pub settlement_address: String,

    /// Reason for the reversal request.
    pub reason: String,
}

impl Revert {
    /// Create a new Revert message
    pub fn new(transaction_id: &str, settlement_address: &str, reason: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            settlement_address: settlement_address.to_string(),
            reason: reason.to_string(),
        }
    }
}

impl Revert {
    /// Custom validation for Revert messages
    pub fn validate_revert(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transaction ID is required in Revert".to_string(),
            ));
        }

        if self.settlement_address.is_empty() {
            return Err(Error::Validation(
                "Settlement address is required in Revert".to_string(),
            ));
        }

        if self.reason.is_empty() {
            return Err(Error::Validation(
                "Reason is required in Revert".to_string(),
            ));
        }

        Ok(())
    }
}
