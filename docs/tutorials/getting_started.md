# Getting Started with TAP-RS

This tutorial provides a step-by-step guide to get started with the TAP-RS library, showing you how to set up a basic TAP environment, create agents, and exchange messages.

## Prerequisites

Before starting, make sure you have:

- Rust 1.71.0 or later installed
- Cargo installed
- Basic familiarity with Rust and async programming
- (Optional) Node.js 14+ for TypeScript examples

## 1. Adding TAP-RS to Your Project

### Rust Project

Add TAP-RS dependencies to your `Cargo.toml`:

```toml
[dependencies]
tap-core = "0.1.0"
tap-agent = "0.1.0"
tap-node = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

### TypeScript/JavaScript Project

If you're using TAP-RS in a browser or Node.js environment:

```bash
# Using npm
npm install @tap-rs/tap-ts

# Using yarn
yarn add @tap-rs/tap-ts
```

## 2. Creating a TAP Agent

### In Rust

```rust
use tap_agent::{Agent, AgentConfig};
use tap_core::did::KeyPair;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate a DID key pair
    let key_pair = KeyPair::generate_ed25519().await?;
    let did = key_pair.get_did_key();
    
    // Create agent configuration
    let config = AgentConfig::new()
        .with_did(did.clone())
        .with_name("My TAP Agent");
    
    // Create the agent
    let agent = Agent::new(config, Arc::new(key_pair))?;
    
    println!("Created agent with DID: {}", did);
    
    Ok(())
}
```

### In TypeScript

```typescript
import { Agent, wasmLoader } from "@tap-rs/tap-ts";

async function main() {
    // Load the WASM module
    await wasmLoader.load();

    // Create an agent (this will generate a DID key by default)
    const agent = new Agent({
        nickname: "My TypeScript Agent"
    });

    console.log(`Created agent with DID: ${agent.did}`);
}

main().catch(console.error);
```

## 3. Creating and Sending TAP Messages

### In Rust

```rust
use tap_core::message::{TransferBody, TapMessageBody, Agent as MessageAgent};
use didcomm::Message;
use tap_caip::AssetId;
use std::collections::HashMap;

