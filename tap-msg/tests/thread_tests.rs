use didcomm::Message;
use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::error::{Error, Result};
use tap_msg::message::types::{
    AddAgents, Authorizable, Authorize, ConfirmRelationship, Participant, RemoveAgent, ReplaceAgent,
};
use tap_msg::message::{TapMessage, TapMessageBody, Transfer};
use uuid::Uuid;

#[test]
fn test_create_reply() -> Result<()> {
    // Create original message
    let _alice_did = "did:example:alice";
    let _bob_did = "did:example:bob";
    // Using an underscore prefix for unused variable
    let _charlie_did = "did:example:charlie";

    // Create a transfer from Alice to Bob
    let transfer = Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap(),
        originator: Participant {
            id: _alice_did.to_string(),
            role: Some("originator".to_string()),
            policies: None,
            leiCode: None,
        },
        beneficiary: Some(Participant {
            id: _bob_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
        }),
        amount: "10.00".to_string(),
        agents: vec![],
        settlement_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Create the initial transfer message
    let transfer_message =
        transfer.to_didcomm_with_route(Some(_alice_did), [_bob_did].iter().copied())?;

    // Create an authorize response from Bob to Alice
    let authorize = Authorize {
        transfer_id: transfer_message.id.clone(), // Get ID from message
        note: None,
    };

    // TODO: Reimplement reply creation using DIDComm message methods
    // let _reply_message =
    //     authorize.create_reply(&transfer_message, bob_did, &[alice_did, bob_did])?;

    // Test the message extension method
    let reply_via_message = transfer_message.create_reply(&authorize, _bob_did)?;

    // Verify the reply created via the Message trait has the same properties
    assert_eq!(reply_via_message.from, Some(_bob_did.to_string()));
    assert!(reply_via_message
        .to
        .as_ref()
        .unwrap()
        .contains(&_alice_did.to_string()));
    assert!(!reply_via_message
        .to
        .as_ref()
        .unwrap()
        .contains(&_bob_did.to_string()));
    assert_eq!(reply_via_message.thid, Some(transfer_message.id.clone()));

    Ok(())
}

#[test]
fn test_add_agents() -> Result<()> {
    let transfer_id = "test-transfer-123";
    let _alice_did = "did:example:alice";
    let _bob_did = "did:example:bob";
    let charlie_did = "did:example:charlie";

    // Create an AddAgents message to add Charlie
    let add_agents = AddAgents {
        transfer_id: transfer_id.to_string(),
        agents: vec![Participant {
            id: charlie_did.to_string(),
            role: Some("observer".to_string()),
            policies: None,
            leiCode: None,
        }],
    };

    // Validate the message
    add_agents.validate()?;

    // Create a DIDComm message from the add_agents
    let message = add_agents
        .to_didcomm_with_route(Some(_alice_did), [_bob_did, charlie_did].iter().copied())?;

    // Set the thread ID
    let message_with_thread = Message {
        thid: Some(transfer_id.to_string()),
        ..message
    };

    // Verify the message properties
    assert_eq!(message_with_thread.from, Some(_alice_did.to_string()));
    assert!(message_with_thread
        .to
        .as_ref()
        .unwrap()
        .contains(&_bob_did.to_string()));
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
    let _alice_did = "did:example:alice";
    let _bob_did = "did:example:bob";
    let charlie_did = "did:example:charlie";

    // Create a ReplaceAgent message to replace Bob with Charlie
    let replace_agent = ReplaceAgent {
        transfer_id: transfer_id.to_string(),
        original: _bob_did.to_string(),
        replacement: Participant {
            id: charlie_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
        },
    };

    // Validate the message
    replace_agent.validate()?;

    // Create a DIDComm message from the replace_agent
    let message = replace_agent
        .to_didcomm_with_route(Some(_alice_did), [_bob_did, charlie_did].iter().copied())?;

    // Set the thread ID
    let message_with_thread = Message {
        thid: Some(transfer_id.to_string()),
        ..message
    };

    // Verify the message properties
    assert_eq!(message_with_thread.from, Some(_alice_did.to_string()));
    assert!(message_with_thread
        .to
        .as_ref()
        .unwrap()
        .contains(&_bob_did.to_string()));
    assert!(message_with_thread
        .to
        .as_ref()
        .unwrap()
        .contains(&charlie_did.to_string()));
    assert_eq!(message_with_thread.thid, Some(transfer_id.to_string()));

    // Extract the body back and verify
    let extracted_replace_agent = ReplaceAgent::from_didcomm(&message_with_thread)?;
    assert_eq!(extracted_replace_agent.transfer_id, transfer_id);
    assert_eq!(extracted_replace_agent.original, _bob_did);
    assert_eq!(extracted_replace_agent.replacement.id, charlie_did);

    Ok(())
}

#[test]
fn test_remove_agent() -> Result<()> {
    let transfer_id = "test-transfer-123";
    let _alice_did = "did:example:alice";
    let _bob_did = "did:example:bob";

    // Create a RemoveAgent message to remove Bob
    let remove_agent = RemoveAgent {
        transfer_id: transfer_id.to_string(),
        agent: _bob_did.to_string(),
    };

    // Validate the message
    remove_agent.validate()?;

    // Create a DIDComm message from the remove_agent
    let message =
        remove_agent.to_didcomm_with_route(Some(_alice_did), [_bob_did].iter().copied())?;

    // Set the thread ID
    let message_with_thread = Message {
        thid: Some(transfer_id.to_string()),
        ..message
    };

    // Verify the message properties
    assert_eq!(message_with_thread.from, Some(_alice_did.to_string()));
    assert!(message_with_thread
        .to
        .as_ref()
        .unwrap()
        .contains(&_bob_did.to_string()));
    assert_eq!(message_with_thread.thid, Some(transfer_id.to_string()));

    // Extract the body back and verify
    let extracted_remove_agent = RemoveAgent::from_didcomm(&message_with_thread)?;
    assert_eq!(extracted_remove_agent.transfer_id, transfer_id);
    assert_eq!(extracted_remove_agent.agent, _bob_did);

    Ok(())
}

