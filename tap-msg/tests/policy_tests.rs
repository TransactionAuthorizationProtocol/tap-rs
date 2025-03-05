//! Tests for TAIP-7 policy implementations.

use std::collections::HashMap;
use tap_msg::message::{
    Participant, Policy, RequireAuthorization, RequirePresentation, RequireProofOfControl,
    TapMessageBody, UpdatePolicies,
};

/// Test creating a participant with policies
#[test]
fn test_participant_with_policies() {
    // Create a policy
    let auth_policy = RequireAuthorization {
        type_: "RequireAuthorization".to_string(),
        from: Some(vec!["did:example:alice".to_string()]),
        from_role: None,
        from_agent: None,
        purpose: Some("Authorization required from Alice".to_string()),
    };

    // Create the participant with the policy
    let participant = Participant {
        id: "did:example:bob".to_string(),
        role: Some("beneficiary".to_string()),
        policies: Some(vec![Policy::RequireAuthorization(auth_policy)]),
    };

    // Verify the policy is correctly included
    assert!(participant.policies.is_some());
    assert_eq!(participant.policies.as_ref().unwrap().len(), 1);

    // Check that we can access the policy
    if let Some(ref policies) = participant.policies {
        if let Policy::RequireAuthorization(policy) = &policies[0] {
            assert_eq!(policy.type_, "RequireAuthorization");
            assert_eq!(policy.from.as_ref().unwrap()[0], "did:example:alice");
            assert_eq!(
                policy.purpose.as_ref().unwrap(),
                "Authorization required from Alice"
            );
        } else {
            panic!("Wrong policy type");
        }
    }
}

/// Test creating UpdatePolicies message
#[test]
fn test_update_policies() {
    // Create a presentation policy
    let presentation_policy = RequirePresentation {
        type_: "RequirePresentation".to_string(),
        context: Some(vec!["https://www.w3.org/2018/credentials/v1".to_string()]),
        from: Some(vec!["did:example:bob".to_string()]),
        from_role: None,
        from_agent: None,
        about_party: Some("originator".to_string()),
        about_agent: None,
        purpose: Some("Please provide KYC credentials".to_string()),
        presentation_definition: Some("https://example.com/presentations/kyc".to_string()),
        credentials: None,
    };

    // Create the UpdatePolicies message
    let update = UpdatePolicies {
        transfer_id: "transfer_12345".to_string(),
        context: "https://tap.rsvp/schemas/1.0".to_string(),
        policies: vec![Policy::RequirePresentation(presentation_policy)],
        metadata: HashMap::new(),
    };

    // Check the message type
    assert_eq!(
        UpdatePolicies::message_type(),
        "https://tap.rsvp/schema/1.0#updatepolicies"
    );

    // Validate the message
    assert!(update.validate().is_ok());
}

/// Test validation of UpdatePolicies message
#[test]
fn test_update_policies_validation() {
    // Create a proof of control policy
    let proof_policy = RequireProofOfControl {
        type_: "RequireProofOfControl".to_string(),
        from: Some(vec!["did:example:dave".to_string()]),
        from_role: None,
        from_agent: None,
        nonce: 12345678,
        purpose: Some("Please prove control of your account".to_string()),
    };

    // Test with empty transfer_id
    let invalid_update = UpdatePolicies {
        transfer_id: "".to_string(),
        context: "https://tap.rsvp/schemas/1.0".to_string(),
        policies: vec![Policy::RequireProofOfControl(proof_policy.clone())],
        metadata: HashMap::new(),
    };
    assert!(invalid_update.validate().is_err());

    // Test with empty context
    let invalid_update = UpdatePolicies {
        transfer_id: "transfer_12345".to_string(),
        context: "".to_string(),
        policies: vec![Policy::RequireProofOfControl(proof_policy.clone())],
        metadata: HashMap::new(),
    };
    assert!(invalid_update.validate().is_err());

    // Test with empty policies
    let invalid_update = UpdatePolicies {
        transfer_id: "transfer_12345".to_string(),
        context: "https://tap.rsvp/schemas/1.0".to_string(),
        policies: vec![],
        metadata: HashMap::new(),
    };
    assert!(invalid_update.validate().is_err());

    // Valid message should pass validation
    let valid_update = UpdatePolicies {
        transfer_id: "transfer_12345".to_string(),
        context: "https://tap.rsvp/schemas/1.0".to_string(),
        policies: vec![Policy::RequireProofOfControl(proof_policy)],
        metadata: HashMap::new(),
    };
    assert!(valid_update.validate().is_ok());
}

/// Test all policy types
#[test]
fn test_all_policy_types() {
    // Create authorization policy
    let auth_policy = RequireAuthorization::default();
    assert_eq!(auth_policy.type_, "RequireAuthorization");

    // Create presentation policy
    let presentation_policy = RequirePresentation::default();
    assert_eq!(presentation_policy.type_, "RequirePresentation");

    // Create proof of control policy
    let proof_policy = RequireProofOfControl::default();
    assert_eq!(proof_policy.type_, "RequireProofOfControl");

    // Ensure nonce is generated
    assert_ne!(proof_policy.nonce, 0);
}
