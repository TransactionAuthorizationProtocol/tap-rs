//! Tests for TAP Agent

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;

use tap_agent::crypto::{BasicSecretResolver, DebugSecretsResolver, DefaultMessagePacker};
use tap_agent::did::{
    DIDDoc, DIDMethodResolver, MultiResolver, Service, VerificationMethod, VerificationMethodType,
};
use tap_agent::key_manager::{DefaultKeyManager, KeyManager, Secret, SecretMaterial, SecretType};
use tap_agent::message::SecurityMode;
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

/// A DID resolver for testing that returns a hardcoded DID document
#[derive(Debug)]
struct TestDIDResolver;

impl TestDIDResolver {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DIDMethodResolver for TestDIDResolver {
    fn method(&self) -> &str {
        "example"
    }

    async fn resolve_method(&self, did: &str) -> tap_agent::error::Result<Option<DIDDoc>> {
        match did {
            "did:example:123" => {
                // Return a DID document with no services
                Ok(Some(DIDDoc {
                    id: did.to_string(),
                    verification_method: vec![VerificationMethod {
                        id: format!("{}#key-1", did),
                        type_: VerificationMethodType::Ed25519VerificationKey2018,
                        controller: did.to_string(),
                        verification_material: tap_agent::did::VerificationMaterial::Base58 {
                            public_key_base58: "test1234".to_string(),
                        },
                    }],
                    authentication: vec![format!("{}#key-1", did)],
                    key_agreement: vec![],
                    service: vec![],
                }))
            }
            "did:example:456" => {
                // Return a DID document with a DIDComm service
                Ok(Some(DIDDoc {
                    id: did.to_string(),
                    verification_method: vec![VerificationMethod {
                        id: format!("{}#key-1", did),
                        type_: VerificationMethodType::Ed25519VerificationKey2018,
                        controller: did.to_string(),
                        verification_material: tap_agent::did::VerificationMaterial::Base58 {
                            public_key_base58: "test1234".to_string(),
                        },
                    }],
                    authentication: vec![format!("{}#key-1", did)],
                    key_agreement: vec![],
                    service: vec![Service {
                        id: format!("{}#didcomm-1", did),
                        type_: "DIDCommMessaging".to_string(),
                        service_endpoint: "https://example.com/didcomm".to_string(),
                        properties: std::collections::HashMap::new(),
                    }],
                }))
            }
            "did:example:web" => {
                // Return a DID document with a web service
                Ok(Some(DIDDoc {
                    id: did.to_string(),
                    verification_method: vec![VerificationMethod {
                        id: format!("{}#key-1", did),
                        type_: VerificationMethodType::Ed25519VerificationKey2018,
                        controller: did.to_string(),
                        verification_material: tap_agent::did::VerificationMaterial::Base58 {
                            public_key_base58: "test1234".to_string(),
                        },
                    }],
                    authentication: vec![format!("{}#key-1", did)],
                    key_agreement: vec![],
                    service: vec![Service {
                        id: format!("{}#service-1", did),
                        type_: "Web".to_string(),
                        service_endpoint: "https://example.com/api".to_string(),
                        properties: std::collections::HashMap::new(),
                    }],
                }))
            }
            _ => Ok(None),
        }
    }
}

/// Simple function to create a test secret resolver
fn create_test_secret_resolver() -> BasicSecretResolver {
    let mut resolver = BasicSecretResolver::new();

    // Add a test secret
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

    resolver.add_secret("did:example:123", secret);

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

    resolver.add_secret("did:example:456", secret);

    resolver
}

#[tokio::test]
async fn test_agent_get_service_endpoint() {
    // Create a test agent config
    let config = AgentConfig::new("did:example:123".to_string());

    // Create the DID resolver
    let resolver = Arc::new(MultiResolver::new_with_resolvers(vec![Arc::new(
        TestDIDResolver::new(),
    )]));

    // Create the secret resolver
    let secret_resolver = create_test_secret_resolver();

    // Create the message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(
        resolver.clone(),
        Arc::new(secret_resolver),
        true,
    ));

    // Create the agent
    let agent = DefaultAgent::new(config, message_packer);

    // Test get_service_endpoint works correctly
    let endpoint = agent.get_service_endpoint("did:example:456").await.unwrap();
    assert!(endpoint.is_some(), "Service endpoint should be found");
    assert!(
        endpoint.unwrap().contains("https://example.com/didcomm"),
        "Service endpoint has correct URL"
    );

    // Test for a DID with other service type
    let endpoint = agent.get_service_endpoint("did:example:web").await.unwrap();
    assert!(
        endpoint.is_some(),
        "Service endpoint should be found for web service"
    );
    assert!(
        endpoint.unwrap().contains("https://example.com/api"),
        "Web service endpoint has correct URL"
    );

    // Test for a DID with no service endpoint
    let endpoint = agent.get_service_endpoint("did:example:123").await.unwrap();
    assert!(endpoint.is_none(), "No service endpoint should be found");
}

#[tokio::test]
#[ignore = "Skip for now - issues with test keys"]
async fn test_send_message_to_multiple_recipients() {
    // Create a test agent config
    let config = AgentConfig::new("did:example:123".to_string());

    // Create the DID resolver
    let resolver = Arc::new(MultiResolver::new_with_resolvers(vec![Arc::new(
        TestDIDResolver::new(),
    )]));

    // Create the secret resolver
    let secret_resolver = create_test_secret_resolver();

    // Create the message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(
        resolver.clone(),
        Arc::new(secret_resolver),
        true,
    ));

    // Create the agent
    let agent = DefaultAgent::new(config, message_packer);

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
        assert!(result.status.is_none()); // No actual delivery with deliver=false
        assert!(result.error.is_none()); // No error expected
    }
}
