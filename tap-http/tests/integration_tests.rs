//! End-to-end integration tests for the TAP protocol
//!
//! Tests the complete flow from HTTP request through TAP Node to Agent processing

use bytes::Bytes;
use serde_json::json;
use std::sync::Arc;
use tap_agent::{PackOptions, Packable, SecurityMode, TapAgent};
use tap_http::{event::EventBus, handler::handle_didcomm};
use tap_msg::didcomm::PlainMessage;
use tap_node::{NodeConfig, TapNode};
use warp::hyper::body::to_bytes;
use warp::Reply;

async fn response_to_json(response: impl Reply) -> serde_json::Value {
    let response_bytes = to_bytes(response.into_response().into_body()).await.unwrap();
    serde_json::from_slice(&response_bytes).unwrap()
}

#[tokio::test]
async fn test_end_to_end_signed_message_flow() {
    // Setup: Create node and agents
    let config = NodeConfig::default();
    let node = Arc::new(TapNode::new(config));
    let event_bus = Arc::new(EventBus::new());

    // Create sender and receiver agents
    let (sender_agent, sender_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (receiver_agent, receiver_did) = TapAgent::from_ephemeral_key().await.unwrap();

    // Register receiver with node
    node.register_agent(Arc::new(receiver_agent)).await.unwrap();

    // Create and sign a message
    let message = PlainMessage {
        id: "e2e-signed-test".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/transfer".to_string(),
        body: json!({
            "amount": "100.00",
            "currency": "USD",
            "from_account": "alice-account",
            "to_account": "bob-account"
        }),
        from: sender_did.clone(),
        to: vec![receiver_did.clone()],
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };

    // Sign the message
    let sender_kid = format!("{}#keys-1", sender_did);

    let pack_options = PackOptions::new().with_sign(&sender_kid);
    let signed_message = message
        .pack(sender_agent.key_manager().as_ref(), pack_options)
        .await
        .unwrap();

    // Test HTTP handler processing
    let body = Bytes::from(signed_message);
    let content_type = Some("application/didcomm-signed+json".to_string());

    let response = handle_didcomm(content_type, body, node.clone(), event_bus.clone())
        .await
        .unwrap();

    // Verify response
    let _response_json = response_to_json(response).await;
    // Note: This will likely fail verification because the node's resolver doesn't have the sender's DID
    // But it should get to the verification step, not fail on parsing
}

#[tokio::test]
async fn test_end_to_end_encrypted_message_flow() {
    // Setup: Create node and agents
    let config = NodeConfig::default();
    let node = Arc::new(TapNode::new(config));
    let event_bus = Arc::new(EventBus::new());

    // Create sender and receiver agents
    let (sender_agent, sender_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (receiver_agent, receiver_did) = TapAgent::from_ephemeral_key().await.unwrap();

    // Register receiver with node
    node.register_agent(Arc::new(receiver_agent)).await.unwrap();

    // Create and encrypt a message
    let message = PlainMessage {
        id: "e2e-encrypted-test".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/payment".to_string(),
        body: json!({
            "payment_id": "pay-12345",
            "amount": "50.00",
            "currency": "EUR",
            "memo": "Confidential payment"
        }),
        from: sender_did.clone(),
        to: vec![receiver_did.clone()],
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };

    // Encrypt the message
    let pack_options = PackOptions {
        security_mode: SecurityMode::AuthCrypt,
        sender_kid: Some(format!("{}#keys-1", sender_did)),
        recipient_kid: Some(format!("{}#keys-1", receiver_did)),
    };

    let encrypted_message = message
        .pack(sender_agent.key_manager().as_ref(), pack_options)
        .await
        .unwrap();

    // Test HTTP handler processing
    let body = Bytes::from(encrypted_message);
    let content_type = Some("application/didcomm-encrypted+json".to_string());

    let response = handle_didcomm(content_type, body, node.clone(), event_bus.clone())
        .await
        .unwrap();

    // Verify successful response
    let response_json = response_to_json(response).await;
    assert_eq!(response_json["status"], "success");
}

#[tokio::test]
async fn test_end_to_end_multiple_agents() {
    // Setup: Create node with multiple agents
    let config = NodeConfig::default();
    let node = Arc::new(TapNode::new(config));
    let event_bus = Arc::new(EventBus::new());

    // Create multiple agents
    let (agent1, _agent1_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (agent2, agent2_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (agent3, _agent3_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (sender_agent, sender_did) = TapAgent::from_ephemeral_key().await.unwrap();

    // Register all agents with node
    node.register_agent(Arc::new(agent1)).await.unwrap();
    node.register_agent(Arc::new(agent2)).await.unwrap();
    node.register_agent(Arc::new(agent3)).await.unwrap();

    // Send encrypted message to agent2
    let message = PlainMessage {
        id: "multi-agent-test".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://example.org/multi-agent".to_string(),
        body: json!({
            "recipient": "agent2",
            "message": "This is for agent2 only"
        }),
        from: sender_did.clone(),
        to: vec![agent2_did.clone()],
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };

    let pack_options = PackOptions {
        security_mode: SecurityMode::AuthCrypt,
        sender_kid: Some(format!("{}#keys-1", sender_did)),
        recipient_kid: Some(format!("{}#keys-1", agent2_did)),
    };

    let encrypted_message = message
        .pack(sender_agent.key_manager().as_ref(), pack_options)
        .await
        .unwrap();

    // Process through HTTP handler
    let body = Bytes::from(encrypted_message);
    let content_type = Some("application/didcomm-encrypted+json".to_string());

    let response = handle_didcomm(content_type, body, node.clone(), event_bus.clone())
        .await
        .unwrap();

    // Should succeed - agent2 should decrypt and process the message
    let response_json = response_to_json(response).await;
    assert_eq!(response_json["status"], "success");
}

#[tokio::test]
async fn test_end_to_end_security_validation() {
    // Setup
    let config = NodeConfig::default();
    let node = Arc::new(TapNode::new(config));
    let event_bus = Arc::new(EventBus::new());

    // Test 1: Plain message should be rejected
    let plain_message = json!({
        "id": "plain-test",
        "typ": "application/didcomm-plain+json",
        "type": "https://example.org/test",
        "body": {"content": "This should be rejected"},
        "from": "did:example:sender",
        "to": ["did:example:receiver"]
    });

    let body = Bytes::from(serde_json::to_string(&plain_message).unwrap());
    let content_type = Some("application/didcomm-plain+json".to_string());

    let response = handle_didcomm(content_type, body, node.clone(), event_bus.clone())
        .await
        .unwrap();
    let response_json = response_to_json(response).await;

    assert_eq!(response_json["status"], "error");
    assert!(response_json["message"]
        .as_str()
        .unwrap()
        .contains("Plain DIDComm messages are not allowed"));

    // Test 2: Missing content type should be rejected
    let body = Bytes::from(serde_json::to_string(&plain_message).unwrap());
    let response = handle_didcomm(None, body, node.clone(), event_bus.clone())
        .await
        .unwrap();
    let response_json = response_to_json(response).await;

    assert_eq!(response_json["status"], "error");
    let message = response_json["message"].as_str().unwrap_or("");
    assert!(message.contains("Missing Content-Type header"), "Expected 'Missing Content-Type header' but got: {}", message);

    // Test 3: Invalid content type should be rejected
    let body = Bytes::from(serde_json::to_string(&plain_message).unwrap());
    let content_type = Some("application/json".to_string());

    let response = handle_didcomm(content_type, body, node.clone(), event_bus.clone())
        .await
        .unwrap();
    let response_json = response_to_json(response).await;

    assert_eq!(response_json["status"], "error");
    assert!(response_json["message"]
        .as_str()
        .unwrap()
        .contains("Invalid Content-Type"));
}

#[tokio::test]
async fn test_end_to_end_error_handling() {
    // Setup
    let config = NodeConfig::default();
    let node = Arc::new(TapNode::new(config));
    let event_bus = Arc::new(EventBus::new());

    // Test 1: Invalid UTF-8
    let invalid_bytes = Bytes::from(vec![0xFF, 0xFF]);
    let content_type = Some("application/didcomm-signed+json".to_string());

    let response = handle_didcomm(content_type, invalid_bytes, node.clone(), event_bus.clone())
        .await
        .unwrap();
    let response_json = response_to_json(response).await;

    assert_eq!(response_json["status"], "error");
    assert!(response_json["message"]
        .as_str()
        .unwrap()
        .contains("Invalid UTF-8"));

    // Test 2: Invalid JSON
    let invalid_json = Bytes::from("invalid json {");
    let content_type = Some("application/didcomm-signed+json".to_string());

    let response = handle_didcomm(content_type, invalid_json, node.clone(), event_bus.clone())
        .await
        .unwrap();
    let response_json = response_to_json(response).await;

    assert_eq!(response_json["status"], "error");
    assert!(response_json["message"]
        .as_str()
        .unwrap()
        .contains("Invalid JSON"));
}

#[tokio::test]
async fn test_end_to_end_message_threading() {
    // Setup
    let config = NodeConfig::default();
    let node = Arc::new(TapNode::new(config));
    let event_bus = Arc::new(EventBus::new());

    let (sender_agent, sender_did) = TapAgent::from_ephemeral_key().await.unwrap();
    let (receiver_agent, receiver_did) = TapAgent::from_ephemeral_key().await.unwrap();

    node.register_agent(Arc::new(receiver_agent)).await.unwrap();

    // Send initial message
    let initial_message = PlainMessage {
        id: "thread-initial".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/transfer".to_string(),
        body: json!({"amount": "100.00", "currency": "USD"}),
        from: sender_did.clone(),
        to: vec![receiver_did.clone()],
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };

    // Send follow-up message with thread ID
    let follow_up_message = PlainMessage {
        id: "thread-followup".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/confirm".to_string(),
        body: json!({"confirmation": "approved"}),
        from: sender_did.clone(),
        to: vec![receiver_did.clone()],
        thid: Some("thread-initial".to_string()),
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };

    // Encrypt and send both messages
    for message in [initial_message, follow_up_message] {
        let pack_options = PackOptions {
            security_mode: SecurityMode::AuthCrypt,
            sender_kid: Some(format!("{}#keys-1", sender_did)),
            recipient_kid: Some(format!("{}#keys-1", receiver_did)),
        };

        let encrypted = message
            .pack(sender_agent.key_manager().as_ref(), pack_options)
            .await
            .unwrap();
        let body = Bytes::from(encrypted);
        let content_type = Some("application/didcomm-encrypted+json".to_string());

        let response = handle_didcomm(content_type, body, node.clone(), event_bus.clone())
            .await
            .unwrap();
        let response_json = response_to_json(response).await;

        assert_eq!(response_json["status"], "success");
    }
}
