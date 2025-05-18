//! Agent management message types for the Transaction Authorization Protocol.
//!
//! This module defines the message types for managing agents in the TAP protocol,
//! including adding, replacing, and removing agents from transactions.

use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::Participant;
use crate::message::tap_message_trait::TapMessageBody;

/// Add agents message body (TAIP-5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAgents {
    /// ID of the transaction to add agents to.
    #[serde(rename = "transfer_id")]
    pub transaction_id: String,

    /// Agents to add.
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
        "https://tap.rsvp/schema/1.0#add-agents"
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

        // Create the message
        let message = PlainMessage {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.to_string(),
            to: Vec::new(), // Empty recipients, will be determined by the framework later
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

impl_tap_message!(AddAgents);

/// Replace agent message body (TAIP-5).
///
/// This message type allows replacing an agent with another agent in a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceAgent {
    /// ID of the transaction to replace agent in.
    #[serde(rename = "transfer_id")]
    pub transaction_id: String,

    /// DID of the original agent to replace.
    pub original: String,

    /// Replacement agent.
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
        "https://tap.rsvp/schema/1.0#replace-agent"
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

        // Create the message
        let message = PlainMessage {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.to_string(),
            to: Vec::new(), // Empty recipients, will be determined by the framework later
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

impl_tap_message!(ReplaceAgent);

/// Remove agent message body (TAIP-5).
///
/// This message type allows removing an agent from a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAgent {
    /// ID of the transaction to remove agent from.
    #[serde(rename = "transfer_id")]
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
        "https://tap.rsvp/schema/1.0#remove-agent"
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

        // Create the message
        let message = PlainMessage {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.to_string(),
            to: Vec::new(), // Empty recipients, will be determined by the framework later
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

impl_tap_message!(RemoveAgent);