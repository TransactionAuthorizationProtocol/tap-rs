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
#[tap(message_type = "https://tap.rsvp/schema/1.0#confirmrelationship")]
pub struct ConfirmRelationship {
    /// ID of the transaction related to this message.
    #[serde(rename = "transfer_id")]
    #[tap(transaction_id)]
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

        if self.for_id.is_empty() {
            return Err(Error::Validation(
                "For ID is required in ConfirmRelationship".to_string(),
            ));
        }

        Ok(())
    }
}
