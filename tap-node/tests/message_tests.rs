//! Tests for message handling in TAP Node

use std::sync::Arc;
use tap_agent::{AgentConfig, Agent, TapAgent};
use tap_core::message::{TapMessage, TapMessageBuilder, TapMessageType};
use tap_node::{NodeConfig, TapNode, Result};
use uuid;

/// Create a test agent with the given DID
fn create_test_agent(did: &str) -> Arc<dyn Agent> {
    let agent_config = AgentConfig::new_with_did(did);
    let agent = TapAgent::with_defaults(
        agent_config,
        did.to_string(),
        Some(format!("Test Agent {}", did)),
    )
    .unwrap();
    
    Arc::new(agent)
}

/// Create a test message
fn create_test_message(from_did: &str, to_did: &str) -> TapMessage {
    TapMessageBuilder::new()
        .id(uuid::Uuid::new_v4().to_string())
        .message_type(TapMessageType::Error)
        .from_did(Some(from_did.to_string()))
        .to_did(Some(to_did.to_string()))
        .body(serde_json::json!({
            "code": "TEST_ERROR",
            "message": "This is a test error message",
            "transaction_id": None::<String>,
            "metadata": {}
        }))
        .build()
        .unwrap()
}

#[tokio::test]
async fn test_message_routing() -> Result<()> {
    // Create a node with logging enabled for testing
    let config = NodeConfig {
        debug: true,
        max_agents: None,
        enable_message_logging: true,
        log_message_content: true,
        processor_pool: None,
    };
    let node = TapNode::new(config);

    // Create and register two test agents
    let agent1_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let agent2_did = "did:key:z6MkgYAFipwyqebCJagYs8XP6EPwXjiwLy8GZ6M1YyYAXMbh";
    
    let agent1 = create_test_agent(agent1_did);
    let agent2 = create_test_agent(agent2_did);
    
    node.register_agent(agent1).await?;
    node.register_agent(agent2).await?;
    
    // Create a test message
    let message = create_test_message(agent1_did, agent2_did);
    
    // Create a channel to receive events from the event bus
    let mut event_receiver = node.event_bus().subscribe_channel();
    
    // Send the message
    let packed_message = node.send_message(agent1_did, agent2_did, message.clone()).await?;
    
    // Verify that we received some packed message data
    assert!(!packed_message.is_empty());
    
    // Verify that we can receive an event about the message being sent
    let _event = tokio::time::timeout(std::time::Duration::from_secs(1), event_receiver.recv()).await
        .expect("Timed out waiting for event")
        .expect("Failed to receive event");
    
    // Note: In a more complete test we would verify the event contents
    // For now, just make sure we received something
    
    Ok(())
}

#[tokio::test]
async fn test_node_agent_communication() -> Result<()> {
    // Create a node
    let config = NodeConfig::default();
    let node = TapNode::new(config);

    // Create and register two test agents
    let agent1_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let agent2_did = "did:key:z6MkgYAFipwyqebCJagYs8XP6EPwXjiwLy8GZ6M1YyYAXMbh";
    
    let agent1 = create_test_agent(agent1_did);
    let agent2 = create_test_agent(agent2_did);
    
    node.register_agent(agent1).await?;
    node.register_agent(agent2).await?;
    
    // Create a test message going from agent1 to agent2
    let message = create_test_message(agent1_did, agent2_did);
    
    // Process the message - this should result in the message being routed to agent2
    node.process_message(message.clone()).await?;
    
    // Test that we can unregister agents
    node.unregister_agent(agent1_did).await?;
    assert_eq!(node.agents().agent_count(), 1);
    
    // Attempting to dispatch a message to an unregistered agent should fail
    let message2 = create_test_message(agent2_did, agent1_did);
    let result = node.dispatch_message(agent1_did, message2).await;
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_invalid_message_handling() -> Result<()> {
    // Create a node
    let config = NodeConfig::default();
    let node = TapNode::new(config);

    // Create and register an agent
    let agent_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let agent = create_test_agent(agent_did);
    node.register_agent(agent).await?;
    
    // Create a test message with missing to field (invalid)
    let mut message = create_test_message(agent_did, "did:example:invalid");
    message.to_did = None;
    
    // Process the message - this should be filtered out by validation
    let result = node.process_message(message).await;
    
    // The operation should complete without error (message was filtered)
    assert!(result.is_ok());
    
    Ok(())
}
