//! Examples for thread management and messaging workflows.

use crate::error::Result;
use crate::message::{
    AddAgents, Authorize, Participant, Reject, ReplaceAgent, RemoveAgent, Settle, TapMessageBody, Transfer, 
    TapMessage,
};
use didcomm::Message;
use std::collections::HashMap;
use tap_caip::AssetId;
use std::str::FromStr;

/// This example demonstrates how to create a reply to a Transfer message
pub fn create_reply_to_transfer_example(
    original_transfer_message: &Message,
    creator_did: &str,
) -> Result<Message> {
    // Extract the Transfer body from the original message
    let transfer = Transfer::from_didcomm(original_transfer_message)?;
    
    // Create an Authorize response
    let authorize = Authorize {
        transfer_id: original_transfer_message.id.clone(),
        note: Some("Transfer authorized".to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
        metadata: HashMap::new(),
    };
    
    // Create a reply using the new create_reply method
    // This will automatically:
    // 1. Set the thread ID (thid) to link to the original message
    // 2. Set the 'from' field to the creator_did
    // 3. Set the 'to' field to include all participants except the creator
    let response_message = authorize.create_reply(
        original_transfer_message,
        creator_did,
        &[
            "did:example:alice",
            "did:example:bob",
            creator_did,
            "did:example:charlie",
        ],
    )?;
    
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
        timestamp: chrono::Utc::now().to_rfc3339(),
        metadata: HashMap::new(),
    };
    
    // Create a reply using the Message trait extension
    // This will automatically gather all participants from the original message
    let response_message = original_message.create_reply(&authorize, creator_did)?;
    
    Ok(response_message)
}

/// This example demonstrates how to add a new participant to an existing thread
pub fn add_participant_to_thread_example(
    transfer_id: &str,
    creator_did: &str,
    existing_participants: &[&str],
    new_participant: &str,
) -> Result<Message> {
    // Create a new participant with an optional role
    let new_participant = Participant {
        id: new_participant.to_string(),
        role: Some("observer".to_string()),
    };
    
    // Create an AddAgents message according to TAIP-5
    let update = AddAgents {
        transfer_id: transfer_id.to_string(),
        agents: vec![new_participant.clone()],
        metadata: HashMap::new(),
    };
    
    // Create the message with proper routing information
    let mut all_participants = existing_participants.to_vec();
    all_participants.push(new_participant.id.as_str());
    
    // Convert the update to a DIDComm message
    let message = update.to_didcomm_with_route(
        Some(creator_did),
        all_participants.iter().filter(|&&did| did != creator_did).map(|&did| did),
    )?;
    
    // Set the thread ID to link this message to the existing thread
    let message_with_thread = Message {
        thid: Some(transfer_id.to_string()),
        ..message
    };
    
    Ok(message_with_thread)
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
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap(),
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
        memo: Some("Initial transfer".to_string()),
        metadata: HashMap::new(),
    };
    
    // Create the initial transfer message
    let transfer_message = transfer.to_didcomm_with_route(
        Some(alice_did),
        [bob_did].iter().copied(),
    )?;
    
    println!("Created initial transfer message: {:?}", transfer_message);
    
    // Let's assume we have a unique thread ID
    let transfer_id = transfer_message.id.clone();
    
    // 2. Now Bob wants to reply to the transfer
    let authorize = Authorize {
        transfer_id: transfer_message.id.clone(),
        note: Some("Transfer approved".to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
        metadata: HashMap::new(),
    };
    
    // Create a reply from Bob to Alice
    let authorize_message = authorize.create_reply(
        &transfer_message,
        bob_did,
        &[alice_did, bob_did],
    )?;
    
    println!("Created authorize message: {:?}", authorize_message);
    
    // 3. Now Alice wants to add Charlie to the thread
    // Create an AddAgents message
    let add_agents = AddAgents {
        transfer_id: transfer_id.clone(),
        agents: vec![Participant {
            id: charlie_did.to_string(),
            role: Some("observer".to_string()),
        }],
        metadata: HashMap::new(),
    };
    
    // Create the add agents message
    let add_agents_message = add_agents.to_didcomm_with_route(
        Some(alice_did),
        [bob_did, charlie_did].iter().copied(),
    )?;
    
    // Set the thread ID
    let add_agents_message_with_thread = Message {
        thid: Some(transfer_id.clone()),
        ..add_agents_message
    };
    
    println!("Created add agents message: {:?}", add_agents_message_with_thread);
    
    // 4. Now Bob wants to replace himself with Dave
    let replace_agent = ReplaceAgent {
        transfer_id: transfer_id.clone(),
        original: bob_did.to_string(),
        replacement: Participant {
            id: dave_did.to_string(),
            role: Some("beneficiary".to_string()),
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
    
    println!("Created replace agent message: {:?}", replace_agent_message_with_thread);
    
    // 5. Alice wants to remove Charlie from the thread
    let remove_agent = RemoveAgent {
        transfer_id: transfer_id.clone(),
        agent: charlie_did.to_string(),
        metadata: HashMap::new(),
    };
    
    // Create the remove agent message
    let remove_agent_message = remove_agent.to_didcomm_with_route(
        Some(alice_did),
        [dave_did, charlie_did].iter().copied(),
    )?;
    
    // Set the thread ID
    let remove_agent_message_with_thread = Message {
        thid: Some(transfer_id.clone()),
        ..remove_agent_message
    };
    
    println!("Created remove agent message: {:?}", remove_agent_message_with_thread);
    
    // 6. Now Dave can settle the transfer
    let settle = Settle {
        transfer_id: transfer_message.id.clone(),
        transaction_id: "tx123456".to_string(),
        transaction_hash: Some("0xabcdef1234567890".to_string()),
        block_height: Some(12345),
        note: Some("Transfer settled".to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
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
