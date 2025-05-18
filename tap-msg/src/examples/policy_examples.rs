//! Examples for using policies according to TAIP-7.

use crate::error::Result;
use crate::message::{
    policy::{Policy, RequireAuthorization, RequirePresentation, RequireProofOfControl},
    tap_message_trait::TapMessageBody,
    Authorize, Participant, Transfer, UpdatePolicies,
};
use crate::didcomm::PlainMessage;

// This trait isn't exported yet, but it should exist somewhere
trait Authorizable {}
use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;

/// This example demonstrates how to create a participant with policies
pub fn create_participant_with_policies_example() -> Result<Participant> {
    // Create a RequireAuthorization policy
    let auth_policy = RequireAuthorization {
        from: Some(vec!["did:example:alice".to_string()]),
        from_role: None,
        from_agent: None,
        purpose: Some("Authorization required from Alice".to_string()),
    };

    // Create a RequirePresentation policy
    let presentation_policy = RequirePresentation {
        context: Some(vec![
            "https://www.w3.org/2018/credentials/v1".to_string(),
            "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
        ]),
        from: Some(vec!["did:example:bob".to_string()]),
        from_role: None,
        from_agent: None,
        about_party: Some("originator".to_string()),
        about_agent: None,
        purpose: Some("Please provide KYC credentials".to_string()),
        presentation_definition: Some("https://example.com/presentations/kyc".to_string()),
        credentials: None,
    };

    // Create the participant with policies
    let participant = Participant {
        id: "did:example:charlie".to_string(),
        role: Some("beneficiary".to_string()),
        leiCode: None,
        policies: Some(vec![
            Policy::RequireAuthorization(auth_policy),
            Policy::RequirePresentation(presentation_policy),
        ]),
    };

    Ok(participant)
}

/// This example demonstrates how to update policies for a transaction
pub fn update_policies_example(
    transaction_id: &str,
    creator_did: &str,
    recipients: &[&str],
) -> Result<Message> {
    // Create a proof of control policy
    let proof_policy = RequireProofOfControl {
        from: Some(vec!["did:example:dave".to_string()]),
        from_role: None,
        from_agent: None,
        address_id: "eip155:1:0x1234567890123456789012345678901234567890".to_string(),
        purpose: Some("Please prove control of your account".to_string()),
    };

    // Create an UpdatePolicies message
    let update = UpdatePolicies {
        transaction_id: transaction_id.to_string(),
        policies: vec![Policy::RequireProofOfControl(proof_policy)],
    };

    // Convert the update to a DIDComm message
    let participants = recipients
        .iter()
        .filter(|&&did| did != creator_did)
        .copied()
        .collect::<Vec<_>>();
    let message = update.to_didcomm_with_route(Some(creator_did), participants)?;

    // Set the thread ID to link this message to the existing thread
    let message_with_thread = Message {
        thid: Some(transaction_id.to_string()),
        ..message
    };

    Ok(message_with_thread)
}

/// This example demonstrates a complete workflow for adding and updating policies
pub fn policy_workflow_example() -> Result<()> {
    println!("=== Starting Policy Workflow Example ===");

    // Define DIDs for our example
    let originator_did = "did:example:originator";
    let beneficiary_did = "did:example:beneficiary";
    let sender_vaspd_did = "did:example:sender_vasp";
    let receiver_vaspd_did = "did:example:receiver_vasp";

    // Step 1: Create a beneficiary with policies
    println!("Step 1: Creating beneficiary with policies");
    let auth_policy = RequireAuthorization {
        from: Some(vec![originator_did.to_string()]),
        from_role: None,
        from_agent: None,
        purpose: Some("Authorization required from originator".to_string()),
    };

    let beneficiary = Participant {
        id: beneficiary_did.to_string(),
        role: Some("beneficiary".to_string()),
        leiCode: None,
        policies: Some(vec![Policy::RequireAuthorization(auth_policy)]),
    };
    println!("  Created beneficiary with policies: {:?}", beneficiary);

    // Step 2: Create a transfer ID (this would normally be generated in practice)
    let transfer_id = "transfer_12345";
    println!("Step 2: Created transfer ID: {}", transfer_id);

    // Step 3: Sender VASP wants to add a presentation requirement policy
    println!("Step 3: Sender VASP adds a presentation requirement");
    let presentation_policy = RequirePresentation {
        context: Some(vec!["https://www.w3.org/2018/credentials/v1".to_string()]),
        from: Some(vec![beneficiary_did.to_string()]),
        from_role: None,
        from_agent: None,
        about_party: Some("beneficiary".to_string()),
        about_agent: None,
        purpose: Some("Please provide identity credentials".to_string()),
        presentation_definition: None,
        credentials: Some(HashMap::from([(
            "type".to_string(),
            vec!["IdentityCredential".to_string()],
        )])),
    };

    let update_message = UpdatePolicies {
        transaction_id: transfer_id.to_string(),
        policies: vec![Policy::RequirePresentation(presentation_policy)],
    };

    // Convert to DIDComm message with proper routing
    let participants = [
        originator_did,
        beneficiary_did,
        sender_vaspd_did,
        receiver_vaspd_did,
    ];
    let to = participants
        .iter()
        .filter(|&&did| did != sender_vaspd_did)
        .copied()
        .collect::<Vec<_>>();
    let message = update_message.to_didcomm_with_route(Some(sender_vaspd_did), to)?;

    // Link to our transfer thread
    let message_with_thread = Message {
        thid: Some(transfer_id.to_string()),
        ..message
    };

    println!(
        "  Created UpdatePolicies message: {:?}",
        message_with_thread
    );
    println!("  This message will be routed to all participants");

    println!("=== Policy Workflow Example Completed ===");
    Ok(())
}

