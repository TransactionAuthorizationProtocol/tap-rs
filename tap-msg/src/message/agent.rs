//! Agent types for TAP messages (TAIP-5).
//!
//! This module defines the structure of agent information used in TAP messages.
//! Agents are services involved in executing transactions such as exchanges,
//! custodial wallet services, wallets, blockchain addresses, DeFi protocols, and bridges.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::message::policy::Policy;

/// Common trait for TAP participants (agents and parties)
pub trait TapParticipant {
    /// Get the identifier of this participant
    fn id(&self) -> &str;
}

/// Helper for serializing/deserializing the `for` field that can be either a string or an array
#[derive(Debug, Clone, PartialEq)]
pub struct ForParties(pub Vec<String>);

impl Serialize for ForParties {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.0.len() == 1 {
            // Serialize as a single string if there's only one party
            self.0[0].serialize(serializer)
        } else {
            // Serialize as an array if there are multiple parties
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for ForParties {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct ForPartiesVisitor;

        impl<'de> Visitor<'de> for ForPartiesVisitor {
            type Value = ForParties;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or an array of strings")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ForParties(vec![value.to_string()]))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut parties = Vec::new();
                while let Some(party) = seq.next_element::<String>()? {
                    parties.push(party);
                }
                Ok(ForParties(parties))
            }
        }

        deserializer.deserialize_any(ForPartiesVisitor)
    }
}

