//! Customer management module for TAP Node
//!
//! This module provides customer data management functionality, including:
//! - Automatic extraction of party information from TAP messages
//! - Schema.org JSON-LD profile storage
//! - Multiple identifier support (DIDs, email, phone, URLs)
//! - Relationship tracking for TAIP-9 compliance
//! - IVMS101 data caching for Travel Rule compliance

use crate::error::{Error, Result};
use crate::storage::{
    Customer, CustomerIdentifier, CustomerRelationship, IdentifierType, SchemaType, Storage,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;
use tap_ivms101::{
    builder::{GeographicAddressBuilder, NaturalPersonBuilder, NaturalPersonNameBuilder},
    message::Person,
    types::AddressType,
};
use tap_msg::message::Party;
use tap_msg::utils::NameHashable;
use uuid::Uuid;

/// Customer manager handles all customer-related operations
pub struct CustomerManager {
    storage: Arc<Storage>,
}

impl CustomerManager {
    /// Generate name hash from IVMS101 Person data using TAIP-12 standard
    pub fn generate_name_hash_from_ivms101(&self, person: &Person) -> Option<String> {
        person
            .get_full_name()
            .map(|name| Customer::hash_name(&name))
    }

    /// Create a new customer manager
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// Get a reference to the storage
    pub fn get_storage(&self) -> &Arc<Storage> {
        &self.storage
    }

    /// Extract and create/update customer from a Party object
    pub async fn extract_customer_from_party(
        &self,
        party: &Party,
        agent_did: &str,
        _role: &str, // "originator", "beneficiary", etc.
    ) -> Result<String> {
        // Determine customer ID and primary identifier
        let (customer_id, primary_identifier) = self.determine_customer_id(&party.id);

        // Check if customer exists
        let existing = self
            .storage
            .get_customer(&customer_id)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        let mut profile = json!({
            "@context": "https://schema.org",
            "@type": "Person",
            "identifier": party.id.clone(),
        });

        // Add metadata fields to profile
        for (key, value) in &party.metadata {
            // Map common fields
            match key.as_str() {
                "name" | "https://schema.org/name" => {
                    profile["name"] = value.clone();
                }
                "givenName" | "https://schema.org/givenName" => {
                    profile["givenName"] = value.clone();
                }
                "familyName" | "https://schema.org/familyName" => {
                    profile["familyName"] = value.clone();
                }
                "addressCountry" | "https://schema.org/addressCountry" => {
                    profile["addressCountry"] = value.clone();
                }
                "nameHash" => {
                    // Preserve existing name hash from party metadata
                    profile["nameHash"] = value.clone();
                }
                _ => {
                    // Add other metadata as-is
                    profile[key] = value.clone();
                }
            }
        }

        // Extract structured data from profile
        let (given_name, family_name, display_name, address_country) =
            self.extract_structured_data(&profile);

        let now = Utc::now().to_rfc3339();

        let mut customer = Customer {
            id: customer_id.clone(),
            agent_did: agent_did.to_string(),
            schema_type: SchemaType::Person, // Default to Person, can be updated later
            given_name,
            family_name,
            display_name: display_name.or_else(|| {
                party
                    .metadata
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            }),
            legal_name: None,
            lei_code: None,
            mcc_code: None,
            address_country,
            address_locality: None,
            postal_code: None,
            street_address: None,
            profile,
            ivms101_data: None,
            verified_at: None,
            created_at: existing
                .as_ref()
                .map(|c| c.created_at.clone())
                .unwrap_or_else(|| now.clone()),
            updated_at: now,
        };

        // Generate and add name hash if not already present
        if customer.get_name_hash().is_none() {
            customer.add_name_hash_to_profile();
        }

        // Upsert customer
        self.storage
            .upsert_customer(&customer)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        // Add identifier
        let identifier = CustomerIdentifier {
            id: primary_identifier.clone(),
            customer_id: customer_id.clone(),
            identifier_type: self.determine_identifier_type(&primary_identifier),
            verified: false,
            verification_method: None,
            verified_at: None,
            created_at: Utc::now().to_rfc3339(),
        };
        self.storage
            .add_customer_identifier(&identifier)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        // Extract additional identifiers from the id string
        self.extract_additional_identifiers(&customer_id, &party.id)
            .await?;

        Ok(customer_id)
    }

    /// Update customer with schema.org data
    pub async fn update_customer_profile(
        &self,
        customer_id: &str,
        profile_data: Value,
    ) -> Result<()> {
        let mut customer = self
            .storage
            .get_customer(customer_id)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?
            .ok_or_else(|| Error::Storage("Customer not found".to_string()))?;

        // Merge profile data
        if let Value::Object(ref mut existing_map) = customer.profile {
            if let Value::Object(new_map) = profile_data {
                for (key, value) in new_map {
                    existing_map.insert(key, value);
                }
            }
        }

        // Re-extract structured data
        let (given_name, family_name, display_name, address_country) =
            self.extract_structured_data(&customer.profile);

        customer.given_name = given_name.or(customer.given_name);
        customer.family_name = family_name.or(customer.family_name);
        customer.display_name = display_name.or(customer.display_name);
        customer.address_country = address_country.or(customer.address_country);
        customer.updated_at = Utc::now().to_rfc3339();

        // Regenerate name hash if names have changed
        customer.add_name_hash_to_profile();

        self.storage
            .upsert_customer(&customer)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;
        Ok(())
    }

    /// Generate IVMS101 data from customer profile
    pub async fn generate_ivms101_data(&self, customer_id: &str) -> Result<Value> {
        let customer = self
            .storage
            .get_customer(customer_id)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?
            .ok_or_else(|| Error::Storage("Customer not found".to_string()))?;

        let person = match customer.schema_type {
            SchemaType::Person => {
                // Build natural person
                let mut person_builder = NaturalPersonBuilder::new();

                // Add name
                if customer.family_name.is_some() || customer.given_name.is_some() {
                    let name = NaturalPersonNameBuilder::new()
                        .legal_name(
                            customer.family_name.as_deref().unwrap_or("Unknown"),
                            customer.given_name.as_deref().unwrap_or(""),
                        )
                        .build()
                        .map_err(|e| Error::Storage(format!("Failed to build name: {}", e)))?;
                    person_builder = person_builder.name(name);
                }

                // Add address only if we have street address (required field)
                if customer.address_country.is_some() && customer.street_address.is_some() {
                    let mut address_builder = GeographicAddressBuilder::new()
                        .address_type(AddressType::Home)
                        .country(customer.address_country.as_deref().unwrap_or(""))
                        .street_name(customer.street_address.as_deref().unwrap_or(""));

                    if let Some(postal) = &customer.postal_code {
                        address_builder = address_builder.post_code(postal);
                    }
                    if let Some(town) = &customer.address_locality {
                        address_builder = address_builder.town_name(town);
                    }

                    let address = address_builder
                        .build()
                        .map_err(|e| Error::Storage(format!("Failed to build address: {}", e)))?;
                    person_builder = person_builder.add_address(address);
                }

                let natural_person = person_builder.build().map_err(|e| {
                    Error::Storage(format!("Failed to build natural person: {}", e))
                })?;

                Person::NaturalPerson(natural_person)
            }
            SchemaType::Organization => {
                // For organizations, we'll use LegalPerson (not implemented in tap-ivms101 yet)
                // For now, return empty JSON
                return Ok(json!({}));
            }
            _ => return Ok(json!({})),
        };

        // Serialize the person to JSON
        let ivms101_json = serde_json::to_value(&person)
            .map_err(|e| Error::Storage(format!("Failed to serialize IVMS101: {}", e)))?;

        // Cache the generated IVMS101 data
        let mut customer = customer;
        customer.ivms101_data = Some(ivms101_json.clone());
        customer.updated_at = Utc::now().to_rfc3339();
        self.storage
            .upsert_customer(&customer)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        Ok(ivms101_json)
    }

    /// Update customer data from IVMS101 data
    pub async fn update_customer_from_ivms101(
        &self,
        customer_id: &str,
        ivms101_data: &Value,
    ) -> Result<()> {
        let mut customer = self
            .storage
            .get_customer(customer_id)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?
            .ok_or_else(|| Error::Storage("Customer not found".to_string()))?;

        // Parse IVMS101 data and update customer fields
        if let Some(natural_person) = ivms101_data.get("naturalPerson") {
            // Update from natural person data
            if let Some(name) = natural_person.get("name") {
                if let Some(name_identifiers) =
                    name.get("nameIdentifiers").and_then(|v| v.as_array())
                {
                    if let Some(first_name_id) = name_identifiers.first() {
                        if let Some(primary) = first_name_id
                            .get("primaryIdentifier")
                            .and_then(|v| v.as_str())
                        {
                            customer.family_name = Some(primary.to_string());
                        }
                        if let Some(secondary) = first_name_id
                            .get("secondaryIdentifier")
                            .and_then(|v| v.as_str())
                        {
                            customer.given_name = Some(secondary.to_string());
                        }
                    }
                }
            }

            // Update address from IVMS101
            if let Some(addresses) = natural_person
                .get("geographicAddress")
                .and_then(|v| v.as_array())
            {
                if let Some(first_addr) = addresses.first() {
                    if let Some(street) = first_addr.get("streetName").and_then(|v| v.as_str()) {
                        customer.street_address = Some(street.to_string());
                    }
                    if let Some(postal) = first_addr.get("postCode").and_then(|v| v.as_str()) {
                        customer.postal_code = Some(postal.to_string());
                    }
                    if let Some(town) = first_addr.get("townName").and_then(|v| v.as_str()) {
                        customer.address_locality = Some(town.to_string());
                    }
                    if let Some(country) = first_addr.get("country").and_then(|v| v.as_str()) {
                        customer.address_country = Some(country.to_string());
                    }
                }
            }
        }

        // Store the IVMS101 data
        customer.ivms101_data = Some(ivms101_data.clone());
        customer.updated_at = Utc::now().to_rfc3339();

        // Regenerate name hash if names have changed
        customer.add_name_hash_to_profile();

        self.storage
            .upsert_customer(&customer)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;
        Ok(())
    }

    /// Add a verified relationship
    pub async fn add_relationship(
        &self,
        customer_id: &str,
        relationship_type: &str,
        related_identifier: &str,
        proof: Option<Value>,
    ) -> Result<()> {
        let relationship = CustomerRelationship {
            id: Uuid::new_v4().to_string(),
            customer_id: customer_id.to_string(),
            relationship_type: relationship_type.to_string(),
            related_identifier: related_identifier.to_string(),
            proof,
            confirmed_at: Some(Utc::now().to_rfc3339()),
            created_at: Utc::now().to_rfc3339(),
        };

        self.storage
            .add_customer_relationship(&relationship)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;
        Ok(())
    }

    // Helper methods

    fn determine_customer_id(&self, account: &str) -> (String, String) {
        // If it's a DID, use it as the customer ID
        if account.starts_with("did:") {
            (account.to_string(), account.to_string())
        } else if account.contains('@') {
            // Email address - create a stable ID
            let id = format!("customer:{}", Uuid::new_v4());
            (id, format!("mailto:{}", account))
        } else if account.starts_with("http://") || account.starts_with("https://") {
            // URL - create did:web
            let domain = account
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            let did_web = format!("did:web:{}", domain.replace('/', ":"));
            (did_web.clone(), did_web)
        } else if account.starts_with('+') || account.chars().all(|c| c.is_digit(10) || c == '-') {
            // Phone number
            let id = format!("customer:{}", Uuid::new_v4());
            (id, format!("tel:{}", account))
        } else {
            // Generic identifier
            let id = format!("customer:{}", Uuid::new_v4());
            (id, account.to_string())
        }
    }

    fn determine_identifier_type(&self, identifier: &str) -> IdentifierType {
        if identifier.starts_with("did:") {
            IdentifierType::Did
        } else if identifier.starts_with("mailto:") {
            IdentifierType::Email
        } else if identifier.starts_with("tel:") || identifier.starts_with("sms:") {
            IdentifierType::Phone
        } else if identifier.starts_with("http://") || identifier.starts_with("https://") {
            IdentifierType::Url
        } else if identifier.contains(':') && identifier.contains('/') {
            // Likely a CAIP account identifier
            IdentifierType::Account
        } else {
            IdentifierType::Other
        }
    }

    async fn extract_additional_identifiers(&self, customer_id: &str, account: &str) -> Result<()> {
        // If account contains multiple identifiers (e.g., "did:key:xyz, email:user@example.com")
        if account.contains(',') {
            for part in account.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    let identifier = CustomerIdentifier {
                        id: trimmed.to_string(),
                        customer_id: customer_id.to_string(),
                        identifier_type: self.determine_identifier_type(trimmed),
                        verified: false,
                        verification_method: None,
                        verified_at: None,
                        created_at: Utc::now().to_rfc3339(),
                    };
                    let _ = self.storage.add_customer_identifier(&identifier).await;
                }
            }
        }
        Ok(())
    }

    fn extract_structured_data(
        &self,
        profile: &Value,
    ) -> (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        let mut given_name = None;
        let mut family_name = None;
        let mut display_name = None;
        let mut address_country = None;

        if let Value::Object(map) = profile {
            // Extract name components
            if let Some(Value::String(gn)) = map.get("givenName") {
                given_name = Some(gn.clone());
            }
            if let Some(Value::String(fn_)) = map.get("familyName") {
                family_name = Some(fn_.clone());
            }
            if let Some(Value::String(name)) = map.get("name") {
                display_name = Some(name.clone());
            }

            // Extract address
            if let Some(Value::Object(addr)) = map.get("address") {
                if let Some(Value::String(country)) = addr.get("addressCountry") {
                    address_country = Some(country.clone());
                }
            } else if let Some(Value::String(country)) = map.get("addressCountry") {
                address_country = Some(country.clone());
            }
        }

        (given_name, family_name, display_name, address_country)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_extract_customer_from_party() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = Arc::new(Storage::new(Some(db_path)).await.unwrap());

        let manager = CustomerManager::new(storage.clone());

        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), json!("Alice Smith"));
        let party = Party::with_metadata(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            metadata,
        );

        let customer_id = manager
            .extract_customer_from_party(&party, "did:key:agent", "originator")
            .await
            .unwrap();

        // Verify customer was created
        let customer = storage.get_customer(&customer_id).await.unwrap().unwrap();
        assert_eq!(customer.display_name, Some("Alice Smith".to_string()));
        assert_eq!(customer.agent_did, "did:key:agent");

        // Verify identifier was created
        let identifiers = storage
            .get_customer_identifiers(&customer_id)
            .await
            .unwrap();
        assert_eq!(identifiers.len(), 1);
        assert_eq!(identifiers[0].identifier_type, IdentifierType::Did);
    }

    #[tokio::test]
    async fn test_email_identifier() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = Arc::new(Storage::new(Some(db_path)).await.unwrap());

        let manager = CustomerManager::new(storage.clone());

        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), json!("Alice"));
        let party = Party::with_metadata("alice@example.com", metadata);

        let customer_id = manager
            .extract_customer_from_party(&party, "did:key:agent", "beneficiary")
            .await
            .unwrap();

        // Verify identifier is mailto format
        let identifiers = storage
            .get_customer_identifiers(&customer_id)
            .await
            .unwrap();
        assert_eq!(identifiers.len(), 1);
        assert_eq!(identifiers[0].id, "mailto:alice@example.com");
        assert_eq!(identifiers[0].identifier_type, IdentifierType::Email);
    }
}
