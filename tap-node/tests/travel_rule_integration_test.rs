//! Integration test for TAIP-10 Travel Rule implementation

use std::str::FromStr;
use std::sync::Arc;
use tap_agent::{AgentKey, LocalAgentKey};
use tap_caip::AssetId;
use tap_ivms101::{
    builder::{
        GeographicAddressBuilder, IvmsMessageBuilder, NaturalPersonBuilder,
        NaturalPersonNameBuilder,
    },
    message::Person,
    types::{AddressType, TransactionDirection},
};
use tap_msg::{
    didcomm::PlainMessage,
    message::{Agent as TapAgent, Party, Transfer},
};
use tap_node::{
    customer::CustomerManager,
    message::{PlainMessageProcessor, TravelRuleProcessor},
    storage::Storage,
};
use tempfile::tempdir;
use uuid::Uuid;

#[tokio::test]
async fn test_travel_rule_flow_with_ivms101() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // Create storage and customer manager
    let storage = Arc::new(Storage::new(Some(db_path.clone())).await.unwrap());
    let customer_manager = Arc::new(CustomerManager::new(storage.clone()));

    // Create agents
    let originator_agent = LocalAgentKey::generate_ed25519("originator").unwrap();
    let beneficiary_agent = LocalAgentKey::generate_ed25519("beneficiary").unwrap();

    // Create customer profiles
    let originator_party = Party::with_metadata(
        "did:example:alice",
        [
            ("givenName".to_string(), serde_json::json!("Alice")),
            ("familyName".to_string(), serde_json::json!("Smith")),
            ("addressCountry".to_string(), serde_json::json!("US")),
            (
                "streetAddress".to_string(),
                serde_json::json!("123 Main St"),
            ),
        ]
        .into(),
    );

    let beneficiary_party = Party::with_metadata(
        "did:example:bob",
        [
            ("givenName".to_string(), serde_json::json!("Bob")),
            ("familyName".to_string(), serde_json::json!("Jones")),
            ("addressCountry".to_string(), serde_json::json!("US")),
            (
                "streetAddress".to_string(),
                serde_json::json!("456 Oak Ave"),
            ),
        ]
        .into(),
    );

    // Extract customers from parties
    customer_manager
        .extract_customer_from_party(&originator_party, originator_agent.did(), "originator")
        .await
        .unwrap();

    customer_manager
        .extract_customer_from_party(&beneficiary_party, beneficiary_agent.did(), "beneficiary")
        .await
        .unwrap();

    // Generate IVMS101 data for originator
    let ivms101_data = customer_manager
        .generate_ivms101_data(&originator_party.id)
        .await
        .unwrap();

    println!(
        "Generated IVMS101 data: {}",
        serde_json::to_string_pretty(&ivms101_data).unwrap()
    );

    // Create a transfer message
    let mut transfer = Transfer::builder()
        .originator(originator_party.clone())
        .beneficiary(beneficiary_party.clone())
        .asset(AssetId::from_str("eip155:1/slip44:60").unwrap())
        .amount("1.23".to_string())
        .build();

    // Add agents
    transfer.agents.push(TapAgent::new(
        originator_agent.did(),
        "originator",
        &originator_party.id,
    ));
    transfer.agents.push(TapAgent::new(
        beneficiary_agent.did(),
        "beneficiary",
        &beneficiary_party.id,
    ));

    // Create travel rule processor
    let travel_rule_processor = TravelRuleProcessor::new(customer_manager.clone());

    // Create a plain message from the transfer
    let mut plain_message = PlainMessage::new(
        Uuid::new_v4().to_string(),
        "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        serde_json::to_value(&transfer).unwrap(),
        originator_agent.did().to_string(),
    );
    plain_message.to = vec![beneficiary_agent.did().to_string()];
    plain_message.created_time = Some(chrono::Utc::now().timestamp_millis() as u64);

    // Process outgoing message (should attach IVMS101 data)
    let processed_message = travel_rule_processor
        .process_outgoing(plain_message.clone())
        .await
        .unwrap()
        .unwrap();

    // Verify IVMS101 attachment was added
    assert!(processed_message.attachments.is_some());
    let attachments = processed_message.attachments.as_ref().unwrap();
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].id, Some("ivms101-vp".to_string()));

    // Simulate receiving the message on the beneficiary side
    let _received_message = travel_rule_processor
        .process_incoming(processed_message)
        .await
        .unwrap()
        .unwrap();

    println!("Travel rule integration test completed successfully!");
}

