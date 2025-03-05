use chrono;
use didcomm::Message;
use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::error::Result;
use tap_msg::message::types::{AddAgents, Authorize, Participant, RemoveAgent, ReplaceAgent};
use tap_msg::message::{TapMessage, TapMessageBody, Transfer};
use uuid::Uuid;

#[test]
fn test_create_reply() -> Result<()> {
    // Create original message
    let alice_did = "did:example:alice";
    let bob_did = "did:example:bob";
    // Using an underscore prefix for unused variable
    let _charlie_did = "did:example:charlie";

    // Create a transfer from Alice to Bob
    let transfer = Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap(),
        originator: Participant {
            id: alice_did.to_string(),
            role: Some("originator".to_string()),
        },
        beneficiary: Some(Participant {
            id: bob_did.to_string(),
            role: Some("beneficiary".to_string()),
        }),
        amount: "10.00".to_string(),
        agents: vec![],
        settlement_id: None,
        memo: Some("Test transfer".to_string()),
        metadata: HashMap::new(),
    };

    // Create the initial transfer message
    let transfer_message =
        transfer.to_didcomm_with_route(Some(alice_did), [bob_did].iter().copied())?;

    // Create an authorize response from Bob to Alice
    let authorize = Authorize {
        transfer_id: transfer_message.id.clone(),
        note: Some("Transfer authorized".to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
        metadata: HashMap::new(),
    };

    // Create a reply using the create_reply method
    let reply_message =
        authorize.create_reply(&transfer_message, bob_did, &[alice_did, bob_did])?;

    // Verify the reply has the correct properties
    assert_eq!(reply_message.from, Some(bob_did.to_string()));
    assert!(reply_message
        .to
        .as_ref()
        .unwrap()
        .contains(&alice_did.to_string()));
    assert!(!reply_message
        .to
        .as_ref()
        .unwrap()
        .contains(&bob_did.to_string()));
    assert_eq!(reply_message.thid, Some(transfer_message.id.clone()));

    // Test the message extension method
    let reply_via_message = transfer_message.create_reply(&authorize, bob_did)?;

    // Verify the reply created via the Message trait has the same properties
    assert_eq!(reply_via_message.from, Some(bob_did.to_string()));
    assert!(reply_via_message
        .to
        .as_ref()
        .unwrap()
        .contains(&alice_did.to_string()));
    assert!(!reply_via_message
        .to
        .as_ref()
        .unwrap()
        .contains(&bob_did.to_string()));
    assert_eq!(reply_via_message.thid, Some(transfer_message.id.clone()));

    Ok(())
}

#[test]
fn test_add_agents() -> Result<()> {
    let transfer_id = "test-transfer-123";
    let alice_did = "did:example:alice";
    let bob_did = "did:example:bob";
    let charlie_did = "did:example:charlie";

    // Create an AddAgents message to add Charlie
    let add_agents = AddAgents {
        transfer_id: transfer_id.to_string(),
        agents: vec![Participant {
            id: charlie_did.to_string(),
            role: Some("observer".to_string()),
        }],
        metadata: HashMap::new(),
    };

    // Validate the message
    add_agents.validate()?;

    // Create a DIDComm message from the add_agents
    let message = add_agents
        .to_didcomm_with_route(Some(alice_did), [bob_did, charlie_did].iter().copied())?;

    // Set the thread ID
    let message_with_thread = Message {
        thid: Some(transfer_id.to_string()),
        ..message
    };

    // Verify the message properties
    assert_eq!(message_with_thread.from, Some(alice_did.to_string()));
    assert!(message_with_thread
        .to
        .as_ref()
        .unwrap()
        .contains(&bob_did.to_string()));
    assert!(message_with_thread
        .to
        .as_ref()
        .unwrap()
        .contains(&charlie_did.to_string()));
    assert_eq!(message_with_thread.thid, Some(transfer_id.to_string()));

    // Extract the body back and verify
    let extracted_add_agents = AddAgents::from_didcomm(&message_with_thread)?;
    assert_eq!(extracted_add_agents.transfer_id, transfer_id);
    assert_eq!(extracted_add_agents.agents.len(), 1);
    assert_eq!(extracted_add_agents.agents[0].id, charlie_did);

    Ok(())
}

