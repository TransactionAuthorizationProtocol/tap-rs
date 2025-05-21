//! Example of using WebSocket messaging with TAP

use std::sync::Arc;
use std::time::Duration;
use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
use tap_agent::config::AgentConfig;
use tap_agent::did::{DIDKeyGenerator, MultiResolver};
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_agent::TapAgent;
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
    let _did_resolver = Arc::new(MultiResolver::default());

    // Create agent key managers
    let mut alice_builder = AgentKeyManagerBuilder::new();
    let mut bob_builder = AgentKeyManagerBuilder::new();

    // Create Alice's secret
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

    // Create Bob's secret
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

    // Add secrets to builders
    alice_builder = alice_builder.add_secret(alice_key.did.clone(), alice_secret);
    bob_builder = bob_builder.add_secret(bob_key.did.clone(), bob_secret);

    // Build the key managers
    let alice_key_manager = alice_builder
        .build()
        .expect("Failed to build Alice's key manager");
    let bob_key_manager = bob_builder
        .build()
        .expect("Failed to build Bob's key manager");

    // Create Agent configurations for Alice and Bob
    let alice_config = AgentConfig::new(alice_key.did.clone())
        .with_security_mode("SIGNED")
        .with_debug(true);

    let bob_config = AgentConfig::new(bob_key.did.clone())
        .with_security_mode("SIGNED")
        .with_debug(true);

    // Create the agents
    let _alice = Arc::new(TapAgent::new(alice_config, Arc::new(alice_key_manager)));
    let _bob = Arc::new(TapAgent::new(bob_config, Arc::new(bob_key_manager)));

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
