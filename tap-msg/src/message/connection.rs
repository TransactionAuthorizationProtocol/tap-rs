//! Connection types for TAP messages.
//!
//! This module defines the structure of connection messages and related types
//! used in the Transaction Authorization Protocol (TAP).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::message::agent::TapParticipant;
use crate::message::tap_message_trait::{TapMessage as TapMessageTrait, TapMessageBody};
use crate::message::{Agent, Party};
use crate::TapMessage;

/// Agent structure specific to Connect messages.
/// Unlike regular agents, Connect agents don't require a "for" field
/// because the principal is specified separately in the Connect message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectAgent {
    /// DID of the agent.
    #[serde(rename = "@id")]
    pub id: String,

    /// Name of the agent (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Type of the agent (optional).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,

    /// Service URL for the agent (optional).
    #[serde(rename = "serviceUrl", skip_serializing_if = "Option::is_none")]
    pub service_url: Option<String>,

    /// Additional metadata.
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TapParticipant for ConnectAgent {
    fn id(&self) -> &str {
        &self.id
    }
}

impl ConnectAgent {
    /// Create a new ConnectAgent with just an ID.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: None,
            agent_type: None,
            service_url: None,
            metadata: HashMap::new(),
        }
    }

    /// Convert to a regular Agent by adding a for_party.
    pub fn to_agent(&self, for_party: &str) -> Agent {
        let mut agent = Agent::new_without_role(&self.id, for_party);

        // Copy metadata fields
        if let Some(name) = &self.name {
            agent
                .metadata
                .insert("name".to_string(), serde_json::Value::String(name.clone()));
        }
        if let Some(agent_type) = &self.agent_type {
            agent.metadata.insert(
                "type".to_string(),
                serde_json::Value::String(agent_type.clone()),
            );
        }
        if let Some(service_url) = &self.service_url {
            agent.metadata.insert(
                "serviceUrl".to_string(),
                serde_json::Value::String(service_url.clone()),
            );
        }

        // Copy any additional metadata
        for (k, v) in &self.metadata {
            agent.metadata.insert(k.clone(), v.clone());
        }

        agent
    }
}

/// Transaction limits for connection constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLimits {
    /// Maximum amount per transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_transaction: Option<String>,

    /// Maximum daily amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily: Option<String>,

    /// Currency for the limits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

/// Connection constraints for the Connect message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConstraints {
    /// Allowed purposes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purposes: Option<Vec<String>>,

    /// Allowed category purposes.
    #[serde(rename = "categoryPurposes", skip_serializing_if = "Option::is_none")]
    pub category_purposes: Option<Vec<String>>,

    /// Transaction limits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<TransactionLimits>,
}

/// Connect message body (TAIP-2).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(
    message_type = "https://tap.rsvp/schema/1.0#Connect",
    initiator,
    authorizable
)]
pub struct Connect {
    /// Transaction ID (only available after creation).
    #[serde(skip)]
    #[tap(transaction_id)]
    pub transaction_id: Option<String>,

    /// Agent DID (kept for backward compatibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    /// Agent object containing agent details.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tap(participant)]
    pub agent: Option<ConnectAgent>,

    /// Principal party this connection is for.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tap(participant)]
    pub principal: Option<Party>,

    /// The entity this connection is for (kept for backward compatibility).
    #[serde(rename = "for", skip_serializing_if = "Option::is_none", default)]
    pub for_: Option<String>,

    /// The role of the agent (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Connection constraints (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<ConnectionConstraints>,
}

impl Connect {
    /// Create a new Connect message (backward compatible).
    pub fn new(transaction_id: &str, agent_id: &str, for_id: &str, role: Option<&str>) -> Self {
        Self {
            transaction_id: Some(transaction_id.to_string()),
            agent_id: Some(agent_id.to_string()),
            agent: None,
            principal: None,
            for_: Some(for_id.to_string()),
            role: role.map(|s| s.to_string()),
            constraints: None,
        }
    }