#[tokio::test]
async fn test_ivms101_generation_and_parsing() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage = Arc::new(Storage::new(Some(db_path)).await.unwrap());
    let customer_manager = CustomerManager::new(storage);

    // Create a detailed customer profile
    let party = Party::with_metadata(
        "did:example:alice",
        [
            ("givenName".to_string(), serde_json::json!("Alice")),
            ("familyName".to_string(), serde_json::json!("Smith")),
            ("addressCountry".to_string(), serde_json::json!("US")),
            ("addressLocality".to_string(), serde_json::json!("New York")),
            ("postalCode".to_string(), serde_json::json!("10001")),
            (
                "streetAddress".to_string(),
                serde_json::json!("123 Main St"),
            ),
        ]
        .into(),
    );

    // Extract customer
    let customer_id = customer_manager
        .extract_customer_from_party(&party, "did:example:agent", "originator")
        .await
        .unwrap();

    // Generate IVMS101 data
    let ivms101_data = customer_manager
        .generate_ivms101_data(&customer_id)
        .await
        .unwrap();

    println!(
        "Generated IVMS101 data: {}",
        serde_json::to_string_pretty(&ivms101_data).unwrap()
    );

    // Verify the structure
    assert!(ivms101_data.get("naturalPerson").is_some());
    let natural_person = ivms101_data.get("naturalPerson").unwrap();
    assert!(natural_person.get("name").is_some());
    // Address will be present after adding street address
    if natural_person.get("geographicAddresses").is_some() {
        assert!(!natural_person
            .get("geographicAddresses")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty());
    }

    // Update customer from IVMS101 data
    let test_ivms101 = serde_json::json!({
        "naturalPerson": {
            "name": {
                "nameIdentifiers": [{
                    "primaryIdentifier": "Johnson",
                    "secondaryIdentifier": "Robert",
                    "nameIdentifierType": "LEGL"
                }]
            },
            "geographicAddress": [{
                "addressType": "HOME",
                "streetName": "456 Oak Ave",
                "postCode": "10002",
                "townName": "Brooklyn",
                "country": "US"
            }]
        }
    });

    customer_manager
        .update_customer_from_ivms101(&customer_id, &test_ivms101)
        .await
        .unwrap();

    // Verify the update
    let updated_ivms = customer_manager
        .generate_ivms101_data(&customer_id)
        .await
        .unwrap();

    println!(
        "Updated IVMS101 data: {}",
        serde_json::to_string_pretty(&updated_ivms).unwrap()
    );

    // Check that the data was updated
    let updated_person = updated_ivms.get("naturalPerson").unwrap();
    let name_ids = updated_person["name"]["nameIdentifiers"]
        .as_array()
        .unwrap();
    assert_eq!(name_ids[0]["primaryIdentifier"], "Johnson");
    assert_eq!(name_ids[0]["secondaryIdentifier"], "Robert");
}

#[tokio::test]
async fn test_ivms101_builder_integration() {
    // Test creating IVMS101 message using builders
    let originator_person = NaturalPersonBuilder::new()
        .name(
            NaturalPersonNameBuilder::new()
                .legal_name("Smith", "Alice")
                .build()
                .unwrap(),
        )
        .add_address(
            GeographicAddressBuilder::new()
                .address_type(AddressType::Home)
                .street_name("123 Main St")
                .post_code("10001")
                .town_name("New York")
                .country("US")
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    let beneficiary_person = NaturalPersonBuilder::new()
        .name(
            NaturalPersonNameBuilder::new()
                .legal_name("Jones", "Bob")
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    let ivms_message = IvmsMessageBuilder::new()
        .originator(vec![Person::NaturalPerson(originator_person)])
        .beneficiary(vec![Person::NaturalPerson(beneficiary_person)])
        .originating_vasp(Person::NaturalPerson(
            NaturalPersonBuilder::new()
                .name(
                    NaturalPersonNameBuilder::new()
                        .legal_name("VASP", "Originating")
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
        ))
        .beneficiary_vasp(Person::NaturalPerson(
            NaturalPersonBuilder::new()
                .name(
                    NaturalPersonNameBuilder::new()
                        .legal_name("VASP", "Beneficiary")
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
        ))
        .transaction(
            "1.23",
            "USD",
            TransactionDirection::Outgoing,
            "eip155:1",
            chrono::Utc::now().to_rfc3339(),
        )
        .unwrap()
        .build()
        .unwrap();

    // Serialize to JSON
    let json = serde_json::to_value(&ivms_message).unwrap();
    println!(
        "Complete IVMS101 message: {}",
        serde_json::to_string_pretty(&json).unwrap()
    );

    // Verify structure
    assert!(json.get("originator").is_some());
    assert!(json.get("beneficiary").is_some());
    assert!(json.get("transaction").is_some());
}
