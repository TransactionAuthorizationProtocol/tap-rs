//! Relationship confirmation message types for the Transaction Authorization Protocol.
//!
//! This module defines the ConfirmRelationship message type, which is used to confirm
//! relationships between agents in the TAP protocol.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::tap_message_trait::TapMessageBody;

/// ConfirmRelationship message body (TAIP-9).
///
/// This message type allows confirming a relationship between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmRelationship {
    /// ID of the transaction related to this message.
    #[serde(rename = "transfer_id")]
    pub transaction_id: String,

    /// DID of the agent whose relationship is being confirmed.
    pub agent_id: String,

    /// DID of the entity that the agent acts on behalf of.
    #[serde(rename = "for")]
    pub for_id: String,

    /// Role of the agent in the transaction (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

impl ConfirmRelationship {
    /// Creates a new ConfirmRelationship message body.
    pub fn new(transaction_id: &str, agent_id: &str, for_id: &str, role: Option<String>) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            agent_id: agent_id.to_string(),
            for_id: for_id.to_string(),
            role,
        }
    }

    /// Validates the ConfirmRelationship message body.
    pub fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transfer ID is required in ConfirmRelationship".to_string(),
            ));
        }

        if self.agent_id.is_empty() {
            return Err(Error::Validation(
                "Agent ID is required in ConfirmRelationship".to_string(),
            ));
        }

        if self.for_id.is_empty() {
            return Err(Error::Validation(
                "For ID is required in ConfirmRelationship".to_string(),
            ));
        }

        Ok(())
    }
}

impl TapMessageBody for ConfirmRelationship {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#confirmrelationship"
    }

    fn validate(&self) -> Result<()> {
        self.validate()
    }

    fn to_didcomm(&self, from_did: &str) -> Result<PlainMessage> {
        // 1. Serialize self to JSON value
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // 2. Add/ensure '@type' field
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
            // Note: serde handles #[serde(rename = "for")] automatically during serialization
        }

        // 3. Generate ID and timestamp
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp_millis() as u64;

        // The from field is required in our PlainMessage
        let from = from_did.to_string();

        // 4. Create the Message struct with empty recipients (since test expects empty 'to')
        let message = PlainMessage {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from,
            to: vec![],
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

impl_tap_message!(ConfirmRelationship);
