//! Test that the ValidationPlainMessageProcessor handles both seconds and milliseconds timestamps

use std::collections::HashMap;
use tap_msg::didcomm::PlainMessage;
use tap_node::message::processor::{PlainMessageProcessor, ValidationPlainMessageProcessor};

#[tokio::test]
async fn test_timestamp_in_seconds_passes_validation() {
    let processor = ValidationPlainMessageProcessor;

    // Create a message with timestamp in seconds (common in some systems)
    let message = PlainMessage {
        id: "test-seconds-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/Transfer".to_string(),
        body: serde_json::json!({
            "test": "data"
        }),
        from: "did:key:test_sender".to_string(),
        to: vec!["did:key:test_recipient".to_string()],
        thid: None,
        pthid: None,
        extra_headers: HashMap::new(),
        attachments: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64), // Timestamp in seconds
        expires_time: None,
        from_prior: None,
    };

    // Should pass validation
    let result = processor.process_incoming(message).await.unwrap();
    assert!(
        result.is_some(),
        "Message with timestamp in seconds should pass validation"
    );
}

#[tokio::test]
async fn test_timestamp_in_milliseconds_passes_validation() {
    let processor = ValidationPlainMessageProcessor;

    // Create a message with timestamp in milliseconds (DIDComm standard)
    let message = PlainMessage {
        id: "test-millis-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/Transfer".to_string(),
        body: serde_json::json!({
            "test": "data"
        }),
        from: "did:key:test_sender".to_string(),
        to: vec!["did:key:test_recipient".to_string()],
        thid: None,
        pthid: None,
        extra_headers: HashMap::new(),
        attachments: None,
        created_time: Some(chrono::Utc::now().timestamp_millis() as u64), // Timestamp in milliseconds
        expires_time: None,
        from_prior: None,
    };

    // Should pass validation
    let result = processor.process_incoming(message).await.unwrap();
    assert!(
        result.is_some(),
        "Message with timestamp in milliseconds should pass validation"
    );
}

#[tokio::test]
async fn test_future_timestamp_seconds_fails_validation() {
    let processor = ValidationPlainMessageProcessor;

    // Create a message with future timestamp in seconds
    let message = PlainMessage {
        id: "test-future-seconds-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/Transfer".to_string(),
        body: serde_json::json!({
            "test": "data"
        }),
        from: "did:key:test_sender".to_string(),
        to: vec!["did:key:test_recipient".to_string()],
        thid: None,
        pthid: None,
        extra_headers: HashMap::new(),
        attachments: None,
        created_time: Some((chrono::Utc::now().timestamp() + 3600) as u64), // 1 hour future in seconds
        expires_time: None,
        from_prior: None,
    };

    // Should fail validation
    let result = processor.process_incoming(message).await.unwrap();
    assert!(
        result.is_none(),
        "Message with future timestamp should fail validation"
    );
}

#[tokio::test]
async fn test_future_timestamp_milliseconds_fails_validation() {
    let processor = ValidationPlainMessageProcessor;

    // Create a message with future timestamp in milliseconds
    let message = PlainMessage {
        id: "test-future-millis-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/Transfer".to_string(),
        body: serde_json::json!({
            "test": "data"
        }),
        from: "did:key:test_sender".to_string(),
        to: vec!["did:key:test_recipient".to_string()],
        thid: None,
        pthid: None,
        extra_headers: HashMap::new(),
        attachments: None,
        created_time: Some((chrono::Utc::now().timestamp_millis() + 3_600_000) as u64), // 1 hour future in milliseconds
        expires_time: None,
        from_prior: None,
    };

    // Should fail validation
    let result = processor.process_incoming(message).await.unwrap();
    assert!(
        result.is_none(),
        "Message with future timestamp should fail validation"
    );
}

#[tokio::test]
async fn test_slightly_future_timestamp_within_tolerance() {
    let processor = ValidationPlainMessageProcessor;

    // Create a message with timestamp 1 minute in the future (within 5 minute tolerance)
    let message_seconds = PlainMessage {
        id: "test-slight-future-seconds-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/Transfer".to_string(),
        body: serde_json::json!({
            "test": "data"
        }),
        from: "did:key:test_sender".to_string(),
        to: vec!["did:key:test_recipient".to_string()],
        thid: None,
        pthid: None,
        extra_headers: HashMap::new(),
        attachments: None,
        created_time: Some((chrono::Utc::now().timestamp() + 60) as u64), // 1 minute future in seconds
        expires_time: None,
        from_prior: None,
    };

    // Should pass validation (within 5 minute tolerance)
    let result = processor.process_incoming(message_seconds).await.unwrap();
    assert!(
        result.is_some(),
        "Message with 1 minute future timestamp should pass validation"
    );

    // Same test with milliseconds
    let message_millis = PlainMessage {
        id: "test-slight-future-millis-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/Transfer".to_string(),
        body: serde_json::json!({
            "test": "data"
        }),
        from: "did:key:test_sender".to_string(),
        to: vec!["did:key:test_recipient".to_string()],
        thid: None,
        pthid: None,
        extra_headers: HashMap::new(),
        attachments: None,
        created_time: Some((chrono::Utc::now().timestamp_millis() + 60_000) as u64), // 1 minute future in milliseconds
        expires_time: None,
        from_prior: None,
    };

    let result = processor.process_incoming(message_millis).await.unwrap();
    assert!(
        result.is_some(),
        "Message with 1 minute future timestamp in milliseconds should pass validation"
    );
}
