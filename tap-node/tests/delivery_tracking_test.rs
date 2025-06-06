//! Test delivery tracking integration with message sending

use std::sync::Arc;
use tap_agent::TapAgent;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{Party, Transfer};
use tap_node::storage::models::{DeliveryStatus, DeliveryType};
use tap_node::{NodeConfig, TapNode};

#[tokio::test]
async fn test_delivery_tracking_with_send_message() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for test storage
    let temp_dir = tempfile::tempdir()?;
    let tap_root = temp_dir.path().to_path_buf();

    // Create node with custom tap root
    let config = NodeConfig {
        tap_root: Some(tap_root.clone()),
        ..Default::default()
    };
    let mut node = TapNode::new(config);
    node.init_storage().await?;

    // Create two agents for internal delivery test
    let (sender_agent, sender_did) = TapAgent::from_ephemeral_key().await?;
    let (recipient_agent, recipient_did) = TapAgent::from_ephemeral_key().await?;

    // Register both agents
    node.register_agent(Arc::new(sender_agent)).await?;
    node.register_agent(Arc::new(recipient_agent)).await?;

    // Create a proper Transfer struct first
    let transfer = Transfer {
        transaction_id: "tx-123".to_string(),
        asset: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".parse()?,
        amount: "100.00".to_string(),
        originator: Party::new(&sender_did),
        beneficiary: Some(Party::new(&recipient_did)),
        agents: vec![],
        settlement_id: None,
        memo: Some("Test transfer".to_string()),
        connection_id: None,
        metadata: std::collections::HashMap::new(),
    };

    // Create DIDComm message from the Transfer
    let test_message = transfer.to_didcomm(&sender_did)?;

    // Send the message
    let message_id = test_message.id.clone(); // Get the actual message ID
    let packed_message = node
        .send_message(sender_did.clone(), recipient_did.clone(), test_message)
        .await?;

    println!("Packed message: {}", packed_message);

    // Wait a moment for async operations to complete
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Check that delivery records were created
    if let Some(storage_manager) = node.agent_storage_manager() {
        let sender_storage = storage_manager.get_agent_storage(&sender_did).await?;

        // Get deliveries for the sender using the actual message ID
        let deliveries = sender_storage
            .get_deliveries_for_message(&message_id)
            .await?;

        println!("Found {} delivery records", deliveries.len());

        // Should have one delivery record for internal delivery
        assert!(!deliveries.is_empty(), "No delivery records found");

        let delivery = &deliveries[0];
        println!("Delivery record: {:?}", delivery);

        // Verify delivery details
        assert_eq!(delivery.message_id, message_id);
        assert_eq!(delivery.recipient_did, recipient_did);
        assert_eq!(delivery.delivery_type, DeliveryType::Internal);
        assert_eq!(delivery.status, DeliveryStatus::Success);
        assert!(delivery.delivered_at.is_some());
        assert!(delivery.error_message.is_none());

        // Verify that the signed message was stored
        assert!(!delivery.message_text.is_empty());
        assert_ne!(delivery.message_text, "test-message-123"); // Should be the packed/signed message

        println!("✅ Internal delivery tracking test passed!");
    } else {
        panic!("Storage manager not available");
    }

    Ok(())
}

#[tokio::test]
async fn test_external_delivery_tracking() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for test storage
    let temp_dir = tempfile::tempdir()?;
    let tap_root = temp_dir.path().to_path_buf();

    // Create node with custom tap root
    let config = NodeConfig {
        tap_root: Some(tap_root.clone()),
        ..Default::default()
    };
    let mut node = TapNode::new(config);
    node.init_storage().await?;

    // Create one agent for external delivery test
    let (sender_agent, sender_did) = TapAgent::from_ephemeral_key().await?;
    node.register_agent(Arc::new(sender_agent)).await?;

    // External recipient (not registered)
    let external_did = "did:example:external-recipient";

    // Create a proper Transfer struct first
    let transfer = Transfer {
        transaction_id: "tx-456".to_string(),
        asset: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".parse()?,
        amount: "200.00".to_string(),
        originator: Party::new(&sender_did),
        beneficiary: Some(Party::new(external_did)),
        agents: vec![],
        settlement_id: None,
        memo: Some("External transfer".to_string()),
        connection_id: None,
        metadata: std::collections::HashMap::new(),
    };

    // Create DIDComm message from the Transfer
    let test_message = transfer.to_didcomm(&sender_did)?;

    // Send the message
    let message_id = test_message.id.clone(); // Get the actual message ID
    let packed_message = node
        .send_message(sender_did.clone(), external_did.to_string(), test_message)
        .await?;

    println!("Packed message for external delivery: {}", packed_message);

    // Wait a moment for async operations to complete
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Check that delivery records were created for external delivery
    if let Some(storage_manager) = node.agent_storage_manager() {
        let sender_storage = storage_manager.get_agent_storage(&sender_did).await?;

        // Get deliveries for the sender using the actual message ID
        let deliveries = sender_storage
            .get_deliveries_for_message(&message_id)
            .await?;

        println!(
            "Found {} delivery records for external message",
            deliveries.len()
        );

        // Should have one delivery record for external delivery
        assert!(!deliveries.is_empty(), "No delivery records found");

        let delivery = &deliveries[0];
        println!("External delivery record: {:?}", delivery);

        // Verify delivery details
        assert_eq!(delivery.message_id, message_id);
        assert_eq!(delivery.recipient_did, external_did);
        assert_eq!(delivery.delivery_type, DeliveryType::Https);
        // External delivery should succeed with HTTP response (even if 403/404)
        assert_eq!(delivery.status, DeliveryStatus::Success);
        assert!(delivery.delivery_url.is_some());
        assert!(delivery.delivered_at.is_some()); // Delivered successfully
                                                  // Should have HTTP status code recorded
        assert!(delivery.last_http_status_code.is_some());

        // Verify that the signed message was stored
        assert!(!delivery.message_text.is_empty());

        println!("✅ External delivery tracking test passed!");
    } else {
        panic!("Storage manager not available");
    }

    Ok(())
}
