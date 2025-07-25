//! Tests for the ValidationPlainMessageProcessor
//!
//! This file contains tests for the validation functionality of the TAP Node,
//! specifically testing the ValidationPlainMessageProcessor implementation.

use serde_json::json;
use tap_msg::didcomm::PlainMessage;
use tap_node::message::processor::{PlainMessageProcessor, ValidationPlainMessageProcessor};

#[tokio::test]
async fn test_valid_message_passes_validation() {
    // Create a processor
    let processor = ValidationPlainMessageProcessor;

    // Create a valid message
    let message = PlainMessage {
        id: "test-id-123".to_string(),
        typ: "https://didcomm.org/basicmessage/2.0/message".to_string(),
        type_: "https://didcomm.org/basicmessage/2.0/message".to_string(),
        body: json!({
            "content": "Hello, this is a basic message!"
        }),
        from: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        to: vec!["did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".to_string()],
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        thid: Some("thread-123".to_string()),
        pthid: None,
        attachments: None,
        from_prior: None,
        extra_headers: Default::default(),
    };

    // Process the message - it should pass validation
    let result = processor.process_incoming(message.clone()).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_some());
    assert_eq!(processed.unwrap().id, "test-id-123");

    // Outgoing message should also pass
    let result = processor.process_outgoing(message).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_some());
}

#[tokio::test]
async fn test_missing_id_fails_validation() {
    // Create a processor
    let processor = ValidationPlainMessageProcessor;

    // Create a message missing the required ID field
    let message = PlainMessage {
        id: "".to_string(), // Empty ID
        typ: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        type_: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        body: json!({
            "amount": "100.00",
            "asset": "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
            "transaction_id": "tx-123456"
        }),
        from: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        to: vec!["did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".to_string()],
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        thid: Some("thread-123".to_string()),
        pthid: None,
        attachments: None,
        from_prior: None,
        extra_headers: Default::default(),
    };

    // Process the message - it should fail validation
    let result = processor.process_incoming(message).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none()); // Message should be dropped
}

#[tokio::test]
async fn test_missing_type_fails_validation() {
    // Create a processor
    let processor = ValidationPlainMessageProcessor;

    // Create a message missing the type field
    let message = PlainMessage {
        id: "test-id-123".to_string(),
        typ: "".to_string(),   // Empty type
        type_: "".to_string(), // Empty type
        body: json!({
            "amount": "100.00",
            "asset": "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
            "transaction_id": "tx-123456"
        }),
        from: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        to: vec!["did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".to_string()],
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        thid: Some("thread-123".to_string()),
        pthid: None,
        attachments: None,
        from_prior: None,
        extra_headers: Default::default(),
    };

    // Process the message - it should fail validation
    let result = processor.process_incoming(message).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none()); // Message should be dropped
}

#[tokio::test]
async fn test_invalid_from_did_fails_validation() {
    // Create a processor
    let processor = ValidationPlainMessageProcessor;

    // Create a message with an invalid FROM DID
    let message = PlainMessage {
        id: "test-id-123".to_string(),
        typ: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        type_: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        body: json!({
            "amount": "100.00",
            "asset": "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
            "transaction_id": "tx-123456"
        }),
        from: "invalid-did-format".to_string(), // Invalid DID format
        to: vec!["did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".to_string()],
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        thid: Some("thread-123".to_string()),
        pthid: None,
        attachments: None,
        from_prior: None,
        extra_headers: Default::default(),
    };

    // Process the message - it should fail validation
    let result = processor.process_incoming(message).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none()); // Message should be dropped
}

#[tokio::test]
async fn test_invalid_to_did_fails_validation() {
    // Create a processor
    let processor = ValidationPlainMessageProcessor;

    // Create a message with an invalid TO DID
    let message = PlainMessage {
        id: "test-id-123".to_string(),
        typ: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        type_: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        body: json!({
            "amount": "100.00",
            "asset": "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
            "transaction_id": "tx-123456"
        }),
        from: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        to: vec!["invalid-recipient-did".to_string()], // Invalid DID format
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        thid: Some("thread-123".to_string()),
        pthid: None,
        attachments: None,
        from_prior: None,
        extra_headers: Default::default(),
    };

    // Process the message - it should fail validation
    let result = processor.process_incoming(message).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none()); // Message should be dropped
}

