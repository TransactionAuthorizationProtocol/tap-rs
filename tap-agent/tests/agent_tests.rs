//! Tests for TAP Agent

use async_trait::async_trait;
use didcomm::did::DIDDoc;
use didcomm::secrets::{Secret, SecretType, SecretMaterial};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{BasicSecretResolver, DefaultMessagePacker, DebugSecretsResolver};
use tap_agent::did::{MultiResolver, SyncDIDResolver, DIDMethodResolver};
use tap_agent::error::{Error, Result};
use tap_core::error::{Error as TapCoreError, Result as TapCoreResult};
use tap_core::message::tap_message_trait::TapMessageBody;
use uuid::Uuid;

// Test message for agent tests
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestMessage {
    pub content: String,
}

impl TapMessageBody for TestMessage {
    fn message_type() -> &'static str {
        "TAP_TEST"
    }

    fn from_didcomm(msg: &didcomm::Message) -> TapCoreResult<Self> {
        // First try to get content directly from message body
        if let Some(content) = msg.body.get("content") {
            if let Some(content_str) = content.as_str() {
                return Ok(Self {
                    content: content_str.to_string(),
                });
            }
        }
        
        // If we get here, we couldn't find a content field that's a string
        // Try to extract as a serde_json value
        Ok(Self {
            content: "".to_string(),
        })
    }

    fn validate(&self) -> TapCoreResult<()> {
        Ok(())
    }

    fn to_didcomm(&self) -> TapCoreResult<didcomm::Message> {
        // Create a new DIDComm message
        let msg = didcomm::Message {
            id: Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: serde_json::to_value(self).map_err(|e| TapCoreError::SerializationError(e.to_string()))?,
            from: None,
            to: None,
            thid: None,
            pthid: None,
            extra_headers: Default::default(),
            created_time: None,
            expires_time: None,
            from_prior: None,
            attachments: None,
        };
        
        Ok(msg)
    }
}

// Create a Presentation message for testing AuthCrypt mode
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PresentationMessage {
    pub presentation_id: String,
    pub data: String,
}

impl TapMessageBody for PresentationMessage {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#Presentation"
    }

    fn from_didcomm(msg: &didcomm::Message) -> TapCoreResult<Self> {
        // First try to get fields directly from message body
        let presentation_id = msg.body.get("presentation_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
            
        let data = msg.body.get("data")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        Ok(Self {
            presentation_id,
            data,
        })
    }

    fn validate(&self) -> TapCoreResult<()> {
        Ok(())
    }

    fn to_didcomm(&self) -> TapCoreResult<didcomm::Message> {
        // Create a new DIDComm message
        let msg = didcomm::Message {
            id: Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: serde_json::to_value(self).map_err(|e| TapCoreError::SerializationError(e.to_string()))?,
            from: None,
            to: None,
            thid: None,
            pthid: None,
            extra_headers: Default::default(),
            created_time: None,
            expires_time: None,
            from_prior: None,
            attachments: None,
        };
        
        Ok(msg)
    }
}

// Create a test DID resolver
#[derive(Debug)]
struct TestDIDResolver;

impl TestDIDResolver {
    fn new() -> Self {
        TestDIDResolver
    }
}

#[async_trait]
impl DIDMethodResolver for TestDIDResolver {
    fn method(&self) -> &str {
        "example"
    }
    
    async fn resolve_method(&self, did: &str) -> Result<Option<DIDDoc>> {
        if !did.starts_with("did:example:") {
            return Err(Error::UnsupportedDIDMethod(format!("Unsupported DID method for test resolver: {}", did)));
        }
        
        // Create a test DID document
        let id = format!("{}#keys-1", did);
        
        let auth_method = didcomm::did::VerificationMethod {
            id: id.clone(),
            type_: didcomm::did::VerificationMethodType::Ed25519VerificationKey2018,
            controller: did.to_string(),
            verification_material: didcomm::did::VerificationMaterial::Base58 {
                public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
            },
        };
        
        let doc = DIDDoc {
            id: did.to_string(),
            verification_method: vec![auth_method.clone()],
            authentication: vec![id.clone()],
            key_agreement: vec![id],
            service: vec![],
        };
        
        Ok(Some(doc))
    }
}