async fn create_transfer_message(
    from_did: &str, 
    to_did: &str
) -> Result<Message, tap_core::error::Error> {
    // Create originator and beneficiary agents
    let originator = MessageAgent {
        id: from_did.to_string(),
        role: Some("originator".to_string()),
    };
    
    let beneficiary = MessageAgent {
        id: to_did.to_string(),
        role: Some("beneficiary".to_string()),
    };
    
    // Create a transfer body
    let transfer_body = TransferBody {
        asset: AssetId::parse("eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap(), // DAI token
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents: vec![],
        settlement_id: None,
        memo: Some("Payment for services".to_string()),
        metadata: HashMap::new(),
    };
    
    // Create a DIDComm message from the transfer body
    let message = transfer_body.to_didcomm()?;
    
    // Set the sender and recipients
    let message = message
        .set_from(Some(from_did.to_string()))
        .set_to(Some(vec![to_did.to_string()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    Ok(message)
}
```

### In TypeScript

```typescript
import { Message, MessageType } from "@tap-rs/tap-ts";

// Create a TAP message
const createTransferMessage = () => {
    const transfer = new Message({
        type: MessageType.TRANSFER,
    });
    
    // Set transfer data
    transfer.setTransferData({
        asset: "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F", // DAI token
        amount: "100.0",
        originatorDid: "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
        beneficiaryDid: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        memo: "Payment for services"
    });
    
    return transfer;
};
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

### In TypeScript

```typescript
import { TapNode, Agent } from "@tap-rs/tap-ts";

// Create a TAP node
const node = new TapNode({
    debug: true,
    network: {
        peers: ["https://example.com/tap"], // Optional peers
    },
});

// Create an agent
const agent = new Agent({
    nickname: "Alice",
});

// Register the agent with the node
node.registerAgent(agent);

// Subscribe to messages on the node
const unsubscribe = node.subscribeToMessages((message, metadata) => {
    console.log(`Received message: ${message.id} from ${metadata.fromDid}`);
});
```

## 5. Processing Incoming Messages

### In Rust

```rust
use tap_core::message::{TransferBody, AuthorizeBody};
use didcomm::Message;
use std::collections::HashMap;

async fn process_transfer_message(
    message: &Message
) -> Result<Message, tap_core::error::Error> {
    // Extract the transfer body
    let transfer_body = TransferBody::from_didcomm(message)?;
    
    println!("Received transfer:");
    println!("  From: {}", transfer_body.originator.id);
    println!("  Amount: {}", transfer_body.amount);
    println!("  Asset: {}", transfer_body.asset);
    
    // Create an authorize response
    let authorize_body = AuthorizeBody {
        transfer_id: message.id.clone(),
        note: Some("Transfer authorized".to_string()),
        metadata: HashMap::new(),
    };
    
    // Convert to DIDComm message
    let response = authorize_body.to_didcomm()?;
    
    // Set sender and recipient
    let response = response
        .set_from(Some(transfer_body.beneficiary.as_ref().unwrap().id.clone()))
        .set_to(Some(vec![transfer_body.originator.id.clone()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    Ok(response)
}
```

### In TypeScript

```typescript
import { Agent, Message, MessageType } from "@tap-rs/tap-ts";

// Register a message handler for a specific message type
agent.registerMessageHandler(MessageType.TRANSFER, (message, metadata) => {
    console.log("Received transfer message:");
    console.log("  From:", metadata.fromDid);
    
    const transferData = message.getTransferData();
    console.log("  Amount:", transferData.amount);
    console.log("  Asset:", transferData.asset);
    
    // Create an authorize response
    const authorize = new Message({
        type: MessageType.AUTHORIZE,
        correlation: message.id, // Link to the transfer message
    });
    
    // Set authorize data
    authorize.setAuthorizeData({
        note: "Transfer authorized"
    });
    
    // Send the response
    return agent.sendMessage(metadata.fromDid, authorize);
});
```

## 6. Complete End-to-End Example

### In Rust

```rust
use tap_agent::{Agent, AgentConfig};
use tap_core::{
    did::KeyPair,
    message::{TransferBody, AuthorizeBody, TapMessageBody, Agent as MessageAgent},
};
use tap_node::{Node, NodeConfig};
use didcomm::Message;
use tap_caip::AssetId;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create key pairs for Alice and Bob
    let alice_key = KeyPair::generate_ed25519().await?;
    let bob_key = KeyPair::generate_ed25519().await?;
    
    let alice_did = alice_key.get_did_key();
    let bob_did = bob_key.get_did_key();
    
    println!("Alice DID: {}", alice_did);
    println!("Bob DID: {}", bob_did);
    
    // Create agents for Alice and Bob
    let alice_agent = Agent::new(
        AgentConfig::new().with_did(alice_did.clone()).with_name("Alice"),
        Arc::new(alice_key),
    )?;
    
    let bob_agent = Agent::new(
        AgentConfig::new().with_did(bob_did.clone()).with_name("Bob"),
        Arc::new(bob_key),
    )?;
    
    // Create and start a node
    let node = Arc::new(Node::new(NodeConfig::default()));
    
    // Register agents with the node
    node.register_agent(Arc::new(alice_agent.clone())).await?;
    node.register_agent(Arc::new(bob_agent.clone())).await?;
    
    // Create a channel to signal when Bob receives and processes a message
    let (tx, rx) = oneshot::channel::<Message>();
    
    // Set up a message handler for Bob
    node.register_message_handler(bob_did.clone(), |message| {
        println!("Bob received a message: {}", message.id);
        
        // Process based on message type
        if message.type_.as_ref().map_or(false, |t| t == "TAP_TRANSFER") {
            // Create an authorize response (in a real scenario, would verify first)
            let authorize_body = AuthorizeBody {
                transfer_id: message.id.clone(),
                note: Some("Transfer authorized".to_string()),
                metadata: HashMap::new(),
            };
            
            match authorize_body.to_didcomm() {
                Ok(response) => {
                    let response = response
                        .set_from(Some(bob_did.clone()))
                        .set_to(Some(vec![alice_did.clone()]))
                        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
                    
                    // Signal that we've processed the message
                    let _ = tx.send(response);
                },
                Err(e) => eprintln!("Error creating response: {}", e),
            }
        }
        
        Ok(())
    }).await?;
    
    // Alice creates a transfer message for Bob
    let originator = MessageAgent {
        id: alice_did.clone(),
        role: Some("originator".to_string()),
    };
    
    let beneficiary = MessageAgent {
        id: bob_did.clone(),
        role: Some("beneficiary".to_string()),
    };
    
    let transfer_body = TransferBody {
        asset: AssetId::parse("eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap(),
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents: vec![],
        settlement_id: None,
        memo: Some("Payment for services".to_string()),
        metadata: HashMap::new(),
    };
    
    let transfer_message = transfer_body.to_didcomm()?
        .set_from(Some(alice_did.clone()))
        .set_to(Some(vec![bob_did.clone()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    // Alice sends the message to the node
    println!("Alice is sending a transfer message...");
    node.receive(transfer_message).await?;
    
    // Wait for Bob's response
    let bob_response = rx.await?;
    
    // Alice processes Bob's response
    println!("Alice received Bob's response");
    let authorize_body = AuthorizeBody::from_didcomm(&bob_response)?;
    println!("Authorization note: {}", authorize_body.note.unwrap_or_default());
    
    // In a real scenario, Alice would now proceed with the on-chain transaction
    println!("Alice can now proceed with the on-chain transaction");
    
    Ok(())
}
```

### In TypeScript

See the [basic_usage.ts](../../tap-ts/examples/basic_usage.ts) example for a complete TypeScript implementation.

## Next Steps

Now that you've learned the basics of using TAP-RS, you might want to explore:

- [Implementing TAP flows](./implementing_tap_flows.md) - A detailed guide on implementing complete TAP flows
- [Security best practices](./security_best_practices.md) - How to secure your TAP implementation
- [WASM integration](./wasm_integration.md) - How to use TAP-RS in browser environments
- [API Reference](../api/index.md) - Complete API documentation

For questions or support, please open an issue on the [TAP-RS GitHub repository](https://github.com/notabene/tap-rs).
