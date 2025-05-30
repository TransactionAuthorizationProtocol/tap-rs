//! Relationship confirmation message types for the Transaction Authorization Protocol.
//!
//! This module defines the ConfirmRelationship message type, which is used to confirm
//! relationships between agents in the TAP protocol.

use crate::error::{Error, Result};
use crate::{TapMessage, TapMessageBody};
use serde::{Deserialize, Serialize};

/// ConfirmRelationship message body (TAIP-9).
///
/// This message type allows confirming a relationship between agents.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage, TapMessageBody)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#ConfirmRelationship")]
pub struct ConfirmRelationship {
    /// ID of the transaction related to this message.
    #[serde(rename = "transfer_id")]
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// DID of the agent whose relationship is being confirmed.
    pub agent_id: String,

    /// Type of relationship being confirmed (e.g., "agent_for", "custodian", etc.).
    pub relationship_type: String,
}

impl ConfirmRelationship {
    /// Creates a new ConfirmRelationship message body.
    pub fn new(transaction_id: &str, agent_id: &str, relationship_type: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            agent_id: agent_id.to_string(),
            relationship_type: relationship_type.to_string(),
        }
    }
}

// Note: The validate() method name conflicts with auto-generated validate,
// so we rename the custom one
impl ConfirmRelationship {
    /// Custom validation for ConfirmRelationship messages
    pub fn validate_relationship(&self) -> Result<()> {
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

        if self.relationship_type.is_empty() {
            return Err(Error::Validation(
                "Relationship type is required in ConfirmRelationship".to_string(),
            ));
        }

        Ok(())
    }
}
