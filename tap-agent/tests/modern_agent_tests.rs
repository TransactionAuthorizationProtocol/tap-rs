use tap_agent::{
    agent::{Agent, AgentConfig, DeliveryResult, ModernAgent, ModernAgentBuilder},
    error::{Error, Result},
    key_manager::{DefaultKeyManager, KeyManagerBuilder},
    message_packing::{PackOptions, Packable, UnpackOptions, Unpackable},
};

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tap_msg::message::{
    participant::Participant, tap_message_trait::TapMessageBody, transfer::Transfer,
};

// Mock message type for testing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TestMessage {
    message_type: String,
    from: String,
    to: Vec<String>,
    content: String,
}

impl TapMessageBody for TestMessage {
    fn get_type(&self) -> &str {
        &self.message_type
    }
}

// Helper to create a basic agent config for testing
fn create_test_config() -> AgentConfig {
    AgentConfig {
        service_endpoint: "https://example.com/endpoint".to_string(),
        agent_id: "test-agent".to_string(),
        service_endpoint_auth: None,
    }
}

#[tokio::test]
async fn test_modern_agent_creation() -> Result<()> {
    // Create a key manager
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("default")
        .build()
        .await?;

    // Create a modern agent
    let agent = ModernAgentBuilder::new()
        .with_config(create_test_config())
        .with_key_manager(Arc::new(key_manager))
        .build()
        .await?;

    // Test agent properties
    assert_eq!(agent.config().agent_id, "test-agent");
    assert_eq!(
        agent.config().service_endpoint,
        "https://example.com/endpoint"
    );

    Ok(())
}

#[tokio::test]
async fn test_message_preparation() -> Result<()> {
    // Create a key manager
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("default")
        .build()
        .await?;

    // Get the key's DID to use as our agent DID
    let key = key_manager.get_signing_key("default").await?;
    let agent_did = key.did().to_string();

    // Create agent config with our DID
    let mut config = create_test_config();
    config.agent_id = agent_did.clone();

    // Create a modern agent
    let agent = ModernAgentBuilder::new()
        .with_config(config)
        .with_key_manager(Arc::new(key_manager))
        .build()
        .await?;

    // Create a test message
    let message = TestMessage {
        message_type: "test/message".to_string(),
        from: agent_did.clone(),
        to: vec!["did:example:recipient".to_string()],
        content: "Test content".to_string(),
    };

    // Test prepare_message (this doesn't actually deliver)
    let (message_id, packed_message) = agent.prepare_message(&message).await?;

    // Verify the message ID format (UUID)
    assert!(message_id.len() > 10); // Simple check for a non-empty ID

    // Packed message should be a JSON string
    assert!(packed_message.starts_with("{"));
    assert!(packed_message.contains("test/message"));

    Ok(())
}

#[tokio::test]
async fn test_transfer_message_creation() -> Result<()> {
    // Create a key manager
    let key_manager = KeyManagerBuilder::new()
        .with_auto_generated_ed25519_key("default")
        .build()
        .await?;

    // Get the key's DID to use as our agent DID
    let key = key_manager.get_signing_key("default").await?;
    let agent_did = key.did().to_string();

    // Create agent config with our DID
    let mut config = create_test_config();
    config.agent_id = agent_did.clone();

    // Create a modern agent
    let agent = ModernAgentBuilder::new()
        .with_config(config)
        .with_key_manager(Arc::new(key_manager))
        .build()
        .await?;

    // Create a simple transfer message
    let originator = Participant {
        id: agent_did.clone(),
        name: Some("Originator".to_string()),
        ..Default::default()
    };

    let beneficiary = Participant {
        id: "did:example:beneficiary".to_string(),
        name: Some("Beneficiary".to_string()),
        ..Default::default()
    };

    let transfer = Transfer::builder()
        .with_id("test-transfer-123".to_string())
        .with_originator(originator)
        .with_beneficiary(beneficiary)
        .with_asset_id("ethereum:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f".to_string())
        .with_amount("100.0".to_string())
        .build()?;

    // Test prepare_message with this more complex type
    let (message_id, packed_message) = agent.prepare_message(&transfer).await?;

    // Verify the message has expected content
    assert!(packed_message.contains("test-transfer-123"));
    assert!(packed_message.contains("Originator"));
    assert!(packed_message.contains("Beneficiary"));

    Ok(())
}
