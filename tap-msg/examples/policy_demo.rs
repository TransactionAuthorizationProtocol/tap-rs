use std::collections::HashMap;
use tap_msg::message::{
    Participant, Policy, RequireAuthorization, RequirePresentation, RequireProofOfControl,
    UpdatePolicies,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== TAP Policies (TAIP-7) Demo ===\n");

    // Example 1: Create a participant with policies
    println!("\n1. Creating a Participant with Policies:");

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
        policies: Some(vec![
            Policy::RequireAuthorization(auth_policy),
            Policy::RequirePresentation(presentation_policy),
        ]),
        lei: None,
    };

    println!("Participant ID: {}", participant.id);
    println!("Participant Role: {:?}", participant.role);

    if let Some(policies) = &participant.policies {
        println!("Policies count: {}", policies.len());
        for (i, policy) in policies.iter().enumerate() {
            match policy {
                Policy::RequireAuthorization(p) => {
                    println!("  Policy {}: RequireAuthorization", i + 1);
                    println!("    Purpose: {:?}", p.purpose);
                    println!("    From: {:?}", p.from);
                }
                Policy::RequirePresentation(p) => {
                    println!("  Policy {}: RequirePresentation", i + 1);
                    println!("    Purpose: {:?}", p.purpose);
                    println!("    From: {:?}", p.from);
                    println!("    About Party: {:?}", p.about_party);
                    println!(
                        "    Presentation Definition: {:?}",
                        p.presentation_definition
                    );
                }
                Policy::RequireProofOfControl(p) => {
                    println!("  Policy {}: RequireProofOfControl", i + 1);
                    println!("    Purpose: {:?}", p.purpose);
                    println!("    From: {:?}", p.from);
                    println!("    Nonce: {}", p.nonce);
                }
            }
        }
    }

    // Example 2: Create and validate an UpdatePolicies message
    println!("\n2. Creating and Validating an UpdatePolicies Message:");

    // Create a proof of control policy
    let proof_policy = RequireProofOfControl {
        from: Some(vec!["did:example:dave".to_string()]),
        from_role: None,
        from_agent: None,
        nonce: 12345678,
        purpose: Some("Please prove control of your account".to_string()),
    };

    // Create an UpdatePolicies message
    let update = UpdatePolicies {
        transfer_id: "transfer_abc123".to_string(),
        policies: vec![Policy::RequireProofOfControl(proof_policy.clone())],
        metadata: HashMap::new(),
    };

    println!("UpdatePolicies message created:");
    println!("  Transfer ID: {}", update.transfer_id);
    println!("  Number of policies: {}", update.policies.len());

    // Validate the message
    match update.validate() {
        Ok(_) => println!("  Validation: SUCCESS - Message is valid"),
        Err(e) => println!("  Validation: FAILED - {}", e),
    }

    // Example 3: Test update validation with errors
    println!("\n3. Testing UpdatePolicies Validation with Errors:");

    // Test with empty transfer_id
    let invalid_update_1 = UpdatePolicies {
        transfer_id: "".to_string(),
        policies: vec![Policy::RequireProofOfControl(proof_policy.clone())],
        metadata: HashMap::new(),
    };

    match invalid_update_1.validate() {
        Ok(_) => println!("  Empty transfer_id validation: Unexpectedly passed"),
        Err(e) => println!("  Empty transfer_id validation: Failed as expected - {}", e),
    }

    // Test with empty context
    let invalid_update_2 = UpdatePolicies {
        transfer_id: "transfer_abc123".to_string(),
        policies: vec![Policy::RequireProofOfControl(proof_policy.clone())],
        metadata: HashMap::new(),
    };

    match invalid_update_2.validate() {
        Ok(_) => println!("  Empty context validation: Unexpectedly passed"),
        Err(e) => println!("  Empty context validation: Failed as expected - {}", e),
    }

    // Test with empty policies
    let invalid_update_3 = UpdatePolicies {
        transfer_id: "transfer_abc123".to_string(),
        policies: vec![],
        metadata: HashMap::new(),
    };

    match invalid_update_3.validate() {
        Ok(_) => println!("  Empty policies validation: Unexpectedly passed"),
        Err(e) => println!("  Empty policies validation: Failed as expected - {}", e),
    }

    println!("\n4. Default Policies:");

    // Default RequireAuthorization
    let default_auth = RequireAuthorization::default();
    println!("\n  Default RequireAuthorization:");
    println!("    From: {:?}", default_auth.from);
    println!("    Purpose: {:?}", default_auth.purpose);

    // Default RequirePresentation
    let default_presentation = RequirePresentation::default();
    println!("\n  Default RequirePresentation:");
    println!("    Context: {:?}", default_presentation.context);
    println!("    From: {:?}", default_presentation.from);
    println!("    Purpose: {:?}", default_presentation.purpose);

    // Default RequireProofOfControl
    let default_proof = RequireProofOfControl::default();
    println!("\n  Default RequireProofOfControl:");
    println!("    From: {:?}", default_proof.from);
    println!("    Nonce: {}", default_proof.nonce);
    println!("    Purpose: {:?}", default_proof.purpose);

    println!("\n=== Demo Completed Successfully ===");
    Ok(())
}
