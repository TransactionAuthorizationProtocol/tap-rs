//! Tests for message processors
//!
//! This file contains integration tests for message processors in the TAP Node.

use tap_msg::didcomm::Message;
use tap_node::message::processor::{MessageProcessor, ValidationMessageProcessor};
use serde_json::json;

/// Create a valid test message for validation
fn create_test_message(id: &str, typ: &str, from: Option<&str>, to: Option<Vec<&str>>) -> Message {
    let from_did = from.map(|s| s.to_string());
    let to_dids = to.map(|v| v.iter().map(|&s| s.to_string()).collect());
    
    Message {
        id: id.to_string(),
        typ: typ.to_string(),
        type_: typ.to_string(),
        body: json!({"test": "body"}),
        from: from_did,
        to: to_dids,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        thid: None,
        pthid: None,
        attachments: None,
        from_prior: None,
        extra_headers: Default::default(),
    }
}

#[tokio::test]
async fn test_validation_processor_accepts_valid_messages() {
    // Create a validator
    let processor = ValidationMessageProcessor;

    // Create a valid message with all required fields
    let message = create_test_message(
        "test-123",
        "https://tap.rsvp/schema/1.0#transfer",
        None,
        None,
    );

    // Process the message
    let result = processor.process_incoming(message).await;

    // Should succeed and return the message
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_some());
}

#[tokio::test]
async fn test_validation_processor_rejects_empty_id() {
    // Create a validator
    let processor = ValidationMessageProcessor;

    // Create a message with an empty ID
    let message = create_test_message(
        "", // Empty ID should be rejected
        "https://tap.rsvp/schema/1.0#transfer",
        None,
        None,
    );

    // Process the message
    let result = processor.process_incoming(message).await;

    // Should return None (rejected message)
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none());
}

#[tokio::test]
async fn test_validation_processor_rejects_empty_type() {
    // Create a validator
    let processor = ValidationMessageProcessor;

    // Create a message with an empty type
    let message = create_test_message(
        "test-123", "", // Empty type should be rejected
        None, None,
    );

    // Process the message
    let result = processor.process_incoming(message).await;

    // Should return None (rejected message)
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none());
}

#[tokio::test]
async fn test_validation_processor_rejects_invalid_from_did() {
    // Create a validator
    let processor = ValidationMessageProcessor;

    // Create a message with an invalid from DID
    let message = create_test_message(
        "test-123",
        "https://tap.rsvp/schema/1.0#transfer",
        Some("invalid-did-format"), // Invalid DID format
        None,
    );

    // Process the message
    let result = processor.process_incoming(message).await;

    // Should return None (rejected message)
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none());
}

#[tokio::test]
async fn test_validation_processor_rejects_invalid_to_did() {
    // Create a validator
    let processor = ValidationMessageProcessor;

    // Create a message with an invalid to DID
    let message = create_test_message(
        "test-123",
        "https://tap.rsvp/schema/1.0#transfer",
        None,
        Some(vec!["invalid-did-format"]), // Invalid DID format
    );

    // Process the message
    let result = processor.process_incoming(message).await;

    // Should return None (rejected message)
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none());
}

#[tokio::test]
async fn test_validation_processor_accepts_valid_did_formats() {
    // Create a validator
    let processor = ValidationMessageProcessor;

    // Create a message with valid DID formats
    let message = create_test_message(
        "test-123",
        "https://tap.rsvp/schema/1.0#transfer",
        Some("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"),
        Some(vec![
            "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
        ]),
    );

    // Process the message
    let result = processor.process_incoming(message).await;

    // Should succeed and return the message
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_some());
}