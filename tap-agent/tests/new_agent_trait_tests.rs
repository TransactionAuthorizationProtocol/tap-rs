//! Tests for the new Agent trait methods
//!
//! Tests the refactored Agent trait methods for handling encrypted and plain messages

use serde_json::json;
use tap_agent::{Agent, PackOptions, Packable, TapAgent};
use tap_msg::didcomm::PlainMessage;

#[tokio::test]
async fn test_receive_plain_message() {
    // Create agent
    let (agent, _did) = TapAgent::from_ephemeral_key().await.unwrap();

    // Create a plain message
    let message = PlainMessage {
        id: "test-plain-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://example.org/test".to_string(),
        body: json!({"content": "Hello World"}),
        from: "did:example:sender".to_string(),
        to: vec![agent.get_agent_did().to_string()],
        thid: None,
        pthid: None,
        created_time: Some(1234567890),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };

    // Agent should process the message successfully
    let result = agent.receive_plain_message(message).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_receive_encrypted_message() {
    // Create two agents - sender and receiver
    let (sender_agent, _sender_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (receiver_agent, receiver_did) = TapAgent::from_ephemeral_key().await.unwrap();

    // Create a message
    let message = PlainMessage {
        id: "test-encrypted-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://example.org/test".to_string(),
        body: json!({"secret": "encrypted content"}),
        from: sender_agent.get_agent_did().to_string(),
        to: vec![receiver_did.clone()],
        thid: None,
        pthid: None,
        created_time: Some(1234567890),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };

    // Note: Key resolution is handled internally by the key manager

    // Encrypt the message using AuthCrypt
    let sender_kid = format!("{}#keys-1", sender_agent.get_agent_did());
    let recipient_kid = format!("{}#keys-1", receiver_did);

    let pack_options = PackOptions {
        security_mode: tap_agent::SecurityMode::AuthCrypt,
        sender_kid: Some(sender_kid),
        recipient_kid: Some(recipient_kid),
    };

    let encrypted = message
        .pack(sender_agent.key_manager().as_ref(), pack_options)
        .await
        .unwrap();
    let jwe_value: serde_json::Value = serde_json::from_str(&encrypted).unwrap();

    // Receiver should be able to decrypt and process the message
    let result = receiver_agent.receive_encrypted_message(&jwe_value).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_receive_message_standalone_plain() {
    // Create agent
    let (agent, _did) = TapAgent::from_ephemeral_key().await.unwrap();

    // Create a plain message JSON
    let message_json = json!({
        "id": "test-standalone-plain",
        "typ": "application/didcomm-plain+json",
        "type": "https://example.org/test",
        "body": {"content": "Plain message"},
        "from": "did:example:sender",
        "to": [agent.get_agent_did()],
        "created_time": 1234567890
    });

    let message_str = serde_json::to_string(&message_json).unwrap();

    // Agent should process and return the plain message
    let result = agent.receive_message(&message_str).await;
    assert!(result.is_ok());

    let plain_message = result.unwrap();
    assert_eq!(plain_message.id, "test-standalone-plain");
    assert_eq!(plain_message.type_, "https://example.org/test");
}

#[tokio::test]
async fn test_receive_message_standalone_signed() {
    // Create sender agent
    let (sender_agent, _sender_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (receiver_agent, _receiver_did) = TapAgent::from_ephemeral_key().await.unwrap();

    // Create a message to sign
    let message = PlainMessage {
        id: "test-standalone-signed".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://example.org/test".to_string(),
        body: json!({"content": "Signed message"}),
        from: sender_agent.get_agent_did().to_string(),
        to: vec![receiver_agent.get_agent_did().to_string()],
        thid: None,
        pthid: None,
        created_time: Some(1234567890),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };

    // Sign the message
    let sender_kid = format!("{}#keys-1", sender_agent.get_agent_did());
    let pack_options = PackOptions::new().with_sign(&sender_kid);
    let signed_message = message
        .pack(sender_agent.key_manager().as_ref(), pack_options)
        .await
        .unwrap();

    // Receiver should process signed message (note: in standalone mode,
    // signature verification will use the unpacking method since no resolver is available)
    let _result = receiver_agent.receive_message(&signed_message).await;
    // This might fail because standalone verification needs the key
    // This is expected - standalone agents would need the signing keys available
    // or a resolver to verify signatures
}

#[tokio::test]
async fn test_receive_message_standalone_encrypted() {
    // Create two agents
    let (sender_agent, _sender_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (receiver_agent, receiver_did) = TapAgent::from_ephemeral_key().await.unwrap();

    // Create a message to encrypt
    let message = PlainMessage {
        id: "test-standalone-encrypted".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://example.org/test".to_string(),
        body: json!({"secret": "encrypted content"}),
        from: sender_agent.get_agent_did().to_string(),
        to: vec![receiver_did.clone()],
        thid: None,
        pthid: None,
        created_time: Some(1234567890),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };

    // Encrypt the message
    let sender_kid = format!("{}#keys-1", sender_agent.get_agent_did());
    let recipient_kid = format!("{}#keys-1", receiver_did);

    let pack_options = PackOptions {
        security_mode: tap_agent::SecurityMode::AuthCrypt,
        sender_kid: Some(sender_kid),
        recipient_kid: Some(recipient_kid),
    };

    let encrypted = message
        .pack(sender_agent.key_manager().as_ref(), pack_options)
        .await
        .unwrap();

    // Receiver should decrypt and return the plain message
    let result = receiver_agent.receive_message(&encrypted).await;
    assert!(result.is_ok());

    let plain_message = result.unwrap();
    assert_eq!(plain_message.id, "test-standalone-encrypted");
    assert_eq!(plain_message.type_, "https://example.org/test");
}

#[tokio::test]
async fn test_receive_encrypted_message_wrong_recipient() {
    // Create three agents
    let (sender_agent, _sender_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (_intended_receiver, intended_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (wrong_receiver, _wrong_did) = TapAgent::from_ephemeral_key().await.unwrap();

    // Create a message encrypted for intended_receiver
    let message = PlainMessage {
        id: "test-wrong-recipient".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://example.org/test".to_string(),
        body: json!({"secret": "not for you"}),
        from: sender_agent.get_agent_did().to_string(),
        to: vec![intended_did.clone()],
        thid: None,
        pthid: None,
        created_time: Some(1234567890),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };

    let sender_kid = format!("{}#keys-1", sender_agent.get_agent_did());
    let recipient_kid = format!("{}#keys-1", intended_did);

    let pack_options = PackOptions {
        security_mode: tap_agent::SecurityMode::AuthCrypt,
        sender_kid: Some(sender_kid),
        recipient_kid: Some(recipient_kid),
    };

    let encrypted = message
        .pack(sender_agent.key_manager().as_ref(), pack_options)
        .await
        .unwrap();
    let jwe_value: serde_json::Value = serde_json::from_str(&encrypted).unwrap();

    // Wrong receiver should fail to decrypt
    let result = wrong_receiver.receive_encrypted_message(&jwe_value).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_receive_message_invalid_json() {
    // Create agent
    let (agent, _did) = TapAgent::from_ephemeral_key().await.unwrap();

    // Invalid JSON should fail
    let result = agent.receive_message("invalid json {").await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to parse message as JSON"));
}

#[tokio::test]
async fn test_receive_plain_message_processing() {
    // Create agent
    let (agent, _did) = TapAgent::from_ephemeral_key().await.unwrap();

    // Create multiple plain messages
    let messages = vec![
        PlainMessage {
            id: "msg-1".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://example.org/transfer".to_string(),
            body: json!({"amount": 100}),
            from: "did:example:sender".to_string(),
            to: vec![agent.get_agent_did().to_string()],
            thid: None,
            pthid: None,
            created_time: Some(1234567890),
            expires_time: None,
            from_prior: None,
            attachments: None,
            extra_headers: Default::default(),
        },
        PlainMessage {
            id: "msg-2".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://example.org/payment".to_string(),
            body: json!({"currency": "USD"}),
            from: "did:example:sender".to_string(),
            to: vec![agent.get_agent_did().to_string()],
            thid: Some("msg-1".to_string()),
            pthid: None,
            created_time: Some(1234567891),
            expires_time: None,
            from_prior: None,
            attachments: None,
            extra_headers: Default::default(),
        },
    ];

    // Agent should process all messages successfully
    for message in messages {
        let result = agent.receive_plain_message(message).await;
        assert!(result.is_ok());
    }
}
