use tap_msg::message::{
    Participant, Policy, RequireAuthorization, RequirePresentation, RequireProofOfControl,
    UpdatePolicies,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TAP Message Policy Demo ===\n");

    // Example 1: Create a Transfer message with policies
    println!("1. Creating a Participant with Policies:");

    // Create an authorization policy
    let auth_policy = RequireAuthorization {
        from: Some(vec!["did:example:alice".to_string()]),
        from_role: None,
        from_agent: None,
        purpose: Some("Authorization required for all transfers".to_string()),
    };

    // Create a presentation policy
    let presentation_policy = RequirePresentation {
        context: Some(vec![
            "https://www.w3.org/2018/credentials/v1".to_string(),
            "https://w3id.org/security/suites/jws-2020/v1".to_string(),
        ]),
        from: Some(vec!["did:example:bob".to_string()]),
        from_role: None,
        from_agent: None,
        about_party: None,
        about_agent: None,
        purpose: Some("Please provide KYC credentials".to_string()),
        presentation_definition: Some("https://example.com/presentations/kyc".to_string()),
        credentials: None,
    };

    // Create the participant with policies
    let participant = Participant {
        id: "did:example:bob".to_string(),
        role: Some("beneficiary".to_string()),
        policies: Some(vec![
            Policy::RequireAuthorization(auth_policy),
            Policy::RequirePresentation(presentation_policy),
        ]),
        leiCode: None,
    };

    // Display the participant info
    println!("  Participant: {}", participant.id);
    println!("  Role: {:?}", participant.role);

    if let Some(policies) = &participant.policies {
        println!("  Policies:");
        println!("  Policies count: {}", policies.len());
        for (i, policy) in policies.iter().enumerate() {
            match policy {
                Policy::RequireAuthorization(p) => {
                    println!("    Policy {}: RequireAuthorization", i + 1);
                    println!("      Purpose: {:?}", p.purpose);
                    println!("      From: {:?}", p.from);
                    println!("      From Role: {:?}", p.from_role);
                    println!("      From Agent: {:?}", p.from_agent);
                }
                Policy::RequirePresentation(p) => {
                    println!("    Policy {}: RequirePresentation", i + 1);
                    println!("      Context: {:?}", p.context);
                    println!("      From: {:?}", p.from);
                    println!("      About Party: {:?}", p.about_party);
                    println!("      Purpose: {:?}", p.purpose);
                    println!(
                        "      Presentation Definition: {:?}",
                        p.presentation_definition
                    );
                }
                Policy::RequireProofOfControl(p) => {
                    println!("    Policy {}: RequireProofOfControl", i + 1);
                    println!("      Purpose: {:?}", p.purpose);
                    println!("      From: {:?}", p.from);
                    println!("      Address ID: {}", p.address_id);
                }
                Policy::RequireRelationshipConfirmation(p) => {
                    println!("    Policy {}: RequireRelationshipConfirmation", i + 1);
                    println!("      Purpose: {:?}", p.purpose);
                    println!("      From Role: {:?}", p.from_role);
                    println!("      Nonce: {:?}", p.nonce);
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
        address_id: "eip155:1:0x1234567890123456789012345678901234567890".to_string(),
        purpose: Some("Please prove control of your account".to_string()),
    };

    // Create an UpdatePolicies message
    let update = UpdatePolicies {
        transaction_id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
        policies: vec![
            Policy::RequireAuthorization(RequireAuthorization {
                from: Some(vec!["did:example:charlie".to_string()]),
                from_role: None,
                from_agent: None,
                purpose: Some("Additional authorization required".to_string()),
            }),
            Policy::RequireProofOfControl(proof_policy),
        ],
    };

    // Display the UpdatePolicies message
    println!("  UpdatePolicies Message:");
    println!("  Transaction ID: {}", update.transaction_id);
    println!("  Policies count: {}", update.policies.len());

    for (i, policy) in update.policies.iter().enumerate() {
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
            }
            Policy::RequireProofOfControl(p) => {
                println!("  Policy {}: RequireProofOfControl", i + 1);
                println!("    Purpose: {:?}", p.purpose);
                println!("    From: {:?}", p.from);
                println!("    Address ID: {}", p.address_id);
            }
            Policy::RequireRelationshipConfirmation(p) => {
                println!("  Policy {}: RequireRelationshipConfirmation", i + 1);
                println!("    Purpose: {:?}", p.purpose);
                println!("    From Role: {:?}", p.from_role);
                println!("    Nonce: {:?}", p.nonce);
            }
        }
    }

    // Example 3: Default values
    println!("\n3. Default Policy Values:");

    // Create default policies
    let default_auth = RequireAuthorization::default();
    println!("\n  Default RequireAuthorization:");
    println!("    From: {:?}", default_auth.from);
    println!("    Purpose: {:?}", default_auth.purpose);

    let default_proof = RequireProofOfControl::default();
    println!("\n  Default RequireProofOfControl:");
    println!("    From: {:?}", default_proof.from);
    println!("    Address ID: {}", default_proof.address_id);
    println!("    Purpose: {:?}", default_proof.purpose);

    println!("\n=== Demo Completed Successfully ===");

    Ok(())
}