#[tokio::test]
async fn test_future_timestamp_fails_validation() {
    // Create a processor
    let processor = ValidationPlainMessageProcessor;

    // Create a message with a timestamp too far in the future
    let message = PlainMessage {
        id: "test-id-123".to_string(),
        typ: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        type_: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        body: json!({
            "amount": "100.00",
            "asset": "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
            "transaction_id": "tx-123456"
        }),
        from: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        to: vec!["did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".to_string()],
        created_time: Some((chrono::Utc::now().timestamp() + 3600) as u64), // 1 hour in the future
        expires_time: None,
        thid: Some("thread-123".to_string()),
        pthid: None,
        attachments: None,
        from_prior: None,
        extra_headers: Default::default(),
    };

    // Process the message - it should fail validation
    let result = processor.process_incoming(message).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none()); // Message should be dropped
}

#[tokio::test]
async fn test_unknown_message_type_fails_validation() {
    // Create a processor
    let processor = ValidationPlainMessageProcessor;

    // Create a message with an unknown type
    let message = PlainMessage {
        id: "test-id-123".to_string(),
        typ: "unknown-message-type".to_string(), // Unknown type
        type_: "unknown-message-type".to_string(), // Unknown type
        body: json!({
            "amount": "100.00",
            "asset": "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
            "transaction_id": "tx-123456"
        }),
        from: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        to: vec!["did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".to_string()],
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        thid: Some("thread-123".to_string()),
        pthid: None,
        attachments: None,
        from_prior: None,
        extra_headers: Default::default(),
    };

    // Process the message - it should fail validation
    let result = processor.process_incoming(message).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none()); // Message should be dropped
}

#[tokio::test]
async fn test_missing_body_fails_validation() {
    // Create a processor
    let processor = ValidationPlainMessageProcessor;

    // Create a TAP message missing a required body
    let message = PlainMessage {
        id: "test-id-123".to_string(),
        typ: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        type_: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        body: json!(null), // Empty body
        from: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        to: vec!["did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".to_string()],
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        thid: Some("thread-123".to_string()),
        pthid: None,
        attachments: None,
        from_prior: None,
        extra_headers: Default::default(),
    };

    // Process the message - it should fail validation
    let result = processor.process_incoming(message).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none()); // Message should be dropped
}

#[tokio::test]
async fn test_invalid_body_format_fails_validation() {
    // Create a processor
    let processor = ValidationPlainMessageProcessor;

    // Create a DIDComm message with an invalid body format
    let message = PlainMessage {
        id: "test-id-123".to_string(),
        typ: "https://didcomm.org/basicmessage/1.0/message".to_string(),
        type_: "https://didcomm.org/basicmessage/1.0/message".to_string(),
        body: json!(null), // null is not a valid body format
        from: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        to: vec!["did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".to_string()],
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        thid: Some("thread-123".to_string()),
        pthid: None,
        attachments: None,
        from_prior: None,
        extra_headers: Default::default(),
    };

    // Process the message - it should fail validation
    let result = processor.process_incoming(message).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none()); // Message should be dropped
}

#[tokio::test]
async fn test_empty_pthid_fails_validation() {
    // Create a processor
    let processor = ValidationPlainMessageProcessor;

    // Create a message with an empty pthid
    let message = PlainMessage {
        id: "test-id-123".to_string(),
        typ: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        type_: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        body: json!({
            "amount": "100.00",
            "asset": "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
            "transaction_id": "tx-123456"
        }),
        from: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        to: vec!["did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".to_string()],
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        thid: Some("thread-123".to_string()),
        pthid: Some("".to_string()), // Empty pthid
        attachments: None,
        from_prior: None,
        extra_headers: Default::default(),
    };

    // Process the message - it should fail validation
    let result = processor.process_incoming(message).await;
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert!(processed.is_none()); // Message should be dropped
}
