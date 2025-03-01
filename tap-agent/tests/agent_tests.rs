//! Tests for TAP Agent

use tap_agent::{Agent, AgentConfig, TapAgent};
use tap_core::message::TapMessageType;

#[tokio::test]
async fn test_agent_creation() {
    // Create a basic agent with did:key method
    let config = AgentConfig::new()
        .with_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
        .with_name("Test Agent")
        .with_endpoint("https://example.com/endpoint");

    let agent_result = TapAgent::with_defaults(
        config,
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        Some("Test Agent".to_string()),
    );

    assert!(agent_result.is_ok());

    let agent = agent_result.unwrap();
    assert_eq!(
        agent.did(),
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
    );
    assert_eq!(agent.name(), Some("Test Agent"));
    assert_eq!(
        agent.config().endpoint,
        Some("https://example.com/endpoint".to_string())
    );
}

#[tokio::test]
async fn test_message_creation() {
    // Create a basic agent
    let config =
        AgentConfig::new_with_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

    let agent = TapAgent::with_defaults(
        config,
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        None,
    )
    .unwrap();

    // Create a simple message
    let message = agent
        .create_message(
            TapMessageType::Custom("test".into()),
            Some(serde_json::json!({
                "test": "value",
                "num": 123
            })),
        )
        .await
        .unwrap();

    // Check message properties
    assert_eq!(message.message_type, TapMessageType::Custom("test".into()));
    assert!(message.body.is_some());
    assert!(!message.id.is_empty());
    assert!(!message.created_time.is_empty());
}
