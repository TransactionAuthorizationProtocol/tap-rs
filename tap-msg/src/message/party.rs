//! Party types for TAP messages (TAIP-6).
//!
//! This module defines the structure of party information used in TAP messages.
//! Parties are the real-world entities involved with a transaction - legal or natural persons.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::message::agent::TapParticipant;

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
}

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
}
