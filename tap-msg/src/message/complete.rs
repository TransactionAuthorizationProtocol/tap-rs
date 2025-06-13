//! Complete message type for the Transaction Authorization Protocol.
//!
//! This module defines the Complete message type, which is used
//! for completing payment transactions in the TAP protocol.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::TapMessage;

/// Complete message body (TAIP-14).
/// 
/// Used to indicate completion of a payment transaction.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#Complete")]
pub struct Complete {
    /// ID of the payment being completed.
    #[tap(thread_id)]
    pub transaction_id: String,

    /// Settlement address (CAIP-10 format) where payment was sent.
    #[serde(rename = "settlementAddress")]
    pub settlement_address: String,

    /// Optional amount completed. If specified, must be less than or equal to the original amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
}

impl Complete {
    /// Create a new Complete message
    pub fn new(transaction_id: &str, settlement_address: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            settlement_address: settlement_address.to_string(),
            amount: None,
        }
    }

    /// Create a new Complete message with an amount
    pub fn with_amount(transaction_id: &str, settlement_address: &str, amount: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            settlement_address: settlement_address.to_string(),
            amount: Some(amount.to_string()),
        }
    }
}

impl Complete {
    /// Custom validation for Complete messages
    pub fn validate_complete(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transaction ID is required in Complete".to_string(),
            ));
        }

        if self.settlement_address.is_empty() {
            return Err(Error::Validation(
                "Settlement address is required in Complete".to_string(),
            ));
        }

        // Validate settlement address format (basic CAIP-10 check)
        if !self.settlement_address.contains(':') {
            return Err(Error::Validation(
                "Settlement address must be in CAIP-10 format".to_string(),
            ));
        }

        if let Some(amount) = &self.amount {
            if amount.is_empty() {
                return Err(Error::Validation(
                    "Amount cannot be empty when provided".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validation method that will be called by TapMessageBody trait
    pub fn validate(&self) -> Result<()> {
        self.validate_complete()
    }
}