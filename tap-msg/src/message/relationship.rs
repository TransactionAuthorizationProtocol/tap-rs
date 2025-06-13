//! Relationship confirmation message types for the Transaction Authorization Protocol.
//!
//! This module defines the ConfirmRelationship message type, which is used to confirm
//! relationships between agents in the TAP protocol.

use crate::error::{Error, Result};
use crate::TapMessage;
use serde::{Deserialize, Serialize};

/// ConfirmRelationship message body (TAIP-9).
///
/// This message type allows confirming a relationship between agents.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#ConfirmRelationship")]
pub struct ConfirmRelationship {
    /// ID of the transaction related to this message.
    #[serde(rename = "transfer_id")]
    #[tap(thread_id)]
    pub transaction_id: String,

    /// DID of the agent whose relationship is being confirmed.
    #[serde(rename = "@id")]
    pub agent_id: String,

    /// The entity this agent is acting for.
    #[serde(rename = "for")]
    pub for_entity: String,

    /// The role of the agent (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

impl ConfirmRelationship {
    /// Creates a new ConfirmRelationship message body.
    pub fn new(transaction_id: &str, agent_id: &str, for_entity: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            agent_id: agent_id.to_string(),
            for_entity: for_entity.to_string(),
            role: None,
        }
    }

    /// Add a role to the confirmation.
    pub fn with_role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
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
                "Agent ID (@id) is required in ConfirmRelationship".to_string(),
            ));
        }

        if self.for_entity.is_empty() {
            return Err(Error::Validation(
                "For entity is required in ConfirmRelationship".to_string(),
            ));
        }

        Ok(())
    }
}
