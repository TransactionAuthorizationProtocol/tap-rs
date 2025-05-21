# Getting Started with TAP-RS

This tutorial provides a step-by-step guide to get started with the TAP-RS library, showing you how to set up a basic TAP environment, create agents, and exchange messages.

## Prerequisites

Before starting, make sure you have:

- Rust 1.71.0 or later installed
- Cargo installed
- Basic familiarity with Rust and async programming

## 1. Adding TAP-RS to Your Project

### Rust Project

Add TAP-RS dependencies to your `Cargo.toml`:

```toml
[dependencies]
tap-msg = "0.2.0"
tap-agent = "0.2.0"
tap-node = "0.2.0"
tap-caip = "0.2.0"
tokio = { version = "1", features = ["full"] }
```

### WebAssembly and TypeScript Support

For information on using TAP-RS in browser environments or with TypeScript, see:
- [tap-wasm README](../../tap-wasm/README.md)
- [tap-ts README](../../tap-ts/README.md)

## 2. Creating a TAP Agent

### In Rust

```rust
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{DefaultMessagePacker, BasicSecretResolver};
use tap_agent::did::{MultiResolver, KeyResolver};
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate or use a pre-existing DID
    let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string();

    // Create agent configuration with the DID
    let config = AgentConfig::new(did.clone());

    // Set up DID resolver with support for did:key
    let mut did_resolver = MultiResolver::new();
    did_resolver.register_method("key", KeyResolver::new());
    let did_resolver = Arc::new(did_resolver);

    // Set up secret resolver with the agent's key
    let mut secret_resolver = BasicSecretResolver::new();
    let secret = Secret {
        id: format!("{}#keys-1", did),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo",
                "d": "nWGxne_9WmC6hEr-BQh-uDpW6n7dZsN4c4C9rFfIz3Yh"
            }),
        },
    };
    secret_resolver.add_secret(&did, secret);
    let secret_resolver = Arc::new(secret_resolver);

    // Create message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(did_resolver, secret_resolver));

    // Create the agent
    let agent = DefaultAgent::new(config, message_packer);

    println!("Created agent with DID: {}", did);

    Ok(())
}
```

### Creating an Ephemeral Agent (Simplest Approach)

For quick testing or development, you can create an ephemeral agent with a randomly generated DID:

```rust
use tap_agent::agent::DefaultAgent;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an agent with an ephemeral did:key (generates a random Ed25519 key)
    let (agent, did) = DefaultAgent::new_ephemeral()?;

    println!("Created ephemeral agent with DID: {}", did);

    Ok(())
}
```


## 3. Creating and Sending TAP Messages

### In Rust

```rust
use tap_agent::agent::Agent;
use tap_msg::message::{Transfer, TapMessageBody, Participant as MessageParticipant};
use tap_caip::AssetId;
use std::{collections::HashMap, str::FromStr};

async fn create_and_send_transfer(
    agent: &impl Agent,
    from_did: &str,
    to_did: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // Create originator and beneficiary participants
    let originator = MessageParticipant {
        id: from_did.to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let beneficiary = MessageParticipant {
        id: to_did.to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    // Create a transfer message
    let transfer = Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap(), // DAI token
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents: vec![],
        settlement_id: None,
        memo: Some("Payment for services".to_string()),
        metadata: HashMap::new(),
    };

    // Send the message
    let recipients = vec![to_did];
    let (packed_message, delivery_results) = agent.send_message(&transfer, recipients, true).await?;

    println!("Message sent: {}", packed_message);

    for result in delivery_results {
        if let Some(status) = result.status {
            println!("Delivered to {} with status {}", result.did, status);
        } else if let Some(error) = &result.error {
            println!("Failed to deliver to {}: {}", result.did, error);
        }
    }

    Ok(())
}
```


## 4. Setting Up a TAP Node

### In Rust

```rust
use tap_node::{Node, NodeConfig};
use std::sync::Arc;

async fn create_node() -> Result<Arc<Node>, Box<dyn std::error::Error>> {
    // Create node configuration
    let config = NodeConfig::default();

    // Create and start the node
    let node = Arc::new(Node::new(config));

    // Register an agent with the node
    // node.register_agent(agent).await?;

    Ok(node)
}
```


