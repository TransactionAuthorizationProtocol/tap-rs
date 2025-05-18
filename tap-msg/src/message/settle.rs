//! Settle message type for the Transaction Authorization Protocol.
//!
//! This module defines the Settle message type, which is used
//! for settling transactions in the TAP protocol.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::tap_message_trait::TapMessageBody;

/// Settle message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settle {
    /// ID of the transaction being settled.
    pub transaction_id: String,

    /// Settlement ID (CAIP-220 identifier of the underlying settlement transaction).
    pub settlement_id: String,

    /// Optional amount settled. If specified, must be less than or equal to the original amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
}

impl Settle {
    /// Create a new Settle message
    pub fn new(transaction_id: &str, settlement_id: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            settlement_id: settlement_id.to_string(),
            amount: None,
        }
    }

    /// Create a new Settle message with an amount
    pub fn with_amount(transaction_id: &str, settlement_id: &str, amount: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            settlement_id: settlement_id.to_string(),
            amount: Some(amount.to_string()),
        }
    }
}

impl TapMessageBody for Settle {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#settle"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transaction ID is required in Settle".to_string(),
            ));
        }

        if self.settlement_id.is_empty() {
            return Err(Error::Validation(
                "Settlement ID is required in Settle".to_string(),
            ));
        }

        if let Some(amount) = &self.amount {
            if amount.is_empty() {
                return Err(Error::Validation(
                    "Amount must be a valid number".to_string(),
                ));
            }
            
            // Validate amount is a positive number if provided
            match amount.parse::<f64>() {
                Ok(amount) if amount <= 0.0 => {
                    return Err(Error::Validation("Amount must be positive".to_string()));
                }
                Err(_) => {
                    return Err(Error::Validation(
                        "Amount must be a valid number".to_string(),
                    ));
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<PlainMessage> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // The from field is required in our PlainMessage, so ensure we have a valid value
        let from = from_did.map_or_else(String::new, |s| s.to_string());

        // Create the message
        let message = PlainMessage {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from,
            to: Vec::new(), // Empty recipients, will be determined by the framework later
            thid: Some(self.transaction_id.clone()),
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

impl_tap_message!(Settle);