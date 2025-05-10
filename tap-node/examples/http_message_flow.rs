use std::sync::Arc;
use serde_json::json;
use tap_agent::{AgentConfig, DefaultAgent};
use tap_agent::crypto::{DefaultMessagePacker, DebugSecretsResolver, BasicSecretResolver}; 
use tap_agent::did::MultiResolver;
use tap_node::{HttpMessageSender, NodeConfig, TapNode};

// Create a simple message
#[derive(serde::Serialize)]
struct SimpleMessage {
    id: String,
    type_: String,
    from: Option<String>,
    to: Option<Vec<String>>,
    body: serde_json::Value,
    created_time: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure and create a TAP node
    let config = NodeConfig::default();
    let node = TapNode::new(config);

    // Create resolvers and message packers
    let resolver = Arc::new(MultiResolver::default());
    let secrets = Arc::new(BasicSecretResolver::new());
    
    let alice_packer = Arc::new(DefaultMessagePacker::new(resolver.clone(), secrets.clone()));
    let bob_packer = Arc::new(DefaultMessagePacker::new(resolver.clone(), secrets.clone()));

    // Create two test agents
    let agent1_config = AgentConfig::new("did:example:alice".to_string());
    let agent1 = Arc::new(DefaultAgent::new(agent1_config, alice_packer));
    
    let agent2_config = AgentConfig::new("did:example:bob".to_string());
    let agent2 = Arc::new(DefaultAgent::new(agent2_config, bob_packer));

    // Register agents with the node
    node.register_agent(agent1).await?;
    node.register_agent(agent2).await?;

    // Create a test message
    let message = tap_msg::didcomm::Message {
        id: uuid::Uuid::new_v4().to_string(),
        typ: "https://tap.rsvp/schema/tap-message-v1".to_string(),
        type_: "".to_string(), // This field is required but unused
        from: Some("did:example:alice".to_string()),
        to: Some(vec!["did:example:bob".to_string()]),
        body: json!({
            "content": "Hello, Bob!",
            "timestamp": chrono::Utc::now().timestamp()
        }),
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        attachments: None,
        thid: None,
        pthid: None,
        from_prior: None,
        extra_headers: Default::default(),
    };

    // Send the message through the node
    let packed_message = node
        .send_message("did:example:alice", "did:example:bob", message)
        .await?;

    println!("Message packed successfully: {}", packed_message);

    // Create an HTTP message sender for external dispatch
    let sender = HttpMessageSender::with_options(
        "https://recipient-node.example.com".to_string(),
        5000,  // 5 second timeout
        2      // 2 retries
    );

    // In a real application, you would send this to the receiving node
    // For this example, we'll just log what would happen
    println!("Would send message to did:example:bob via HTTP");
    
    // This would actually send the message in a real environment
    // sender.send(packed_message, vec!["did:example:bob".to_string()]).await?;
    
    // For demonstration, let's show how to configure HTTP sender for different environments
    
    #[cfg(feature = "reqwest")]
    {
        println!("Using native HTTP implementation with reqwest");
    }
    
    #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
    {
        println!("Using WASM HTTP implementation with web-sys");
    }
    
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "reqwest")))]
    {
        println!("Using fallback implementation - no actual HTTP requests will be made");
    }
    
    #[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
    {
        println!("Using WASM fallback implementation - no actual HTTP requests will be made");
    }

    println!("Message flow completed successfully");
    Ok(())
}