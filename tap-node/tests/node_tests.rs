//! Tests for TAP Node

use std::sync::Arc;
use tap_agent::{AgentConfig, TapAgent};
use tap_node::{NodeConfig, TapNode};

#[tokio::test]
async fn test_node_creation() {
    // Create a basic node
    let config = NodeConfig::default();
    let node = TapNode::new(config);

    // Check node properties
    assert!(!node.config().debug);
    assert_eq!(node.agents().agent_count(), 0);
}

#[tokio::test]
async fn test_agent_registration() {
    // Create a node
    let config = NodeConfig::default();
    let node = TapNode::new(config);

    // Create an agent
    let agent_config =
        AgentConfig::new_with_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

    let agent = TapAgent::with_defaults(
        agent_config,
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        Some("Test Agent".to_string()),
    )
    .unwrap();

    // Register the agent with the node
    node.register_agent(Arc::new(agent)).await.unwrap();

    // Check that the agent is registered
    assert_eq!(node.agents().agent_count(), 1);
    assert!(node
        .agents()
        .has_agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"));
}

#[tokio::test]
async fn test_agent_unregistration() {
    // Create a node
    let config = NodeConfig::default();
    let node = TapNode::new(config);

    // Create and register an agent
    let agent_config =
        AgentConfig::new_with_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

    let agent = TapAgent::with_defaults(
        agent_config,
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        Some("Test Agent".to_string()),
    )
    .unwrap();

    node.register_agent(Arc::new(agent)).await.unwrap();
    assert_eq!(node.agents().agent_count(), 1);

    // Unregister the agent
    node.unregister_agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
        .await
        .unwrap();

    // Check that the agent is no longer registered
    assert_eq!(node.agents().agent_count(), 0);
    assert!(!node
        .agents()
        .has_agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"));
}

#[tokio::test]
async fn test_node_config() {
    // Create a node with custom config
    let config = NodeConfig {
        debug: true,
        max_agents: Some(10),
        enable_message_logging: true,
        log_message_content: true,
        processor_pool: None,
    };

    let node = TapNode::new(config);

    // Verify config values
    assert!(node.config().debug);
    assert_eq!(node.config().max_agents, Some(10));
    assert!(node.config().enable_message_logging);
    assert!(node.config().log_message_content);
}
