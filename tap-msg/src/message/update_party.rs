//! Update Party message type for the Transaction Authorization Protocol.
//!
//! This module defines the UpdateParty message type, which is used to update
//! party information in an existing transaction.

use crate::error::{Error, Result};
use crate::message::agent::TapParticipant;
use crate::message::Party;
use crate::TapMessage;
use serde::{Deserialize, Serialize};

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
/// use tap_msg::message::Party;
/// use std::collections::HashMap;
///
/// // Create a party with updated information
/// let updated_party = Party::new("did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx")
///     .with_country("de");
///
/// // Create an UpdateParty message
/// let update_party = UpdateParty::new(
///     "transfer-123",
///     "originator",
///     updated_party
/// );
///
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#UpdateParty")]
pub struct UpdateParty {
    /// ID of the transaction this update relates to.
    #[tap(thread_id)]
    pub transaction_id: String,

    /// Type of party being updated (e.g., 'originator', 'beneficiary').
    #[serde(rename = "partyType")]
    pub party_type: String,

    /// Updated party information.
    #[tap(participant)]
    pub party: Party,

    /// Optional context for the update.
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl UpdateParty {
    /// Creates a new UpdateParty message body.
    pub fn new(transaction_id: &str, party_type: &str, party: Party) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            party_type: party_type.to_string(),
            party,
            context: None,
        }
    }

    /// Custom validation for UpdateParty messages
    pub fn validate_update_party(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "transaction_id cannot be empty".to_string(),
            ));
        }

        if self.party_type.is_empty() {
            return Err(Error::Validation("partyType cannot be empty".to_string()));
        }

        if self.party.id().is_empty() {
            return Err(Error::Validation("party.id cannot be empty".to_string()));
        }

        Ok(())
    }
}

impl UpdateParty {
    /// Implementation of the validate method for the TapMessageBody trait
    /// This delegates to the custom validation method
    pub fn validate(&self) -> Result<()> {
        self.validate_update_party()
    }
}
