//! Tests for message processors
//!
//! This file contains integration tests for message processors in the TAP Node.

use serde_json::json;
use tap_msg::didcomm::PlainMessage;
use tap_node::message::processor::{PlainMessageProcessor, ValidationPlainMessageProcessor};

/// Create a valid test message for validation
fn create_test_message(
    id: &str,
    typ: &str,
    from: Option<&str>,
    to: Option<Vec<&str>>,
) -> PlainMessage {
    let from_did = from.unwrap_or("did:example:default_sender").to_string();
    let to_dids = to
        .unwrap_or_else(|| vec!["did:example:default_recipient"])
        .iter()
        .map(|&s| s.to_string())
        .collect();

    PlainMessage {
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
    let processor = ValidationPlainMessageProcessor;

    // Create a valid message with all required fields
    let message = create_test_message(
        "test-123",
        "https://tap.rsvp/schema/1.0#Transfer",
        Some("did:example:sender"),
        Some(vec!["did:example:recipient"]),
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
    let processor = ValidationPlainMessageProcessor;

    // Create a message with an empty ID
    let message = create_test_message(
        "", // Empty ID should be rejected
        "https://tap.rsvp/schema/1.0#Transfer",
        Some("did:example:sender"),
        Some(vec!["did:example:recipient"]),
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
    let processor = ValidationPlainMessageProcessor;

    // Create a message with an empty type
    let message = create_test_message(
        "test-123",
        "", // Empty type should be rejected
        Some("did:example:sender"),
        Some(vec!["did:example:recipient"]),
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
    let processor = ValidationPlainMessageProcessor;

    // Create a message with an invalid from DID
    let message = create_test_message(
        "test-123",
        "https://tap.rsvp/schema/1.0#Transfer",
        Some("invalid-did-format"), // Invalid DID format
        Some(vec!["did:example:recipient"]),
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
    let processor = ValidationPlainMessageProcessor;

    // Create a message with an invalid to DID
    let message = create_test_message(
        "test-123",
        "https://tap.rsvp/schema/1.0#Transfer",
        Some("did:example:sender"),
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
    let processor = ValidationPlainMessageProcessor;

    // Create a message with valid DID formats
    let message = create_test_message(
        "test-123",
        "https://tap.rsvp/schema/1.0#Transfer",
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
