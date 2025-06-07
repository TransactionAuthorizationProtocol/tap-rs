//! Test that transactions are stored in all involved agents' databases

use std::sync::Arc;
use tap_agent::TapAgent;
use tap_node::{NodeConfig, TapNode};
use tempfile::TempDir;

#[tokio::test]
async fn test_transfer_stored_in_all_agent_databases() {
    // Create temporary directory for storage
    let temp_dir = TempDir::new().unwrap();
    let tap_root = temp_dir.path().to_path_buf();

    // Create TAP Node with agent storage manager
    let mut config = NodeConfig::default();
    config.tap_root = Some(tap_root.clone());
    config.enable_message_logging = true;
    config.log_message_content = true;

    let node = Arc::new(TapNode::new(config));

    // Create three agents: originator, beneficiary, and a custodian
    let (originator_agent, originator_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (beneficiary_agent, beneficiary_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (custodian_agent, custodian_did) = TapAgent::from_ephemeral_key().await.unwrap();

    println!("Originator: {}", originator_did);
    println!("Beneficiary: {}", beneficiary_did);
    println!("Custodian: {}", custodian_did);

    // Register all agents
    node.register_agent(Arc::new(originator_agent))
        .await
        .unwrap();
    node.register_agent(Arc::new(beneficiary_agent))
        .await
        .unwrap();
    node.register_agent(Arc::new(custodian_agent))
        .await
        .unwrap();

    // Create a proper Transfer struct first
    let transfer = tap_msg::message::Transfer {
        transaction_id: "tx-456".to_string(),
        asset: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
            .parse()
            .unwrap(),
        amount: "1000.00".to_string(),
        originator: tap_msg::message::Party::new(&originator_did),
        beneficiary: Some(tap_msg::message::Party::new(&beneficiary_did)),
        agents: vec![tap_msg::message::Agent::new(
            &custodian_did,
            "Custodian",
            &originator_did,
        )],
        memo: None,
        settlement_id: None,
        connection_id: None,
        metadata: Default::default(),
    };

    // Create a proper PlainMessage
    let transfer_message = tap_msg::didcomm::PlainMessage {
        id: "transfer-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/message/transfer".to_string(),
        body: serde_json::to_value(&transfer).unwrap(),
        from: originator_did.clone(),
        to: vec![beneficiary_did.clone()],
        thid: None,
        pthid: None,
        extra_headers: Default::default(),
        attachments: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
    };

    // Convert to JSON for the receive_message call
    let transfer_message_json = serde_json::to_value(&transfer_message).unwrap();

    // Process the message
    node.receive_message(transfer_message_json).await.unwrap();

    // Allow some time for storage operations
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify the transaction appears in all three agents' databases
    let storage_manager = node.agent_storage_manager().unwrap();

    // Check originator's storage
    let originator_storage = storage_manager
        .get_agent_storage(&originator_did)
        .await
        .unwrap();
    let originator_transactions = originator_storage.list_transactions(10, 0).await.unwrap();
    assert!(
        !originator_transactions.is_empty(),
        "Originator should have the transaction"
    );
    assert_eq!(originator_transactions[0].reference_id, "transfer-123");
    // Verify the actual transaction_id is in the message body
    let tx_id = originator_transactions[0]
        .message_json
        .get("body")
        .and_then(|b| b.get("transaction_id"))
        .and_then(|t| t.as_str())
        .unwrap();
    assert_eq!(tx_id, "tx-456");
    println!("✓ Transaction found in originator's database");

    // Check beneficiary's storage
    let beneficiary_storage = storage_manager
        .get_agent_storage(&beneficiary_did)
        .await
        .unwrap();
    let beneficiary_transactions = beneficiary_storage.list_transactions(10, 0).await.unwrap();
    assert!(
        !beneficiary_transactions.is_empty(),
        "Beneficiary should have the transaction"
    );
    assert_eq!(beneficiary_transactions[0].reference_id, "transfer-123");
    println!("✓ Transaction found in beneficiary's database");

    // Check custodian's storage
    let custodian_storage = storage_manager
        .get_agent_storage(&custodian_did)
        .await
        .unwrap();
    let custodian_transactions = custodian_storage.list_transactions(10, 0).await.unwrap();
    assert!(
        !custodian_transactions.is_empty(),
        "Custodian should have the transaction"
    );
    assert_eq!(custodian_transactions[0].reference_id, "transfer-123");
    println!("✓ Transaction found in custodian's database");

    println!("✓ All agents have the transaction in their respective databases");
}

#[tokio::test]
async fn test_payment_stored_in_all_agent_databases() {
    // Create temporary directory for storage
    let temp_dir = TempDir::new().unwrap();
    let tap_root = temp_dir.path().to_path_buf();

    // Create TAP Node
    let mut config = NodeConfig::default();
    config.tap_root = Some(tap_root.clone());
    config.enable_message_logging = true;

    let node = Arc::new(TapNode::new(config));

    // Create three agents: customer, merchant, and payment processor
    let (customer_agent, customer_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (merchant_agent, merchant_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (processor_agent, processor_did) = TapAgent::from_ephemeral_key().await.unwrap();

    // Register all agents
    node.register_agent(Arc::new(customer_agent)).await.unwrap();
    node.register_agent(Arc::new(merchant_agent)).await.unwrap();
    node.register_agent(Arc::new(processor_agent))
        .await
        .unwrap();

    // Create a payment message
    let payment_message = serde_json::json!({
        "id": "payment-789",
        "type": "https://tap.rsvp/message/payment",
        "from": customer_did,
        "to": [merchant_did.clone()],
        "created_time": chrono::Utc::now().timestamp(),
        "body": {
            "transaction_id": "pay-101",
            "amount": "50.00",
            "currency": "USD",
            "customer": {
                "@id": customer_did
            },
            "merchant": {
                "@id": merchant_did
            },
            "agents": [
                {
                    "@id": processor_did,
                    "role": "PaymentProcessor",
                    "for": merchant_did
                }
            ]
        }
    });

    // Process the message
    node.receive_message(payment_message).await.unwrap();

    // Allow time for storage
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify all agents have the transaction
    let storage_manager = node.agent_storage_manager().unwrap();

    // Check each agent's storage
    for (agent_did, agent_name) in [
        (&customer_did, "Customer"),
        (&merchant_did, "Merchant"),
        (&processor_did, "Payment Processor"),
    ] {
        let storage = storage_manager.get_agent_storage(agent_did).await.unwrap();
        let transactions = storage.list_transactions(10, 0).await.unwrap();
        assert!(
            !transactions.is_empty(),
            "{} should have the transaction",
            agent_name
        );
        assert_eq!(transactions[0].reference_id, "payment-789");
        println!("✓ Transaction found in {}'s database", agent_name);
    }
}

#[tokio::test]
async fn test_non_transaction_message_single_storage() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let tap_root = temp_dir.path().to_path_buf();

    // Create TAP Node
    let mut config = NodeConfig::default();
    config.tap_root = Some(tap_root.clone());
    config.enable_message_logging = true;

    let node = Arc::new(TapNode::new(config));

    // Create two agents
    let (agent1, did1) = TapAgent::from_ephemeral_key().await.unwrap();
    let (agent2, did2) = TapAgent::from_ephemeral_key().await.unwrap();

    node.register_agent(Arc::new(agent1)).await.unwrap();
    node.register_agent(Arc::new(agent2)).await.unwrap();

    // Create a non-transaction message (e.g., a basic message)
    let basic_message = serde_json::json!({
        "id": "msg-111",
        "type": "basic-message",
        "from": did1,
        "to": [did2.clone()],
        "created_time": chrono::Utc::now().timestamp(),
        "body": {
            "content": "Hello, this is not a transaction"
        }
    });

    // Process the message
    node.receive_message(basic_message).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify the message is only in recipient's message log (not in transactions)
    let storage_manager = node.agent_storage_manager().unwrap();

    // Check sender's storage - should have no transactions
    let sender_storage = storage_manager.get_agent_storage(&did1).await.unwrap();
    let sender_transactions = sender_storage.list_transactions(10, 0).await.unwrap();
    assert!(
        sender_transactions.is_empty(),
        "Sender should have no transactions"
    );

    // Check recipient's storage - should have no transactions
    let recipient_storage = storage_manager.get_agent_storage(&did2).await.unwrap();
    let recipient_transactions = recipient_storage.list_transactions(10, 0).await.unwrap();
    assert!(
        recipient_transactions.is_empty(),
        "Recipient should have no transactions"
    );

    // But the message should be in the message log
    let recipient_messages = recipient_storage.list_messages(10, 0, None).await.unwrap();
    assert!(
        !recipient_messages.is_empty(),
        "Recipient should have the message in log"
    );
    assert_eq!(recipient_messages[0].message_id, "msg-111");

    println!("✓ Non-transaction messages are not stored as transactions");
}

#[tokio::test]
async fn test_outgoing_transaction_multi_storage() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let tap_root = temp_dir.path().to_path_buf();

    // Create TAP Node
    let mut config = NodeConfig::default();
    config.tap_root = Some(tap_root.clone());
    config.enable_message_logging = true;

    let node = Arc::new(TapNode::new(config));

    // Create agents
    let (sender_agent, sender_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (recipient_agent, recipient_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (escrow_agent, escrow_did) = TapAgent::from_ephemeral_key().await.unwrap();

    node.register_agent(Arc::new(sender_agent)).await.unwrap();
    node.register_agent(Arc::new(recipient_agent))
        .await
        .unwrap();
    node.register_agent(Arc::new(escrow_agent)).await.unwrap();

    // Create an outgoing transfer message
    let transfer = tap_msg::message::Transfer {
        transaction_id: "out-tx-222".to_string(),
        asset: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
            .parse()
            .unwrap(),
        amount: "500.00".to_string(),
        originator: tap_msg::message::Party::new(&sender_did),
        beneficiary: Some(tap_msg::message::Party::new(&recipient_did)),
        agents: vec![tap_msg::message::Agent::new(
            &escrow_did,
            "Escrow",
            &sender_did,
        )],
        memo: Some("Test outgoing transfer".to_string()),
        settlement_id: None,
        connection_id: None,
        metadata: Default::default(),
    };

    // Convert to DIDComm message
    use tap_msg::message::tap_message_trait::TapMessageBody;
    let didcomm_message = transfer.to_didcomm(&sender_did).unwrap();

    // Send the message (this should trigger outgoing storage)
    node.send_message(sender_did.clone(), didcomm_message)
        .await
        .unwrap();

    // Allow time for storage
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify all involved agents have the transaction
    let storage_manager = node.agent_storage_manager().unwrap();

    for (agent_did, agent_name) in [
        (&sender_did, "Sender"),
        (&recipient_did, "Recipient"),
        (&escrow_did, "Escrow Agent"),
    ] {
        let storage = storage_manager.get_agent_storage(agent_did).await.unwrap();
        let transactions = storage.list_transactions(10, 0).await.unwrap();
        assert!(
            !transactions.is_empty(),
            "{} should have the outgoing transaction",
            agent_name
        );
        // Check that the transaction is stored (reference_id will be a UUID generated by DIDComm)
        // We'll verify the transaction_id from the message body instead
        let tx_body = transactions[0]
            .message_json
            .get("body")
            .and_then(|b| b.get("transaction_id"))
            .and_then(|t| t.as_str())
            .unwrap();
        assert_eq!(tx_body, "out-tx-222");
        println!("✓ Outgoing transaction found in {}'s database", agent_name);
    }

    println!("✓ Outgoing transactions are stored in all involved agents' databases");
}

#[tokio::test]
async fn test_message_delivered_to_all_recipients() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let tap_root = temp_dir.path().to_path_buf();

    // Create TAP Node
    let mut config = NodeConfig::default();
    config.tap_root = Some(tap_root.clone());
    config.enable_message_logging = true;

    let node = Arc::new(TapNode::new(config));

    // Create three agents that will all be recipients
    let (agent1, did1) = TapAgent::from_ephemeral_key().await.unwrap();
    let (agent2, did2) = TapAgent::from_ephemeral_key().await.unwrap();
    let (agent3, did3) = TapAgent::from_ephemeral_key().await.unwrap();
    let (sender_agent, sender_did) = TapAgent::from_ephemeral_key().await.unwrap();

    node.register_agent(Arc::new(agent1)).await.unwrap();
    node.register_agent(Arc::new(agent2)).await.unwrap();
    node.register_agent(Arc::new(agent3)).await.unwrap();
    node.register_agent(Arc::new(sender_agent)).await.unwrap();

    // Create a message with multiple recipients in the 'to' field
    let multi_recipient_message = serde_json::json!({
        "id": "multi-msg-123",
        "type": "basic-message",
        "from": sender_did,
        "to": [did1.clone(), did2.clone(), did3.clone()],
        "created_time": chrono::Utc::now().timestamp(),
        "body": {
            "content": "Hello to all recipients"
        }
    });

    // Process the message
    node.receive_message(multi_recipient_message).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify the message was logged in all three recipients' storage
    let storage_manager = node.agent_storage_manager().unwrap();

    for (agent_did, agent_name) in [(&did1, "Agent 1"), (&did2, "Agent 2"), (&did3, "Agent 3")] {
        let storage = storage_manager.get_agent_storage(agent_did).await.unwrap();
        let messages = storage.list_messages(10, 0, None).await.unwrap();

        // Find the message by ID
        let found_message = messages.iter().find(|m| m.message_id == "multi-msg-123");
        assert!(
            found_message.is_some(),
            "{} should have received the message",
            agent_name
        );

        println!("✓ Message delivered to {}", agent_name);
    }

    println!("✓ Message successfully delivered to all recipients");
}

#[tokio::test]
async fn test_transaction_delivered_to_all_recipients() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let tap_root = temp_dir.path().to_path_buf();

    // Create TAP Node
    let mut config = NodeConfig::default();
    config.tap_root = Some(tap_root.clone());
    config.enable_message_logging = true;

    let node = Arc::new(TapNode::new(config));

    // Create three agents that will all be recipients
    let (originator_agent, originator_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (beneficiary_agent, beneficiary_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (custodian_agent, custodian_did) = TapAgent::from_ephemeral_key().await.unwrap();

    node.register_agent(Arc::new(originator_agent))
        .await
        .unwrap();
    node.register_agent(Arc::new(beneficiary_agent))
        .await
        .unwrap();
    node.register_agent(Arc::new(custodian_agent))
        .await
        .unwrap();

    // Create a transfer message with multiple recipients in the 'to' field
    let transfer = tap_msg::message::Transfer {
        transaction_id: "multi-recipient-tx-123".to_string(),
        asset: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
            .parse()
            .unwrap(),
        amount: "500.00".to_string(),
        originator: tap_msg::message::Party::new(&originator_did),
        beneficiary: Some(tap_msg::message::Party::new(&beneficiary_did)),
        agents: vec![tap_msg::message::Agent::new(
            &custodian_did,
            "Custodian",
            &originator_did,
        )],
        memo: None,
        settlement_id: None,
        connection_id: None,
        metadata: Default::default(),
    };

    // Create a PlainMessage with ALL three agents as recipients
    let transfer_message = tap_msg::didcomm::PlainMessage {
        id: "multi-recipient-transfer-456".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/message/transfer".to_string(),
        body: serde_json::to_value(&transfer).unwrap(),
        from: originator_did.clone(),
        to: vec![
            originator_did.clone(),
            beneficiary_did.clone(),
            custodian_did.clone(),
        ], // All three as recipients
        thid: None,
        pthid: None,
        extra_headers: Default::default(),
        attachments: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
    };

    let transfer_message_json = serde_json::to_value(&transfer_message).unwrap();

    // Process the message
    node.receive_message(transfer_message_json).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify the transaction was stored in all three recipients' databases
    let storage_manager = node.agent_storage_manager().unwrap();

    for (agent_did, agent_name) in [
        (&originator_did, "Originator"),
        (&beneficiary_did, "Beneficiary"),
        (&custodian_did, "Custodian"),
    ] {
        let storage = storage_manager.get_agent_storage(agent_did).await.unwrap();

        // Check transactions (should be stored because it's a transfer)
        let transactions = storage.list_transactions(10, 0).await.unwrap();
        let found_transaction = transactions
            .iter()
            .find(|t| t.reference_id == "multi-recipient-transfer-456");
        assert!(
            found_transaction.is_some(),
            "{} should have the transaction",
            agent_name
        );

        // Check messages (should be logged because they're recipients)
        let messages = storage.list_messages(10, 0, None).await.unwrap();
        let found_message = messages
            .iter()
            .find(|m| m.message_id == "multi-recipient-transfer-456");
        assert!(
            found_message.is_some(),
            "{} should have received the message",
            agent_name
        );

        println!("✓ Transaction delivered and stored for {}", agent_name);
    }

    println!("✓ Transaction successfully delivered to all recipients");
}
