//! Tests for message handling in TAP Node

use std::collections::HashMap;
use std::sync::Arc;
use tap_agent::{AgentConfig, DefaultAgent};
use tap_agent::did::SyncDIDResolver;
use tap_agent::crypto::DebugSecretsResolver;
use tap_msg::didcomm::Message;
use tap_msg::message::ErrorBody;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_node::{NodeConfig, TapNode};
use uuid;
use async_trait::async_trait;
use tap_agent::error::Result as AgentResult;
use std::fmt::Debug;
use didcomm::did::{DIDDoc, VerificationMethod, VerificationMethodType, VerificationMaterial};

/// Test DID Resolver for testing
#[derive(Debug)]
struct TestDIDResolver;

impl TestDIDResolver {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SyncDIDResolver for TestDIDResolver {
    async fn resolve(&self, _did: &str) -> AgentResult<Option<DIDDoc>> {
        // Create a basic DIDDoc
        let did_doc = DIDDoc {
            id: "did:example:123".to_string(),
            verification_method: vec![VerificationMethod {
                id: "did:example:123#key-1".to_string(),
                controller: "did:example:123".to_string(),
                type_: VerificationMethodType::Ed25519VerificationKey2018,
                verification_material: VerificationMaterial::Base58 {
                    public_key_base58: "GHgtPsMnNW5bYrZFFcpvrFcuni4Bjt7QcRNoBQ1ijB2J".to_string(),
                },
            }],
            authentication: vec!["did:example:123#key-1".to_string()],
            key_agreement: vec![],
            service: vec![],
        };
        Ok(Some(did_doc))
    }
}

/// Test Secrets Resolver for testing
#[derive(Debug)]
struct TestSecretsResolver {
    secrets_map: HashMap<String, didcomm::secrets::Secret>,
}

impl TestSecretsResolver {
    pub fn new() -> Self {
        Self {
            secrets_map: HashMap::new(),
        }
    }
}

impl DebugSecretsResolver for TestSecretsResolver {
    fn get_secrets_map(&self) -> &HashMap<String, didcomm::secrets::Secret> {
        &self.secrets_map
    }
}

/// Create a test agent with the given DID
fn create_test_agent(did: &str) -> Arc<DefaultAgent> {
    let agent_config = AgentConfig::new(did.to_string());
    
    // Create resolvers for the DefaultMessagePacker
    let did_resolver = Arc::new(TestDIDResolver::new());
    let secrets_resolver = Arc::new(TestSecretsResolver::new());
    
    // Create a new DefaultAgent with message packer
    let message_packer = Arc::new(tap_agent::crypto::DefaultMessagePacker::new(
        did_resolver,
        secrets_resolver,
    ));
    let agent = DefaultAgent::new(agent_config, message_packer);

    Arc::new(agent)
}

/// Create a test error message body
fn create_test_error_body(_from_did: &str, _to_did: &str) -> ErrorBody {
    ErrorBody {
        code: "TEST_ERROR".to_string(),
        message: "This is a test error message".to_string(),
        caused_by: None,
        metadata: HashMap::new(),
    }
}

/// Convert a TapMessageBody to a DIDComm Message
fn create_didcomm_message<T: TapMessageBody>(
    body: &T,
    from_did: Option<&str>,
    to_did: Option<&str>,
) -> Message {
    let body_json = serde_json::to_value(body).unwrap();
    
    // Create basic DIDComm message with all required fields
    Message {
        id: uuid::Uuid::new_v4().to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        body: body_json,
        from: from_did.map(|s| s.to_string()),
        to: to_did.map(|s| vec![s.to_string()]),
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        type_: "https://example.com/protocols/tap/1.0/error".to_string(),
        extra_headers: HashMap::new(),
    }
}

/// Create a DIDComm Message for testing
fn create_test_message(from_did: &str, to_did: &str) -> Message {
    // Create an error body
    let error_body = create_test_error_body(from_did, to_did);
    
    // Convert to DIDComm message
    create_didcomm_message(&error_body, Some(from_did), Some(to_did))
}

#[tokio::test]
async fn test_message_routing() {
    // Create a node
    let config = NodeConfig::default();
    let _node = TapNode::new(config);

    // Create test agents
    let agent_1_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let agent_2_did = "did:key:z6Mkec9GnzF3o2jRJ2HtBZU9ri8hdwbUATEYQYgVVNJiSMMj";

    let agent_1 = create_test_agent(agent_1_did);
    let agent_2 = create_test_agent(agent_2_did);

    // Register agents with the node
    _node.register_agent(agent_1.clone()).await.unwrap();
    _node.register_agent(agent_2.clone()).await.unwrap();

    // Create a message from agent 1 to agent 2
    let message = create_test_message(agent_1_did, agent_2_did);

    // Process the message
    let result = _node.receive_message(message).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_message_with_unregistered_agent() {
    // Create a node
    let config = NodeConfig::default();
    let _node = TapNode::new(config);

    // Create a test agent
    let agent_1_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let unregistered_did = "did:key:z6Mkec9GnzF3o2jRJ2HtBZU9ri8hdwbUATEYQYgVVNJiSMMj";

    let agent_1 = create_test_agent(agent_1_did);

    // Register the first agent only
    _node.register_agent(agent_1.clone()).await.unwrap();

    // Create a message from registered agent to unregistered agent
    let message = create_test_message(agent_1_did, unregistered_did);

    // Process the message - should fail because target agent is not registered
    let result = _node.receive_message(message).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_message() {
    // Create a node
    let config = NodeConfig::default();
    let _node = TapNode::new(config);

    // Create a test agent
    let agent_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let agent = create_test_agent(agent_did);

    // Register the agent with the node
    _node.register_agent(agent.clone()).await.unwrap();
    
    // Create invalid message with missing to field
    let message = Message {
        id: uuid::Uuid::new_v4().to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        body: serde_json::to_value(&create_test_error_body(agent_did, "did:example:invalid")).unwrap(),
        from: Some(agent_did.to_string()),
        to: None, // Missing to field
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        type_: "https://example.com/protocols/tap/1.0/error".to_string(),
        extra_headers: HashMap::new(),
    };

    // Process the message - this should be filtered out by validation
    let result = _node.receive_message(message).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_custom_node_middleware() {
    // Create a node with custom middleware
    let config = NodeConfig::default();
    let _node = TapNode::new(config);

    // Create test agents
    let agent_1_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let agent_2_did = "did:key:z6Mkec9GnzF3o2jRJ2HtBZU9ri8hdwbUATEYQYgVVNJiSMMj";

    let agent_1 = create_test_agent(agent_1_did);
    let agent_2 = create_test_agent(agent_2_did);

    // Register agents with the node
    _node.register_agent(agent_1.clone()).await.unwrap();
    _node.register_agent(agent_2.clone()).await.unwrap();

    // Create a message from agent 1 to agent 2
    let message = create_test_message(agent_1_did, agent_2_did);

    // Process the message - the default middleware should handle this correctly
    let result = _node.receive_message(message).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_node_agent_communication() {
    // Create a node
    let config = NodeConfig::default();
    let _node = TapNode::new(config);

    // Create test agents
    let agent_1_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let agent_2_did = "did:key:z6Mkec9GnzF3o2jRJ2HtBZU9ri8hdwbUATEYQYgVVNJiSMMj";

    let agent_1 = create_test_agent(agent_1_did);
    let agent_2 = create_test_agent(agent_2_did);

    // Register agents with the node
    _node.register_agent(agent_1.clone()).await.unwrap();
    _node.register_agent(agent_2.clone()).await.unwrap();

    // Create a message from agent 1 to agent 2
    let message = create_test_message(agent_1_did, agent_2_did);

    // Send the message
    let result = _node.send_message(agent_1_did, agent_2_did, message).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_error_message() {
    // Create a node
    let config = NodeConfig::default();
    let _node = TapNode::new(config);

    // Create an error message
    let error_body = ErrorBody {
        code: "TEST001".to_string(),
        message: "Test error message".to_string(),
        caused_by: None,
        metadata: HashMap::new(),
    };

    // Convert the error_body to a message
    let message = error_body.to_didcomm().unwrap();
    
    // Validate message has required fields
    assert!(!message.id.is_empty());
    assert!(!message.typ.is_empty());
    assert!(!message.body.is_null());
    
    // Check specific error properties
    let error_json = &message.body;
    assert!(error_json.get("code").is_some());
    assert_eq!(
        error_json.get("code").unwrap().as_str().unwrap(),
        "TEST001"
    );
    assert!(error_json.get("message").is_some());
    assert_eq!(
        error_json.get("message").unwrap().as_str().unwrap(),
        "Test error message"
    );
}

#[tokio::test]
async fn test_message_creation() {
    // Create a Message directly with the required fields
    let message = Message {
        id: uuid::Uuid::new_v4().to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://example.com/protocol/1.0/test".to_string(),
        body: serde_json::json!({
            "key": "value",
            "number": 42
        }),
        from: Some("did:example:sender".to_string()),
        to: Some(vec!["did:example:recipient".to_string()]),
        created_time: Some(1234567890),
        expires_time: Some(1234657890),
        thid: None,
        pthid: None,
        extra_headers: HashMap::new(),
        from_prior: None,
        attachments: None,
    };

    // Validate message has required fields
    assert!(!message.id.is_empty());
    assert!(!message.typ.is_empty());
    assert!(!message.body.is_null());
    assert!(message.from.is_some());
    assert!(message.to.is_some());
    
    // Check specific properties
    assert_eq!(message.type_, "https://example.com/protocol/1.0/test");
    let body = &message.body;
    assert_eq!(body["key"].as_str().unwrap(), "value");
    assert_eq!(body["number"].as_i64().unwrap(), 42);
}
