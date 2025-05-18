use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::didcomm::PlainMessage;
use tap_msg::message::tap_message_trait::{Connectable, TapMessageBody};
use tap_msg::message::{Connect, Participant, PaymentRequest, Transfer};

/// This module contains fuzzing tests for TAP message types.
/// These tests are designed to ensure that our code handles malformed inputs gracefully.

#[test]
fn test_fuzz_transfer_deserialization() {
    // Test with valid JSON but invalid Transfer structure
    let invalid_json = r#"{
        "asset": "not-a-valid-asset",
        "originator": {
            "id": "not-a-valid-did"
        },
        "amount": "not-a-number",
        "agents": "not-an-array"
    }"#;

    let result = serde_json::from_str::<Transfer>(invalid_json);
    assert!(
        result.is_err(),
        "Should fail to deserialize invalid Transfer JSON"
    );

    // Test with malformed JSON
    let malformed_json = r#"{
        "asset": "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "originator": {
            "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
        },
        "amount": "100.0",
        "agents": [
            {
                "id": "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx"
            }
        ],
        "metadata": {"key": "value"
    }"#; // Missing closing brace

    let result = serde_json::from_str::<Transfer>(malformed_json);
    assert!(result.is_err(), "Should fail to deserialize malformed JSON");
}

#[test]
fn test_fuzz_connect_deserialization() {
    // Test with valid JSON but invalid Connect structure
    let invalid_json = r#"{
        "agents": "not-an-array",
        "constraints": "not-a-valid-constraints-object"
    }"#;

    let result = serde_json::from_str::<Connect>(invalid_json);
    assert!(
        result.is_err(),
        "Should fail to deserialize invalid Connect JSON"
    );

    // Test with empty agents array (which should be invalid)
    let empty_agents_json = r#"{
        "agents": [],
        "metadata": {}
    }"#;

    let result = serde_json::from_str::<Connect>(empty_agents_json);
    // This might not fail at deserialization time, but should fail validation
    if let Ok(connect) = result {
        assert!(
            connect.validate().is_err(),
            "Connect with empty agents should fail validation"
        );
    }
}

#[test]
fn test_fuzz_payment_request_deserialization() {
    // Test with valid JSON but invalid PaymentRequest structure
    let invalid_json = r#"{
        "amount": "not-a-number",
        "merchant": "not-a-valid-participant",
        "agents": "not-an-array"
    }"#;

    let result = serde_json::from_str::<PaymentRequest>(invalid_json);
    assert!(
        result.is_err(),
        "Should fail to deserialize invalid PaymentRequest JSON"
    );

    // Test with missing required fields
    let missing_fields_json = r#"{
        "amount": "100.0",
        "agents": []
    }"#; // Missing merchant

    let result = serde_json::from_str::<PaymentRequest>(missing_fields_json);
    assert!(
        result.is_err(),
        "Should fail to deserialize PaymentRequest with missing fields"
    );
}

#[test]
fn test_fuzz_message_deserialization() {
    // Test with valid JSON but invalid Message structure
    let invalid_json = r#"{
        "id": 12345,
        "type": "not-a-valid-type",
        "body": "not-a-valid-body"
    }"#;

    let result = serde_json::from_str::<Message>(invalid_json);
    assert!(
        result.is_err(),
        "Should fail to deserialize invalid Message JSON"
    );

    // Test with missing required fields
    let missing_fields_json = r#"{
        "body": {}
    }"#; // Missing id and type

    let result = serde_json::from_str::<Message>(missing_fields_json);
    assert!(
        result.is_err(),
        "Should fail to deserialize Message with missing fields"
    );
}

#[test]
fn test_fuzz_connectable_with_invalid_id() {
    // Create a Transfer message
    let mut transfer = create_test_transfer();

    // Test with empty connect_id
    transfer.with_connection("");
    assert!(transfer.has_connection());
    assert_eq!(transfer.connection_id(), Some(""));

    // Test with very long connect_id
    let long_id = "a".repeat(10000);
    transfer.with_connection(&long_id);
    assert!(transfer.has_connection());
    assert_eq!(transfer.connection_id(), Some(long_id.as_str()));

    // Test with special characters
    let special_chars = "!@#$%^&*()_+{}|:<>?";
    transfer.with_connection(special_chars);
    assert!(transfer.has_connection());
    assert_eq!(transfer.connection_id(), Some(special_chars));
}

#[test]
fn test_fuzz_tap_message_body_validation() {
    // Create a Transfer with invalid data
    let mut transfer = create_test_transfer();

    // Test with empty amount
    transfer.amount = "".to_string();
    assert!(
        transfer.validate().is_err(),
        "Transfer with empty amount should fail validation"
    );

    // Test with negative amount
    transfer.amount = "-100.0".to_string();
    assert!(
        transfer.validate().is_err(),
        "Transfer with negative amount should fail validation"
    );

    // Test with non-numeric amount
    transfer.amount = "not-a-number".to_string();
    assert!(
        transfer.validate().is_err(),
        "Transfer with non-numeric amount should fail validation"
    );

    // Test with missing originator
    let mut transfer = create_test_transfer();
    transfer.originator = Participant {
        id: "".to_string(),
        role: None,
        policies: None,
        leiCode: None,
    };
    assert!(
        transfer.validate().is_err(),
        "Transfer with empty originator ID should fail validation"
    );
}

#[test]
fn test_fuzz_didcomm_conversion() {
    // Create a Transfer with valid data
    let transfer = create_test_transfer();

    // Convert to DIDComm message
    let didcomm_message = transfer
        .to_didcomm(None)
        .expect("Failed to convert to DIDComm");

    // Modify the message to have an invalid type
    let mut invalid_message = didcomm_message.clone();
    invalid_message.type_ = "invalid-type".to_string();

    // Try to convert back to Transfer
    let result = Transfer::from_didcomm(&invalid_message);
    assert!(
        result.is_err(),
        "Should fail to convert message with invalid type to Transfer"
    );

    // Modify the message to have invalid body JSON
    let mut invalid_body_message = didcomm_message.clone();
    invalid_body_message.body = serde_json::json!({
        "asset": "not-a-valid-asset",
        "amount": "not-a-number"
    });

    // Try to convert back to Transfer
    let result = Transfer::from_didcomm(&invalid_body_message);
    assert!(
        result.is_err(),
        "Should fail to convert message with invalid body to Transfer"
    );
}

// Helper function to create a test Transfer
fn create_test_transfer() -> Transfer {
    let asset =
        AssetId::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

    let originator = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let beneficiary = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    let agents = vec![Participant {
        id: "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx".to_string(),
        role: None,
        policies: None,
        leiCode: None,
    }];

    Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset,
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents,
        settlement_id: None,
        metadata: HashMap::new(),
        memo: None,
    }
}
