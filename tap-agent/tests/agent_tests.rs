//! Tests for TAP Agent

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{DefaultMessagePacker, DidResolver};
use tap_agent::did::DefaultDIDResolver;
use tap_agent::policy::DefaultPolicyHandler;
use tap_core::message::tap_message_trait::TapMessageBody;

// Test message for agent tests
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestMessage {
    pub content: String,
}

impl TapMessageBody for TestMessage {
    fn message_type() -> &'static str {
        "TAP_TEST"
    }

    fn from_didcomm(msg: &didcomm::Message) -> tap_core::error::Result<Self> {
        let body = &msg.body;
        serde_json::from_value(body.clone())
            .map_err(|e| tap_core::error::Error::Validation(e.to_string()))
    }

    fn validate(&self) -> tap_core::error::Result<()> {
        Ok(())
    }
}

// Create a wrapper for DefaultDIDResolver to implement DidResolver locally
#[derive(Debug)]
struct TestDIDResolver {
    #[allow(dead_code)]
    inner: DefaultDIDResolver,
}

impl TestDIDResolver {
    fn new() -> Self {
        TestDIDResolver {
            inner: DefaultDIDResolver::new(),
        }
    }
}

#[async_trait]
impl DidResolver for TestDIDResolver {
    async fn resolve(&self, _did: &str) -> tap_agent::error::Result<String> {
        // Return a simple mock DID document for testing
        Ok(r#"{
            "@context": "https://www.w3.org/ns/did/v1",
            "id": "did:example:123",
            "authentication": [{
                "id": "did:example:123#keys-1",
                "type": "Ed25519VerificationKey2018",
                "controller": "did:example:123",
                "publicKeyBase58": "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV"
            }]
        }"#
        .to_string())
    }
}

#[tokio::test]
async fn test_agent_creation() {
    // Create a test agent
    let config = AgentConfig::new("did:example:123".to_string());

    // Create the DID resolver
    let resolver = Arc::new(TestDIDResolver::new());

    // Create the message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(resolver.clone()));

    // Create the policy handler
    let policy_handler = Arc::new(DefaultPolicyHandler::new());

    // Create the agent
    let agent = DefaultAgent::new(config, message_packer, policy_handler);

    // Check that the agent was created successfully
    assert_eq!(agent.get_agent_did(), "did:example:123");
}

#[tokio::test]
async fn test_send_receive_message() {
    // Create a test agent
    let config = AgentConfig::new("did:example:123".to_string());

    // Create the DID resolver
    let resolver = Arc::new(TestDIDResolver::new());

    // Create the message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(resolver.clone()));

    // Create the policy handler
    let policy_handler = Arc::new(DefaultPolicyHandler::new());

    // Create the agent
    let agent = DefaultAgent::new(config, message_packer, policy_handler);

    // Create a simple message
    let test_message = TestMessage {
        content: "value".to_string(),
    };

    // Pack the message for sending
    let packed = agent
        .send_message(&test_message, "did:example:456")
        .await
        .unwrap();

    // The agent should be able to decode its own messages
    let received: TestMessage = agent.receive_message(&packed).await.unwrap();

    // Check that the message was received correctly
    assert_eq!(received.content, "value");
}
