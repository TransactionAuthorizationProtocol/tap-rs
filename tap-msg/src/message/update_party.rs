//! Update Party message type for the Transaction Authorization Protocol.
//!
//! This module defines the UpdateParty message type, which is used to update
//! party information in an existing transaction.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::tap_message_trait::TapMessageBody;
use crate::message::Participant;

/// UpdateParty message body (TAIP-6).
///
/// This message type allows agents to update party information in a transaction.
/// It enables a participant to modify their details or role within an existing transfer without
/// creating a new transaction. This is particularly useful for situations where participant
/// information changes during the lifecycle of a transaction.
///
/// # TAIP-6 Specification
/// The UpdateParty message follows the TAIP-6 specification for updating party information
/// in a TAP transaction. It includes JSON-LD compatibility with an optional @context field.
///
/// # Example
/// ```
/// use tap_msg::message::update_party::UpdateParty;
/// use tap_msg::message::Participant;
/// use std::collections::HashMap;
///
/// // Create a participant with updated information
/// let updated_participant = Participant {
///     id: "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx".to_string(),
///     role: Some("new_role".to_string()),
///     policies: None,
///     leiCode: None,
/// };
///
/// // Create an UpdateParty message
/// let update_party = UpdateParty::new(
///     "transfer-123",
///     "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx",
///     updated_participant
/// );
///
/// // Add an optional note
/// let update_party_with_note = UpdateParty {
///     note: Some("Updating role after compliance check".to_string()),
///     ..update_party
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateParty {
    /// ID of the transaction this update relates to.
    pub transaction_id: String,

    /// Type of party being updated (e.g., 'originator', 'beneficiary').
    #[serde(rename = "partyType")]
    pub party_type: String,

    /// Updated party information.
    pub party: Participant,

    /// Optional note regarding the update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Optional context for the update.
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl UpdateParty {
    /// Creates a new UpdateParty message body.
    pub fn new(transaction_id: &str, party_type: &str, party: Participant) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            party_type: party_type.to_string(),
            party,
            note: None,
            context: None,
        }
    }

    /// Creates a new UpdateParty message body with a note.
    pub fn with_note(
        transaction_id: &str,
        party_type: &str,
        party: Participant,
        note: &str,
    ) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            party_type: party_type.to_string(),
            party,
            note: Some(note.to_string()),
            context: None,
        }
    }

    /// Validates the UpdateParty message body.
    pub fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "transaction_id cannot be empty".to_string(),
            ));
        }

        if self.party_type.is_empty() {
            return Err(Error::Validation("partyType cannot be empty".to_string()));
        }

        if self.party.id.is_empty() {
            return Err(Error::Validation("party.id cannot be empty".to_string()));
        }

        Ok(())
    }
}

impl TapMessageBody for UpdateParty {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#update-party"
    }

    fn validate(&self) -> Result<()> {
        self.validate()
    }

    fn to_didcomm(&self, from_did: &str) -> Result<PlainMessage> {
        // Serialize the UpdateParty to a JSON value
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        let now = Utc::now().timestamp() as u64;

        // The from field is required in our PlainMessage
        let from = from_did.to_string();

        // Create a new Message with required fields
        let message = PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from,
            to: Vec::new(),
            thid: Some(self.transaction_id.clone()),
            pthid: None,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            attachments: None,
        };

        Ok(message)
    }
}

impl_tap_message!(UpdateParty);
