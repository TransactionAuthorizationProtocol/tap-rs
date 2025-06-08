//! Tests for TAIP-12 name hashing in customer records

use serde_json::json;
use std::sync::Arc;
use tap_msg::message::Party;
use tap_node::customer::CustomerManager;
use tap_node::storage::{Customer, SchemaType, Storage};
use tempfile::tempdir;

#[tokio::test]
async fn test_customer_name_hash_generation() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage = Arc::new(Storage::new(Some(db_path)).await.unwrap());

    let manager = CustomerManager::new(storage.clone());

    // Create a party with name metadata
    let mut party = Party::new("did:example:alice");
    party.add_metadata("name".to_string(), json!("Alice Lee"));
    party.add_metadata("givenName".to_string(), json!("Alice"));
    party.add_metadata("familyName".to_string(), json!("Lee"));

    let customer_id = manager
        .extract_customer_from_party(&party, "did:example:agent", "originator")
        .await
        .unwrap();

    // Retrieve the customer and check the name hash
    let customer = storage.get_customer(&customer_id).await.unwrap().unwrap();

    // Verify name hash was generated
    let name_hash = customer.get_name_hash();
    assert_eq!(
        name_hash,
        Some("b117f44426c9670da91b563db728cd0bc8bafa7d1a6bb5e764d1aad2ca25032e".to_string())
    );
}

#[tokio::test]
async fn test_customer_preserve_existing_name_hash() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage = Arc::new(Storage::new(Some(db_path)).await.unwrap());

    let manager = CustomerManager::new(storage.clone());

    // Create a party with existing name hash
    let mut party = Party::new("did:example:bob");
    party.add_metadata("name".to_string(), json!("Bob Smith"));
    party.add_metadata("nameHash".to_string(), json!("existing_hash_12345"));

    let customer_id = manager
        .extract_customer_from_party(&party, "did:example:agent", "beneficiary")
        .await
        .unwrap();

    // Retrieve the customer and check the name hash
    let customer = storage.get_customer(&customer_id).await.unwrap().unwrap();

    // Should preserve the existing hash
    let name_hash = customer.get_name_hash();
    assert_eq!(name_hash, Some("existing_hash_12345".to_string()));
}

#[tokio::test]
async fn test_customer_organization_name_hash() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage = Arc::new(Storage::new(Some(db_path)).await.unwrap());

    // Create an organization customer manually
    let mut customer = Customer {
        id: "did:web:example.com".to_string(),
        agent_did: "did:example:agent".to_string(),
        schema_type: SchemaType::Organization,
        given_name: None,
        family_name: None,
        display_name: None,
        legal_name: Some("Example VASP Ltd.".to_string()),
        lei_code: None,
        mcc_code: None,
        address_country: None,
        address_locality: None,
        postal_code: None,
        street_address: None,
        profile: json!({
            "@context": "https://schema.org",
            "@type": "Organization",
            "legalName": "Example VASP Ltd."
        }),
        ivms101_data: None,
        verified_at: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    // Generate and add name hash
    customer.add_name_hash_to_profile();

    // Verify the hash was generated correctly
    let name_hash = customer.get_name_hash();
    assert!(name_hash.is_some());
    assert_eq!(name_hash.unwrap().len(), 64); // SHA-256 produces 64 hex chars
}

#[tokio::test]
async fn test_customer_update_regenerates_hash() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage = Arc::new(Storage::new(Some(db_path)).await.unwrap());

    let manager = CustomerManager::new(storage.clone());

    // Create initial customer
    let mut party = Party::new("did:example:charlie");
    party.add_metadata("givenName".to_string(), json!("Charlie"));
    party.add_metadata("familyName".to_string(), json!("Brown"));

    let customer_id = manager
        .extract_customer_from_party(&party, "did:example:agent", "originator")
        .await
        .unwrap();

    // Get initial hash
    let customer = storage.get_customer(&customer_id).await.unwrap().unwrap();
    let initial_hash = customer.get_name_hash();
    assert!(initial_hash.is_some());

    // Update customer profile with new name
    let update_data = json!({
        "givenName": "Charles",
        "familyName": "Brown"
    });

    manager
        .update_customer_profile(&customer_id, update_data)
        .await
        .unwrap();

    // Get updated customer
    let updated_customer = storage.get_customer(&customer_id).await.unwrap().unwrap();
    let updated_hash = updated_customer.get_name_hash();

    // Hash should be different due to name change
    assert!(updated_hash.is_some());
    assert_ne!(initial_hash, updated_hash);
}

#[tokio::test]
async fn test_customer_ivms101_name_hash() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage = Arc::new(Storage::new(Some(db_path)).await.unwrap());

    let manager = CustomerManager::new(storage.clone());

    // Create a customer with full details
    let mut party = Party::new("did:example:david");
    party.add_metadata("givenName".to_string(), json!("David"));
    party.add_metadata("familyName".to_string(), json!("Wilson"));
    party.add_metadata("addressCountry".to_string(), json!("US"));

    let customer_id = manager
        .extract_customer_from_party(&party, "did:example:agent", "originator")
        .await
        .unwrap();

    // Generate IVMS101 data (this should preserve the name hash)
    let _ivms_data = manager.generate_ivms101_data(&customer_id).await.unwrap();

    // Check that customer still has name hash after IVMS101 generation
    let customer = storage.get_customer(&customer_id).await.unwrap().unwrap();
    let name_hash = customer.get_name_hash();
    assert!(name_hash.is_some());

    // The hash should be for "David Wilson"
    let expected_hash = tap_msg::utils::hash_name("David Wilson");
    assert_eq!(name_hash.unwrap(), expected_hash);
}

#[tokio::test]
async fn test_customer_from_ivms101_generates_hash() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage = Arc::new(Storage::new(Some(db_path)).await.unwrap());

    let manager = CustomerManager::new(storage.clone());

    // First create a basic customer
    let party = Party::new("did:example:eve");
    let customer_id = manager
        .extract_customer_from_party(&party, "did:example:agent", "beneficiary")
        .await
        .unwrap();

    // Update customer from IVMS101 data
    let ivms101_data = json!({
        "naturalPerson": {
            "name": {
                "nameIdentifiers": [{
                    "primaryIdentifier": "Smith",
                    "secondaryIdentifier": "Eve",
                    "nameIdentifierType": "LEGL"
                }]
            }
        }
    });

    manager
        .update_customer_from_ivms101(&customer_id, &ivms101_data)
        .await
        .unwrap();

    // Check that name hash was generated
    let customer = storage.get_customer(&customer_id).await.unwrap().unwrap();
    let name_hash = customer.get_name_hash();
    assert!(name_hash.is_some());

    // The hash should be for "Eve Smith"
    let expected_hash = tap_msg::utils::hash_name("Eve Smith");
    assert_eq!(name_hash.unwrap(), expected_hash);
}
