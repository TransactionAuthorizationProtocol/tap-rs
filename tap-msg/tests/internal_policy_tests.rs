//! Integration tests for internal policy functionality

use didcomm::Message;
use serde_json;
use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::error::Result;
use tap_msg::message::{
    types::Authorizable, AddAgents, Authorize, Participant, Policy, RemoveAgent, ReplaceAgent,
    RequireAuthorization, RequireProofOfControl, TapMessage, TapMessageBody, Transfer,
    UpdatePolicies,
};
use tap_msg::utils::get_current_time;

// Helper function to create a test transfer message
fn create_test_transfer() -> Result<Message> {
    let originator_did = "did:example:originator";
    let beneficiary_did = "did:example:beneficiary";
    let _sender_vasp_did = "did:example:sender_vasp";
    let receiver_vasp_did = "did:example:receiver_vasp";

    let transfer = Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap(),
        originator: Participant {
            id: originator_did.to_string(),
            role: Some("originator".to_string()),
            policies: None,
        },
        beneficiary: Some(Participant {
            id: beneficiary_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
        }),
        amount: "100.00".to_string(),
        agents: vec![
            Participant {
                id: "did:example:sender_vasp".to_string(),
                role: Some("sender_vasp".to_string()),
                policies: None,
            },
            Participant {
                id: receiver_vasp_did.to_string(),
                role: Some("receiver_vasp".to_string()),
                policies: None,
            },
        ],
        settlement_id: None,
        memo: Some("Test transfer".to_string()),
        metadata: HashMap::new(),
    };

    transfer.to_didcomm_with_route(
        Some(originator_did),
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
        nonce: 12345678,
        purpose: Some("Please prove control of your account".to_string()),
    };

    // Use the Authorizable trait to create an UpdatePolicies message
    let update_policies = transfer_message.update_policies(
        vec![
            Policy::RequireAuthorization(auth_policy),
            Policy::RequireProofOfControl(proof_policy),
        ],
        HashMap::new(),
    );

    // Validate the created message
    assert_eq!(update_policies.transfer_id, transfer_message.id);
    assert_eq!(update_policies.policies.len(), 2);

    // Convert to DIDComm message
    let didcomm_message = update_policies.to_didcomm()?;

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
    };

    // Use the Authorizable trait to create an AddAgents message
    let add_agents = transfer_message.add_agents(vec![new_agent.clone()], HashMap::new());

    // Validate the created message
    assert_eq!(add_agents.transfer_id, transfer_message.id);
    assert_eq!(add_agents.agents.len(), 1);
    assert_eq!(add_agents.agents[0].id, new_agent_did);
    assert_eq!(add_agents.agents[0].role, Some("observer".to_string()));

    // Convert to DIDComm message and check that it can be properly deserialized
    let didcomm_message = add_agents.to_didcomm()?;
    let parsed_body = didcomm_message.body_as::<AddAgents>()?;
    assert_eq!(parsed_body.agents.len(), 1);
    assert_eq!(parsed_body.agents[0].id, new_agent_did);

    Ok(())
}

#[test]
fn test_replace_agent() -> Result<()> {
    // Create a test message
    let transfer_message = create_test_transfer()?;
    let original_agent_did = "did:example:beneficiary";
    let replacement_agent_did = "did:example:new_beneficiary";

    // Create a replacement participant
    let replacement = Participant {
        id: replacement_agent_did.to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
    };

    // Use the Authorizable trait to create a ReplaceAgent message
    let replace_agent = transfer_message.replace_agent(
        original_agent_did.to_string(),
        replacement.clone(),
        HashMap::new(),
    );

    // Validate the created message
    assert_eq!(replace_agent.transfer_id, transfer_message.id);
    assert_eq!(replace_agent.original, original_agent_did);
    assert_eq!(replace_agent.replacement.id, replacement_agent_did);
    assert_eq!(
        replace_agent.replacement.role,
        Some("beneficiary".to_string())
    );

    // Convert to DIDComm message and check that it can be properly deserialized
    let didcomm_message = replace_agent.to_didcomm()?;
    let parsed_body = didcomm_message.body_as::<ReplaceAgent>()?;
    assert_eq!(parsed_body.original, original_agent_did);
    assert_eq!(parsed_body.replacement.id, replacement_agent_did);

    Ok(())
}

