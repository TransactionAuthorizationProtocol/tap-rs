//! Party types for TAP messages (TAIP-6).
//!
//! This module defines the structure of party information used in TAP messages.
//! Parties are the real-world entities involved with a transaction - legal or natural persons.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::message::agent::TapParticipant;
use crate::utils::NameHashable;

/// Party in a transaction (TAIP-6).
///
/// Parties are identified using an IRI as the @id attribute in a JSON-LD object.
/// They represent real-world entities (legal or natural persons) that are parties to a transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Party {
    /// IRI of the party (DID, email, phone number, etc).
    #[serde(rename = "@id")]
    pub id: String,

    /// Additional JSON-LD metadata for the party.
    /// This allows for extensible metadata like country codes, LEI codes, MCC codes, etc.
    /// Example: {"https://schema.org/addressCountry": "de", "lei": "..."}
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TapParticipant for Party {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Party {
    /// Create a new party with the given IRI.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new party with metadata.
    pub fn with_metadata(id: &str, metadata: HashMap<String, serde_json::Value>) -> Self {
        Self {
            id: id.to_string(),
            metadata,
        }
    }

    /// Add a metadata field to the party.
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Add metadata using the builder pattern.
    pub fn with_metadata_field(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Add country code metadata.
    pub fn with_country(mut self, country_code: &str) -> Self {
        self.metadata.insert(
            "https://schema.org/addressCountry".to_string(),
            serde_json::Value::String(country_code.to_string()),
        );
        self
    }

    /// Add LEI code metadata.
    pub fn with_lei(mut self, lei_code: &str) -> Self {
        self.metadata.insert(
            "https://schema.org/leiCode".to_string(),
            serde_json::Value::String(lei_code.to_string()),
        );
        self
    }

    /// Add merchant category code (MCC) metadata.
    pub fn with_mcc(mut self, mcc: &str) -> Self {
        self.metadata.insert(
            "mcc".to_string(),
            serde_json::Value::String(mcc.to_string()),
        );
        self
    }

    /// Get a metadata value by key.
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Get country code if present.
    pub fn country(&self) -> Option<String> {
        self.get_metadata("https://schema.org/addressCountry")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Get LEI code if present.
    pub fn lei_code(&self) -> Option<String> {
        self.get_metadata("https://schema.org/leiCode")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Get MCC code if present.
    pub fn mcc(&self) -> Option<String> {
        self.get_metadata("mcc")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Add name hash metadata according to TAIP-12.
    pub fn with_name_hash(mut self, name: &str) -> Self {
        let hash = Self::hash_name(name);
        self.metadata
            .insert("nameHash".to_string(), serde_json::Value::String(hash));
        self
    }

    /// Get name hash if present.
    pub fn name_hash(&self) -> Option<String> {
        self.get_metadata("nameHash")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Set name hash directly.
    pub fn set_name_hash(&mut self, hash: String) {
        self.metadata
            .insert("nameHash".to_string(), serde_json::Value::String(hash));
    }

    // Schema.org Organization field accessors and builders

    /// Add a name field (schema.org/Organization or schema.org/Person).
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

    /// Add an email field (schema.org/Organization or schema.org/Person).
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

    /// Add a telephone field (schema.org/Organization or schema.org/Person).
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
}

// Implement NameHashable for Party
impl NameHashable for Party {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_party_creation() {
        let party = Party::new("did:example:alice");
        assert_eq!(party.id, "did:example:alice");
        assert!(party.metadata.is_empty());
    }

    #[test]
    fn test_party_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "name".to_string(),
            serde_json::Value::String("Alice".to_string()),
        );

        let party = Party::with_metadata("did:example:alice", metadata);
        assert_eq!(party.id, "did:example:alice");
        assert_eq!(
            party.get_metadata("name").unwrap().as_str().unwrap(),
            "Alice"
        );
    }

    #[test]
    fn test_party_with_country() {
        let party = Party::new("did:example:alice").with_country("de");
        assert_eq!(party.country().unwrap(), "de");
    }

    #[test]
    fn test_party_with_lei() {
        let party = Party::new("did:web:example.com").with_lei("LEI123456789");
        assert_eq!(party.lei_code().unwrap(), "LEI123456789");
    }

    #[test]
    fn test_party_with_mcc() {
        let party = Party::new("did:web:merchant.com").with_mcc("5812");
        assert_eq!(party.mcc().unwrap(), "5812");
    }

    #[test]
    fn test_party_serialization() {
        let party = Party::new("did:example:alice")
            .with_country("de")
            .with_lei("LEI123");

        let json = serde_json::to_string(&party).unwrap();
        let deserialized: Party = serde_json::from_str(&json).unwrap();

        assert_eq!(party, deserialized);
        assert_eq!(deserialized.country().unwrap(), "de");
        assert_eq!(deserialized.lei_code().unwrap(), "LEI123");
    }

    #[test]
    fn test_party_json_ld_format() {
        let party = Party::new("did:example:alice").with_country("de");
        let json = serde_json::to_value(&party).unwrap();

        assert_eq!(json["@id"], "did:example:alice");
        assert_eq!(json["https://schema.org/addressCountry"], "de");
    }

    #[test]
    fn test_party_with_name_hash() {
        let party = Party::new("did:example:alice").with_name_hash("Alice Lee");
        assert_eq!(
            party.name_hash().unwrap(),
            "b117f44426c9670da91b563db728cd0bc8bafa7d1a6bb5e764d1aad2ca25032e"
        );
    }

    #[test]
    fn test_party_name_hash_serialization() {
        let party = Party::new("did:example:alice")
            .with_name_hash("Alice Lee")
            .with_country("US");

        let json = serde_json::to_value(&party).unwrap();
        assert_eq!(json["@id"], "did:example:alice");
        assert_eq!(
            json["nameHash"],
            "b117f44426c9670da91b563db728cd0bc8bafa7d1a6bb5e764d1aad2ca25032e"
        );
        assert_eq!(json["https://schema.org/addressCountry"], "US");
    }

    // Schema.org Organization field tests

    #[test]
    fn test_party_with_name_field() {
        let party = Party::new("did:example:alice").with_name("Alice Corporation");

        assert_eq!(party.name(), Some("Alice Corporation"));

        // Test serialization
        let json = serde_json::to_value(&party).unwrap();
        assert_eq!(json["name"], "Alice Corporation");

        // Test deserialization
        let deserialized: Party = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.name(), Some("Alice Corporation"));
    }

    #[test]
    fn test_party_with_url_field() {
        let party = Party::new("did:example:alice").with_url("https://alice.example.com");

        assert_eq!(party.url(), Some("https://alice.example.com"));

        let json = serde_json::to_value(&party).unwrap();
        assert_eq!(json["url"], "https://alice.example.com");
    }

    #[test]
    fn test_party_with_logo_field() {
        let party = Party::new("did:example:alice").with_logo("https://alice.example.com/logo.png");

        assert_eq!(party.logo(), Some("https://alice.example.com/logo.png"));

        let json = serde_json::to_value(&party).unwrap();
        assert_eq!(json["logo"], "https://alice.example.com/logo.png");
    }

    #[test]
    fn test_party_with_description_field() {
        let party =
            Party::new("did:example:alice").with_description("A trusted financial institution");

        assert_eq!(party.description(), Some("A trusted financial institution"));

        let json = serde_json::to_value(&party).unwrap();
        assert_eq!(json["description"], "A trusted financial institution");
    }

    #[test]
    fn test_party_with_email_field() {
        let party = Party::new("did:example:alice").with_email("contact@alice.example.com");

        assert_eq!(party.email(), Some("contact@alice.example.com"));

        let json = serde_json::to_value(&party).unwrap();
        assert_eq!(json["email"], "contact@alice.example.com");
    }

    #[test]
    fn test_party_with_telephone_field() {
        let party = Party::new("did:example:alice").with_telephone("+1-555-0200");

        assert_eq!(party.telephone(), Some("+1-555-0200"));

        let json = serde_json::to_value(&party).unwrap();
        assert_eq!(json["telephone"], "+1-555-0200");
    }

    #[test]
    fn test_party_with_multiple_organization_fields() {
        let party = Party::new("did:example:alice")
            .with_name("Alice Corporation")
            .with_url("https://alice.example.com")
            .with_logo("https://alice.example.com/logo.png")
            .with_description("A trusted financial institution")
            .with_email("contact@alice.example.com")
            .with_telephone("+1-555-0200")
            .with_country("US")
            .with_lei("123456789012345678");

        assert_eq!(party.name(), Some("Alice Corporation"));
        assert_eq!(party.url(), Some("https://alice.example.com"));
        assert_eq!(party.logo(), Some("https://alice.example.com/logo.png"));
        assert_eq!(party.description(), Some("A trusted financial institution"));
        assert_eq!(party.email(), Some("contact@alice.example.com"));
        assert_eq!(party.telephone(), Some("+1-555-0200"));
        assert_eq!(party.country(), Some("US".to_string()));
        assert_eq!(party.lei_code(), Some("123456789012345678".to_string()));

        // Test JSON serialization includes all fields
        let json = serde_json::to_value(&party).unwrap();
        assert_eq!(json["@id"], "did:example:alice");
        assert_eq!(json["name"], "Alice Corporation");
        assert_eq!(json["url"], "https://alice.example.com");
        assert_eq!(json["logo"], "https://alice.example.com/logo.png");
        assert_eq!(json["description"], "A trusted financial institution");
        assert_eq!(json["email"], "contact@alice.example.com");
        assert_eq!(json["telephone"], "+1-555-0200");
        assert_eq!(json["https://schema.org/addressCountry"], "US");
        assert_eq!(json["https://schema.org/leiCode"], "123456789012345678");

        // Test deserialization preserves all fields
        let deserialized: Party = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.name(), Some("Alice Corporation"));
        assert_eq!(deserialized.url(), Some("https://alice.example.com"));
        assert_eq!(deserialized.country(), Some("US".to_string()));
    }

    #[test]
    fn test_party_json_ld_compliance_with_organization_fields() {
        let party = Party::new("did:example:alice")
            .with_name("Alice Corporation")
            .with_metadata_field(
                "ivms101".to_string(),
                serde_json::json!({
                    "naturalPerson": {
                        "name": {
                            "primaryIdentifier": "Smith",
                            "secondaryIdentifier": "Alice"
                        }
                    }
                }),
            );

        let json = serde_json::to_value(&party).unwrap();

        // Verify JSON-LD structure
        assert_eq!(json["@id"], "did:example:alice");
        assert_eq!(json["name"], "Alice Corporation");
        assert!(json["ivms101"]["naturalPerson"]["name"]["primaryIdentifier"].is_string());

        // Fields should be at root level, not nested under metadata
        assert!(json.get("metadata").is_none());
    }

    #[test]
    fn test_party_with_ivms101_and_schema_org_fields() {
        // Test that IVMS101 data can coexist with schema.org fields
        let party = Party::new("did:example:alice")
            .with_name("Alice Corporation")
            .with_country("US")
            .with_metadata_field(
                "ivms101".to_string(),
                serde_json::json!({
                    "legalPerson": {
                        "name": {
                            "nameIdentifier": [{
                                "legalPersonName": "Alice Corporation",
                                "legalPersonNameIdentifierType": "LEGL"
                            }]
                        },
                        "nationalIdentification": {
                            "nationalIdentifier": "123456789",
                            "nationalIdentifierType": "LEIX"
                        }
                    }
                }),
            );

        assert_eq!(party.name(), Some("Alice Corporation"));
        assert_eq!(party.country(), Some("US".to_string()));

        let json = serde_json::to_value(&party).unwrap();
        assert_eq!(json["name"], "Alice Corporation");
        assert_eq!(json["https://schema.org/addressCountry"], "US");
        assert!(json["ivms101"]["legalPerson"]["name"]["nameIdentifier"].is_array());
    }
}
