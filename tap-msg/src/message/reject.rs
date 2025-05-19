//! Reject message type for the Transaction Authorization Protocol.
//!
//! This module defines the Reject message type, which is used
//! for rejecting transactions in the TAP protocol.

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::tap_message_trait::TapMessageBody;

/// Reject message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reject {
    /// ID of the transaction being rejected.
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

impl TapMessageBody for Reject {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#reject"
    }

    fn validate(&self) -> Result<()> {
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

    fn to_didcomm(&self, from_did: &str) -> Result<PlainMessage> {
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

        // The from field is required in our PlainMessage
        let from = from_did.to_string();

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
            attachments: None,
        };

        Ok(message)
    }
}

impl_tap_message!(Reject);
