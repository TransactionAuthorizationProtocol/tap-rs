//! Example of using HTTP messaging between TAP agents

use std::sync::Arc;
use std::time::Duration;
use tap_agent::agent::DefaultAgent;
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{BasicSecretResolver, DefaultMessagePacker};
use tap_agent::did::{DIDDoc, MultiResolver, Service, VerificationMethod};
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_msg::didcomm::PlainMessage;
use tokio::time::sleep;

/// A test resolver that resolves DIDs to predefined DID documents
#[derive(Debug)]
struct TestDIDResolver;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a DID resolver
    let resolver = Arc::new(MultiResolver::default());

    // Create a secrets resolver
    let mut secrets = BasicSecretResolver::new();

    // Add Alice's secret
    let alice_secret = create_test_secret("did:example:alice", "alice-key");
    secrets.add_secret("did:example:alice", alice_secret);

    // Add Bob's secret
    let bob_secret = create_test_secret("did:example:bob", "bob-key");
    secrets.add_secret("did:example:bob", bob_secret);

    // Create the message packers
    let alice_packer = Arc::new(DefaultMessagePacker::new(
        resolver.clone(),
        Arc::new(secrets.clone()),
        true,
    ));
    let bob_packer = Arc::new(DefaultMessagePacker::new(
        resolver.clone(),
        Arc::new(secrets.clone()),
        true,
    ));

    // Create Agent configurations for Alice and Bob
    let alice_config = AgentConfig::new("did:example:alice".to_string())
        .with_security_mode("SIGNED")
        .with_debug(true);

    let bob_config = AgentConfig::new("did:example:bob".to_string())
        .with_security_mode("SIGNED")
        .with_debug(true);

    // Create the agents
    let alice = DefaultAgent::new(alice_config, alice_packer);
    let bob = DefaultAgent::new(bob_config, bob_packer);

    // Let's create a PlainMessage to simulate a message flow
    let plain_message = PlainMessage {
        id: "msg-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "example.message".to_string(),
        body: serde_json::json!({"content": "Hello, Bob!"}),
        from: "did:example:alice".to_string(),
        to: vec!["did:example:bob".to_string()],
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: std::collections::HashMap::new(),
    };

    // Serialize the message
    let message_json = serde_json::to_string(&plain_message)?;
    println!("Original message: {}", message_json);

    // In a real HTTP flow:
    // 1. Alice would sign the message
    // 2. Alice would send it to Bob's endpoint
    // 3. Bob would receive and verify the message

    // Wait to see output
    sleep(Duration::from_secs(1)).await;

    Ok(())
}

// Helper function to create a test secret
fn create_test_secret(did: &str, key_id: &str) -> Secret {
    Secret {
        id: did.to_string(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "base64-encoded-public-key",
                "d": "base64-encoded-private-key",
                "kid": format!("{}#{}", did, key_id)
            }),
        },
    }
}
