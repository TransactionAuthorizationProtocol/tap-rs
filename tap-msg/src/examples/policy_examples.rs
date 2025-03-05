//! Examples for using policies according to TAIP-7.

use crate::error::Result;
use crate::message::{
    Participant, Policy, RequireAuthorization, RequirePresentation, RequireProofOfControl,
    TapMessageBody, UpdatePolicies, TapMessage,
};
use didcomm::Message;
use std::collections::HashMap;

/// This example demonstrates how to create a participant with policies
pub fn create_participant_with_policies_example() -> Result<Participant> {
    // Create a RequireAuthorization policy
    let auth_policy = RequireAuthorization {
        type_: "RequireAuthorization".to_string(),
        from: Some(vec!["did:example:alice".to_string()]),
        from_role: None,
        from_agent: None,
        purpose: Some("Authorization required from Alice".to_string()),
    };

    // Create a RequirePresentation policy
    let presentation_policy = RequirePresentation {
        type_: "RequirePresentation".to_string(),
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
        policies: Some(vec![
            Policy::RequireAuthorization(auth_policy),
            Policy::RequirePresentation(presentation_policy),
        ]),
    };

    Ok(participant)
}

/// This example demonstrates how to update policies for a transaction
pub fn update_policies_example(
    transfer_id: &str,
    creator_did: &str,
    recipients: &[&str],
) -> Result<Message> {
    // Create a proof of control policy
    let proof_policy = RequireProofOfControl {
        type_: "RequireProofOfControl".to_string(),
        from: Some(vec!["did:example:dave".to_string()]),
        from_role: None,
        from_agent: None,
        nonce: 12345678,
        purpose: Some("Please prove control of your account".to_string()),
    };

    // Create an UpdatePolicies message
    let update = UpdatePolicies {
        transfer_id: transfer_id.to_string(),
        context: "https://tap.rsvp/schemas/1.0".to_string(),
        policies: vec![Policy::RequireProofOfControl(proof_policy)],
        metadata: HashMap::new(),
    };

    // Convert the update to a DIDComm message
    let message = update.to_didcomm_with_route(
        Some(creator_did),
        recipients.iter().filter(|&&did| did != creator_did).map(|&did| did),
    )?;

    // Set the thread ID to link this message to the existing thread
    let message_with_thread = Message {
        thid: Some(transfer_id.to_string()),
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
        type_: "RequireAuthorization".to_string(),
        from: Some(vec![originator_did.to_string()]),
        from_role: None,
        from_agent: None,
        purpose: Some("Authorization required from originator".to_string()),
    };

    let beneficiary = Participant {
        id: beneficiary_did.to_string(),
        role: Some("beneficiary".to_string()),
        policies: Some(vec![Policy::RequireAuthorization(auth_policy)]),
    };
    println!("  Created beneficiary with policies: {:?}", beneficiary);

    // Step 2: Create a transfer ID (this would normally be generated in practice)
    let transfer_id = "transfer_12345";
    println!("Step 2: Created transfer ID: {}", transfer_id);

    // Step 3: Sender VASP wants to add a presentation requirement policy
    println!("Step 3: Sender VASP adds a presentation requirement");
    let presentation_policy = RequirePresentation {
        type_: "RequirePresentation".to_string(),
        context: Some(vec!["https://www.w3.org/2018/credentials/v1".to_string()]),
        from: Some(vec![beneficiary_did.to_string()]),
        from_role: None,
        from_agent: None,
        about_party: Some("beneficiary".to_string()),
        about_agent: None,
        purpose: Some("Please provide identity credentials".to_string()),
        presentation_definition: None,
        credentials: Some(HashMap::from([
            ("type".to_string(), vec!["IdentityCredential".to_string()]),
        ])),
    };

    let update_message = UpdatePolicies {
        transfer_id: transfer_id.to_string(),
        context: "https://tap.rsvp/schemas/1.0".to_string(),
        policies: vec![Policy::RequirePresentation(presentation_policy)],
        metadata: HashMap::new(),
    };

    // Convert to DIDComm message with proper routing
    let participants = [originator_did, beneficiary_did, sender_vaspd_did, receiver_vaspd_did];
    let message = update_message.to_didcomm_with_route(
        Some(sender_vaspd_did),
        participants.iter().filter(|&&did| did != sender_vaspd_did).map(|&did| did),
    )?;

    // Link to our transfer thread
    let message_with_thread = Message {
        thid: Some(transfer_id.to_string()),
        ..message
    };

    println!("  Created UpdatePolicies message: {:?}", message_with_thread);
    println!("  This message will be routed to all participants");

    println!("=== Policy Workflow Example Completed ===");
    Ok(())
}
