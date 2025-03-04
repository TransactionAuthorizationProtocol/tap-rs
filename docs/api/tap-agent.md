# tap-agent API Reference

The `tap-agent` crate provides the Agent implementation for the TAP protocol. Agents are the core entities in TAP that can send and receive messages, manage keys, and process transactions.

## Core Types

### `Agent`

The main Agent implementation for TAP.

```rust
pub struct Agent {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new Agent with the given configuration
pub fn new(config: AgentConfig, key_pair: Arc<KeyPair>) -> Result<Self, Error>;

/// Get the agent's DID
pub fn did(&self) -> &str;

/// Get the agent's name/alias
pub fn name(&self) -> &str;

/// Process an incoming DIDComm message
pub async fn process_message(&self, message: Message) -> Result<Option<Message>, Error>;

/// Create and sign a new TAP message
pub async fn create_message<T: TapMessageBody>(&self, body: T, to: &[&str]) -> Result<Message, Error>;

/// Send a message to another agent
pub async fn send_message(&self, to: &str, message: Message) -> Result<(), Error>;

/// Encrypt a message for the recipient(s)
pub async fn encrypt_message(&self, message: Message, to: &[&str]) -> Result<Message, Error>;

/// Decrypt an encrypted message
pub async fn decrypt_message(&self, message: Message) -> Result<Message, Error>;

/// Set a message handler for a specific message type
pub fn set_message_handler<F>(&self, message_type: &str, handler: F)
where
    F: Fn(Message) -> Result<Option<Message>, Error> + Send + Sync + 'static;

/// Update the agent's key pair
pub async fn update_key_pair(&mut self, key_pair: Arc<KeyPair>) -> Result<(), Error>;
```

### `AgentConfig`

Configuration options for creating an Agent.

```rust
pub struct AgentConfig {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new default AgentConfig
pub fn new() -> Self;

/// Set the DID for this agent
pub fn with_did(mut self, did: String) -> Self;

/// Set the name/alias for this agent
pub fn with_name(mut self, name: String) -> Self;

/// Enable logging for this agent
pub fn with_logging(mut self, enable: bool) -> Self;

/// Set the key resolver implementation
pub fn with_key_resolver(mut self, resolver: Arc<dyn KeyResolver>) -> Self;
```

### `DefaultAgent`

The standard implementation of Agent.

```rust
pub struct DefaultAgent {
    // Internal implementation details
}
```

This type implements the primary agent functionality and is the default concrete type returned by `Agent::new()`.

## Message Processing

### `MessageHandler`

A trait for handling TAP messages.

```rust
pub trait MessageHandler: Send + Sync {
    /// Process a message and optionally return a response
    async fn handle(&self, message: Message) -> Result<Option<Message>, Error>;
}
```

#### Implementing a Custom Message Handler

```rust
struct MyTransferHandler {
    // Handler state
}

#[async_trait]
impl MessageHandler for MyTransferHandler {
    async fn handle(&self, message: Message) -> Result<Option<Message>, Error> {
        // Check if this is a transfer message
        if message.type_.as_ref().map_or(false, |t| t == "TAP_TRANSFER") {
            // Process the transfer
            let transfer_body = TransferBody::from_didcomm(&message)?;
            
            // Create an authorize response
            let authorize = AuthorizeBody {
                transfer_id: message.id.clone(),
                note: Some("Transfer authorized".to_string()),
                metadata: HashMap::new(),
            };
            
            // Convert to DIDComm message
            let response = authorize.to_didcomm()?
                .set_from(Some("did:example:bob".to_string()))
                .set_to(Some(vec![transfer_body.originator.id]))
                .set_created_time(Some(get_current_time()));
            
            return Ok(Some(response));
        }
        
        // Not a transfer message, return None
        Ok(None)
    }
}
```

## Key Management

### `AgentKeyManager`

Manages keys for an agent.

```rust
pub struct AgentKeyManager {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new AgentKeyManager with the given key pair
pub fn new(key_pair: Arc<KeyPair>) -> Self;

/// Sign data with the agent's key
pub async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, Error>;

/// Verify a signature using the agent's key
pub async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, Error>;

/// Get the agent's DID
pub fn did(&self) -> &str;

/// Update the agent's key pair
pub fn update_key_pair(&mut self, key_pair: Arc<KeyPair>) -> Result<(), Error>;
```

## Error Handling

```rust
pub enum Error {
    /// Error from the core TAP library
    Core(tap_core::error::Error),
    
    /// Error related to message processing
    MessageProcessing(String),
    
    /// Error related to key management
    KeyManagement(String),
    
    /// Error related to DID resolution
    DIDResolution(String),
    
    /// General error
    General(String),
}
```

## Examples

### Creating an Agent

```rust
use tap_agent::{Agent, AgentConfig};
use tap_core::did::KeyPair;
use std::sync::Arc;

async fn create_agent() -> Result<Agent, Box<dyn std::error::Error>> {
    // Generate a new key pair
    let key_pair = KeyPair::generate_ed25519().await?;
    
    // Create a configuration
    let config = AgentConfig::new()
        .with_name("Alice".to_string())
        .with_logging(true);
    
    // Create the agent
    let agent = Agent::new(config, Arc::new(key_pair))?;
    
    println!("Created agent with DID: {}", agent.did());
    
    Ok(agent)
}
```

### Creating and Sending a Transfer Message

```rust
use tap_agent::Agent;
use tap_core::{
    message::{TransferBody, Agent as MessageAgent},
    did::KeyPair,
};
use tap_caip::AssetId;
use std::{collections::HashMap, sync::Arc, str::FromStr};

async fn send_transfer(
    agent: &Agent,
    beneficiary_did: &str,
    asset_id: &str,
    amount: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create agent representations
    let originator = MessageAgent {
        id: agent.did().to_string(),
        role: Some("originator".to_string()),
    };
    
    let beneficiary = MessageAgent {
        id: beneficiary_did.to_string(),
        role: Some("beneficiary".to_string()),
    };
    
    // Parse the asset ID
    let asset = AssetId::from_str(asset_id)?;
    
    // Create transfer body
    let transfer_body = TransferBody {
        asset,
        originator,
        beneficiary: Some(beneficiary),
        amount: amount.to_string(),
        agents: vec![],
        settlement_id: None,
        memo: Some("Payment for services".to_string()),
        metadata: HashMap::new(),
    };
    
    // Create and sign the message
    let message = agent.create_message(transfer_body, &[beneficiary_did]).await?;
    
    // Send the message
    agent.send_message(beneficiary_did, message).await?;
    
    Ok(())
}
```

### Setting Up a Message Handler

```rust
use tap_agent::Agent;
use tap_core::message::AuthorizeBody;
use didcomm::Message;
use std::collections::HashMap;

async fn setup_message_handler(agent: &Agent) {
    // Set up a handler for TAP_TRANSFER messages
    agent.set_message_handler("TAP_TRANSFER", |message| {
        println!("Received transfer with ID: {}", message.id);
        
        // Create an authorize response
        let authorize = AuthorizeBody {
            transfer_id: message.id.clone(),
            note: Some("Transfer authorized automatically".to_string()),
            metadata: HashMap::new(),
        };
        
        // Assume we can extract the sender from the message
        let sender = message.from.as_ref()
            .ok_or_else(|| tap_agent::Error::General("Missing sender".to_string()))?;
        
        // Convert to DIDComm message and set routing
        let response = authorize.to_didcomm()?
            .set_from(Some(agent.did().to_string()))
            .set_to(Some(vec![sender.clone()]))
            .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
        
        Ok(Some(response))
    });
}
```
