//! WebSocket message flow example
//!
//! This example demonstrates how to use the WebSocketMessageSender
//! to establish persistent connections for real-time messaging.
//!
//! Run with: cargo run --example websocket_message_flow --features websocket

use serde_json::json;
use std::sync::Arc;
use tap_agent::crypto::BasicSecretResolver;
use tap_agent::did::MultiResolver;
use tap_agent::{AgentConfig, DefaultAgent, DefaultMessagePacker};
use tap_msg::didcomm::Message;
use tap_node::{MessageSender, NodeConfig, TapNode, WebSocketMessageSender};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    env_logger::init();

    println!("WebSocket Message Flow Example");
    println!("-----------------------------");
    println!("This example demonstrates using WebSockets for real-time messaging.");

    // Create a TAP node
    let node = TapNode::new(NodeConfig::default());

    // Create sender and recipient DIDs
    let sender_did = "did:example:sender".to_string();
    let recipient_did = "did:example:recipient".to_string();

    // Create agent configurations
    let sender_config = AgentConfig::new(sender_did.clone());
    let recipient_config = AgentConfig::new(recipient_did.clone());

    // Create resolvers
    let did_resolver = Arc::new(MultiResolver::default());
    let secrets_resolver = Arc::new(BasicSecretResolver::new());

    // Create message packers
    let message_packer = Arc::new(DefaultMessagePacker::new(
        did_resolver.clone(),
        secrets_resolver.clone(),
    ));

    // Create agents
    let sender_agent = Arc::new(DefaultAgent::new(sender_config, message_packer.clone()));
    let recipient_agent = Arc::new(DefaultAgent::new(recipient_config, message_packer.clone()));

    // Register agents with the node
    node.register_agent(sender_agent).await?;
    node.register_agent(recipient_agent).await?;

    // Create a sample message
    let message = Message {
        id: "test-123".to_string(),
        typ: "https://tap.rsvp/schema/1.0#transfer".to_string(),
        type_: "https://tap.rsvp/schema/1.0#transfer".to_string(),
        body: json!({
            "amount": "100.00",
            "asset": "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
            "transaction_id": "tx-123456",
            "memo": "Test WebSocket transfer"
        }),
        from: Some(sender_did.clone()),
        to: Some(vec![recipient_did.clone()]),
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        thid: Some("thread-123".to_string()),
        pthid: None,
        attachments: None,
        from_prior: None,
        extra_headers: Default::default(),
    };

    // Pack the message for sending
    println!("Packing message...");
    let packed_message = node
        .send_message(&sender_did, &recipient_did, message)
        .await?;

    // Create a WebSocket message sender with options
    let websocket_sender = WebSocketMessageSender::with_options(
        "https://api.example.com".to_string(), // Your actual WebSocket server URL
    );

    println!("Sending message via WebSocket...");

    // In a real scenario, this would connect to an actual WebSocket server
    // For demonstration, we'll catch the expected error
    match websocket_sender
        .send(packed_message, vec![recipient_did.clone()])
        .await
    {
        Ok(_) => println!("Message sent successfully (unlikely in this example)"),
        Err(e) => println!("Expected error (no actual WebSocket server): {}", e),
    }

    println!("\nIn a real application:");
    println!("1. The WebSocket connection would be established to a real endpoint");
    println!("2. The connection would remain open for bidirectional communication");
    println!("3. Multiple messages could be sent over the same connection");
    println!("4. Incoming messages would be handled by the WebSocket task");

    Ok(())
}