## 5. Processing Incoming Messages

### In Rust

```rust
use tap_agent::agent::Agent;
use tap_msg::message::{Transfer, Authorize, TapMessageBody, Participant as MessageParticipant};
use std::collections::HashMap;

async fn process_incoming_message(
    agent: &impl Agent,
    packed_message: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // Receive and unpack the message
    let transfer: Transfer = agent.receive_message(packed_message).await?;

    println!("Received transfer:");
    println!("  From: {}", transfer.originator.id);
    println!("  Amount: {}", transfer.amount);
    println!("  Asset: {}", transfer.asset);

    // Create an authorize response
    let authorize = Authorize {
        transfer_id: "transfer-123".to_string(), // In a real scenario, use the actual transfer ID
        note: Some("Transfer authorized".to_string()),
        metadata: HashMap::new(),
    };

    // Send the authorize message back to the originator
    let recipient_did = &transfer.originator.id;
    let (packed_response, _) = agent.send_message(&authorize, vec![recipient_did], true).await?;

    println!("Sent authorize response: {}", packed_response);

    Ok(())
}
```


## 6. Complete End-to-End Example

### In Rust

```rust
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{DefaultMessagePacker, BasicSecretResolver};
use tap_agent::did::{MultiResolver, KeyResolver};
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_msg::message::{Transfer, Authorize, TapMessageBody, Participant as MessageParticipant};
use tap_caip::AssetId;
use std::{collections::HashMap, sync::Arc, str::FromStr};
use tokio::time::sleep;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create two ephemeral agents for simplicity
    let (alice_agent, alice_did) = DefaultAgent::new_ephemeral()?;
    let (bob_agent, bob_did) = DefaultAgent::new_ephemeral()?;

    println!("Alice DID: {}", alice_did);
    println!("Bob DID: {}", bob_did);

    // Alice creates and sends a transfer to Bob
    let originator = MessageParticipant {
        id: alice_did.clone(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let beneficiary = MessageParticipant {
        id: bob_did.clone(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let transfer = Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap(),
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents: vec![],
        settlement_id: None,
        memo: Some("Payment for services".to_string()),
        metadata: HashMap::new(),
    };

    println!("Alice is sending a transfer to Bob...");
    let (packed_transfer, _) = alice_agent.send_message(&transfer, vec![&bob_did], false).await?;

    // In a real scenario, this message would be delivered over a transport layer
    // For simplicity, we directly give Bob the packed message

    println!("Bob receives the transfer...");
    let received_transfer: Transfer = bob_agent.receive_message(&packed_transfer).await?;

    println!("Bob unpacked the transfer:");
    println!("  From: {}", received_transfer.originator.id);
    println!("  Amount: {}", received_transfer.amount);
    println!("  Asset: {}", received_transfer.asset);

    // Bob creates and sends an authorize response
    let authorize = Authorize {
        transfer_id: "transfer-123".to_string(), // In a real scenario, use the ID from the transfer
        note: Some("Transfer authorized".to_string()),
        metadata: HashMap::new(),
    };

    println!("Bob is sending an authorize response to Alice...");
    let (packed_authorize, _) = bob_agent.send_message(&authorize, vec![&alice_did], false).await?;

    // Alice receives Bob's response
    println!("Alice receives Bob's response...");
    let received_authorize: Authorize = alice_agent.receive_message(&packed_authorize).await?;

    println!("Alice unpacked the authorize response:");
    println!("  Note: {}", received_authorize.note.unwrap_or_default());

    // In a real scenario, Alice would now proceed with the on-chain transaction
    println!("Alice can now proceed with the on-chain transaction");

    Ok(())
}
```


## Next Steps

Now that you've learned the basics of using TAP-RS, you might want to explore:

- [Implementing TAP flows](./implementing_tap_flows.md) - A detailed guide on implementing complete TAP flows
- [Security best practices](./security_best_practices.md) - How to secure your TAP implementation
- [API Reference](../api/index.md) - Complete API documentation

For questions or support, please open an issue on the [TAP-RS GitHub repository](https://github.com/notabene/tap-rs).