#[async_trait]
impl SyncDIDResolver for TestDIDResolver {
    async fn resolve(&self, did: &str) -> Result<Option<DIDDoc>> {
        self.resolve_method(did).await
    }
}

// Create a test secret resolver with a test key
fn create_test_secret_resolver() -> Arc<dyn DebugSecretsResolver> {
    let mut resolver = BasicSecretResolver::new();
    
    // Create a test key for the sender
    let test_key = Secret {
        id: "did:example:123#keys-1".to_string(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "kid": "did:example:123#keys-1",
                "crv": "Ed25519",
                "x": "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo",
                "d": "nWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Yh"
            })
        },
    };
    
    resolver.add_secret("did:example:123", test_key);
    
    // Add a test key for the recipient as well
    let recipient_key = Secret {
        id: "did:example:456#keys-1".to_string(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "kid": "did:example:456#keys-1",
                "crv": "Ed25519",
                "x": "12qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo",
                "d": "oWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Yh"
            })
        },
    };
    
    resolver.add_secret("did:example:456", recipient_key);
    
    // Return the resolver directly as it implements DebugSecretsResolver
    Arc::new(resolver)
}

#[tokio::test]
async fn test_agent_creation() {
    // Create a test agent
    let config = AgentConfig::new("did:example:123".to_string());

    // Create the DID resolver
    let resolver = Arc::new(MultiResolver::new_with_resolvers(vec![Arc::new(TestDIDResolver::new())]));
    
    // Create the secret resolver
    let secret_resolver = create_test_secret_resolver();

    // Create the message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(resolver.clone(), secret_resolver));

    // Create the agent
    let agent = DefaultAgent::new(config, message_packer);

    // Check that the agent was created successfully
    assert_eq!(agent.get_agent_did(), "did:example:123");
}

#[tokio::test]
async fn test_send_receive_message() {
    // Create a test agent
    let config = AgentConfig::new("did:example:123".to_string());

    // Create the DID resolver
    let resolver = Arc::new(MultiResolver::new_with_resolvers(vec![Arc::new(TestDIDResolver::new())]));
    
    // Create the secret resolver
    let secret_resolver = create_test_secret_resolver();

    // Create the message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(resolver.clone(), secret_resolver));

    // Create the agent
    let agent = DefaultAgent::new(config, message_packer);

    // Create a simple message
    let test_message = TestMessage {
        content: "value".to_string(),
    };

    // Pack the message for sending - this should use Signed mode automatically
    let packed = agent
        .send_message(&test_message, "did:example:456")
        .await
        .unwrap();

    // The agent should be able to decode its own messages
    let received: TestMessage = agent.receive_message(&packed).await.unwrap();
    
    // Check that the message was received correctly
    assert_eq!(received.content, "value");
}

#[tokio::test]
async fn test_presentation_message() {
    // Create a test agent
    let config = AgentConfig::new("did:example:123".to_string());

    // Create the DID resolver
    let resolver = Arc::new(MultiResolver::new_with_resolvers(vec![Arc::new(TestDIDResolver::new())]));
    
    // Create the secret resolver
    let secret_resolver = create_test_secret_resolver();

    // Create the message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(resolver.clone(), secret_resolver));

    // Create the agent
    let agent = DefaultAgent::new(config, message_packer);

    // Create a presentation message that should use AuthCrypt
    let presentation = PresentationMessage {
        presentation_id: "test123".to_string(),
        data: "secure-data".to_string(),
    };

    // Pack the message for sending - this should use AuthCrypt mode automatically
    let packed = agent
        .send_message(&presentation, "did:example:456")
        .await
        .unwrap();

    // The agent should be able to decode its own messages
    let received: PresentationMessage = agent.receive_message(&packed).await.unwrap();

    // Check that the message was received correctly
    assert_eq!(received.presentation_id, "test123");
    assert_eq!(received.data, "secure-data");
}
