# tap-agent: TAP Agent Implementation

The `tap-agent` crate provides an implementation of a Transaction Authorization Protocol (TAP) agent. This library facilitates message handling, DID resolution, policy evaluation, and message storage for TAP communications.

## Features

- Complete TAP agent implementation with DID identity support
- Message packing and unpacking using DIDComm v2
- Multiple DID method resolution (did:key, did:web, did:pkh)
- Policy handling for message evaluation
- In-memory message storage with query capabilities
- Asynchronous API with Rust's async/await
- WASM compatibility for browser environments

## Usage

Add `tap-agent` to your `Cargo.toml`:

```toml
[dependencies]
tap-agent = "0.1.0"
tap-core = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Creating a TAP Agent

```rust
use tap_agent::{Agent, AgentConfig, TapAgent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the agent
    let config = AgentConfig::new()
        .with_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
        .with_name("My TAP Agent")
        .with_endpoint("https://example.com/endpoint");

    // Create the agent
    let agent = TapAgent::with_defaults(
        config,
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        Some("My TAP Agent".to_string()),
    )?;

    println!("Agent created successfully!");
    println!("DID: {}", agent.did());
    println!("Name: {:?}", agent.name());
    
    Ok(())
}
```

### Creating and Sending Messages

```rust
use tap_agent::{Agent, AgentConfig, TapAgent};
use tap_core::message::TapMessageType;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the agent
    let config = AgentConfig::new_with_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
    let agent = TapAgent::with_defaults(
        config,
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        None,
    )?;

    // Create a transaction proposal message
    let message = agent.create_message(
        TapMessageType::TransactionProposal,
        Some(json!({
            "transaction": {
                "amount": "100.00",
                "currency": "USD",
                "sender": agent.did(),
                "receiver": "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL"
            }
        })),
    ).await?;

    // Store the message
    agent.store_outgoing_message(&message).await?;
    println!("Created message with ID: {}", message.id);

    // Pack the message for sending
    let packed_message = agent.pack_message(
        &message,
        &"did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL".to_string()
    ).await?;
    
    // Send the packed message using your preferred transport protocol
    // Example: HTTP POST request to the recipient's endpoint
    println!("Message packed and ready to send: {}", packed_message);

    Ok(())
}
```

### Receiving and Processing Messages

```rust
use tap_agent::{Agent, AgentConfig, TapAgent};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the agent
    let config = AgentConfig::new_with_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
    let agent = TapAgent::with_defaults(
        config,
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        None,
    )?;

    // Assume we received a packed message
    let received_packed_message = r#"{"protected":"eyJhbGciOiJFZERTQSIsImtpZCI6ImRpZDpr..."}"#;
    
    // Add metadata about the message source
    let mut metadata = HashMap::new();
    metadata.insert(
        "from".to_string(),
        "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL".to_string(),
    );
    
    // Receive the message (unpack, validate, and store)
    let message = agent.receive_message(received_packed_message, Some(metadata)).await?;
    
    println!("Received message:");
    println!("ID: {}", message.id);
    println!("Type: {:?}", message.message_type);
    println!("From: {:?}", message.from);
    
    Ok(())
}
```

### Querying Stored Messages

```rust
use tap_agent::{Agent, AgentConfig, TapAgent};
use tap_agent::storage::MessageQuery;
use tap_core::message::TapMessageType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the agent
    let config = AgentConfig::new_with_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
    let agent = TapAgent::with_defaults(
        config,
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        None,
    )?;

    // Query for all transaction proposals
    let query = MessageQuery::new()
        .with_message_type(TapMessageType::TransactionProposal);
    
    let messages = agent.query_messages(query).await?;
    
    println!("Found {} transaction proposals:", messages.len());
    for message in messages {
        println!("ID: {}", message.id);
        println!("Created: {}", message.created_time);
        println!("From: {:?}", message.from);
        println!("To: {:?}", message.to);
        println!("---");
    }
    
    Ok(())
}
```

## Advanced Usage

### Creating a Custom DID Resolver

```rust
use tap_agent::did::{DidResolver, DidDoc};
use tap_agent::error::{Error, Result};
use async_trait::async_trait;
use std::fmt::Debug;

#[derive(Debug)]
struct CustomResolver;

#[async_trait]
impl DidResolver for CustomResolver {
    async fn resolve(&self, did: &str) -> Result<DidDoc> {
        // Implement your custom DID resolution logic here
        if did.starts_with("did:custom:") {
            // Create and return a DidDoc
            let doc = DidDoc {
                id: did.to_string(),
                verification_method: vec![/* ... */],
                // ... other fields
            };
            Ok(doc)
        } else {
            Err(Error::DidResolution(format!("Unsupported DID method: {}", did)))
        }
    }
}

// Then use it with MultiResolver
use tap_agent::did::MultiResolver;
use std::sync::Arc;

fn setup_custom_resolver() {
    let mut resolver = MultiResolver::new();
    resolver.add_resolver(CustomResolver);
    
    // Use the resolver with your agent or directly
    let resolver_arc = Arc::new(resolver);
    // ...
}
```

### Implementing a Custom Policy Handler

```rust
use tap_agent::policy::{PolicyHandler, PolicyResult};
use tap_core::message::TapMessage;
use tap_agent::error::Result;
use async_trait::async_trait;
use std::fmt::Debug;

#[derive(Debug)]
struct AmountLimitPolicyHandler {
    max_amount: f64,
}

impl AmountLimitPolicyHandler {
    fn new(max_amount: f64) -> Self {
        Self { max_amount }
    }
}

#[async_trait]
impl PolicyHandler for AmountLimitPolicyHandler {
    async fn evaluate(&self, message: &TapMessage) -> Result<PolicyResult> {
        // For transaction proposals, check the amount
        if message.message_type == tap_core::message::TapMessageType::TransactionProposal {
            if let Some(body) = &message.body {
                if let Some(transaction) = body.get("transaction") {
                    if let Some(amount_str) = transaction.get("amount").and_then(|a| a.as_str()) {
                        if let Ok(amount) = amount_str.parse::<f64>() {
                            if amount > self.max_amount {
                                return Ok(PolicyResult::Reject(format!(
                                    "Transaction amount ${} exceeds limit of ${}",
                                    amount, self.max_amount
                                )));
                            }
                        }
                    }
                }
            }
        }
        
        // Allow by default
        Ok(PolicyResult::Allow)
    }
}

// Use the policy handler when creating your agent
```

### Custom Message Storage

The `tap-agent` crate provides an in-memory message store by default, but you can implement your own storage backend:

```rust
use tap_agent::storage::{MessageStore, MessageQuery};
use tap_core::message::TapMessage;
use tap_agent::error::Result;
use async_trait::async_trait;
use std::fmt::Debug;

#[derive(Debug)]
struct DatabaseMessageStore {
    // Your database connection or client
    // db_client: DbClient,
}

#[async_trait]
impl MessageStore for DatabaseMessageStore {
    async fn store_message(&self, message: &TapMessage) -> Result<()> {
        // Store the message in your database
        // self.db_client.insert_message(message).await?;
        Ok(())
    }
    
    async fn get_message(&self, id: &str) -> Result<Option<TapMessage>> {
        // Retrieve the message from your database
        // let message = self.db_client.get_message(id).await?;
        // Ok(message)
        unimplemented!("Database storage not implemented")
    }
    
    async fn query_messages(&self, query: MessageQuery) -> Result<Vec<TapMessage>> {
        // Query messages from your database
        // let messages = self.db_client.query_messages(query).await?;
        // Ok(messages)
        unimplemented!("Database storage not implemented")
    }
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
