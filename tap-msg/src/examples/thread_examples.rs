//! Examples for thread management and messaging workflows.

use crate::error::Result;
use crate::message::{
    types::Authorizable, AddAgents, Authorize, Participant, Policy, RemoveAgent, ReplaceAgent,
    RequireProofOfControl, Settle, TapMessage, TapMessageBody, Transfer,
};
use crate::utils::get_current_time;
use didcomm::Message;
use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;

/// This example demonstrates how to create a reply to a Transfer message
pub fn create_reply_to_transfer_example() -> Result<Message> {
    let alice_did = "did:example:alice";
    let bob_did = "did:example:bob";

    // Create a Transfer message
    let transfer = Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap(),
        originator: Participant {
            id: alice_did.to_string(),
            role: Some("originator".to_string()),
            policies: None,
            leiCode: None,
        },
        beneficiary: Some(Participant {
            id: bob_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
        }),
        amount: "100.00".to_string(),
        agents: vec![],
        settlement_id: None,
        memo: Some("Test transfer".to_string()),
        metadata: HashMap::new(),
    };

    // Create the initial transfer message
    let transfer_message =
        transfer.to_didcomm_with_route(Some(alice_did), [bob_did].iter().copied())?;

    // Create an Authorize message
    let authorize = Authorize {
        transfer_id: transfer_message.id.clone(),
        note: Some("I authorize this transfer".to_string()),
        timestamp: get_current_time()?.to_string(),
        settlement_address: None,
        metadata: HashMap::new(),
    };

    // Create a reply from Alice to Bob
    let response_message =
        authorize.create_reply(&transfer_message, alice_did, &[alice_did, bob_did])?;

    Ok(response_message)
}

/// This example demonstrates how to create a reply using the Message trait extension
pub fn create_reply_using_message_trait_example(
    original_message: &Message,
    creator_did: &str,
) -> Result<Message> {
    // Create an Authorize response
    let authorize = Authorize {
        transfer_id: original_message.id.clone(),
        note: Some("Transfer authorized".to_string()),
        timestamp: get_current_time()?.to_string(),
        settlement_address: None,
        metadata: HashMap::new(),
    };

    // Create a reply using the Message trait extension
    // This will automatically gather all participants from the original message
    let response_message = original_message.create_reply(&authorize, creator_did)?;

    Ok(response_message)
}

/// This example demonstrates adding agents using the Authorizable trait
pub fn create_add_agents_example() -> Result<Message> {
    let originator_did = "did:example:originator";
    let beneficiary_did = "did:example:beneficiary";
    let sender_vasp_did = "did:example:sender_vasp";
    let receiver_vasp_did = "did:example:receiver_vasp";
    let new_agent_did = "did:example:new_agent";

    // Create a Transfer
    let transfer = Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap(),
        originator: Participant {
            id: originator_did.to_string(),
            role: Some("originator".to_string()),
            policies: None,
            leiCode: None,
        },
        beneficiary: Some(Participant {
            id: beneficiary_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
        }),
        amount: "100.00".to_string(),
        agents: vec![
            Participant {
                id: sender_vasp_did.to_string(),
                role: Some("sender_vasp".to_string()),
                policies: None,
                leiCode: None,
            },
            Participant {
                id: receiver_vasp_did.to_string(),
                role: Some("receiver_vasp".to_string()),
                policies: None,
                leiCode: None,
            },
        ],
        settlement_id: None,
        memo: Some("Test transfer".to_string()),
        metadata: HashMap::new(),
    };

    // Create the initial transfer message
    let transfer_message =
        transfer.to_didcomm_with_route(Some(originator_did), [beneficiary_did].iter().copied())?;

    // Create an Authorize message first
    let authorize = Authorize {
        transfer_id: transfer_message.id.clone(),
        note: Some("I authorize this transfer".to_string()),
        timestamp: get_current_time()?.to_string(),
        settlement_address: None,
        metadata: HashMap::new(),
    };

    // Create a reply from the originator to the beneficiary
    let authorize_message = authorize.create_reply(
        &transfer_message,
        originator_did,
        &[originator_did, beneficiary_did],
    )?;

    // Create an AddAgents message
    let add_agents = authorize_message.add_agents(
        vec![Participant {
            id: new_agent_did.to_string(),
            role: Some("compliance".to_string()),
            policies: None,
            leiCode: None,
        }],
        HashMap::new(),
    );

    // Create a reply using the TapMessage trait
    let response = transfer_message.create_reply(&add_agents, originator_did)?;

    Ok(response)
}

/// This example demonstrates replacing an agent using the Authorizable trait
pub fn create_replace_agent_example(
    original_message: &Message,
    creator_did: &str,
    original_agent_id: &str,
    replacement_agent_id: &str,
    replacement_agent_role: Option<&str>,
) -> Result<Message> {
    // Create a replacement participant
    let replacement = Participant {
        id: replacement_agent_id.to_string(),
        role: replacement_agent_role.map(ToString::to_string),
        policies: None, // No policies for this participant
        leiCode: None,
    };

    // Create a ReplaceAgent message using the Authorizable trait
    let replace_agent =
        original_message.replace_agent(original_agent_id.to_string(), replacement, HashMap::new());

    // Create a reply using the TapMessage trait
    let response = original_message.create_reply(&replace_agent, creator_did)?;

    Ok(response)
}

/// This example demonstrates removing an agent using the Authorizable trait
pub fn create_remove_agent_example(
    original_message: &Message,
    creator_did: &str,
    agent_to_remove: &str,
) -> Result<Message> {
    // Create a RemoveAgent message using the Authorizable trait
    let remove_agent = original_message.remove_agent(agent_to_remove.to_string(), HashMap::new());

    // Create a reply using the TapMessage trait
    let response = original_message.create_reply(&remove_agent, creator_did)?;

    Ok(response)
}

