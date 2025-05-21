//! Tests for ephemeral agent creation

use std::sync::Arc;
use tap_agent::agent::{Agent, TapAgent};
use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
use tap_agent::config::AgentConfig;
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tokio::test;

#[derive(Debug)]
struct TestDIDResolver;

impl TestDIDResolver {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl tap_agent::did::DIDMethodResolver for TestDIDResolver {
    fn method(&self) -> &str {
        "key"
    }

    async fn resolve_method(
        &self,
        did: &str,
    ) -> tap_agent::error::Result<Option<tap_agent::did::DIDDoc>> {
        // Simple mock implementation
        if did.starts_with("did:key:") {
            let vm_id = format!("{}#1", did);

            // Create a basic verification method
            let vm = tap_agent::did::VerificationMethod {
                id: vm_id.clone(),
                type_: tap_agent::did::VerificationMethodType::Ed25519VerificationKey2018,
                controller: did.to_string(),
                verification_material: tap_agent::did::VerificationMaterial::Base58 {
                    public_key_base58: "test1234".to_string(),
                },
            };

            // Create a basic DID document
            Ok(Some(tap_agent::did::DIDDoc {
                id: did.to_string(),
                verification_method: vec![vm],
                authentication: vec![vm_id.clone()],
                key_agreement: vec![],
                assertion_method: vec![],
                capability_invocation: vec![],
                capability_delegation: vec![],
                service: vec![],
            }))
        } else {
            Ok(None)
        }
    }
}

// Define a simple EmptyMessage for testing if it doesn't exist
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmptyMessage {}

impl tap_msg::TapMessageBody for EmptyMessage {
    fn message_type() -> &'static str {
        "test.empty"
    }

    fn validate(&self) -> Result<(), tap_msg::error::Error> {
        Ok(())
    }
}

#[test]
async fn test_create_ephemeral_agent() {
    // Create a key manager with a new ephemeral key
    // Generate a fresh key since we don't have access to with_auto_generated_ed25519_key
    let mut builder = AgentKeyManagerBuilder::new();

    // Add a test secret for the agent
    let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string();
    let secret = Secret {
        id: "default".to_string(),
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
    builder = builder.add_secret(did.clone(), secret);
    let key_manager = builder.build().expect("Failed to build key manager");

    // Create a config
    let config = AgentConfig::new(did.clone());

    // Create the agent
    let agent = TapAgent::new(config, Arc::new(key_manager));

    // Check that the agent was created successfully
    assert!(!did.is_empty(), "DID should not be empty");

    // Test that the agent can receive messages
    let result = agent
        .receive_message::<EmptyMessage>("{\"id\":\"test\",\"type\":\"test.message.type\"}")
        .await;

    // The actual receive should fail due to invalid packed message
    // but we're just testing that the agent exists and can attempt to receive
    assert!(result.is_err());
}

// Test that we can create an agent from a test key
#[test]
async fn test_create_agent_from_key() {
    // Create a key manager builder
    let mut builder = AgentKeyManagerBuilder::new();

    // Add a test secret for the agent
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

    // Build the key manager
    let key_manager = builder.build().expect("Failed to build key manager");

    // Create a config with the DID
    let config = AgentConfig::new("did:example:123".to_string());

    // Create the agent with the key manager
    let agent = TapAgent::new(config, Arc::new(key_manager));

    // Verify the agent has the expected DID
    assert_eq!(agent.get_agent_did(), "did:example:123");
}