/// Agent in a transaction (TAIP-5).
///
/// Agents are identified using Decentralized Identifiers (DIDs) and can be:
/// - Centralized services (exchanges, custodial wallets)
/// - End-user software (self-hosted wallets)  
/// - Decentralized protocols (DeFi protocols, bridges)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    /// DID of the agent.
    #[serde(rename = "@id")]
    pub id: String,

    /// Role of the agent in this transaction (optional).
    /// Examples: "SettlementAddress", "SourceAddress", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// DID or IRI of another Agent or Party that this agent acts on behalf of (REQUIRED per TAIP-5).
    /// Can be a single party or multiple parties.
    #[serde(rename = "for")]
    pub for_parties: ForParties,

    /// Policies of the agent according to TAIP-7 (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub policies: Option<Vec<Policy>>,

    /// Additional JSON-LD metadata for the agent.
    /// This allows for extensible metadata beyond the core fields.
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TapParticipant for Agent {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Agent {
    /// Create a new agent with the given DID, role, and for_party.
    pub fn new(id: &str, role: &str, for_party: &str) -> Self {
        Self {
            id: id.to_string(),
            role: Some(role.to_string()),
            for_parties: ForParties(vec![for_party.to_string()]),
            policies: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new agent with multiple parties.
    pub fn new_for_parties(id: &str, role: &str, for_parties: Vec<String>) -> Self {
        Self {
            id: id.to_string(),
            role: Some(role.to_string()),
            for_parties: ForParties(for_parties),
            policies: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new agent without a role.
    pub fn new_without_role(id: &str, for_party: &str) -> Self {
        Self {
            id: id.to_string(),
            role: None,
            for_parties: ForParties(vec![for_party.to_string()]),
            policies: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new agent with metadata.
    pub fn with_metadata(
        id: &str,
        role: &str,
        for_party: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            id: id.to_string(),
            role: Some(role.to_string()),
            for_parties: ForParties(vec![for_party.to_string()]),
            policies: None,
            metadata,
        }
    }

    /// Add policies to this agent.
    pub fn with_policies(mut self, policies: Vec<Policy>) -> Self {
        self.policies = Some(policies);
        self
    }

    /// Add a single policy to this agent.
    pub fn add_policy(mut self, policy: Policy) -> Self {
        match &mut self.policies {
            Some(policies) => policies.push(policy),
            None => self.policies = Some(vec![policy]),
        }
        self
    }

    /// Add a metadata field to the agent.
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Add metadata using the builder pattern.
    pub fn with_metadata_field(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get a metadata value by key.
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Check if this agent has a specific role.
    pub fn has_role(&self, role: &str) -> bool {
        self.role.as_ref().is_some_and(|r| r == role)
    }

    /// Check if this agent acts for a specific party.
    pub fn acts_for(&self, party_id: &str) -> bool {
        self.for_parties.0.contains(&party_id.to_string())
    }

    /// Get all parties this agent acts for.
    pub fn for_parties(&self) -> &[String] {
        &self.for_parties.0
    }

    /// Get the first party this agent acts for (for backward compatibility).
    pub fn primary_party(&self) -> Option<&str> {
        self.for_parties.0.first().map(|s| s.as_str())
    }

    /// Add a party this agent acts for.
    pub fn add_for_party(&mut self, party_id: &str) {
        if !self.for_parties.0.contains(&party_id.to_string()) {
            self.for_parties.0.push(party_id.to_string());
        }
    }

    /// Set all parties this agent acts for.
    pub fn set_for_parties(&mut self, parties: Vec<String>) {
        self.for_parties.0 = parties;
    }

    // Schema.org Organization field accessors and builders

    /// Add a name field (schema.org/Organization).
    pub fn with_name(mut self, name: &str) -> Self {
        self.metadata.insert(
            "name".to_string(),
            serde_json::Value::String(name.to_string()),
        );
        self
    }

    /// Get the name field if present.
    pub fn name(&self) -> Option<&str> {
        self.metadata.get("name").and_then(|v| v.as_str())
    }

    /// Add a URL field (schema.org/Organization).
    pub fn with_url(mut self, url: &str) -> Self {
        self.metadata.insert(
            "url".to_string(),
            serde_json::Value::String(url.to_string()),
        );
        self
    }

    /// Get the URL field if present.
    pub fn url(&self) -> Option<&str> {
        self.metadata.get("url").and_then(|v| v.as_str())
    }

    /// Add a logo field (schema.org/Organization).
    pub fn with_logo(mut self, logo: &str) -> Self {
        self.metadata.insert(
            "logo".to_string(),
            serde_json::Value::String(logo.to_string()),
        );
        self
    }

    /// Get the logo field if present.
    pub fn logo(&self) -> Option<&str> {
        self.metadata.get("logo").and_then(|v| v.as_str())
    }

    /// Add a description field (schema.org/Organization).
    pub fn with_description(mut self, description: &str) -> Self {
        self.metadata.insert(
            "description".to_string(),
            serde_json::Value::String(description.to_string()),
        );
        self
    }

    /// Get the description field if present.
    pub fn description(&self) -> Option<&str> {
        self.metadata.get("description").and_then(|v| v.as_str())
    }

    /// Add an email field (schema.org/Organization).
    pub fn with_email(mut self, email: &str) -> Self {
        self.metadata.insert(
            "email".to_string(),
            serde_json::Value::String(email.to_string()),
        );
        self
    }

    /// Get the email field if present.
    pub fn email(&self) -> Option<&str> {
        self.metadata.get("email").and_then(|v| v.as_str())
    }

    /// Add a telephone field (schema.org/Organization).
    pub fn with_telephone(mut self, telephone: &str) -> Self {
        self.metadata.insert(
            "telephone".to_string(),
            serde_json::Value::String(telephone.to_string()),
        );
        self
    }

    /// Get the telephone field if present.
    pub fn telephone(&self) -> Option<&str> {
        self.metadata.get("telephone").and_then(|v| v.as_str())
    }

    /// Add a serviceUrl field for DIDComm endpoint fallback (TAIP-5).
    pub fn with_service_url(mut self, service_url: &str) -> Self {
        self.metadata.insert(
            "serviceUrl".to_string(),
            serde_json::Value::String(service_url.to_string()),
        );
        self
    }

    /// Get the serviceUrl field if present.
    pub fn service_url(&self) -> Option<&str> {
        self.metadata.get("serviceUrl").and_then(|v| v.as_str())
    }
}

/// Common agent roles used in TAP transactions.
pub mod roles {
    /// Settlement address role for blockchain transactions.
    pub const SETTLEMENT_ADDRESS: &str = "SettlementAddress";

    /// Source address role for originating transactions.
    pub const SOURCE_ADDRESS: &str = "SourceAddress";

    /// Custodial service role.
    pub const CUSTODIAL_SERVICE: &str = "CustodialService";

    /// Wallet service role.
    pub const WALLET_SERVICE: &str = "WalletService";

    /// Exchange service role.
    pub const EXCHANGE: &str = "Exchange";

    /// Bridge service role for cross-chain transactions.
    pub const BRIDGE: &str = "Bridge";

    /// DeFi protocol role.
    pub const DEFI_PROTOCOL: &str = "DeFiProtocol";
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice");

        assert_eq!(agent.id, "did:web:example.com");
        assert_eq!(agent.role, Some("Exchange".to_string()));
        assert_eq!(agent.for_parties.0, vec!["did:example:alice"]);
        assert!(agent.policies.is_none());
        assert!(agent.metadata.is_empty());
    }

    #[test]
    fn test_agent_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "name".to_string(),
            serde_json::Value::String("Example Exchange".to_string()),
        );

        let agent = Agent::with_metadata(
            "did:web:example.com",
            "Exchange",
            "did:example:alice",
            metadata,
        );

        assert_eq!(
            agent.get_metadata("name").unwrap().as_str().unwrap(),
            "Example Exchange"
        );
    }

    #[test]
    fn test_agent_with_policies() {
        use crate::message::policy::{Policy, RequireAuthorization};

        let auth_req = RequireAuthorization {
            from: Some(vec!["did:example:kyc".to_string()]),
            from_role: None,
            from_agent: None,
            purpose: Some("KYC verification".to_string()),
        };
        let policy = Policy::RequireAuthorization(auth_req);

        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice")
            .with_policies(vec![policy]);

        assert!(agent.policies.is_some());
        assert_eq!(agent.policies.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_agent_serialization() {
        let agent = Agent::new(
            "did:web:example.com",
            "SettlementAddress",
            "did:example:alice",
        )
        .with_metadata_field(
            "name".to_string(),
            serde_json::Value::String("Test Agent".to_string()),
        );

        let json = serde_json::to_string(&agent).unwrap();
        let deserialized: Agent = serde_json::from_str(&json).unwrap();

        assert_eq!(agent, deserialized);
        assert_eq!(deserialized.role, Some("SettlementAddress".to_string()));
        assert_eq!(deserialized.for_parties.0, vec!["did:example:alice"]);
    }

    #[test]
    fn test_agent_json_ld_format() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice");
        let json = serde_json::to_value(&agent).unwrap();

        assert_eq!(json["@id"], "did:web:example.com");
        assert_eq!(json["role"], "Exchange");
        assert_eq!(json["for"], "did:example:alice"); // Should serialize as string for single party
    }

    #[test]
    fn test_agent_helper_methods() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice");

        assert!(agent.has_role("Exchange"));
        assert!(!agent.has_role("Wallet"));
        assert!(agent.acts_for("did:example:alice"));
        assert!(!agent.acts_for("did:example:bob"));
    }

    #[test]
    fn test_agent_roles_constants() {
        assert_eq!(roles::SETTLEMENT_ADDRESS, "SettlementAddress");
        assert_eq!(roles::SOURCE_ADDRESS, "SourceAddress");
        assert_eq!(roles::EXCHANGE, "Exchange");
    }

    #[test]
    fn test_agent_multiple_parties() {
        let parties = vec![
            "did:example:alice".to_string(),
            "did:example:bob".to_string(),
        ];
        let agent = Agent::new_for_parties("did:web:example.com", "Exchange", parties.clone());

        assert_eq!(agent.for_parties.0, parties);
        assert!(agent.acts_for("did:example:alice"));
        assert!(agent.acts_for("did:example:bob"));
        assert!(!agent.acts_for("did:example:charlie"));
    }

    #[test]
    fn test_agent_for_parties_serialization_single() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice");
        let json = serde_json::to_value(&agent).unwrap();

        // Single party should serialize as string
        assert_eq!(json["for"], "did:example:alice");

        // Test deserialization
        let deserialized: Agent = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.for_parties.0, vec!["did:example:alice"]);
    }

    #[test]
    fn test_agent_for_parties_serialization_multiple() {
        let parties = vec![
            "did:example:alice".to_string(),
            "did:example:bob".to_string(),
        ];
        let agent = Agent::new_for_parties("did:web:example.com", "Exchange", parties.clone());
        let json = serde_json::to_value(&agent).unwrap();

        // Multiple parties should serialize as array
        assert_eq!(
            json["for"],
            serde_json::Value::Array(vec![
                serde_json::Value::String("did:example:alice".to_string()),
                serde_json::Value::String("did:example:bob".to_string())
            ])
        );

        // Test deserialization
        let deserialized: Agent = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.for_parties.0, parties);
    }

    #[test]
    fn test_agent_for_parties_deserialization_from_string() {
        let json = serde_json::json!({
            "@id": "did:web:example.com",
            "role": "Exchange",
            "for": "did:example:alice"
        });

        let agent: Agent = serde_json::from_value(json).unwrap();
        assert_eq!(agent.for_parties.0, vec!["did:example:alice"]);
    }

    #[test]
    fn test_agent_for_parties_deserialization_from_array() {
        let json = serde_json::json!({
            "@id": "did:web:example.com",
            "role": "Exchange",
            "for": ["did:example:alice", "did:example:bob"]
        });

        let agent: Agent = serde_json::from_value(json).unwrap();
        assert_eq!(
            agent.for_parties.0,
            vec!["did:example:alice", "did:example:bob"]
        );
    }

    #[test]
    fn test_agent_for_parties_methods() {
        let mut agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice");

        assert_eq!(agent.for_parties(), &["did:example:alice"]);
        assert_eq!(agent.primary_party(), Some("did:example:alice"));

        agent.add_for_party("did:example:bob");
        assert_eq!(
            agent.for_parties(),
            &["did:example:alice", "did:example:bob"]
        );

        agent.set_for_parties(vec!["did:example:charlie".to_string()]);
        assert_eq!(agent.for_parties(), &["did:example:charlie"]);
        assert_eq!(agent.primary_party(), Some("did:example:charlie"));
    }

    // Schema.org Organization field tests

    #[test]
    fn test_agent_with_name_field() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice")
            .with_name("Example Exchange Inc.");

        assert_eq!(agent.name(), Some("Example Exchange Inc."));

        // Test serialization
        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["name"], "Example Exchange Inc.");

        // Test deserialization
        let deserialized: Agent = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.name(), Some("Example Exchange Inc."));
    }

    #[test]
    fn test_agent_with_url_field() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice")
            .with_url("https://example.com");

        assert_eq!(agent.url(), Some("https://example.com"));

        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["url"], "https://example.com");
    }

    #[test]
    fn test_agent_with_logo_field() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice")
            .with_logo("https://example.com/logo.png");

        assert_eq!(agent.logo(), Some("https://example.com/logo.png"));

        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["logo"], "https://example.com/logo.png");
    }

    #[test]
    fn test_agent_with_description_field() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice")
            .with_description("A leading cryptocurrency exchange");

        assert_eq!(
            agent.description(),
            Some("A leading cryptocurrency exchange")
        );

        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["description"], "A leading cryptocurrency exchange");
    }

    #[test]
    fn test_agent_with_email_field() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice")
            .with_email("support@example.com");

        assert_eq!(agent.email(), Some("support@example.com"));

        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["email"], "support@example.com");
    }

    #[test]
    fn test_agent_with_telephone_field() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice")
            .with_telephone("+1-555-0100");

        assert_eq!(agent.telephone(), Some("+1-555-0100"));

        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["telephone"], "+1-555-0100");
    }

    #[test]
    fn test_agent_with_service_url_field() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice")
            .with_service_url("https://example.com/didcomm");

        assert_eq!(agent.service_url(), Some("https://example.com/didcomm"));

        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["serviceUrl"], "https://example.com/didcomm");
    }

    #[test]
    fn test_agent_with_multiple_organization_fields() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice")
            .with_name("Example Exchange Inc.")
            .with_url("https://example.com")
            .with_logo("https://example.com/logo.png")
            .with_description("A leading cryptocurrency exchange")
            .with_email("support@example.com")
            .with_telephone("+1-555-0100")
            .with_service_url("https://example.com/didcomm");

        assert_eq!(agent.name(), Some("Example Exchange Inc."));
        assert_eq!(agent.url(), Some("https://example.com"));
        assert_eq!(agent.logo(), Some("https://example.com/logo.png"));
        assert_eq!(
            agent.description(),
            Some("A leading cryptocurrency exchange")
        );
        assert_eq!(agent.email(), Some("support@example.com"));
        assert_eq!(agent.telephone(), Some("+1-555-0100"));
        assert_eq!(agent.service_url(), Some("https://example.com/didcomm"));

        // Test JSON serialization includes all fields
        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["@id"], "did:web:example.com");
        assert_eq!(json["role"], "Exchange");
        assert_eq!(json["for"], "did:example:alice");
        assert_eq!(json["name"], "Example Exchange Inc.");
        assert_eq!(json["url"], "https://example.com");
        assert_eq!(json["logo"], "https://example.com/logo.png");
        assert_eq!(json["description"], "A leading cryptocurrency exchange");
        assert_eq!(json["email"], "support@example.com");
        assert_eq!(json["telephone"], "+1-555-0100");
        assert_eq!(json["serviceUrl"], "https://example.com/didcomm");

        // Test deserialization preserves all fields
        let deserialized: Agent = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.name(), Some("Example Exchange Inc."));
        assert_eq!(deserialized.url(), Some("https://example.com"));
        assert_eq!(
            deserialized.service_url(),
            Some("https://example.com/didcomm")
        );
    }

    #[test]
    fn test_agent_json_ld_compliance_with_organization_fields() {
        let agent = Agent::new("did:web:example.com", "Exchange", "did:example:alice")
            .with_name("Example Exchange")
            .with_metadata_field(
                "lei:leiCode".to_string(),
                serde_json::Value::String("123456789012345678".to_string()),
            );

        let json = serde_json::to_value(&agent).unwrap();

        // Verify JSON-LD structure
        assert_eq!(json["@id"], "did:web:example.com");
        assert_eq!(json["name"], "Example Exchange");
        assert_eq!(json["lei:leiCode"], "123456789012345678");

        // Fields should be at root level, not nested
        assert!(json.get("metadata").is_none());
    }
}