#[test]
fn test_confirm_relationship() -> Result<()> {
    let transfer_id = "test-transfer-123";
    let _alice_did = "did:example:alice";
    let _bob_did = "did:example:bob";
    let org_did = "did:example:organization";

    // Create a ConfirmRelationship message to confirm Bob is acting on behalf of an organization
    let confirm_relationship = ConfirmRelationship {
        transfer_id: transfer_id.to_string(),
        agent_id: _bob_did.to_string(),
        for_id: org_did.to_string(),
        role: Some("custodian".to_string()),
    };

    // Validate the message
    confirm_relationship.validate()?;

    // Create a DIDComm message from the confirm_relationship
    let message =
        confirm_relationship.to_didcomm_with_route(Some(_alice_did), [_bob_did].iter().copied())?;

    // Set the thread ID
    let message_with_thread = Message {
        thid: Some(transfer_id.to_string()),
        ..message
    };

    // Verify the message properties
    assert_eq!(message_with_thread.from, Some(_alice_did.to_string()));
    assert!(message_with_thread
        .to
        .as_ref()
        .unwrap()
        .contains(&_bob_did.to_string()));
    assert_eq!(message_with_thread.thid, Some(transfer_id.to_string()));

    // Extract the body back and verify
    let extracted_confirm = ConfirmRelationship::from_didcomm(&message_with_thread)?;
    assert_eq!(extracted_confirm.transfer_id, transfer_id);
    assert_eq!(extracted_confirm.agent_id, _bob_did);
    assert_eq!(extracted_confirm.for_id, org_did);
    assert_eq!(extracted_confirm.role, Some("custodian".to_string()));

    // Test using the Authorizable trait
    let transfer = Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap(),
        originator: Participant::new(_alice_did),
        beneficiary: Some(Participant::new(_bob_did)),
        amount: "10.00".to_string(),
        agents: vec![],
        settlement_id: None,
        metadata: HashMap::new(),
        memo: Some("Test memo".to_string()),
    };

    // Create a DIDComm message from the transfer
    let transfer_message = transfer.to_didcomm(None)?;
    let mut metadata = HashMap::new();
    metadata.insert(
        "context".to_string(),
        serde_json::Value::String("test".to_string()),
    );

    // Extract the message body from transfer_message
    let transfer_body_json = transfer_message.body;

    if transfer_body_json.is_null() {
        return Err(Error::SerializationError(
            "Missing transfer body".to_string(),
        ));
    }

    let transfer_body: Transfer = serde_json::from_value(transfer_body_json.clone())?;

    // Create a ConfirmRelationship message using the Authorizable trait
    let confirm_body = transfer_body.confirm_relationship(
        transfer_id.to_string(),       // Argument 1: transfer_id
        _bob_did.to_string(),          // Argument 2: agent_id
        org_did.to_string(),           // Argument 3: for_id
        Some("custodian".to_string()), // Argument 4: Role
    );

    // Create a DIDComm message from the confirm_body
    let confirm_message = confirm_body.to_didcomm(Some(_alice_did))?;

    // Verify the created message
    assert_eq!(confirm_message.from, Some(_alice_did.to_string()));
    assert!(confirm_message
        .to
        .as_ref()
        .unwrap()
        .contains(&_bob_did.to_string()));
    // The thid should match the transfer_id provided in the ConfirmRelationship body
    assert_eq!(confirm_message.thid, Some(confirm_body.transfer_id.clone()));

    // Check body content (role)
    assert_eq!(confirm_message.body["role"].as_str().unwrap(), "custodian",);

    Ok(())
}

#[test]
fn test_get_all_participants() -> Result<()> {
    let _alice_did = "did:example:alice";
    let _bob_did = "did:example:bob";
    let charlie_did = "did:example:charlie";

    // Create a message with some participants
    let message = Message {
        id: Uuid::new_v4().to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0#transfer".to_string(),
        body: serde_json::json!({}),
        from: Some(_alice_did.to_string()),
        to: Some(vec![_bob_did.to_string(), charlie_did.to_string()]),
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
    assert!(participants.contains(&_alice_did.to_string()));
    assert!(participants.contains(&_bob_did.to_string()));
    assert!(participants.contains(&charlie_did.to_string()));

    Ok(())
}

#[test]
fn test_add_agents_missing_transfer_id() {
    // Test adding agents to a transfer that doesn't exist
    let _tx_id = "".to_string();
    let agents = vec![Participant {
        id: "did:key:z6Mkk7yqnGF3YwTrLpqrW6PGsKci7dNqh1CjnvMbzrMerSeL".to_string(),
        role: Some("sender_agent".to_string()),
        leiCode: None,
        policies: None,
    }];

    // Create an AddAgents message to add the agents
    let add_agents = AddAgents {
        transfer_id: _tx_id,
        agents,
    };

    // Validate the message
    assert!(add_agents.validate().is_err());
}

#[test]
fn test_add_agents_empty() {
    let transfer_id = "test-transfer-123";
    let _alice_did = "did:example:alice";
    let _bob_did = "did:example:bob";

    // Create an AddAgents message to add no agents
    let add_agents = AddAgents {
        transfer_id: transfer_id.to_string(),
        agents: vec![],
    };

    // Validate the message
    let err = add_agents.validate().unwrap_err();
    match err {
        Error::Validation(s) => assert!(s.contains("At least one agent")),
        _ => panic!("Expected Validation error"),
    }
}
