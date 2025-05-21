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

#[tokio::test]
async fn test_from_private_key_ed25519() {
    use chrono::Utc;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use tap_agent::KeyType;
    use tap_msg::message::Presentation;
    use uuid::Uuid;

    // Generate a random Ed25519 key
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let private_key_bytes = signing_key.to_bytes();

    // Create a TapAgent from the private key
    let (agent, did) = TapAgent::from_private_key(&private_key_bytes, KeyType::Ed25519, true)
        .await
        .unwrap();

    // Verify the agent was created correctly
    assert!(did.starts_with("did:key:z"), "DID should be a did:key");
    assert_eq!(agent.get_agent_did(), &did);

    // Create a simple presentation message to test with
    let presentation = Presentation::builder()
        .presentation_id(Uuid::new_v4().to_string())
        .thread_id(Uuid::new_v4().to_string())
        .created_time(Utc::now())
        .from(did.clone())
        .to(vec!["did:example:123".to_string()])
        .build()
        .unwrap();

    // Pack the message (no delivery)
    let (packed, _) = agent
        .send_message(&presentation, vec!["did:example:123"], false)
        .await
        .unwrap();

    // Verify that the packed message is not empty
    assert!(!packed.is_empty(), "Packed message should not be empty");

    // Create another agent with the same private key
    let (agent2, did2) = TapAgent::from_private_key(&private_key_bytes, KeyType::Ed25519, false)
        .await
        .unwrap();

    // Verify that both agents have the same DID
    assert_eq!(
        did, did2,
        "DIDs should be identical for the same private key"
    );

    // Verify that the second agent can unpack the message packed by the first
    let received_presentation: Presentation = agent2.receive_message(&packed).await.unwrap();

    // Verify the received message is the same as the sent message
    assert_eq!(
        presentation.presentation_id,
        received_presentation.presentation_id
    );
}

#[tokio::test]
async fn test_from_private_key_p256() {
    use chrono::Utc;
    use p256::ecdsa::SigningKey as P256SigningKey;
    use rand::rngs::OsRng;
    use tap_agent::KeyType;
    use tap_msg::message::Presentation;
    use uuid::Uuid;

    // Generate a random P-256 key
    let mut rng = OsRng;
    let signing_key = P256SigningKey::random(&mut rng);
    let private_key_bytes = signing_key.to_bytes().to_vec();

    // Create a TapAgent from the private key
    let (agent, did) = TapAgent::from_private_key(&private_key_bytes, KeyType::P256, true)
        .await
        .unwrap();

    // Verify the agent was created correctly
    assert!(did.starts_with("did:key:z"), "DID should be a did:key");
    assert_eq!(agent.get_agent_did(), &did);

    // Create a simple presentation message to test with
    let presentation = Presentation::builder()
        .presentation_id(Uuid::new_v4().to_string())
        .thread_id(Uuid::new_v4().to_string())
        .created_time(Utc::now())
        .from(did.clone())
        .to(vec!["did:example:123".to_string()])
        .build()
        .unwrap();

    // Pack the message (no delivery)
    let (packed, _) = agent
        .send_message(&presentation, vec!["did:example:123"], false)
        .await
        .unwrap();

    // Verify that the packed message is not empty
    assert!(!packed.is_empty(), "Packed message should not be empty");

    // Create another agent with the same private key
    let (agent2, did2) = TapAgent::from_private_key(&private_key_bytes, KeyType::P256, false)
        .await
        .unwrap();

    // Verify that both agents have the same DID
    assert_eq!(
        did, did2,
        "DIDs should be identical for the same private key"
    );

    // Verify that the second agent can unpack the message packed by the first
    let received_presentation: Presentation = agent2.receive_message(&packed).await.unwrap();

    // Verify the received message is the same as the sent message
    assert_eq!(
        presentation.presentation_id,
        received_presentation.presentation_id
    );
}
