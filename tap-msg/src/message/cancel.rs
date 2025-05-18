//! Cancel message type for the Transaction Authorization Protocol.
//!
//! This module defines the Cancel message type, which is used
//! for canceling transactions in the TAP protocol.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::tap_message_trait::TapMessageBody;

/// Cancel message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cancel {
    /// ID of the transfer being cancelled.
    pub transaction_id: String,

    /// Optional reason for cancellation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Cancel {
    /// Create a new Cancel message
    pub fn new(transaction_id: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            reason: None,
            note: None,
        }
    }

    /// Create a new Cancel message with a reason
    pub fn with_reason(transaction_id: &str, reason: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            reason: Some(reason.to_string()),
            note: None,
        }
    }

    /// Create a new Cancel message with a reason and note
    pub fn with_reason_and_note(transaction_id: &str, reason: &str, note: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            reason: Some(reason.to_string()),
            note: Some(note.to_string()),
        }
    }
}

impl TapMessageBody for Cancel {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#cancel"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Cancel message must have a transaction_id".into(),
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

        // Create the message
        let message = PlainMessage {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: from_did.to_string(),
            to: Vec::new(),
            thid: Some(self.transaction_id.clone()),
            pthid: None,
            created_time: Some(Utc::now().timestamp() as u64),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
        };

        Ok(message)
    }
}

impl_tap_message!(Cancel);