/// This example demonstrates the use of the Authorizable trait's update_policies method
pub fn create_update_policies_using_authorizable_example(
    original_message: &Result<Message>,
    policies: Vec<Policy>,
    _transaction_id: &str,
    creator_did: &str,
    participant_dids: &[String],
) -> Result<Message> {
    // 1. Extract the body from the original DIDComm message
    let body_json = original_message
        .as_ref()
        .map_err(Clone::clone)?
        .body
        .clone();
    // 2. Deserialize the body into a Transfer struct
    let transfer_body: Transfer = serde_json::from_value(body_json.clone())
        .map_err(|e| crate::error::Error::SerializationError(e.to_string()))?;
    // 3. Call update_policies on the Transfer struct (Authorizable trait impl)
    // Extract or generate a transaction ID
    let transaction_id = transfer_body.transaction_id.clone();
    let update_policies_message = transfer_body.update_policies(transaction_id, policies);

    // Convert the update to a DIDComm message
    let mut update_policies_reply = update_policies_message.to_didcomm(Some(creator_did))?;

    // Set thread ID to maintain conversation
    update_policies_reply.thid = Some(original_message.as_ref().map_err(Clone::clone)?.id.clone());

    // Set recipients
    update_policies_reply.to = Some(participant_dids.iter().map(|s| s.to_string()).collect());

    Ok(update_policies_reply)
}

/// This example demonstrates a modified policy workflow using the Authorizable trait
pub fn policy_workflow_with_authorizable_example() -> Result<()> {
    println!("=== Starting Policy Workflow with Authorizable Example ===");

    // Define DIDs for our example
    let originator_did = "did:example:originator";
    let beneficiary_did = "did:example:beneficiary";
    let sender_vasp_did = "did:example:sender_vasp";
    let receiver_vasp_did = "did:example:receiver_vasp";

    // Step 1: Create a transfer message to initiate the workflow
    let transfer = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset: AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap(),
        originator: Participant {
            id: originator_did.to_string(),
            role: Some("originator".to_string()),
            leiCode: None,
            policies: None,
        },
        beneficiary: Some(Participant {
            id: beneficiary_did.to_string(),
            role: Some("beneficiary".to_string()),
            leiCode: None,
            policies: None,
        }),
        amount: "100.00".to_string(),
        memo: None,
        agents: vec![
            Participant {
                id: sender_vasp_did.to_string(),
                role: Some("sender_vasp".to_string()),
                leiCode: None,
                policies: None,
            },
            Participant {
                id: receiver_vasp_did.to_string(),
                role: Some("receiver_vasp".to_string()),
                leiCode: None,
                policies: None,
            },
        ],
        settlement_id: None,
        metadata: HashMap::new(),
    };

    // Convert the transfer to a DIDComm message
    let transfer_message = transfer.to_didcomm_with_route(
        Some(originator_did),
        [beneficiary_did, sender_vasp_did, receiver_vasp_did]
            .iter()
            .copied(),
    )?;

    println!("Transfer message created: {:?}", transfer_message);

    // Step 2: Create an UpdatePolicies message using the Authorizable trait
    // This would typically be created by a VASP to enforce compliance
    let participants = [
        originator_did.to_string(),
        beneficiary_did.to_string(),
        sender_vasp_did.to_string(),
        receiver_vasp_did.to_string(),
    ];

    let cloned_transfer_id = transfer_message.id.clone();
    let update_policies_message = create_update_policies_using_authorizable_example(
        &Ok(transfer_message),
        vec![],
        &cloned_transfer_id,
        sender_vasp_did,
        &participants,
    )?;

    println!(
        "Update policies message created: {:?}",
        update_policies_message
    );

    // Step 3: Create an authorization message in response to the updated policies
    let authorize_body =
        transfer.authorize(Some("Authorization with policy constraints".to_string()));

    // Create a reply to the update policies message
    let mut authorize_reply = authorize_body.to_didcomm(Some(beneficiary_did))?;

    // Set thread ID to maintain conversation
    authorize_reply.thid = Some(update_policies_message.id.clone());

    // Set recipients
    authorize_reply.to = Some(participants.iter().map(|s| s.to_string()).collect());

    println!("Authorization message created: {:?}", authorize_reply);

    Ok(())
}

/// This example demonstrates a modified policy workflow using the Authorizable trait
pub fn create_authorize_example() -> Result<()> {
    // Create an example Authorize message body
    let authorize_message = Authorize {
        transaction_id: "transfer_12345".to_string(),
        note: Some("Authorized with policy constraints".to_string()),
    };

    println!("Authorize message: {:#?}", authorize_message);

    Ok(())
}
