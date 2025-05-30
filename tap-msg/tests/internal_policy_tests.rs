//! Integration tests for internal policy functionality

use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::didcomm::PlainMessage;
use tap_msg::error::Result;
use tap_msg::message::tap_message_trait::{TapMessage, TapMessageBody, Transaction};
use tap_msg::message::{
    AddAgents, Authorize, Participant, Policy as TapPolicy, RequireAuthorization,
    RequireProofOfControl, Transfer, UpdatePolicies,
};

#[allow(dead_code)]
const POLICY_ENGINE_DID: &str = "did:policy:engine";

// Helper function to create a test transfer message
fn create_test_transfer() -> Result<PlainMessage> {
    let originator_did = "did:example:originator";
    let beneficiary_did = "did:example:beneficiary";
    let _sender_vasp_did = "did:example:sender_vasp";
    let receiver_vasp_did = "did:example:receiver_vasp";

    let transfer = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap(),
        originator: Participant {
            id: originator_did.to_string(),
            role: Some("originator".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        },
        beneficiary: Some(Participant {
            id: beneficiary_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        }),
        amount: "100.00".to_string(),
        agents: vec![
            Participant {
                id: "did:example:sender_vasp".to_string(),
                role: Some("sender_vasp".to_string()),
                policies: None,
                leiCode: None,
                name: None,
            },
            Participant {
                id: receiver_vasp_did.to_string(),
                role: Some("receiver_vasp".to_string()),
                policies: None,
                leiCode: None,
                name: None,
            },
        ],
        settlement_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    transfer.to_didcomm_with_route(
        originator_did,
        [
            beneficiary_did,
            "did:example:sender_vasp",
            receiver_vasp_did,
        ]
        .iter()
        .copied(),
    )
}

#[test]
fn test_update_policies() -> Result<()> {
    // Create a test message
    let transfer_message = create_test_transfer()?;
    let _creator_did = "did:example:sender_vasp";

    // Create policies
    let auth_policy = RequireAuthorization {
        from: Some(vec!["did:example:originator".to_string()]),
        from_role: None,
        from_agent: None,
        purpose: Some("Authorization required".to_string()),
    };

    let proof_policy = RequireProofOfControl {
        from: Some(vec!["did:example:beneficiary".to_string()]),
        from_role: None,
        from_agent: None,
        address_id: "eip155:1:0x1234567890123456789012345678901234567890".to_string(),
        purpose: Some("Please prove control of your account".to_string()),
    };

    // Use the Authorizable trait to create an UpdatePolicies message
    let update_policies = UpdatePolicies {
        transaction_id: transfer_message.id.clone(),
        policies: vec![
            TapPolicy::RequireAuthorization(auth_policy),
            TapPolicy::RequireProofOfControl(proof_policy),
        ],
    };

    // Validate the created message
    assert_eq!(update_policies.policies.len(), 2);

    // Convert to DIDComm message
    let didcomm_message = update_policies.to_didcomm("did:example:sender_vasp")?;

    // DEBUG: Print the JSON structure of the message body
    println!(
        "DEBUG: Message body JSON: {}",
        serde_json::to_string_pretty(&didcomm_message.body).unwrap()
    );

    assert_eq!(
        didcomm_message.body_as::<UpdatePolicies>()?.policies.len(),
        2
    );

    Ok(())
}

#[test]
fn test_add_agents() -> Result<()> {
    // Create a test message
    let transfer_message = create_test_transfer()?;
    let _creator_did = "did:example:sender_vasp";
    let new_agent_did = "did:example:new_agent";

    // Create a new participant
    let new_agent = Participant {
        id: new_agent_did.to_string(),
        role: Some("observer".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    // Use the Authorizable trait to create an AddAgents message
    let add_agents = AddAgents {
        transaction_id: transfer_message.id.clone(),
        agents: vec![new_agent.clone()],
    };

    // Validate the created message
    assert_eq!(add_agents.agents.len(), 1);
    assert_eq!(add_agents.agents[0].id, new_agent_did);
    assert_eq!(add_agents.agents[0].role, Some("observer".to_string()));

    // Convert to DIDComm message and check that it can be properly deserialized
    let didcomm_message = add_agents.to_didcomm("did:example:sender_vasp")?;
    let parsed_body = didcomm_message.body_as::<AddAgents>()?;
    assert_eq!(parsed_body.agents.len(), 1);
    assert_eq!(parsed_body.agents[0].id, new_agent_did);

    Ok(())
}

#[test]
fn test_authorizable_trait_methods() -> Result<()> {
    // Create a base transfer message
    let transfer = create_test_transfer_struct()?;
    let original_agent_did = "did:example:original_agent";
    let replacement_agent_did = "did:example:replacement_agent";
    let agent_to_remove = "did:example:agent_to_remove";

    // Create a replacement participant
    let replacement = Participant {
        id: replacement_agent_did.to_string(),
        role: Some("replacement_agent".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    // Use the Transaction trait methods on the Transfer struct
    // replace_agent expects creator_did, original DID, and replacement Participant
    let replace_agent_message = Transaction::replace_agent(
        &transfer,
        "test-transfer-123", // creator_did
        original_agent_did,
        replacement.clone(),
    );

    // The replace_agent_message is already a PlainMessage<ReplaceAgent>, so we can access the body directly
    assert_eq!(
        replace_agent_message.body.transaction_id,
        transfer.transaction_id
    ); // Use transfer's transaction_id
    assert_eq!(replace_agent_message.body.original, original_agent_did);
    assert_eq!(
        replace_agent_message.body.replacement.id,
        replacement_agent_did
    );
    assert_eq!(
        replace_agent_message.body.replacement.role,
        Some("replacement_agent".to_string())
    );

    // Test RemoveAgent
    // Pass creator_did and agent DID
    let remove_agent_message = Transaction::remove_agent(
        &transfer,
        "test-transfer-123", // creator_did
        agent_to_remove,
    );

    // The remove_agent_message is already a PlainMessage<RemoveAgent>, so we can access the body directly
    assert_eq!(
        remove_agent_message.body.transaction_id,
        transfer.transaction_id
    ); // Use transfer's transaction_id
    assert_eq!(remove_agent_message.body.agent, agent_to_remove);

    // It seems we don't need to convert these specific bodies to full messages for this test
    // We are just testing the body creation logic here.
    Ok(())
}

#[test]
fn test_reply_creation_maintains_thread() -> Result<()> {
    // Create a base transfer message
    let transfer_message = create_test_transfer()?;
    let creator_did = "did:example:sender_vasp";
    let _participants = &[
        "did:example:originator",
        "did:example:beneficiary",
        "did:example:sender_vasp",
        "did:example:receiver_vasp",
    ];

    // Create an UpdatePolicies message using the Authorizable trait
    let proof_policy = RequireProofOfControl {
        from: Some(vec!["did:example:beneficiary".to_string()]),
        from_role: None,
        from_agent: None,
        address_id: "eip155:1:0x1234567890123456789012345678901234567890".to_string(),
        purpose: Some("Please prove control of your account".to_string()),
    };

    let update_policies = UpdatePolicies {
        transaction_id: transfer_message.id.clone(),
        policies: vec![TapPolicy::RequireProofOfControl(proof_policy)],
    };

    // Create a reply using the TapMessageBody trait
    let mut reply = update_policies.to_didcomm("did:example:sender_vasp")?;

    // Manually set the thread ID for the reply
    reply.thid = Some(transfer_message.id.clone());

    // Verify that thread correlation is maintained
    assert_eq!(reply.thid, Some(transfer_message.id.clone()));
    assert_eq!(reply.from, creator_did.to_string());

    // Verify that the message body contains the right content
    let body = reply.body_as::<UpdatePolicies>()?;
    assert_eq!(body.transaction_id, transfer_message.id);
    assert_eq!(body.policies.len(), 1);

    Ok(())
}

#[test]
fn test_reply_chain() -> Result<()> {
    // Create a test message chain to simulate a conversation
    let transfer_message = create_test_transfer()?;
    let originator_did = "did:example:originator";
    let beneficiary_did = "did:example:beneficiary";
    let sender_vasp_did = "did:example:sender_vasp";
    let _participants = &[originator_did, beneficiary_did, sender_vasp_did];

    // Step 1: VASP updates policies
    let update_policies = UpdatePolicies {
        transaction_id: transfer_message.id.clone(),
        policies: vec![TapPolicy::RequireAuthorization(RequireAuthorization {
            from: Some(vec![beneficiary_did.to_string()]),
            from_role: None,
            from_agent: None,
            purpose: Some("Please authorize".to_string()),
        })],
    };

    let mut policies_message = update_policies.to_didcomm("did:example:sender_vasp")?;
    policies_message.thid = Some(transfer_message.id.clone());

    // Step 2: Beneficiary authorizes in response
    let authorize = Authorize {
        transaction_id: transfer_message.id.clone(),
        settlement_address: None,
        expiry: None,
        note: Some("I authorize this transfer".to_string()),
    };

    // Pass sender DID as required by compiler here
    let mut authorize_message = authorize.to_didcomm("did:example:beneficiary")?;
    authorize_message.thid = Some(transfer_message.id.clone());

    // Step 3: VASP adds an agent
    let add_agents = AddAgents {
        transaction_id: transfer_message.id.clone(),
        agents: vec![Participant {
            id: "did:example:compliance".to_string(),
            role: Some("compliance".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        }],
    };

    // Pass sender DID as required by compiler here
    let mut add_agents_message = add_agents.to_didcomm("did:example:sender_vasp")?;
    add_agents_message.thid = Some(transfer_message.id.clone());

    // Verify thread correlation is maintained throughout the chain
    assert_eq!(policies_message.thid, Some(transfer_message.id.clone()));
    assert_eq!(authorize_message.thid, Some(transfer_message.id.clone()));
    assert_eq!(add_agents_message.thid, Some(transfer_message.id.clone()));

    // Verify the message sequence
    let body1 = policies_message.body_as::<UpdatePolicies>()?;
    assert_eq!(body1.transaction_id, transfer_message.id);

    let body2 = authorize_message.body_as::<Authorize>()?;
    assert_eq!(body2.transaction_id, transfer_message.id);
    assert_eq!(body2.note, Some("I authorize this transfer".to_string()));

    let body3 = add_agents_message.body_as::<AddAgents>()?;
    assert_eq!(body3.transaction_id, transfer_message.id);
    assert_eq!(body3.agents[0].id, "did:example:compliance");

    Ok(())
}

fn create_test_transfer_struct() -> Result<Transfer> {
    let originator = Participant {
        id: "did:example:originator".to_string(),
        role: Some("originator".to_string()),
        leiCode: None,
        name: None,
        policies: None,
    };
    let beneficiary = Participant {
        id: "did:example:beneficiary".to_string(),
        role: Some("beneficiary".to_string()),
        leiCode: None,
        name: None,
        policies: None,
    };
    let transfer = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap(),
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.00".to_string(),
        agents: vec![
            Participant {
                id: "did:example:sender_vasp".to_string(),
                role: Some("sender_vasp".to_string()),
                leiCode: None,
                name: None,
                policies: None,
            },
            Participant {
                id: "did:example:receiver_vasp".to_string(),
                role: Some("receiver_vasp".to_string()),
                leiCode: None,
                name: None,
                policies: None,
            },
        ],
        settlement_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    Ok(transfer)
}