#[test]
fn test_remove_agent() -> Result<()> {
    // Create a test message
    let transfer_message = create_test_transfer()?;
    let agent_to_remove = "did:example:receiver_vasp";

    // Use the Authorizable trait to create a RemoveAgent message
    let remove_agent = transfer_message.remove_agent(agent_to_remove.to_string(), HashMap::new());

    // Validate the created message
    assert_eq!(remove_agent.transfer_id, transfer_message.id);
    assert_eq!(remove_agent.agent, agent_to_remove);

    // Convert to DIDComm message and check that it can be properly deserialized
    let didcomm_message = remove_agent.to_didcomm()?;
    let parsed_body = didcomm_message.body_as::<RemoveAgent>()?;
    assert_eq!(parsed_body.agent, agent_to_remove);

    Ok(())
}

#[test]
fn test_create_reply_maintains_thread_correlation() -> Result<()> {
    // Create a test message
    let transfer_message = create_test_transfer()?;
    let creator_did = "did:example:sender_vasp";
    let participants = &[
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
        nonce: 12345678,
        purpose: Some("Please prove control of your account".to_string()),
    };

    let update_policies = UpdatePolicies {
        transfer_id: transfer_message.id.clone(),
        policies: vec![Policy::RequireProofOfControl(proof_policy)],
        metadata: HashMap::new(),
    };

    // Create a reply using the TapMessageBody trait
    let reply = update_policies.create_reply(&transfer_message, creator_did, participants)?;

    // Verify that thread correlation is maintained
    assert_eq!(reply.thid, Some(transfer_message.id.clone()));
    assert_eq!(reply.from, Some(creator_did.to_string()));

    // Verify that the message body contains the right content
    let body = reply.body_as::<UpdatePolicies>()?;
    assert_eq!(body.transfer_id, transfer_message.id);
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
    let participants = &[originator_did, beneficiary_did, sender_vasp_did];

    // Step 1: VASP updates policies
    let update_policies = UpdatePolicies {
        transfer_id: transfer_message.id.clone(),
        policies: vec![Policy::RequireAuthorization(RequireAuthorization {
            from: Some(vec![beneficiary_did.to_string()]),
            from_role: None,
            from_agent: None,
            purpose: Some("Please authorize".to_string()),
        })],
        metadata: HashMap::new(),
    };

    let policies_message =
        update_policies.create_reply(&transfer_message, sender_vasp_did, participants)?;

    // Step 2: Beneficiary authorizes in response
    let authorize = Authorize {
        transfer_id: transfer_message.id.clone(),
        note: Some("I authorize this transfer".to_string()),
        timestamp: get_current_time()?.to_string(),
        metadata: HashMap::new(),
    };

    let authorize_message =
        authorize.create_reply(&policies_message, beneficiary_did, participants)?;

    // Step 3: VASP adds an agent
    let add_agents = AddAgents {
        transfer_id: transfer_message.id.clone(),
        agents: vec![Participant {
            id: "did:example:compliance".to_string(),
            role: Some("compliance".to_string()),
            policies: None,
        }],
        metadata: HashMap::new(),
    };

    let add_agents_message = add_agents.create_reply(
        &authorize_message,
        sender_vasp_did,
        &[
            originator_did,
            beneficiary_did,
            sender_vasp_did,
            "did:example:compliance",
        ],
    )?;

    // Verify thread correlation is maintained throughout the chain
    assert_eq!(policies_message.thid, Some(transfer_message.id.clone()));
    assert_eq!(authorize_message.thid, Some(transfer_message.id.clone()));
    assert_eq!(add_agents_message.thid, Some(transfer_message.id.clone()));

    // Verify the message sequence
    let body1 = policies_message.body_as::<UpdatePolicies>()?;
    assert_eq!(body1.transfer_id, transfer_message.id);

    let body2 = authorize_message.body_as::<Authorize>()?;
    assert_eq!(body2.transfer_id, transfer_message.id);
    assert_eq!(body2.note, Some("I authorize this transfer".to_string()));

    let body3 = add_agents_message.body_as::<AddAgents>()?;
    assert_eq!(body3.transfer_id, transfer_message.id);
    assert_eq!(body3.agents[0].id, "did:example:compliance");

    Ok(())
}
