//! Tests for TAP Node

use async_trait::async_trait;
use didcomm::did::{DIDDoc, VerificationMaterial, VerificationMethod, VerificationMethodType};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tap_agent::crypto::DebugSecretsResolver;
use tap_agent::did::SyncDIDResolver;
use tap_agent::error::Result as AgentResult;
use tap_agent::{AgentConfig, DefaultAgent};
use tap_node::{NodeConfig, TapNode};

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

#[tokio::test]
async fn test_node_creation() {
    // Create a basic node
    let config = NodeConfig::default();
    let node = TapNode::new(config);

    // Check node properties - these should be checked against public accessor methods
    assert!(!node.config().debug);
    assert_eq!(node.agents().agent_count(), 0);
}

#[tokio::test]
async fn test_agent_registration() {
    // Create a node
    let config = NodeConfig::default();
    let node = TapNode::new(config);

    // Create and register a test agent
    let agent_config =
        AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());

    // Create resolvers for the DefaultMessagePacker
    let did_resolver = Arc::new(TestDIDResolver::new());
    let secrets_resolver = Arc::new(TestSecretsResolver::new());

    // Create a new DefaultAgent with message packer
    let message_packer = Arc::new(tap_agent::crypto::DefaultMessagePacker::new(
        did_resolver,
        secrets_resolver,
    ));
    let agent = DefaultAgent::new(agent_config, message_packer);

    let agent = Arc::new(agent);
    let result = node.register_agent(agent.clone()).await;
    assert!(result.is_ok());

    // Check agent count
    assert_eq!(node.agents().agent_count(), 1);

    // Unregister the agent
    let result = node
        .unregister_agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
        .await;
    assert!(result.is_ok());

    // Check agent count again
    assert_eq!(node.agents().agent_count(), 0);
    assert!(!node
        .agents()
        .has_agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"));
}

#[tokio::test]
async fn test_agent_unregistration() {
    // Create a node
    let config = NodeConfig::default();
    let node = TapNode::new(config);

    // Create and register an agent
    let agent_config =
        AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());

    // Create resolvers for the DefaultMessagePacker
    let did_resolver = Arc::new(TestDIDResolver::new());
    let secrets_resolver = Arc::new(TestSecretsResolver::new());

    // Create a new DefaultAgent with message packer
    let message_packer = Arc::new(tap_agent::crypto::DefaultMessagePacker::new(
        did_resolver,
        secrets_resolver,
    ));
    let agent = DefaultAgent::new(agent_config, message_packer);

    node.register_agent(Arc::new(agent)).await.unwrap();
    assert_eq!(node.agents().agent_count(), 1);

    // Unregister the agent
    node.unregister_agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
        .await
        .unwrap();

    // Check that the agent is no longer registered
    assert_eq!(node.agents().agent_count(), 0);
    assert!(!node
        .agents()
        .has_agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"));
}

#[tokio::test]
async fn test_node_configuration() {
    // Create a node with custom configuration
    let config = NodeConfig {
        debug: true,
        max_agents: Some(10),
        enable_message_logging: true,
        log_message_content: true,
        processor_pool: None,
        event_logger: None,
    };
    let node = TapNode::new(config);

    // Create and register a test agent
    let agent_config =
        AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());

    // Create resolvers for the DefaultMessagePacker
    let did_resolver = Arc::new(TestDIDResolver::new());
    let secrets_resolver = Arc::new(TestSecretsResolver::new());

    // Create a new DefaultAgent with message packer
    let message_packer = Arc::new(tap_agent::crypto::DefaultMessagePacker::new(
        did_resolver,
        secrets_resolver,
    ));
    let agent = DefaultAgent::new(agent_config, message_packer);

    let agent = Arc::new(agent);
    let result = node.register_agent(agent).await;
    assert!(result.is_ok());

    // Check configuration
    assert!(node.config().debug);
    assert!(node.config().log_message_content);
    assert_eq!(node.config().max_agents, Some(10));
}
