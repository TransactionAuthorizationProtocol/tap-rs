//! Test DIDComm message validation in TAP Node
//!
//! This test verifies that DIDComm messages (Trust Ping, Basic Message)
//! pass through the ValidationPlainMessageProcessor correctly.

use std::collections::HashMap;
use tap_msg::didcomm::PlainMessage;
use tap_node::message::processor::{PlainMessageProcessor, ValidationPlainMessageProcessor};

#[tokio::test]
async fn test_trust_ping_passes_validation() {
    let processor = ValidationPlainMessageProcessor;

    // Create a Trust Ping message
    let trust_ping = PlainMessage {
        id: "test-ping-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://didcomm.org/trust-ping/2.0/ping".to_string(),
        body: serde_json::json!({
            "response_requested": true,
            "comment": "Test ping"
        }),
        from: "did:key:test_sender".to_string(),
        to: vec!["did:key:test_recipient".to_string()],
        thid: None,
        pthid: None,
        extra_headers: HashMap::new(),
        attachments: None,
        created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
        expires_time: None,
        from_prior: None,
    };

    // Test incoming validation
    let result = processor
        .process_incoming(trust_ping.clone())
        .await
        .unwrap();
    assert!(
        result.is_some(),
        "Trust Ping should pass incoming validation"
    );

    // Test outgoing validation
    let result = processor.process_outgoing(trust_ping).await.unwrap();
    assert!(
        result.is_some(),
        "Trust Ping should pass outgoing validation"
    );
}

#[tokio::test]
async fn test_basic_message_passes_validation() {
    let processor = ValidationPlainMessageProcessor;

    // Create a Basic Message
    let basic_message = PlainMessage {
        id: "test-basic-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://didcomm.org/basicmessage/2.0/message".to_string(),
        body: serde_json::json!({
            "content": "Hello, this is a test message!",
            "locale": "en"
        }),
        from: "did:key:test_sender".to_string(),
        to: vec!["did:key:test_recipient".to_string()],
        thid: None,
        pthid: None,
        extra_headers: HashMap::new(),
        attachments: None,
        created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
        expires_time: None,
        from_prior: None,
    };

    // Test incoming validation
    let result = processor
        .process_incoming(basic_message.clone())
        .await
        .unwrap();
    assert!(
        result.is_some(),
        "Basic Message should pass incoming validation"
    );

    // Test outgoing validation
    let result = processor.process_outgoing(basic_message).await.unwrap();
    assert!(
        result.is_some(),
        "Basic Message should pass outgoing validation"
    );
}

#[tokio::test]
async fn test_unknown_didcomm_message_passes_validation() {
    let processor = ValidationPlainMessageProcessor;

    // Create an unknown DIDComm message type
    let unknown_message = PlainMessage {
        id: "test-unknown-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://didcomm.org/unknown-protocol/1.0/unknown-message".to_string(),
        body: serde_json::json!({
            "test": "data"
        }),
        from: "did:key:test_sender".to_string(),
        to: vec!["did:key:test_recipient".to_string()],
        thid: None,
        pthid: None,
        extra_headers: HashMap::new(),
        attachments: None,
        created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
        expires_time: None,
        from_prior: None,
    };

    // Test incoming validation - should pass (we allow unknown DIDComm types)
    let result = processor
        .process_incoming(unknown_message.clone())
        .await
        .unwrap();
    assert!(
        result.is_some(),
        "Unknown DIDComm message should pass incoming validation"
    );

    // Test outgoing validation - should pass (we allow unknown DIDComm types)
    let result = processor.process_outgoing(unknown_message).await.unwrap();
    assert!(
        result.is_some(),
        "Unknown DIDComm message should pass outgoing validation"
    );
}

#[tokio::test]
async fn test_unknown_protocol_fails_validation() {
    let processor = ValidationPlainMessageProcessor;

    // Create a message with completely unknown protocol
    let unknown_protocol = PlainMessage {
        id: "test-unknown-protocol-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://unknown.example.com/protocol/1.0/message".to_string(),
        body: serde_json::json!({
            "test": "data"
        }),
        from: "did:key:test_sender".to_string(),
        to: vec!["did:key:test_recipient".to_string()],
        thid: None,
        pthid: None,
        extra_headers: HashMap::new(),
        attachments: None,
        created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
        expires_time: None,
        from_prior: None,
    };

    // Test incoming validation - should fail
    let result = processor
        .process_incoming(unknown_protocol.clone())
        .await
        .unwrap();
    assert!(
        result.is_none(),
        "Unknown protocol should fail incoming validation"
    );

    // Test outgoing validation - should fail
    let result = processor.process_outgoing(unknown_protocol).await.unwrap();
    assert!(
        result.is_none(),
        "Unknown protocol should fail outgoing validation"
    );
}