    /// Create a new Connect message with Agent and Principal.
    pub fn new_with_agent_and_principal(
        transaction_id: &str,
        agent: ConnectAgent,
        principal: Party,
    ) -> Self {
        Self {
            transaction_id: Some(transaction_id.to_string()),
            agent_id: None,
            agent: Some(agent),
            principal: Some(principal),
            for_: None,
            role: None,
            constraints: None,
        }
    }

    /// Add constraints to the Connect message.
    pub fn with_constraints(mut self, constraints: ConnectionConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }
}

impl Connect {
    /// Custom validation for Connect messages
    pub fn validate_connect(&self) -> Result<()> {
        // transaction_id is optional for initiator messages
        // It will be set when creating the DIDComm message

        // Either agent_id or agent must be present
        if self.agent_id.is_none() && self.agent.is_none() {
            return Err(Error::Validation(
                "either agent_id or agent is required".to_string(),
            ));
        }

        // Either for_ or principal must be present and non-empty
        let for_empty = self.for_.as_ref().is_none_or(|s| s.is_empty());
        if for_empty && self.principal.is_none() {
            return Err(Error::Validation(
                "either for or principal is required".to_string(),
            ));
        }

        // Constraints are required for Connect messages
        if self.constraints.is_none() {
            return Err(Error::Validation(
                "Connection request must include constraints".to_string(),
            ));
        }

        Ok(())
    }

    /// Validation method that will be called by TapMessageBody trait
    pub fn validate(&self) -> Result<()> {
        self.validate_connect()
    }
}

/// Out of Band invitation for TAP connections.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#OutOfBand")]
pub struct OutOfBand {
    /// The goal code for this invitation.
    pub goal_code: String,

    /// The goal for this invitation.
    pub goal: String,

    /// The public DID or endpoint URL for the inviter.
    pub service: String,

    /// Accept media types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept: Option<Vec<String>>,

    /// Handshake protocols supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handshake_protocols: Option<Vec<String>>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl OutOfBand {
    /// Create a new OutOfBand message.
    pub fn new(goal_code: String, goal: String, service: String) -> Self {
        Self {
            goal_code,
            goal,
            service,
            accept: None,
            handshake_protocols: None,
            metadata: HashMap::new(),
        }
    }
}

/// Authorization Required message body.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#AuthorizationRequired")]
pub struct AuthorizationRequired {
    /// Authorization URL.
    #[serde(rename = "authorization_url")]
    pub url: String,

    /// Agent ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    /// Expiry date/time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AuthorizationRequired {
    /// Create a new AuthorizationRequired message.
    pub fn new(url: String, expires: String) -> Self {
        Self {
            url,
            agent_id: None,
            expires: Some(expires),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the message.
    pub fn add_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

impl OutOfBand {
    /// Custom validation for OutOfBand messages
    pub fn validate_out_of_band(&self) -> Result<()> {
        if self.goal_code.is_empty() {
            return Err(Error::Validation("Goal code is required".to_string()));
        }

        if self.service.is_empty() {
            return Err(Error::Validation("Service is required".to_string()));
        }

        Ok(())
    }

    /// Validation method that will be called by TapMessageBody trait
    pub fn validate(&self) -> Result<()> {
        self.validate_out_of_band()
    }
}

impl AuthorizationRequired {
    /// Custom validation for AuthorizationRequired messages
    pub fn validate_authorization_required(&self) -> Result<()> {
        if self.url.is_empty() {
            return Err(Error::Validation(
                "Authorization URL is required".to_string(),
            ));
        }

        // Validate expiry date if present
        if let Some(expires) = &self.expires {
            // Simple format check
            if !expires.contains('T') || !expires.contains(':') {
                return Err(Error::Validation(
                    "Invalid expiry date format. Expected ISO8601/RFC3339 format".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validation method that will be called by TapMessageBody trait
    pub fn validate(&self) -> Result<()> {
        self.validate_authorization_required()
    }
}
