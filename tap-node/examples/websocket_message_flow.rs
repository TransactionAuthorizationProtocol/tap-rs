//! Example of using WebSocket messaging with TAP

use base64;
use std::sync::Arc;
use std::time::Duration;
use tap_agent::agent::DefaultAgent;
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{BasicSecretResolver, DefaultMessagePacker};
use tap_agent::did::{DIDKeyGenerator, MultiResolver};
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create two agents - Alice and Bob
    println!("1. Creating Alice and Bob agents...");

    // Create a DID key generator
    let key_generator = Arc::new(DIDKeyGenerator::new());

    // Generate keys for Alice and Bob
    let alice_key = key_generator.generate_did(tap_agent::did::DIDGenerationOptions::default())?;
    let bob_key = key_generator.generate_did(tap_agent::did::DIDGenerationOptions::default())?;

    println!("   - Alice DID: {}", alice_key.did);
    println!("   - Bob DID: {}", bob_key.did);

    // Create a DID resolver that knows about both DIDs
    let did_resolver = Arc::new(MultiResolver::default());

    // Create a secrets resolver with both sets of keys
    let mut secrets_resolver = BasicSecretResolver::new();

    // Create and add secrets for Alice and Bob
    let alice_secret = Secret {
        id: alice_key.did.clone(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "x": base64::encode(&alice_key.public_key),
                "d": base64::encode(&alice_key.private_key)
            }),
        },
    };
    secrets_resolver.add_secret(&alice_key.did, alice_secret);

    let bob_secret = Secret {
        id: bob_key.did.clone(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "x": base64::encode(&bob_key.public_key),
                "d": base64::encode(&bob_key.private_key)
            }),
        },
    };
    secrets_resolver.add_secret(&bob_key.did, bob_secret);

    // Create a message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(
        did_resolver.clone(),
        Arc::new(secrets_resolver.clone()),
        true,
    ));

    // Create Agent configurations for Alice and Bob
    let alice_config = AgentConfig::new(alice_key.did.clone())
        .with_security_mode("SIGNED")
        .with_debug(true);

    let bob_config = AgentConfig::new(bob_key.did.clone())
        .with_security_mode("SIGNED")
        .with_debug(true);

    // Create the agents
    let _alice = Arc::new(DefaultAgent::new(alice_config, message_packer.clone()));
    let _bob = Arc::new(DefaultAgent::new(bob_config, message_packer.clone()));

    // Create WebSocket message routers
    println!("2. Creating message routers...");

    // Create a router config
    let _alice_router = tap_node::message::DefaultPlainMessageRouter::new();
    let _bob_router = tap_node::message::DefaultPlainMessageRouter::new();

    // Create message senders (simplified here - would need actual endpoints)
    let _alice_sender =
        tap_node::WebSocketPlainMessageSender::new("ws://localhost:3001/ws".to_string());
    let _bob_sender =
        tap_node::WebSocketPlainMessageSender::new("ws://localhost:3002/ws".to_string());

    // Start the WebSocket servers (would usually be in separate processes)
    println!("3. Starting WebSocket servers...");

    // This is a simplification - in a real system, you would have separate services
    // Instead, we'll just simulate the message flow

    // Simulate sending a message from Alice to Bob
    println!("4. Sending a message from Alice to Bob...");

    // In a real application, we would:
    // 1. Create a TAP message
    // 2. Pack it using the agent
    // 3. Send it via the MessageSender
    // 4. The recipient would receive it via their MessageRouter

    println!("5. WebSocket messaging simulation complete");

    // Wait to see output
    sleep(Duration::from_secs(1)).await;

    Ok(())
}
