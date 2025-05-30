//! Agent management message types for the Transaction Authorization Protocol.
//!
//! This module defines the message types for managing agents in the TAP protocol,
//! including adding, replacing, and removing agents from transactions.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::message::Participant;
use crate::{TapMessage, TapMessageBody};

/// Add agents message body (TAIP-5).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
pub struct AddAgents {
    /// ID of the transaction to add agents to.
    #[serde(rename = "transfer_id")]
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// Agents to add.
    #[tap(participant_list)]
    pub agents: Vec<Participant>,
}

impl AddAgents {
    /// Create a new AddAgents message
    pub fn new(transaction_id: &str, agents: Vec<Participant>) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            agents,
        }
    }

    /// Add a single agent to this message
    pub fn add_agent(mut self, agent: Participant) -> Self {
        self.agents.push(agent);
        self
    }
}

impl TapMessageBody for AddAgents {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#AddAgents"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transaction ID is required in AddAgents".to_string(),
            ));
        }

        if self.agents.is_empty() {
            return Err(Error::Validation(
                "At least one agent must be specified in AddAgents".to_string(),
            ));
        }

        // Validate each agent
        for agent in &self.agents {
            if agent.id.is_empty() {
                return Err(Error::Validation("Agent ID cannot be empty".to_string()));
            }
        }

        Ok(())
    }
}

/// Replace agent message body (TAIP-5).
///
/// This message type allows replacing an agent with another agent in a transaction.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
pub struct ReplaceAgent {
    /// ID of the transaction to replace agent in.
    #[serde(rename = "transfer_id")]
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// DID of the original agent to replace.
    pub original: String,

    /// Replacement agent.
    #[tap(participant)]
    pub replacement: Participant,
}

impl ReplaceAgent {
    /// Create a new ReplaceAgent message
    pub fn new(transaction_id: &str, original: &str, replacement: Participant) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            original: original.to_string(),
            replacement,
        }
    }
}

impl TapMessageBody for ReplaceAgent {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#ReplaceAgent"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transaction ID is required in ReplaceAgent".to_string(),
            ));
        }

        if self.original.is_empty() {
            return Err(Error::Validation(
                "Original agent ID is required in ReplaceAgent".to_string(),
            ));
        }

        if self.replacement.id.is_empty() {
            return Err(Error::Validation(
                "Replacement agent ID is required in ReplaceAgent".to_string(),
            ));
        }

        Ok(())
    }
}

/// Remove agent message body (TAIP-5).
///
/// This message type allows removing an agent from a transaction.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
pub struct RemoveAgent {
    /// ID of the transaction to remove agent from.
    #[serde(rename = "transfer_id")]
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// DID of the agent to remove.
    pub agent: String,
}

impl RemoveAgent {
    /// Create a new RemoveAgent message
    pub fn new(transaction_id: &str, agent: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            agent: agent.to_string(),
        }
    }
}

impl TapMessageBody for RemoveAgent {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#RemoveAgent"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transaction ID is required in RemoveAgent".to_string(),
            ));
        }

        if self.agent.is_empty() {
            return Err(Error::Validation(
                "Agent ID is required in RemoveAgent".to_string(),
            ));
        }

        Ok(())
    }
}
