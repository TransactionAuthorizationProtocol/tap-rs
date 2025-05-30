//! Connection types for TAP messages.
//!
//! This module defines the structure of connection messages and related types
//! used in the Transaction Authorization Protocol (TAP).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::message::tap_message_trait::{
    Authorizable, Connectable, TapMessage as TapMessageTrait, TapMessageBody,
};
use crate::message::{
    Authorize, Cancel, Participant, Policy, Reject, RemoveAgent, ReplaceAgent, Revert, Settle,
    UpdatePolicies,
};
use crate::TapMessage;

/// Transaction limits for connection constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLimits {
    /// Maximum amount for a transaction.
    pub max_amount: Option<String>,

    /// Maximum total amount for all transactions.
    pub max_total_amount: Option<String>,

    /// Maximum number of transactions allowed.
    pub max_transactions: Option<u64>,
}

/// Connection constraints for the Connect message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConstraints {
    /// Limit on transaction amount.
    pub transaction_limits: Option<TransactionLimits>,
}

/// Connect message body (TAIP-2).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
pub struct Connect {
    /// Transaction ID.
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// Agent DID.
    pub agent_id: String,

    /// The entity this connection is for.
    #[serde(rename = "for")]
    pub for_: String,

    /// The role of the agent (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Connection constraints (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<ConnectionConstraints>,
}

impl Connect {
    /// Create a new Connect message.
    pub fn new(transaction_id: &str, agent_id: &str, for_id: &str, role: Option<&str>) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            agent_id: agent_id.to_string(),
            for_: for_id.to_string(),
            role: role.map(|s| s.to_string()),
            constraints: None,
        }
    }

    /// Add constraints to the Connect message.
    pub fn with_constraints(mut self, constraints: ConnectionConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }
}

impl Connectable for Connect {
    fn with_connection(&mut self, _connect_id: &str) -> &mut Self {
        // Connect messages don't have a connection ID
        self
    }

    fn has_connection(&self) -> bool {
        false
    }

    fn connection_id(&self) -> Option<&str> {
        None
    }
}

impl TapMessageBody for Connect {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#connect"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation("transaction_id is required".to_string()));
        }
        if self.agent_id.is_empty() {
            return Err(Error::Validation("agent_id is required".to_string()));
        }
        if self.for_.is_empty() {
            return Err(Error::Validation("for is required".to_string()));
        }
        Ok(())
    }
}

impl Authorizable for Connect {
    fn authorize(
        &self,
        creator_did: &str,
        settlement_address: Option<&str>,
        expiry: Option<&str>,
        note: Option<&str>,
    ) -> crate::didcomm::PlainMessage<Authorize> {
        let authorize = Authorize::with_all(&self.transaction_id, settlement_address, expiry, note);
        // Create a PlainMessage from self first, then create the reply
        let original_message = self
            .to_didcomm(creator_did)
            .expect("Failed to create DIDComm message");
        original_message
            .create_reply(&authorize, creator_did)
            .expect("Failed to create reply")
    }

    fn settle(
        &self,
        creator_did: &str,
        settlement_id: &str,
        amount: Option<&str>,
    ) -> crate::didcomm::PlainMessage<Settle> {
        let settle = Settle {
            transaction_id: self.transaction_id.clone(),
            settlement_id: settlement_id.to_string(),
            amount: amount.map(|s| s.to_string()),
        };
        // Create a PlainMessage from self first, then create the reply
        let original_message = self
            .to_didcomm(creator_did)
            .expect("Failed to create DIDComm message");
        original_message
            .create_reply(&settle, creator_did)
            .expect("Failed to create reply")
    }

    fn reject(&self, creator_did: &str, reason: &str) -> crate::didcomm::PlainMessage<Reject> {
        let reject = Reject {
            transaction_id: self.transaction_id.clone(),
            reason: reason.to_string(),
        };
        // Create a PlainMessage from self first, then create the reply
        let original_message = self
            .to_didcomm(creator_did)
            .expect("Failed to create DIDComm message");
        original_message
            .create_reply(&reject, creator_did)
            .expect("Failed to create reply")
    }

