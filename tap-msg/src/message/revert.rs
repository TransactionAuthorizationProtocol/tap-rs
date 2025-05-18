//! Revert message type for the Transaction Authorization Protocol.
//!
//! This module defines the Revert message type, which is used
//! for requesting reversal of settled transactions in the TAP protocol.

use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::tap_message_trait::TapMessageBody;

/// Revert message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revert {
    /// ID of the transfer being reverted.
    pub transaction_id: String,

    /// Settlement address in CAIP-10 format to return the funds to.
    pub settlement_address: String,

    /// Reason for the reversal request.
    pub reason: String,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Revert {
    /// Create a new Revert message
    pub fn new(transaction_id: &str, settlement_address: &str, reason: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            settlement_address: settlement_address.to_string(),
            reason: reason.to_string(),
            note: None,
        }
    }

    /// Create a new Revert message with a note
    pub fn with_note(transaction_id: &str, settlement_address: &str, reason: &str, note: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            settlement_address: settlement_address.to_string(),
            reason: reason.to_string(),
            note: Some(note.to_string()),
        }
    }
}

impl TapMessageBody for Revert {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#revert"
    }

    fn validate(&self) -> Result<()> {
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

    fn to_didcomm(&self, from_did: &str) -> Result<PlainMessage> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // The from field is required in our PlainMessage
        let from = from_did.to_string();

        // Create the message
        let message = PlainMessage {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from,
            to: Vec::new(), // Empty recipients
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

impl_tap_message!(Revert);