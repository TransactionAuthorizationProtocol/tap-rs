//! Tests for ephemeral agent creation

use std::sync::Arc;
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::crypto::{BasicSecretResolver, DefaultMessagePacker};
use tap_agent::did::{DIDGenerationOptions, KeyType, MultiResolver};
use tap_agent::key_manager::{DefaultKeyManager, KeyManager, Secret, SecretMaterial, SecretType};
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
    // Test that we can create an ephemeral agent with a new key
    let (agent, did) = tap_agent::agent::DefaultAgent::new_ephemeral().unwrap();

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
    // Create a key manager
    let key_manager = DefaultKeyManager::new();

    // Generate a key
    let key = key_manager
        .generate_key(DIDGenerationOptions {
            key_type: KeyType::Ed25519,
        })
        .unwrap();

    // Create a DID resolver
    let did_resolver = Arc::new(MultiResolver::new_with_resolvers(vec![Arc::new(
        TestDIDResolver::new(),
    )]));

    // Create a secret resolver
    let mut secret_resolver = BasicSecretResolver::new();

    // Add the key as a secret
    let secret = Secret {
        id: key.did.clone(),
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

    secret_resolver.add_secret(&key.did, secret);

    // Create a message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(
        did_resolver,
        Arc::new(secret_resolver),
        true,
    ));

    // Create agent configuration
    let config = tap_agent::config::AgentConfig {
        agent_did: key.did.clone(),
        parameters: std::collections::HashMap::new(),
        security_mode: Some("SIGNED".to_string()),
        debug: true,
        timeout_seconds: Some(30),
    };

    // Create the agent
    let agent = DefaultAgent::new(config, message_packer);

    // Verify the agent has the expected DID
    assert_eq!(agent.get_agent_did(), key.did);
}
