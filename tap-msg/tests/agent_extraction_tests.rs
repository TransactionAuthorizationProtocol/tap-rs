extern crate tap_msg;

use std::collections::HashMap;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::{create_tap_message, TapMessageBody};
use tap_msg::message::types::{Participant, Transfer};

/// Tests that the to_didcomm method automatically extracts all agent DIDs when no sender is specified
#[test]
fn test_to_didcomm_extracts_all_agents_when_no_sender() {
    // Create a Transfer message with multiple agents
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    let agent1 = Participant {
        id: "did:web:agent1.example".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let agent2 = Participant {
        id: "did:web:agent2.example".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    let agent3 = Participant {
        id: "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb".to_string(),
        role: Some("settlementAddress".to_string()),
        policies: None,
        leiCode: None,
    };

    let body = Transfer {
        asset,
        originator: agent1.clone(),
        beneficiary: Some(agent2.clone()),
        amount: "100.00".to_string(),
        agents: vec![agent1.clone(), agent2.clone(), agent3.clone()],
        settlement_id: None,
        memo: Some("Test extraction".to_string()),
        metadata: HashMap::new(),
    };

    // Convert to DIDComm message with no sender specified
    let message = body.to_didcomm(None).unwrap();

    // Debug: Print the message
    println!("DEBUG: Message: {:?}", message);
    println!("DEBUG: Message to field: {:?}", message.to);
    println!(
        "DEBUG: Message body: {}",
        serde_json::to_string_pretty(&message.body).unwrap()
    );

    // Verify all agents are in the 'to' field
    assert!(message.from.is_none());
    assert!(message.to.is_some());

    let recipients = message.to.unwrap();
    assert_eq!(recipients.len(), 3);
    assert!(recipients.contains(&"did:web:agent1.example".to_string()));
    assert!(recipients.contains(&"did:web:agent2.example".to_string()));
    assert!(recipients
        .contains(&"did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb".to_string()));
}

/// Tests that the to_didcomm method excludes the sender from the 'to' field
#[test]
fn test_to_didcomm_excludes_sender_from_recipients() {
    // Create a Transfer message with multiple agents
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    let agent1 = Participant {
        id: "did:web:agent1.example".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let agent2 = Participant {
        id: "did:web:agent2.example".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    let agent3 = Participant {
        id: "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb".to_string(),
        role: Some("settlementAddress".to_string()),
        policies: None,
        leiCode: None,
    };

    let body = Transfer {
        asset,
        originator: agent1.clone(),
        beneficiary: Some(agent2.clone()),
        amount: "100.00".to_string(),
        agents: vec![agent1.clone(), agent2.clone(), agent3.clone()],
        settlement_id: None,
        memo: Some("Test extraction".to_string()),
        metadata: HashMap::new(),
    };

    // Convert to DIDComm message with sender specified as agent1
    let sender_did = "did:web:agent1.example";
    let message = body.to_didcomm(Some(sender_did)).unwrap();

    // Verify sender is in 'from' and not in 'to'
    assert_eq!(message.from, Some(sender_did.to_string()));
    assert!(message.to.is_some());

    let recipients = message.to.unwrap();
    assert_eq!(recipients.len(), 2);
    assert!(!recipients.contains(&sender_did.to_string()));
    assert!(recipients.contains(&"did:web:agent2.example".to_string()));
    assert!(recipients
        .contains(&"did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb".to_string()));
}

/// Tests that to_didcomm_with_route overrides the automatically extracted recipients
#[test]
fn test_to_didcomm_with_route_overrides_extracted_recipients() {
    // Create a Transfer message with multiple agents
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    let agent1 = Participant {
        id: "did:web:agent1.example".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let agent2 = Participant {
        id: "did:web:agent2.example".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    let agent3 = Participant {
        id: "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb".to_string(),
        role: Some("settlementAddress".to_string()),
        policies: None,
        leiCode: None,
    };

    let body = Transfer {
        asset,
        originator: agent1.clone(),
        beneficiary: Some(agent2.clone()),
        amount: "100.00".to_string(),
        agents: vec![agent1.clone(), agent2.clone(), agent3.clone()],
        settlement_id: None,
        memo: Some("Test extraction".to_string()),
        metadata: HashMap::new(),
    };

    // Create explicit recipients list that's different from the agents
    let sender_did = "did:web:agent1.example";
    let explicit_recipient = "did:web:explicit.recipient";
    let message = body
        .to_didcomm_with_route(Some(sender_did), [explicit_recipient].iter().copied())
        .unwrap();

    // Verify sender is in 'from' and only the explicit recipient is in 'to'
    assert_eq!(message.from, Some(sender_did.to_string()));
    assert!(message.to.is_some());

    let recipients = message.to.unwrap();
    assert_eq!(recipients.len(), 1);
    assert_eq!(recipients[0], explicit_recipient);
}

/// Tests that create_tap_message works correctly with automatic agent extraction
#[test]
fn test_create_tap_message_with_automatic_extraction() {
    // Create a Transfer message with multiple agents
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    let agent1 = Participant {
        id: "did:web:agent1.example".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let agent2 = Participant {
        id: "did:web:agent2.example".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    let body = Transfer {
        asset,
        originator: agent1.clone(),
        beneficiary: Some(agent2.clone()),
        amount: "100.00".to_string(),
        agents: vec![agent1.clone(), agent2.clone()],
        settlement_id: None,
        memo: Some("Test extraction".to_string()),
        metadata: HashMap::new(),
    };

    // Create message with specific ID but use automatic extraction for recipients
    let message_id = "test-message-id-123";
    let sender_did = "did:web:agent1.example";
    let message =
        create_tap_message(&body, Some(message_id.to_string()), Some(sender_did), &[]).unwrap();

    // Verify custom ID is set, sender is in 'from', and recipient is extracted from agents
    assert_eq!(message.id, message_id);
    assert_eq!(message.from, Some(sender_did.to_string()));
    assert!(message.to.is_some());

    let recipients = message.to.unwrap();
    assert_eq!(recipients.len(), 1);
    assert!(recipients.contains(&"did:web:agent2.example".to_string()));
}

/// Tests that create_tap_message overrides automatic extraction when explicit recipients are provided
#[test]
fn test_create_tap_message_with_explicit_recipients() {
    // Create a Transfer message with multiple agents
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    let agent1 = Participant {
        id: "did:web:agent1.example".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let agent2 = Participant {
        id: "did:web:agent2.example".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    let body = Transfer {
        asset,
        originator: agent1.clone(),
        beneficiary: Some(agent2.clone()),
        amount: "100.00".to_string(),
        agents: vec![agent1.clone(), agent2.clone()],
        settlement_id: None,
        memo: Some("Test extraction".to_string()),
        metadata: HashMap::new(),
    };

    // Create message with explicit recipients
    let message_id = "test-message-id-456";
    let sender_did = "did:web:agent1.example";
    let explicit_recipient = "did:web:explicit.recipient";
    let message = create_tap_message(
        &body,
        Some(message_id.to_string()),
        Some(sender_did),
        &[explicit_recipient],
    )
    .unwrap();

    // Verify only the explicit recipient is in 'to'
    assert_eq!(message.id, message_id);
    assert_eq!(message.from, Some(sender_did.to_string()));
    assert!(message.to.is_some());

    let recipients = message.to.unwrap();
    assert_eq!(recipients.len(), 1);
    assert_eq!(recipients[0], explicit_recipient);
}

/// Tests handling of empty agents array
#[test]
fn test_to_didcomm_with_empty_agents() {
    // Create a Transfer message with no agents
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    let originator = Participant {
        id: "did:web:originator.example".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let beneficiary = Participant {
        id: "did:web:beneficiary.example".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    let body = Transfer {
        asset,
        originator: originator.clone(),
        beneficiary: Some(beneficiary.clone()),
        amount: "100.00".to_string(),
        agents: vec![], // Empty agents array
        settlement_id: None,
        memo: Some("Test empty agents".to_string()),
        metadata: HashMap::new(),
    };

    // Convert to DIDComm message
    let message = body.to_didcomm(None).unwrap();

    // Verify 'to' field contains originator and beneficiary DIDs even when agents array is empty
    assert!(message.from.is_none());
    assert!(message.to.is_some());
    let to = message.to.as_ref().unwrap();
    assert_eq!(to.len(), 2);
    assert!(to.contains(&"did:web:originator.example".to_string()));
    assert!(to.contains(&"did:web:beneficiary.example".to_string()));
}
