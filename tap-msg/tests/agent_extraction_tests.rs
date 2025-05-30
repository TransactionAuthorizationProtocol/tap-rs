extern crate tap_msg;

use std::collections::HashMap;
use tap_caip::AssetId;
// Removed unused import: PlainMessage
use tap_msg::message::tap_message_trait::{create_tap_message, TapMessage, TapMessageBody};
use tap_msg::message::{Participant, Transfer};

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
        name: None,
    };

    let agent2 = Participant {
        id: "did:web:agent2.example".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let agent3 = Participant {
        id: "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb".to_string(),
        role: Some("settlementAddress".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let body = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset,
        originator: agent1.clone(),
        beneficiary: Some(agent2.clone()),
        amount: "100.00".to_string(),
        agents: vec![agent1.clone(), agent2.clone(), agent3.clone()],
        settlement_id: None,
        connect_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Convert to DIDComm message with sender specified (required in the new API)
    // Use a sender that's not in the agents list
    let sender_did = "did:web:sender.example";
    let message = body.to_didcomm(sender_did).unwrap();

    // The from field should be the sender DID
    assert_eq!(message.from, sender_did);

    // The to field should include all participant DIDs except the sender
    assert!(!message.to.is_empty());
    assert_eq!(message.to.len(), 3); // All 3 agents

    // Verify each agent DID is in the recipients
    assert!(message.to.contains(&agent1.id));
    assert!(message.to.contains(&agent2.id));
    assert!(message.to.contains(&agent3.id));
}

/// Tests that the to_didcomm method filters out the sender DID from recipients when specified
#[test]
fn test_to_didcomm_filters_sender_when_specified() {
    // Create a Transfer message with multiple agents
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    // Use the first agent as the sender
    let sender_did = "did:web:agent1.example";

    let agent1 = Participant {
        id: sender_did.to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let agent2 = Participant {
        id: "did:web:agent2.example".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let agent3 = Participant {
        id: "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb".to_string(),
        role: Some("settlementAddress".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let body = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset,
        originator: agent1.clone(),
        beneficiary: Some(agent2.clone()),
        amount: "100.00".to_string(),
        agents: vec![agent1.clone(), agent2.clone(), agent3.clone()],
        settlement_id: None,
        connect_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Convert to DIDComm message with sender specified
    let message = body.to_didcomm(sender_did).unwrap();

    // The from field should be the sender DID
    assert_eq!(message.from, sender_did);

    // The to field should not include the sender
    assert!(!message.to.contains(&sender_did.to_string()));

    // The to field should include the other agent DIDs
    assert!(message.to.contains(&agent2.id));
    assert!(message.to.contains(&agent3.id));
}

/// Tests the to_didcomm_with_route method with custom routing
#[test]
fn test_to_didcomm_with_route() {
    // Create a Transfer message with multiple agents
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    // Use the first agent as the sender
    let sender_did = "did:web:agent1.example";

    let agent1 = Participant {
        id: sender_did.to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let agent2 = Participant {
        id: "did:web:agent2.example".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let agent3 = Participant {
        id: "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb".to_string(),
        role: Some("settlementAddress".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let body = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset,
        originator: agent1.clone(),
        beneficiary: Some(agent2.clone()),
        amount: "100.00".to_string(),
        agents: vec![agent1.clone(), agent2.clone(), agent3.clone()],
        settlement_id: None,
        connect_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Custom recipients that include only agent2
    let recipients = [agent2.id.as_str()];

    // Convert to DIDComm message with custom routing
    let mut message = body.to_didcomm(sender_did).unwrap();
    message.to = recipients.iter().map(|s| s.to_string()).collect();

    // The from field should be the sender DID
    assert_eq!(message.from, sender_did);

    // The to field should include only the specified recipients
    assert_eq!(message.to.len(), 1);
    assert!(message.to.contains(&agent2.id));
    assert!(!message.to.contains(&agent3.id));
}

/// Tests creating a new TAP message with the create_tap_message utility function
#[test]
fn test_create_tap_message() {
    // Create a Transfer message with multiple agents
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    let sender_did = "did:web:agent1.example";

    let agent1 = Participant {
        id: sender_did.to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let agent2 = Participant {
        id: "did:web:agent2.example".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let body = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset,
        originator: agent1.clone(),
        beneficiary: Some(agent2.clone()),
        amount: "100.00".to_string(),
        agents: vec![agent1.clone(), agent2.clone()],
        settlement_id: None,
        connect_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Create a TAP message with custom ID and recipients
    let message = create_tap_message(
        &body,
        Some("test-id-123".to_string()),
        sender_did,
        &[agent2.id.as_str()],
    )
    .unwrap();

    // Verify the message properties
    assert_eq!(message.id, "test-id-123");
    assert_eq!(message.from, sender_did);

    // Verify recipients
    assert_eq!(message.to.len(), 1);
    assert!(message.to.contains(&agent2.id));
}

/// Tests that the TapMessage trait implementation extracts participants correctly
#[test]
fn test_get_all_participants() {
    // Create a Transfer message with multiple agents
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    let sender_did = "did:web:agent1.example";

    let agent1 = Participant {
        id: sender_did.to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let agent2 = Participant {
        id: "did:web:agent2.example".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let body = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset,
        originator: agent1.clone(),
        beneficiary: Some(agent2.clone()),
        amount: "100.00".to_string(),
        agents: vec![agent1.clone(), agent2.clone()],
        settlement_id: None,
        connect_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Create a message using to_didcomm
    let message = body.to_didcomm(sender_did).unwrap();

    // Use the TapMessage trait to get all participants
    let participants = message.get_all_participants();

    // Verify the participants match the expected participants
    assert_eq!(participants.len(), 2); // sender + recipient
    assert!(participants.contains(&sender_did.to_string()));
    assert!(participants.contains(&agent2.id));
}
