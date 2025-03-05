# TAP Node

TAP node orchestration and message routing for the Transaction Authorization Protocol (TAP).

## Features

- **Multi-Agent Coordination**: Manage multiple TAP agents within a single node
- **Message Routing**: Route incoming messages to the appropriate agent
- **DID Resolution**: Resolve DIDs for message routing and key discovery
- **Asynchronous Processing**: Process messages concurrently using Tokio
- **Event System**: Publish/subscribe system for TAP events
- **Message Queueing**: Queue management for pending messages

## Usage

```rust
use tap_node::node::{TapNode, DefaultTapNode};
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{DefaultMessagePacker, BasicSecretResolver};
use tap_agent::did::DefaultDIDResolver;
use std::sync::Arc;

// Create resolvers
let did_resolver = Arc::new(DefaultDIDResolver::new());
let secret_resolver = Arc::new(BasicSecretResolver::new());

// Create the TAP node
let mut node = DefaultTapNode::new(did_resolver.clone(), secret_resolver.clone());

// Create and register an agent
let config = AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());
let message_packer = Arc::new(DefaultMessagePacker::new(did_resolver, secret_resolver));
let agent = Arc::new(DefaultAgent::new(config, message_packer));
node.register_agent(agent).await?;

// Process an incoming message
let result = node.process_message("incoming_message").await;
```

## Components

### TapNode

The core node interface that manages agents and routes messages:

```rust
pub trait TapNode: Send + Sync {
    /// Register a new agent with the node
    async fn register_agent(&mut self, agent: Arc<dyn Agent>) -> Result<(), Error>;
    
    /// Process an incoming message
    async fn process_message(&self, message: &str) -> Result<(), Error>;
    
    /// Route a message to the appropriate agent
    async fn route_message(&self, message: &didcomm::Message) -> Result<(), Error>;
    
    /// Get an agent by DID
    fn get_agent(&self, did: &str) -> Option<Arc<dyn Agent>>;
}
```

### Message Processing Flow

1. An incoming message is received as a serialized string
2. The node deserializes and unpacks the message
3. The message is validated as a valid TAP message
4. The node determines the target agent based on the recipient DID
5. The message is routed to the appropriate agent for processing
6. The agent processes the message and returns a response if needed

### Event System

The node includes a pub/sub event system for TAP events:

```rust
// Subscribe to all transfer events
let mut subscription = node.subscribe_to_events("transfer").await?;

// Handle events asynchronously
tokio::spawn(async move {
    while let Some(event) = subscription.recv().await {
        println!("Received event: {:?}", event);
    }
});
```

### DID Resolution

The node handles DID resolution for message routing and agent discovery:

```rust
// Resolve a DID to find the recipient agent
let did_doc = node.resolve_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK").await?;
```

## Integration with Other Crates

- **tap-agent**: Uses agents for message processing
- **tap-msg**: Uses TAP message types for protocol handling
- **tap-http**: Can be used with tap-http for HTTP-based DIDComm messaging

## Examples

See the [examples directory](./examples) for more detailed usage examples.
