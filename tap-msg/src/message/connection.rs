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

/// Authorization Required message body (TAIP-4, TAIP-15).
///
/// Indicates that authorization is required to proceed with a transaction or connection.
/// This message was moved from TAIP-15 to TAIP-4 as a standard authorization message.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#AuthorizationRequired")]
pub struct AuthorizationRequired {
    /// Authorization URL where the user can authorize the transaction.
    #[serde(rename = "authorizationUrl")]
    pub authorization_url: String,

    /// ISO 8601 timestamp when the authorization URL expires (REQUIRED per TAIP-4).
    pub expires: String,

    /// Optional party type (e.g., "customer", "principal", "originator") that is required to open the URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AuthorizationRequired {
    /// Create a new AuthorizationRequired message.
    pub fn new(authorization_url: String, expires: String) -> Self {
        Self {
            authorization_url,
            expires,
            from: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new AuthorizationRequired message with a specified party type.
    pub fn new_with_from(authorization_url: String, expires: String, from: String) -> Self {
        Self {
            authorization_url,
            expires,
            from: Some(from),
            metadata: HashMap::new(),
        }
    }

    /// Set the party type that is required to open the URL.
    pub fn with_from(mut self, from: String) -> Self {
        self.from = Some(from);
        self
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
        if self.authorization_url.is_empty() {
            return Err(Error::Validation(
                "Authorization URL is required".to_string(),
            ));
        }

        // Validate expiry date (now required per TAIP-4)
        if self.expires.is_empty() {
            return Err(Error::Validation(
                "Expires timestamp is required".to_string(),
            ));
        }

        // Simple format check for ISO 8601
        if !self.expires.contains('T') || !self.expires.contains(':') {
            return Err(Error::Validation(
                "Invalid expiry date format. Expected ISO8601/RFC3339 format".to_string(),
            ));
        }

        // Validate 'from' field if present
        if let Some(ref from) = self.from {
            let valid_from_values = ["customer", "principal", "originator", "beneficiary"];
            if !valid_from_values.contains(&from.as_str()) {
                return Err(Error::Validation(
                    format!("Invalid 'from' value '{}'. Expected one of: customer, principal, originator, beneficiary", from),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_authorization_required_creation() {
        let auth_req = AuthorizationRequired::new(
            "https://vasp.com/authorize?request=abc123".to_string(),
            "2024-12-31T23:59:59Z".to_string(),
        );

        assert_eq!(
            auth_req.authorization_url,
            "https://vasp.com/authorize?request=abc123"
        );
        assert_eq!(auth_req.expires, "2024-12-31T23:59:59Z");
        assert!(auth_req.from.is_none());
        assert!(auth_req.metadata.is_empty());
    }

    #[test]
    fn test_authorization_required_with_from() {
        let auth_req = AuthorizationRequired::new_with_from(
            "https://vasp.com/authorize".to_string(),
            "2024-12-31T23:59:59Z".to_string(),
            "customer".to_string(),
        );

        assert_eq!(auth_req.from, Some("customer".to_string()));
    }

    #[test]
    fn test_authorization_required_builder_pattern() {
        let auth_req = AuthorizationRequired::new(
            "https://vasp.com/authorize".to_string(),
            "2024-12-31T23:59:59Z".to_string(),
        )
        .with_from("principal".to_string())
        .add_metadata("custom_field", serde_json::json!("value"));

        assert_eq!(auth_req.from, Some("principal".to_string()));
        assert_eq!(
            auth_req.metadata.get("custom_field"),
            Some(&serde_json::json!("value"))
        );
    }

    #[test]
    fn test_authorization_required_serialization() {
        let auth_req = AuthorizationRequired::new_with_from(
            "https://vasp.com/authorize?request=abc123".to_string(),
            "2024-12-31T23:59:59Z".to_string(),
            "customer".to_string(),
        );

        let json = serde_json::to_value(&auth_req).unwrap();

        // Check field names match TAIP-4 specification
        assert_eq!(
            json["authorizationUrl"],
            "https://vasp.com/authorize?request=abc123"
        );
        assert_eq!(json["expires"], "2024-12-31T23:59:59Z");
        assert_eq!(json["from"], "customer");

        // Test deserialization
        let deserialized: AuthorizationRequired = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.authorization_url, auth_req.authorization_url);
        assert_eq!(deserialized.expires, auth_req.expires);
        assert_eq!(deserialized.from, auth_req.from);
    }

    #[test]
    fn test_authorization_required_validation_success() {
        let auth_req = AuthorizationRequired::new(
            "https://vasp.com/authorize".to_string(),
            "2024-12-31T23:59:59Z".to_string(),
        );

        assert!(auth_req.validate().is_ok());
    }

    #[test]
    fn test_authorization_required_validation_with_valid_from() {
        let valid_from_values = ["customer", "principal", "originator", "beneficiary"];

        for from_value in &valid_from_values {
            let auth_req = AuthorizationRequired::new_with_from(
                "https://vasp.com/authorize".to_string(),
                "2024-12-31T23:59:59Z".to_string(),
                from_value.to_string(),
            );

            assert!(
                auth_req.validate().is_ok(),
                "Validation failed for from value: {}",
                from_value
            );
        }
    }

    #[test]
    fn test_authorization_required_validation_empty_url() {
        let auth_req =
            AuthorizationRequired::new("".to_string(), "2024-12-31T23:59:59Z".to_string());

        let result = auth_req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Authorization URL is required"));
    }

    #[test]
    fn test_authorization_required_validation_empty_expires() {
        let auth_req = AuthorizationRequired {
            authorization_url: "https://vasp.com/authorize".to_string(),
            expires: "".to_string(),
            from: None,
            metadata: HashMap::new(),
        };

        let result = auth_req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expires timestamp is required"));
    }

    #[test]
    fn test_authorization_required_validation_invalid_expires_format() {
        let auth_req = AuthorizationRequired::new(
            "https://vasp.com/authorize".to_string(),
            "2024-12-31".to_string(), // Missing time component
        );

        let result = auth_req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid expiry date format"));
    }

    #[test]
    fn test_authorization_required_validation_invalid_from() {
        let auth_req = AuthorizationRequired::new_with_from(
            "https://vasp.com/authorize".to_string(),
            "2024-12-31T23:59:59Z".to_string(),
            "invalid_party".to_string(),
        );

        let result = auth_req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid 'from' value"));
    }

    #[test]
    fn test_authorization_required_json_compliance_with_taip4() {
        // Test that the JSON structure matches TAIP-4 example
        let auth_req = AuthorizationRequired::new_with_from(
            "https://beneficiary.vasp/authorize?request=abc123".to_string(),
            "2024-01-01T12:00:00Z".to_string(),
            "customer".to_string(),
        );

        let json = serde_json::to_value(&auth_req).unwrap();

        // Verify field names match TAIP-4 specification
        assert!(json.get("authorizationUrl").is_some());
        assert!(json.get("expires").is_some());
        assert!(json.get("from").is_some());

        // Verify old field names are not present
        assert!(json.get("authorization_url").is_none());
        assert!(json.get("url").is_none());
        assert!(json.get("agent_id").is_none());
    }
}
