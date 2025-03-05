# tap-agent: TAP Agent Implementation

The `tap-agent` crate provides an implementation of a Transaction Authorization Protocol (TAP) agent. This library facilitates message handling, DID resolution, and secure communication using DIDComm for TAP.

## Features

- Complete TAP agent implementation with DID identity support
- Message packing and unpacking using DIDComm v2
- Multiple security modes: Plain, Signed, and AuthCrypt
- Multiple DID method resolution (did:key, did:web, did:pkh)
- Asynchronous API with Rust's async/await
- WASM compatibility for browser environments

## Usage

Add `tap-agent` to your `Cargo.toml`:

```toml
[dependencies]
tap-agent = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Creating a TAP Agent

```rust
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{DefaultMessagePacker, BasicSecretResolver};
use tap_agent::did::DefaultDIDResolver;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the agent
    let config = AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());

    // Set up components
    let did_resolver = Arc::new(DefaultDIDResolver::new());
    let secret_resolver = Arc::new(BasicSecretResolver::new());
    let message_packer = Arc::new(DefaultMessagePacker::new(did_resolver, secret_resolver));
    
    // Create the agent
    let agent = DefaultAgent::new(config, message_packer);

    println!("Agent created successfully!");
    println!("DID: {}", agent.get_agent_did());
    
    Ok(())
}
```

### Creating and Sending Messages

```rust
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{DefaultMessagePacker, BasicSecretResolver};
use tap_agent::did::DefaultDIDResolver;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tap_msg::message::tap_message_trait::TapMessageBody;

// Define a custom message type
#[derive(Debug, Serialize, Deserialize)]
struct TransactionProposal {
    pub amount: String,
    pub currency: String,
}

impl TapMessageBody for TransactionProposal {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#TransactionProposal"
    }

    fn from_didcomm(msg: &didcomm::Message) -> tap_msg::error::Result<Self> {
        serde_json::from_value(msg.body.clone())
            .map_err(|e| tap_msg::error::Error::Validation(e.to_string()))
    }

    fn validate(&self) -> tap_msg::error::Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up agent components
    let config = AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());
    let did_resolver = Arc::new(DefaultDIDResolver::new());
    let secret_resolver = Arc::new(BasicSecretResolver::new());
    let message_packer = Arc::new(DefaultMessagePacker::new(did_resolver, secret_resolver));
    
    // Create the agent
    let agent = DefaultAgent::new(config, message_packer);

    // Create a transaction proposal message
    let proposal = TransactionProposal {
        amount: "100.00".to_string(),
        currency: "USD".to_string(),
    };

    // Pack and send the message
    // The security mode will be automatically determined based on the message type
    let packed_message = agent
        .send_message(&proposal, "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL")
        .await?;
    
    // Send the packed message using your preferred transport protocol
    println!("Message packed and ready to send: {}", packed_message);

    Ok(())
}
```

### Receiving and Processing Messages

```rust
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{DefaultMessagePacker, BasicSecretResolver};
use tap_agent::did::DefaultDIDResolver;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tap_msg::message::tap_message_trait::TapMessageBody;

// Define the same message type for receiving
#[derive(Debug, Serialize, Deserialize)]
struct TransactionProposal {
    pub amount: String,
    pub currency: String,
}

impl TapMessageBody for TransactionProposal {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#TransactionProposal"
    }

    fn from_didcomm(msg: &didcomm::Message) -> tap_msg::error::Result<Self> {
        serde_json::from_value(msg.body.clone())
            .map_err(|e| tap_msg::error::Error::Validation(e.to_string()))
    }

    fn validate(&self) -> tap_msg::error::Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up agent components
    let config = AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());
    let did_resolver = Arc::new(DefaultDIDResolver::new());
    let secret_resolver = Arc::new(BasicSecretResolver::new());
    let message_packer = Arc::new(DefaultMessagePacker::new(did_resolver.clone(), secret_resolver.clone()));
    
    // Create the agent
    let agent = DefaultAgent::new(config, message_packer.clone());

    // Assume we received a packed message
    let received_packed_message = r#"{"protected":"eyJhbGciOiJFZERTQSIsImtpZCI6ImRpZDpr..."}"#;
    
    // Receive and unpack the message
    let message: TransactionProposal = agent.receive_message(received_packed_message).await?;
    
    println!("Received transaction proposal:");
    println!("Amount: {}", message.amount);
    println!("Currency: {}", message.currency);
    
    Ok(())
}
```

## Advanced Usage

### Creating a Custom DID Resolver

```rust
use tap_agent::did::{DidResolver};
use tap_agent::error::{Error, Result};
use async_trait::async_trait;
use std::fmt::Debug;

#[derive(Debug)]
struct CustomResolver;

#[async_trait]
impl DidResolver for CustomResolver {
    async fn resolve(&self, did: &str) -> Result<String> {
        // Implement your custom DID resolution logic here
        if did.starts_with("did:custom:") {
            // Return a DID document as a JSON string
            let doc = format!(
                r#"{{
                    "@context": "https://www.w3.org/ns/did/v1",
                    "id": "{}",
                    "authentication": [
                        "{}#keys-1"
                    ]
                }}"#,
                did, did
            );
            Ok(doc)
        } else {
            Err(Error::Validation(format!("Unsupported DID method: {}", did)))
        }
    }
}

// Then use it with your agent
use tap_agent::did::MultiResolver;
use std::sync::Arc;

fn setup_custom_resolver() {
    let mut resolver = MultiResolver::new();
    resolver.add_resolver(CustomResolver{});
    
    // Use the resolver with your agent
    let resolver_arc = Arc::new(resolver);
    // ...
}
```

### Working with Different Security Modes

The TAP agent supports three security modes:

1. **Plain**: Unencrypted, unsigned messages
2. **Signed**: Signed but not encrypted messages
3. **AuthCrypt**: Authenticated and encrypted messages (most secure)

The agent automatically determines the appropriate security mode based on the message type, but you can also specify it explicitly:

```rust
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{DefaultMessagePacker, BasicSecretResolver, MessagePacker};
use tap_agent::did::DefaultDIDResolver;
use tap_agent::message::SecurityMode;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up agent components
    let config = AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());
    let did_resolver = Arc::new(DefaultDIDResolver::new());
    let secret_resolver = Arc::new(BasicSecretResolver::new());
    let message_packer = Arc::new(DefaultMessagePacker::new(did_resolver.clone(), secret_resolver.clone()));
    
    // Create the agent
    let agent = DefaultAgent::new(config, message_packer.clone());

    // Create a message object
    let message = serde_json::json!({
        "type": "https://tap.rsvp/schema/1.0#SimpleMessage",
        "content": "Hello, World!"
    });

    // Pack with a specific security mode
    let packed = message_packer
        .pack_message(
            &message,
            "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL",
            Some("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"),
            SecurityMode::Signed
        )
        .await?;
    
    println!("Signed message: {}", packed);
    
    Ok(())
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
