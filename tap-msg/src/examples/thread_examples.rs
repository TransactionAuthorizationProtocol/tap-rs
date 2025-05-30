//! Examples for thread management and messaging workflows.

use crate::error::{Error, Result};
use crate::message::policy::{Policy, RequireProofOfControl};
use crate::message::tap_message_trait::{TapMessageBody, Transaction};
use crate::message::{
    AddAgents, Authorize, Participant, RemoveAgent, ReplaceAgent, Settle, Transfer,
};

use crate::didcomm::PlainMessage;
use serde_json;
use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;

/// This example demonstrates how to create a reply to a Transfer message
pub fn create_reply_to_transfer_example() -> Result<PlainMessage> {
    let alice_did = "did:example:alice";
    let bob_did = "did:example:bob";

    // Create a Transfer message
    let transfer = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap(),
        originator: Participant {
            id: alice_did.to_string(),
            role: Some("originator".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        },
        beneficiary: Some(Participant {
            id: bob_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        }),
        amount: "100.00".to_string(),
        agents: vec![],
        settlement_id: None,
        memo: None,
        metadata: HashMap::new(),
    };

    // Create the initial transfer message
    let transfer_message = transfer.to_didcomm_with_route(alice_did, [bob_did].iter().copied())?;

    // Create an Authorize message
    let authorize = Authorize {
        transaction_id: transfer_message.id.clone(),
        settlement_address: None,
        expiry: None,
    };

    // Create a reply to the transfer message
    let mut authorize_reply = authorize.to_didcomm(alice_did)?;

    // Set thread ID to maintain conversation
    authorize_reply.thid = Some(transfer_message.id.clone());

    // Set recipients to all participants except the creator
    let recipients: Vec<String> = [alice_did, bob_did]
        .iter()
        .filter(|&did| *did != alice_did)
        .map(|s| s.to_string())
        .collect();

    authorize_reply.to = recipients;

    Ok(authorize_reply)
}

/// This example demonstrates how to create a reply using the Message trait extension
pub fn create_reply_using_message_trait_example(
    original_message: &PlainMessage,
    creator_did: &str,
) -> Result<PlainMessage> {
    // Create an Authorize response
    let authorize = Authorize {
        transaction_id: original_message.id.clone(),
        settlement_address: None,
        expiry: None,
    };

    // Create a reply using the Message trait extension
    // This will automatically gather all participants from the original message
    let mut response_message = authorize.to_didcomm(creator_did)?;

    // Set thread ID to maintain conversation
    response_message.thid = original_message.thid.clone();

    // Set recipients to all participants except the creator
    let recipients: Vec<String> = original_message
        .to
        .iter()
        .filter(|did| did.as_str() != creator_did)
        .cloned()
        .collect();

    response_message.to = recipients;

    Ok(response_message)
}