#[test]
fn test_replace_agent() -> Result<()> {
    let transfer_id = "test-transfer-123";
    let alice_did = "did:example:alice";
    let bob_did = "did:example:bob";
    let charlie_did = "did:example:charlie";

    // Create a ReplaceAgent message to replace Bob with Charlie
    let replace_agent = ReplaceAgent {
        transfer_id: transfer_id.to_string(),
        original: bob_did.to_string(),
        replacement: Participant {
            id: charlie_did.to_string(),
            role: Some("beneficiary".to_string()),
        },
        metadata: HashMap::new(),
    };

    // Validate the message
    replace_agent.validate()?;

    // Create a DIDComm message from the replace_agent
    let message = replace_agent
        .to_didcomm_with_route(Some(alice_did), [bob_did, charlie_did].iter().copied())?;

    // Set the thread ID
    let message_with_thread = Message {
        thid: Some(transfer_id.to_string()),
        ..message
    };

    // Verify the message properties
    assert_eq!(message_with_thread.from, Some(alice_did.to_string()));
    assert!(message_with_thread
        .to
        .as_ref()
        .unwrap()
        .contains(&bob_did.to_string()));
    assert!(message_with_thread
        .to
        .as_ref()
        .unwrap()
        .contains(&charlie_did.to_string()));
    assert_eq!(message_with_thread.thid, Some(transfer_id.to_string()));

    // Extract the body back and verify
    let extracted_replace_agent = ReplaceAgent::from_didcomm(&message_with_thread)?;
    assert_eq!(extracted_replace_agent.transfer_id, transfer_id);
    assert_eq!(extracted_replace_agent.original, bob_did);
    assert_eq!(extracted_replace_agent.replacement.id, charlie_did);

    Ok(())
}

#[test]
fn test_remove_agent() -> Result<()> {
    let transfer_id = "test-transfer-123";
    let alice_did = "did:example:alice";
    let bob_did = "did:example:bob";

    // Create a RemoveAgent message to remove Bob
    let remove_agent = RemoveAgent {
        transfer_id: transfer_id.to_string(),
        agent: bob_did.to_string(),
        metadata: HashMap::new(),
    };

    // Validate the message
    remove_agent.validate()?;

    // Create a DIDComm message from the remove_agent
    let message = remove_agent.to_didcomm_with_route(Some(alice_did), [bob_did].iter().copied())?;

    // Set the thread ID
    let message_with_thread = Message {
        thid: Some(transfer_id.to_string()),
        ..message
    };

    // Verify the message properties
    assert_eq!(message_with_thread.from, Some(alice_did.to_string()));
    assert!(message_with_thread
        .to
        .as_ref()
        .unwrap()
        .contains(&bob_did.to_string()));
    assert_eq!(message_with_thread.thid, Some(transfer_id.to_string()));

    // Extract the body back and verify
    let extracted_remove_agent = RemoveAgent::from_didcomm(&message_with_thread)?;
    assert_eq!(extracted_remove_agent.transfer_id, transfer_id);
    assert_eq!(extracted_remove_agent.agent, bob_did);

    Ok(())
}

#[test]
fn test_get_all_participants() -> Result<()> {
    let alice_did = "did:example:alice";
    let bob_did = "did:example:bob";
    let charlie_did = "did:example:charlie";

    // Create a message with some participants
    let message = Message {
        id: Uuid::new_v4().to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0#transfer".to_string(),
        body: serde_json::json!({}),
        from: Some(alice_did.to_string()),
        to: Some(vec![bob_did.to_string(), charlie_did.to_string()]),
        thid: None,
        pthid: None,
        extra_headers: HashMap::new(),
        created_time: Some(1645556488),
        expires_time: None,
        from_prior: None,
        attachments: None,
    };

    // Get all participants
    let participants = message.get_all_participants();

    // Verify results
    assert_eq!(participants.len(), 3);
    assert!(participants.contains(&alice_did.to_string()));
    assert!(participants.contains(&bob_did.to_string()));
    assert!(participants.contains(&charlie_did.to_string()));

    Ok(())
}