/// This example demonstrates creating an UpdatePolicies message using the Authorizable trait
pub fn create_update_policies_example(
    original_message: &Message,
    creator_did: &str,
    _recipients: &[&str],
) -> Result<Message> {
    // Create a proof of control policy
    let proof_policy = RequireProofOfControl {
        from: Some(vec!["did:example:dave".to_string()]),
        from_role: None,
        from_agent: None,
        nonce: 12345678,
        purpose: Some("Please prove control of your account".to_string()),
    };

    // Create an UpdatePolicies message using the Authorizable trait
    let update_policies = original_message.update_policies(
        vec![Policy::RequireProofOfControl(proof_policy)],
        HashMap::new(),
    );

    // Create a reply using the TapMessage trait, which maintains thread correlation
    let response = original_message.create_reply(&update_policies, creator_did)?;

    Ok(response)
}

/// This example demonstrates a complete workflow for managing thread participants
pub fn thread_participant_workflow_example() -> Result<()> {
    // 1. Let's start with a transfer message
    let alice_did = "did:example:alice";
    let bob_did = "did:example:bob";
    let charlie_did = "did:example:charlie";
    let dave_did = "did:example:dave";

    // Create a transfer from Alice to Bob
    let transfer = Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap(),
        originator: Participant {
            id: alice_did.to_string(),
            role: Some("originator".to_string()),
            policies: None,
            leiCode: None,
        },
        beneficiary: Some(Participant {
            id: bob_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
        }),
        amount: "10.00".to_string(),
        agents: vec![],
        settlement_id: None,
        memo: Some("Initial transfer".to_string()),
        metadata: HashMap::new(),
    };

    // Create the initial transfer message
    let transfer_message =
        transfer.to_didcomm_with_route(Some(alice_did), [bob_did].iter().copied())?;

    println!("Created initial transfer message: {:?}", transfer_message);

    // Let's assume we have a unique thread ID
    let transfer_id = transfer_message.id.clone();

    // 2. Now Bob wants to reply to the transfer
    let authorize = Authorize {
        transfer_id: transfer_message.id.clone(),
        note: Some("Transfer approved".to_string()),
        timestamp: get_current_time()?.to_string(),
        settlement_address: None,
        metadata: HashMap::new(),
    };

    // Create a reply from Bob to Alice
    let authorize_message =
        authorize.create_reply(&transfer_message, bob_did, &[alice_did, bob_did])?;

    println!("Created authorize message: {:?}", authorize_message);

    // 3. Now Alice wants to add Charlie to the thread
    // Create an AddAgents message
    let add_agents = AddAgents {
        transfer_id: transfer_id.clone(),
        agents: vec![Participant {
            id: charlie_did.to_string(),
            role: Some("observer".to_string()),
            policies: None,
            leiCode: None,
        }],
        metadata: HashMap::new(),
    };

    // Create the add agents message
    let add_agents_message = add_agents
        .to_didcomm_with_route(Some(alice_did), [bob_did, charlie_did].iter().copied())?;

    // Set the thread ID
    let add_agents_message_with_thread = Message {
        thid: Some(transfer_id.clone()),
        ..add_agents_message
    };

    println!(
        "Created add agents message: {:?}",
        add_agents_message_with_thread
    );

    // 4. Now Bob wants to replace himself with Dave
    let replace_agent = ReplaceAgent {
        transfer_id: transfer_id.clone(),
        original: bob_did.to_string(),
        replacement: Participant {
            id: dave_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
        },
        metadata: HashMap::new(),
    };

    // Create the replace agent message
    let replace_agent_message = replace_agent.to_didcomm_with_route(
        Some(bob_did),
        [alice_did, dave_did, charlie_did].iter().copied(),
    )?;

    // Set the thread ID
    let replace_agent_message_with_thread = Message {
        thid: Some(transfer_id.clone()),
        ..replace_agent_message
    };

    println!(
        "Created replace agent message: {:?}",
        replace_agent_message_with_thread
    );

    // 5. Alice wants to remove Charlie from the thread
    let remove_agent = RemoveAgent {
        transfer_id: transfer_id.clone(),
        agent: charlie_did.to_string(),
        metadata: HashMap::new(),
    };

    // Create the remove agent message
    let remove_agent_message = remove_agent
        .to_didcomm_with_route(Some(alice_did), [dave_did, charlie_did].iter().copied())?;

    // Set the thread ID
    let remove_agent_message_with_thread = Message {
        thid: Some(transfer_id.clone()),
        ..remove_agent_message
    };

    println!(
        "Created remove agent message: {:?}",
        remove_agent_message_with_thread
    );

    // 6. Now Dave can settle the transfer
    let settle = Settle {
        transfer_id: transfer_message.id.clone(),
        transaction_id: "tx123456".to_string(),
        transaction_hash: Some("0xabcdef1234567890".to_string()),
        block_height: Some(12345),
        note: Some("Transfer settled".to_string()),
        timestamp: get_current_time()?.to_string(),
        metadata: HashMap::new(),
    };

    // Dave's reply should go to Alice (originator)
    let settle_message = settle.create_reply(
        &remove_agent_message_with_thread,
        dave_did,
        &[alice_did, dave_did],
    )?;

    println!("Created settle message from Dave: {:?}", settle_message);

    // Verify that the 'to' field in the settle message includes Alice
    if let Some(to) = &settle_message.to {
        assert!(to.contains(&alice_did.to_string()));
        assert!(!to.contains(&dave_did.to_string())); // Dave is the sender
        println!("Verified that the settle message is addressed correctly");
    }

    Ok(())
}
