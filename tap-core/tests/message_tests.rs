use std::collections::HashMap;
use tap_core::message::{TapMessage, TapMessageType, TransactionProposalBody, Validate};

#[test]
fn test_transaction_proposal_message() {
    // Create a valid transaction proposal
    let transaction_id = "123e4567-e89b-12d3-a456-426614174000";
    let body = TransactionProposalBody {
        transaction_id: transaction_id.to_string(),
        network: "eip155:1".to_string(),
        sender: "eip155:1:0x1234567890abcdef1234567890abcdef12345678".to_string(),
        recipient: "eip155:1:0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
        asset: "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7".to_string(),
        amount: "100.0".to_string(),
        memo: Some("Test transaction".to_string()),
        tx_reference: None,
        metadata: HashMap::new(),
    };

    let message = TapMessage::new(TapMessageType::TransactionProposal)
        .with_id("msg-id-1")
        .with_body(&body);

    // Validation should pass
    assert!(message.validate().is_ok());

    // Create a message with empty ID
    let empty_id_message = TapMessage::new(TapMessageType::TransactionProposal)
        .with_id("")
        .with_body(&body);

    // Validation should fail for empty ID
    assert!(empty_id_message.validate().is_err());
}

#[test]
fn test_identity_exchange_message() {
    // Test the identity exchange message type
    let message = TapMessage::new(TapMessageType::IdentityExchange).with_id("msg-id-2");

    // Basic validation should pass
    assert!(message.validate().is_ok());
}

#[test]
fn test_travel_rule_info_message() {
    // Test the travel rule info message type
    let message = TapMessage::new(TapMessageType::TravelRuleInfo).with_id("msg-id-3");

    // Basic validation should pass
    assert!(message.validate().is_ok());
}

#[test]
fn test_authorization_response_message() {
    // Test the authorization response message type
    let message = TapMessage::new(TapMessageType::AuthorizationResponse).with_id("msg-id-4");

    // Basic validation should pass
    assert!(message.validate().is_ok());
}

#[test]
fn test_error_message() {
    // Test the error message type
    let message = TapMessage::new(TapMessageType::Error).with_id("msg-id-5");

    // Basic validation should pass
    assert!(message.validate().is_ok());
}

#[test]
fn test_custom_message_type() {
    // Test a custom message type
    let message =
        TapMessage::new(TapMessageType::Custom("test-custom-type".to_string())).with_id("msg-id-6");

    // Basic validation should pass
    assert!(message.validate().is_ok());
}
