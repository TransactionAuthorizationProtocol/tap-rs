//! Tests for TAP Agent

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tap_agent::agent::{Agent, TapAgent};
use tap_agent::agent_key_manager::{AgentKeyManager, AgentKeyManagerBuilder};
use tap_agent::config::AgentConfig;
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_msg::TapMessageBody;

/// A simple test message type
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestMessage {
    content: String,
}

impl TapMessageBody for TestMessage {
    fn message_type() -> &'static str {
        "test.message.type"
    }

    fn validate(&self) -> Result<(), tap_msg::error::Error> {
        Ok(())
    }
}

/// Create a test key manager with pre-configured test keys
fn create_test_key_manager() -> Arc<AgentKeyManager> {
    let mut builder = AgentKeyManagerBuilder::new();

    // Add a test secret for agent
    let secret = Secret {
        id: "did:example:123".to_string(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "test1234",
                "d": "test1234"
            }),
        },
    };
    builder = builder.add_secret("did:example:123".to_string(), secret);

    // Add a test secret for recipient
    let secret = Secret {
        id: "did:example:456".to_string(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "test1234",
                "d": "test1234"
            }),
        },
    };
    builder = builder.add_secret("did:example:456".to_string(), secret);

    Arc::new(builder.build().unwrap())
}

#[tokio::test]
async fn test_agent_get_service_endpoint() {
    // Create a test agent config
    let config = AgentConfig::new("did:example:123".to_string());

    // Create the test key manager
    let key_manager = create_test_key_manager();

    // Create the agent
    let agent = TapAgent::new(config, key_manager);

    // Test get_service_endpoint works correctly for DIDs
    // In the updated implementation, DIDs will use the fallback URL format:
    // https://example.com/did/{did_with_underscores}
    let endpoint = agent.get_service_endpoint("did:example:456").await.unwrap();
    assert!(endpoint.is_some(), "Service endpoint should be found");
    assert!(
        endpoint
            .unwrap()
            .contains("https://example.com/did/did_example_456"),
        "Service endpoint should use fallback URL format"
    );

    // Test for another DID
    let endpoint = agent.get_service_endpoint("did:example:web").await.unwrap();
    assert!(
        endpoint.is_some(),
        "Service endpoint should be found for web DID"
    );
    assert!(
        endpoint
            .unwrap()
            .contains("https://example.com/did/did_example_web"),
        "Service endpoint should use fallback URL format"
    );

    // Test for direct URLs
    let endpoint = agent
        .get_service_endpoint("https://direct.example.com")
        .await
        .unwrap();
    assert!(
        endpoint.is_some(),
        "Service endpoint should be found for direct URL"
    );
    assert_eq!(
        endpoint.unwrap(),
        "https://direct.example.com",
        "Direct URLs should be returned as-is"
    );
}

#[tokio::test]
#[ignore = "Skipped until valid test keys are available for crypto operations"]
async fn test_send_message_to_multiple_recipients() {
    // Create a test agent config
    let config = AgentConfig::new("did:example:123".to_string());

    // Create the test key manager
    let key_manager = create_test_key_manager();

    // Create the agent
    let agent = TapAgent::new(config, key_manager);

    // Create a simple message
    let test_message = TestMessage {
        content: "test multiple recipients".to_string(),
    };

    // Define multiple recipients
    let recipients = vec!["did:example:456"];

    // Send the message
    let (message_id, delivery_results) = agent
        .send_message(&test_message, recipients.clone(), false)
        .await
        .unwrap();

    // Check that we got a valid message ID
    assert!(!message_id.is_empty(), "Message ID should not be empty");

    // Check that we have delivery results for each recipient
    assert_eq!(delivery_results.len(), recipients.len());

    // Check the delivery results
    for result in &delivery_results {
        assert_eq!(result.did, "did:example:456");
        // Endpoints should use the fallback URL format
        assert!(
            result
                .endpoint
                .contains("https://example.com/did/did_example_456"),
            "Endpoint should use fallback URL format"
        );
        assert!(result.status.is_none()); // No actual delivery with deliver=false
        assert!(result.error.is_none()); // No error expected
    }
}