/// This example demonstrates adding agents using the Authorizable trait
pub fn create_add_agents_example() -> Result<PlainMessage> {
    let originator_did = "did:example:originator";
    let beneficiary_did = "did:example:beneficiary";
    let sender_vasp_did = "did:example:sender_vasp";
    let receiver_vasp_did = "did:example:receiver_vasp";
    let new_agent_did = "did:example:new_agent";

    // Create a Transfer
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
        memo: None,
        agents: vec![
            Participant {
                id: sender_vasp_did.to_string(),
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
    };

    // Create the initial transfer message
    let transfer_message =
        transfer.to_didcomm_with_route(originator_did, [beneficiary_did].iter().copied())?;

    // Create an Authorize message first
    let authorize = Authorize {
        transaction_id: transfer_message.id.clone(),
        settlement_address: None,
        expiry: None,
    };

    // Create a reply from the originator to the beneficiary
    let mut authorize_message = authorize.to_didcomm(originator_did)?;

    // Set thread ID to maintain conversation
    authorize_message.thid = Some(transfer_message.id.clone());

    // Set recipients to all participants except the creator
    let recipients: Vec<String> = [originator_did, beneficiary_did]
        .iter()
        .filter(|&did| *did != originator_did)
        .map(|s| s.to_string())
        .collect();

    authorize_message.to = recipients;

    // Create an AddAgents message
    let add_agents = AddAgents {
        transaction_id: transfer_message.id.clone(),
        agents: vec![Participant {
            id: new_agent_did.to_string(),
            role: Some("compliance".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        }],
    };

    // Create a reply using the TapMessage trait
    let mut response = add_agents.to_didcomm(originator_did)?;

    // Set thread ID to maintain conversation
    response.thid = Some(transfer_message.id.clone());

    // Set recipients to all participants except the creator
    let recipients: Vec<String> = [originator_did, beneficiary_did, new_agent_did]
        .iter()
        .filter(|&did| *did != originator_did)
        .map(|s| s.to_string())
        .collect();

    response.to = recipients;

    Ok(response)
}

/// This example demonstrates replacing an agent using the Authorizable trait
pub fn create_replace_agent_example(
    original_message: &PlainMessage,
    creator_did: &str,
    original_agent_id: &str,
    replacement_agent_id: &str,
    replacement_agent_role: Option<&str>,
) -> Result<PlainMessage> {
    // Extract body and deserialize to Transfer
    let original_body_json = original_message.body.clone();
    let original_transfer: Transfer = serde_json::from_value(original_body_json)
        .map_err(|e| Error::SerializationError(e.to_string()))?;

    // Create a replacement participant
    let replacement = Participant {
        id: replacement_agent_id.to_string(),
        role: replacement_agent_role.map(ToString::to_string),
        policies: None, // No policies for this participant
        leiCode: None,
        name: None,
    };

    // Call replace_agent on the Transfer instance
    let mut response = original_transfer.replace_agent(creator_did, original_agent_id, replacement);

    // Set thread ID to maintain conversation
    response.thid = original_message.thid.clone();

    // Set recipients to all participants except the creator
    let recipients: Vec<String> = original_message
        .to
        .iter()
        .filter(|did| did.as_str() != creator_did)
        .cloned()
        .collect();

    response.to = recipients;

    // Convert typed PlainMessage to untyped PlainMessage
    response.to_plain_message()
}

/// This example demonstrates removing an agent using the Authorizable trait
pub fn create_remove_agent_example(
    original_message: &PlainMessage,
    creator_did: &str,
    agent_to_remove: &str,
) -> Result<PlainMessage> {
    // Extract body and deserialize to Transfer
    let original_body_json = original_message.body.clone();
    let original_transfer: Transfer = serde_json::from_value(original_body_json)
        .map_err(|e| Error::SerializationError(e.to_string()))?;

    // Call remove_agent on the Transfer instance
    let mut response = original_transfer.remove_agent(creator_did, agent_to_remove);

    // Set thread ID to maintain conversation
    response.thid = original_message.thid.clone();

    // Set recipients to all participants except the creator
    let recipients: Vec<String> = original_message
        .to
        .iter()
        .filter(|did| did.as_str() != creator_did)
        .cloned()
        .collect();

    response.to = recipients;

    // Convert typed PlainMessage to untyped PlainMessage
    response.to_plain_message()
}

/// This example demonstrates creating an UpdatePolicies message using the Authorizable trait
pub fn create_update_policies_example(
    original_message: &PlainMessage,
    creator_did: &str,
    _recipients: &[&str],
) -> Result<PlainMessage> {
    // Extract body and deserialize to Transfer
    let original_body_json = original_message.body.clone();
    let original_transfer: Transfer = serde_json::from_value(original_body_json)
        .map_err(|e| Error::SerializationError(e.to_string()))?;

    // Create a proof of control policy
    let proof_policy = RequireProofOfControl {
        from: Some(vec!["did:example:dave".to_string()]),
        from_role: None,
        from_agent: None,
        address_id: "eip155:1:0x1234567890123456789012345678901234567890".to_string(),
        purpose: Some("Please prove control of your account".to_string()),
    };

    // Call update_policies on the Transfer instance
    let mut response = original_transfer.update_policies(
        creator_did,
        vec![Policy::RequireProofOfControl(proof_policy)],
    );

    // Set thread ID to maintain conversation
    response.thid = original_message.thid.clone();

    // Set recipients to all participants except the creator
    let recipients: Vec<String> = original_message
        .to
        .iter()
        .filter(|did| did.as_str() != creator_did)
        .cloned()
        .collect();

    response.to = recipients;

    // Convert typed PlainMessage to untyped PlainMessage
    response.to_plain_message()
}

/// This example demonstrates creating a Settle message
///
/// # Arguments
///
/// * `transfer_id` - ID of the transfer to settle
/// * `settlement_id` - Settlement ID
/// * `amount` - Optional amount settled
///
/// # Returns
///
/// A DIDComm message containing the Settle message
pub fn settle_example(
    transaction_id: String,
    settlement_id: String,
    amount: Option<String>,
) -> Result<PlainMessage> {
    // Create a Settle message
    let settle = Settle {
        transaction_id,
        settlement_id,
        amount,
    };

    // Convert to DIDComm message
    let settle_message = settle.to_didcomm("did:example:dave")?;

    Ok(settle_message)
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
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap(),
        originator: Participant {
            id: alice_did.to_string(),
            role: Some("originator".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        },
        memo: None,
        beneficiary: Some(Participant {
            id: bob_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        }),
        amount: "10.00".to_string(),
        agents: vec![],
        settlement_id: None,
        metadata: HashMap::new(),
    };

    // Create the initial transfer message
    let transfer_message = transfer.to_didcomm_with_route(alice_did, [bob_did].iter().copied())?;

    println!("Created initial transfer message: {:?}", transfer_message);

    // Let's assume we have a unique thread ID
    let transfer_id = transfer_message.id.clone();

    // 2. Now Bob wants to reply to the transfer
    let authorize = Authorize {
        transaction_id: transfer_message.id.clone(),
        settlement_address: None,
        expiry: None,
    };

    // Create a reply from Bob to Alice
    let mut authorize_message = authorize.to_didcomm(bob_did)?;

    // Set thread ID to maintain conversation
    authorize_message.thid = Some(transfer_message.id.clone());

    // Set recipients to all participants except the creator
    let recipients: Vec<String> = [alice_did, bob_did]
        .iter()
        .filter(|&did| *did != bob_did)
        .map(|s| s.to_string())
        .collect();

    authorize_message.to = recipients;

    println!("Created authorize message: {:?}", authorize_message);

    // 3. Now Alice wants to add Charlie to the thread
    // Create an AddAgents message
    let add_agents = AddAgents {
        transaction_id: transfer_id.clone(),
        agents: vec![Participant {
            id: charlie_did.to_string(),
            role: Some("observer".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        }],
    };

    // Create the add agents message
    let mut add_agents_message = add_agents.to_didcomm(alice_did)?;

    // Set thread ID to maintain conversation
    add_agents_message.thid = Some(transfer_id.clone());

    // Set recipients to all participants except the creator
    let recipients: Vec<String> = [alice_did, bob_did, charlie_did]
        .iter()
        .filter(|&did| *did != alice_did)
        .map(|s| s.to_string())
        .collect();

    add_agents_message.to = recipients;

    println!("Created add agents message: {:?}", add_agents_message);

    // 4. Now Bob wants to replace himself with Dave
    let replace_agent = ReplaceAgent {
        transaction_id: transfer_id.clone(),
        original: bob_did.to_string(),
        replacement: Participant {
            id: dave_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        },
    };

    // Create the replace agent message
    let mut replace_agent_message = replace_agent.to_didcomm(bob_did)?;

    // Set thread ID to maintain conversation
    replace_agent_message.thid = Some(transfer_id.clone());

    // Set recipients to all participants except the creator
    let recipients: Vec<String> = [alice_did, dave_did, charlie_did]
        .iter()
        .filter(|&did| *did != bob_did)
        .map(|s| s.to_string())
        .collect();

    replace_agent_message.to = recipients;

    println!("Created replace agent message: {:?}", replace_agent_message);

    // 5. Alice wants to remove Charlie from the thread
    let remove_agent = RemoveAgent {
        transaction_id: transfer_id.clone(),
        agent: charlie_did.to_string(),
    };

    // Create the remove agent message
    let mut remove_agent_message = remove_agent.to_didcomm(alice_did)?;

    // Set thread ID to maintain conversation
    remove_agent_message.thid = Some(transfer_id.clone());

    // Set recipients to all participants except the creator
    let recipients: Vec<String> = [alice_did, dave_did, charlie_did]
        .iter()
        .filter(|&did| *did != alice_did)
        .map(|s| s.to_string())
        .collect();

    remove_agent_message.to = recipients;

    println!("Created remove agent message: {:?}", remove_agent_message);

    // 6. Now Dave can settle the transfer
    println!("Step 5: Settling the transfer");
    let settle_message = settle_example(
        transfer_id.clone(),
        "tx123456".to_string(),
        Some("100.0".to_string()),
    )?;

    // Verify that the 'to' field in the settle message includes Alice
    assert!(settle_message.to.contains(&alice_did.to_string()));
    assert!(!settle_message.to.contains(&dave_did.to_string())); // Dave is the sender
    println!("Verified that the settle message is addressed correctly");

    Ok(())
}
