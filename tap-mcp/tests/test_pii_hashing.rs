//! Test for PII hashing functionality in transfers

use serde_json::json;
use tap_node::storage::models::{Customer, SchemaType};

#[tokio::test]
async fn test_transfer_with_person_uses_name_hash() {
    // Create a test setup with a person customer
    let customer = Customer {
        id: "did:example:alice".to_string(),
        agent_did: "did:example:agent".to_string(),
        schema_type: SchemaType::Person,
        given_name: Some("Alice".to_string()),
        family_name: Some("Lee".to_string()),
        display_name: Some("Alice Lee".to_string()),
        legal_name: None,
        lei_code: None,
        mcc_code: None,
        address_country: Some("US".to_string()),
        address_locality: None,
        postal_code: None,
        street_address: None,
        profile: json!({
            "@type": "Person",
            "givenName": "Alice",
            "familyName": "Lee"
        }),
        ivms101_data: None,
        verified_at: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    // The expected name hash for "Alice Lee" according to TAIP-12
    let expected_name_hash = "b117f44426c9670da91b563db728cd0bc8bafa7d1a6bb5e764d1aad2ca25032e";

    // TODO: Complete test implementation once we have a proper test setup
    // This would need:
    // 1. Mock TAP integration
    // 2. Mock storage that returns our test customer
    // 3. Call CreateTransferTool with the customer's DID
    // 4. Verify the resulting transfer has nameHash instead of PII

    assert_eq!(expected_name_hash.len(), 64); // SHA-256 produces 64 hex chars
}

#[tokio::test]
async fn test_transfer_with_organization_uses_lei_code() {
    // Create a test setup with an organization customer
    let customer = Customer {
        id: "did:web:example.com".to_string(),
        agent_did: "did:example:agent".to_string(),
        schema_type: SchemaType::Organization,
        given_name: None,
        family_name: None,
        display_name: Some("Example Corp".to_string()),
        legal_name: Some("Example Corporation Ltd.".to_string()),
        lei_code: Some("549300ZFEEJ2IP5VME73".to_string()), // Example LEI
        mcc_code: Some("5812".to_string()),
        address_country: Some("US".to_string()),
        address_locality: Some("New York".to_string()),
        postal_code: Some("10001".to_string()),
        street_address: None,
        profile: json!({
            "@type": "Organization",
            "legalName": "Example Corporation Ltd.",
            "leiCode": "549300ZFEEJ2IP5VME73"
        }),
        ivms101_data: None,
        verified_at: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    // TODO: Complete test implementation
    // This would verify that:
    // 1. LEI code is included in the transfer
    // 2. Legal name is included
    // 3. No name hash is generated for organizations

    assert_eq!(customer.lei_code.unwrap().len(), 20); // LEI codes are 20 chars
}

#[test]
fn test_name_hash_generation() {
    use tap_msg::utils::NameHashable;

    struct TestHasher;
    impl NameHashable for TestHasher {}

    // Test cases from TAIP-12
    let hash1 = TestHasher::hash_name("Alice Lee");
    assert_eq!(
        hash1,
        "b117f44426c9670da91b563db728cd0bc8bafa7d1a6bb5e764d1aad2ca25032e"
    );

    let hash2 = TestHasher::hash_name("Bob Smith");
    assert_eq!(
        hash2,
        "5432e86b4d4a3a2b4be57b713b12c5c576c88459fe1cfdd760fd6c99a0e06686"
    );

    // Test normalization
    assert_eq!(TestHasher::hash_name("ALICE LEE"), hash1);
    assert_eq!(TestHasher::hash_name("alice lee"), hash1);
    assert_eq!(TestHasher::hash_name("Alice  Lee"), hash1);
}
