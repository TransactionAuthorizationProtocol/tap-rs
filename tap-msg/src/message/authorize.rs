//! Authorize message type for the Transaction Authorization Protocol.
//!
//! This module defines the Authorize message type, which is used
//! for authorizing transactions in the TAP protocol.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::TapMessage;

/// Authorize message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#Authorize")]
pub struct Authorize {
    /// ID of the transaction being authorized.
    #[tap(thread_id)]
    pub transaction_id: String,

    /// Optional settlement address in CAIP-10 format.
    /// Required when sent by a VASP representing the beneficiary unless the original
    /// request contains an agent with the settlementAddress role.
    #[serde(rename = "settlementAddress", skip_serializing_if = "Option::is_none")]
    pub settlement_address: Option<String>,

    /// Optional expiry timestamp in ISO 8601 format.
    /// After this time, if settlement has not occurred, the authorization should be
    /// considered invalid and settlement should not proceed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
}

impl Authorize {
    /// Create a new Authorize message
    pub fn new(transaction_id: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            settlement_address: None,
            expiry: None,
        }
    }

    /// Create a new Authorize message with a settlement address
    pub fn with_settlement_address(transaction_id: &str, settlement_address: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            settlement_address: Some(settlement_address.to_string()),
            expiry: None,
        }
    }

    /// Create a new Authorize message with all optional fields
    pub fn with_all(
        transaction_id: &str,
        settlement_address: Option<&str>,
        expiry: Option<&str>,
    ) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            settlement_address: settlement_address.map(|s| s.to_string()),
            expiry: expiry.map(|s| s.to_string()),
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
