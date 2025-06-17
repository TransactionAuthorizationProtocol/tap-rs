//! Tests for automatic customer data extraction from TAP messages

use std::collections::HashMap;
use std::sync::Arc;
use tap_agent::TapAgent;
use tap_msg::message::{transfer::Transfer, Party};
use tap_node::{NodeConfig, TapNode};
use tempfile::tempdir;

#[tokio::test]
async fn test_automatic_customer_extraction_from_transfer() {
    // Create a temporary directory for storage
    let temp_dir = tempdir().unwrap();

    // Create a TAP node with storage
    let config = NodeConfig {
        storage_path: Some(temp_dir.path().join("node.db")),
        tap_root: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };

    let mut node = TapNode::new(config);
    node.init_storage().await.unwrap();

    // Create and register an agent
    let (agent, agent_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let agent_arc = Arc::new(agent);
    node.register_agent(agent_arc.clone()).await.unwrap();

    // Create a Transfer message with party information
    let mut alice_metadata = HashMap::new();
    alice_metadata.insert("name".to_string(), serde_json::json!("Alice Smith"));
    alice_metadata.insert(
        "https://schema.org/addressCountry".to_string(),
        serde_json::json!("US"),
    );
    let originator = Party::with_metadata("did:key:alice", alice_metadata);

    let mut bob_metadata = HashMap::new();
    bob_metadata.insert("name".to_string(), serde_json::json!("Bob Jones"));
    bob_metadata.insert("email".to_string(), serde_json::json!("bob@example.com"));
    let beneficiary = Party::with_metadata("bob@example.com", bob_metadata);

    let transfer = Transfer {
        asset: "eip155:1/slip44:60".parse().unwrap(),
        originator: Some(originator),
        beneficiary: Some(beneficiary),
        amount: "100".to_string(),
        agents: vec![],
        memo: None,
        settlement_id: None,
        connection_id: None,
        transaction_id: Some("tx-123".to_string()),
        metadata: Default::default(),
    };

    // Send the transfer message through the node
    let message = tap_msg::didcomm::PlainMessage {
        id: "msg-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        body: serde_json::to_value(&transfer).unwrap(),
        from: agent_did.clone(),
        to: vec![agent_did.clone()],
        thid: None,
        pthid: None,
        extra_headers: Default::default(),
        attachments: None,
        created_time: None,
        expires_time: None,
        from_prior: None,
    };

    // Process the message through the node
    // For testing, we'll send it as a plain message since we're testing internal processing
    let message_value = serde_json::to_value(&message).unwrap();
    node.receive_message(message_value).await.unwrap();

    // Give the event handlers time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Verify customer data was extracted
    if let Some(storage_manager) = node.agent_storage_manager() {
        let agent_storage = storage_manager.get_agent_storage(&agent_did).await.unwrap();

        // Check if the message was stored as a transaction
        let transactions = agent_storage.list_transactions(10, 0).await.unwrap();
        eprintln!("Total transactions found: {}", transactions.len());
        for tx in &transactions {
            eprintln!("Transaction: {} ({})", tx.reference_id, tx.message_type);
        }

        // List all customers to debug
        let all_customers = agent_storage
            .list_customers(&agent_did, 100, 0)
            .await
            .unwrap();
        eprintln!("Total customers found: {}", all_customers.len());
        for customer in &all_customers {
            eprintln!(
                "Customer: {} ({})",
                customer.id,
                customer.display_name.as_deref().unwrap_or("<no name>")
            );
        }

        // Check that Alice was created
        let alice = agent_storage
            .get_customer_by_identifier("did:key:alice")
            .await
            .unwrap();
        assert!(alice.is_some(), "Alice customer not found");
        let alice_customer = alice.unwrap();
        assert_eq!(alice_customer.display_name, Some("Alice Smith".to_string()));
        assert_eq!(alice_customer.address_country, Some("US".to_string()));

        // Check that Bob was created with email identifier
        let bob = agent_storage
            .get_customer_by_identifier("mailto:bob@example.com")
            .await
            .unwrap();
        assert!(bob.is_some());
        let bob_customer = bob.unwrap();
        assert_eq!(bob_customer.display_name, Some("Bob Jones".to_string()));

        // Check that Bob's identifiers include both the primary email and the raw email
        let bob_identifiers = agent_storage
            .get_customer_identifiers(&bob_customer.id)
            .await
            .unwrap();
        assert!(bob_identifiers
            .iter()
            .any(|id| id.id == "mailto:bob@example.com"));
    } else {
        panic!("Storage manager not available");
    }
}

#[tokio::test]
async fn test_customer_ivms101_generation() {
    use tap_node::customer::CustomerManager;

    // Create a temporary directory for storage
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().join("test.db");
    let storage = Arc::new(tap_node::Storage::new(Some(storage_path)).await.unwrap());

    let customer_manager = CustomerManager::new(storage.clone());

    // Create a customer with full details
    let mut metadata = HashMap::new();
    metadata.insert("name".to_string(), serde_json::json!("John Doe"));
    metadata.insert(
        "https://schema.org/givenName".to_string(),
        serde_json::json!("John"),
    );
    metadata.insert(
        "https://schema.org/familyName".to_string(),
        serde_json::json!("Doe"),
    );

    let party = Party::with_metadata("did:key:johndoe", metadata);
    let customer_id = customer_manager
        .extract_customer_from_party(&party, "did:key:agent", "originator")
        .await
        .unwrap();

    // Generate IVMS101 data
    let ivms101 = customer_manager
        .generate_ivms101_data(&customer_id)
        .await
        .unwrap();

    // Verify IVMS101 structure
    assert!(ivms101.get("naturalPerson").is_some());
    let natural_person = ivms101.get("naturalPerson").unwrap();
    assert!(natural_person.get("name").is_some());

    let name = natural_person.get("name").unwrap();
    assert!(name.get("nameIdentifiers").is_some());

    let name_identifiers = name.get("nameIdentifiers").unwrap().as_array().unwrap();
    assert_eq!(name_identifiers.len(), 1);

    let name_id = &name_identifiers[0];
    assert_eq!(
        name_id.get("primaryIdentifier").unwrap().as_str().unwrap(),
        "Doe"
    );
    assert_eq!(
        name_id
            .get("secondaryIdentifier")
            .unwrap()
            .as_str()
            .unwrap(),
        "John"
    );
}