    fn cancel(
        &self,
        creator_did: &str,
        by: &str,
        reason: Option<&str>,
    ) -> crate::didcomm::PlainMessage<Cancel> {
        let cancel = if let Some(reason) = reason {
            Cancel::with_reason(&self.transaction_id, by, reason)
        } else {
            Cancel::new(&self.transaction_id, by)
        };
        // Create a PlainMessage from self first, then create the reply
        let original_message = self
            .to_didcomm(creator_did)
            .expect("Failed to create DIDComm message");
        original_message
            .create_reply(&cancel, creator_did)
            .expect("Failed to create reply")
    }

    fn revert(
        &self,
        creator_did: &str,
        settlement_address: &str,
        reason: &str,
    ) -> crate::didcomm::PlainMessage<Revert> {
        let revert = Revert {
            transaction_id: self.transaction_id.clone(),
            settlement_address: settlement_address.to_string(),
            reason: reason.to_string(),
            note: None,
        };
        // Create a PlainMessage from self first, then create the reply
        let original_message = self
            .to_didcomm(creator_did)
            .expect("Failed to create DIDComm message");
        original_message
            .create_reply(&revert, creator_did)
            .expect("Failed to create reply")
    }

    fn update_policies(
        &self,
        creator_did: &str,
        policies: Vec<Policy>,
    ) -> crate::didcomm::PlainMessage<UpdatePolicies> {
        let update_policies = UpdatePolicies {
            transaction_id: self.transaction_id.clone(),
            policies,
        };
        // Create a PlainMessage from self first, then create the reply
        let original_message = self
            .to_didcomm(creator_did)
            .expect("Failed to create DIDComm message");
        original_message
            .create_reply(&update_policies, creator_did)
            .expect("Failed to create reply")
    }

    fn replace_agent(
        &self,
        creator_did: &str,
        original_agent: &str,
        replacement: Participant,
    ) -> crate::didcomm::PlainMessage<ReplaceAgent> {
        let replace_agent = ReplaceAgent {
            transaction_id: self.transaction_id.clone(),
            original: original_agent.to_string(),
            replacement,
        };
        // Create a PlainMessage from self first, then create the reply
        let original_message = self
            .to_didcomm(creator_did)
            .expect("Failed to create DIDComm message");
        original_message
            .create_reply(&replace_agent, creator_did)
            .expect("Failed to create reply")
    }

    fn remove_agent(
        &self,
        creator_did: &str,
        agent: &str,
    ) -> crate::didcomm::PlainMessage<RemoveAgent> {
        let remove_agent = RemoveAgent {
            transaction_id: self.transaction_id.clone(),
            agent: agent.to_string(),
        };
        // Create a PlainMessage from self first, then create the reply
        let original_message = self
            .to_didcomm(creator_did)
            .expect("Failed to create DIDComm message");
        original_message
            .create_reply(&remove_agent, creator_did)
            .expect("Failed to create reply")
    }
}

/// Out of Band invitation for TAP connections.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(generated_id)]
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
#[tap(generated_id)]
pub struct AuthorizationRequired {
    /// Authorization URL.
    pub url: String,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AuthorizationRequired {
    /// Create a new AuthorizationRequired message.
    pub fn new(url: String, expires: String) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("expires".to_string(), serde_json::Value::String(expires));

        Self { url, metadata }
    }

    /// Add metadata to the message.
    pub fn add_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

impl TapMessageBody for OutOfBand {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#outofband"
    }

    fn validate(&self) -> Result<()> {
        if self.goal_code.is_empty() {
            return Err(Error::Validation("Goal code is required".to_string()));
        }

        if self.service.is_empty() {
            return Err(Error::Validation("Service is required".to_string()));
        }

        Ok(())
    }
}

impl TapMessageBody for AuthorizationRequired {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#authorizationrequired"
    }

    fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            return Err(Error::Validation(
                "Authorization URL is required".to_string(),
            ));
        }

        // Validate expiry date if present
        if let Some(expires) = self.metadata.get("expires") {
            if let Some(expires_str) = expires.as_str() {
                // Simple format check
                if !expires_str.contains('T') || !expires_str.contains(':') {
                    return Err(Error::Validation(
                        "Invalid expiry date format. Expected ISO8601/RFC3339 format".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}
