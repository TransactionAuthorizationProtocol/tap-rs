# TAP Node

A high-performance, asynchronous node implementation for the Transaction Authorization Protocol (TAP). This crate provides a complete node infrastructure for managing TAP agents, routing messages, and coordinating secure financial transactions.

## Overview

The TAP Node acts as a central hub for TAP communications, managing multiple agents, processing messages, and coordinating the transaction lifecycle. It is designed for high-throughput environments, with support for concurrent message processing, event-driven architecture, and robust error handling.

## Key Features

- **Multi-Agent Management**: Register and manage multiple TAP agents with different roles and capabilities
- **Message Processing Pipeline**: Process messages through configurable middleware chains
- **Message Routing**: Intelligently route messages to the appropriate agent based on DID addressing
- **Concurrent Processing**: Scale to high throughput with worker pools for message processing
- **Event Publishing**: Comprehensive event system for monitoring and reacting to node activities
- **Flexible Message Delivery**: Send messages via HTTP or WebSockets with robust error handling
- **Cross-Platform Support**: Native and WASM environments for both HTTP and WebSocket transports
- **DID Resolution**: Resolve DIDs for message verification and routing
- **Configurable Components**: Customize node behavior with pluggable components
- **Thread-Safe Design**: Safely share the node across threads with appropriate synchronization
- **WASM Compatibility**: Optional WASM support for browser environments

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
tap-node = { path = "../tap-node" }
tap-agent = { path = "../tap-agent" }
tap-msg = { path = "../tap-msg" }

# Optional features
tap-node = { path = "../tap-node", features = ["native"] } # Enable HTTP support
tap-node = { path = "../tap-node", features = ["websocket"] } # Enable WebSocket support
tap-node = { path = "../tap-node", features = ["native-with-websocket"] } # Enable both HTTP and WebSocket
tap-node = { path = "../tap-node", features = ["wasm"] } # Enable WASM support
tap-node = { path = "../tap-node", features = ["wasm-with-websocket"] } # Enable WASM with WebSocket
```

## Architecture

The TAP Node is built with a modular architecture:

```
┌───────────────────────────────────────────────┐
│                   TAP Node                     │
├───────────────┬───────────────┬───────────────┤
│ Agent Registry│ Message Router│  Event Bus    │
├───────────────┼───────────────┼───────────────┤
│ Message       │ Processor Pool│  Resolver     │
│ Processors    │               │               │
└───────────────┴───────────────┴───────────────┘
        │               │               │
        ▼               ▼               ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│   TAP Agent   │ │   TAP Agent   │ │   TAP Agent   │
└───────────────┘ └───────────────┘ └───────────────┘
```

## Usage

### Basic Setup

```rust
use tap_node::{NodeConfig, TapNode};
use tap_agent::{AgentConfig, DefaultAgent};
use tap_agent::crypto::{DefaultMessagePacker, BasicSecretResolver};
use tap_agent::did::MultiResolver;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the node
    let config = NodeConfig {
        debug: true,
        max_agents: Some(10),
        enable_message_logging: true,
        log_message_content: false,
        processor_pool: None,
    };

    // Create a new node
    let mut node = TapNode::new(config);

    // Start processor pool for high throughput
    let pool_config = tap_node::message::processor_pool::ProcessorPoolConfig {
        workers: 4,
        channel_capacity: 100,
        worker_timeout: Duration::from_secs(30),
    };
    node.start(pool_config).await?;

    // Create and register an agent
    let agent_config = AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());
    let did_resolver = Arc::new(MultiResolver::default());
    let secret_resolver = Arc::new(BasicSecretResolver::new());
    let message_packer = Arc::new(DefaultMessagePacker::new(did_resolver, secret_resolver));
    let agent = DefaultAgent::new(agent_config, message_packer);
    node.register_agent(Arc::new(agent)).await?;

    // The node is now ready to process messages

    Ok(())
}
```

### Processing Messages

```rust
use tap_msg::didcomm::Message;

// Receive and process an incoming message
async fn handle_message(node: &TapNode, message: Message) -> Result<(), tap_node::Error> {
    // Process through the node's pipeline
    node.receive_message(message).await?;
    Ok(())
}

// Send a message from one agent to another
async fn send_message(node: &TapNode, from_did: &str, to_did: &str, message: Message) -> Result<String, tap_node::Error> {
    // Process and dispatch the message, returns the packed message
    let packed = node.send_message(from_did, to_did, message).await?;
    Ok(packed)
}
```

### Event Handling

```rust
use std::sync::Arc;
use async_trait::async_trait;
use tap_node::event::{EventBus, EventSubscriber, NodeEvent};

// Create a custom event subscriber
struct MyEventHandler;

#[async_trait]
impl EventSubscriber for MyEventHandler {
    async fn handle_event(&self, event: NodeEvent) {
        match event {
            NodeEvent::MessageReceived { message } => {
                println!("Message received: {:?}", message);
            },
            NodeEvent::AgentRegistered { did } => {
                println!("Agent registered: {}", did);
            },
            _ => {}
        }
    }
}

// Subscribe to events
async fn subscribe_to_events(node: &TapNode) {
    let event_bus = node.get_event_bus();
    let handler = Arc::new(MyEventHandler);
    event_bus.subscribe(handler).await;

    // Or use a channel-based approach
    let mut receiver = event_bus.subscribe_channel();
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            println!("Event received: {:?}", event);
        }
    });
}
```

## Custom Message Processors

You can create custom message processors to extend the node's capabilities:

```rust
use async_trait::async_trait;
use tap_node::error::Result;
use tap_node::message::processor::MessageProcessor;
use tap_msg::didcomm::Message;

#[derive(Clone, Debug)]
struct MyCustomProcessor;

#[async_trait]
impl MessageProcessor for MyCustomProcessor {
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>> {
        // Custom processing logic here
        println!("Processing message: {}", message.id);

        // Return the processed message
        Ok(Some(message))
    }

    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>> {
        // Custom outgoing message processing
        Ok(Some(message))
    }
}
```

## Message Transport

TAP Node provides multiple options for sending messages between nodes:

### HTTP Message Sender

For standard request-response communication patterns:

```rust
use tap_node::{HttpMessageSender, MessageSender};

// Create an HTTP sender with default settings
let sender = HttpMessageSender::new("https://recipient-endpoint.example.com".to_string());

// Create with custom settings (timeout and retries)
let sender = HttpMessageSender::with_options(
    "https://recipient-endpoint.example.com".to_string(),
    5000,  // 5 second timeout
    3      // 3 retries with exponential backoff
);

// Send a packed message to recipients
sender.send(
    packed_message,
    vec!["did:example:recipient".to_string()]
).await?;
```

### WebSocket Message Sender

For real-time bidirectional communication:

```rust
use tap_node::{WebSocketMessageSender, MessageSender};

// Create a WebSocket sender with default settings
let sender = WebSocketMessageSender::new("https://recipient-endpoint.example.com".to_string());

// Create with custom settings
let sender = WebSocketMessageSender::with_options(
    "https://recipient-endpoint.example.com".to_string(),
    30000,  // 30 second connection timeout
    5       // 5 reconnection attempts
);

// Send a message over an established WebSocket connection
sender.send(
    packed_message,
    vec!["did:example:recipient".to_string()]
).await?;
```

Key benefits of the WebSocket transport:
- Persistent connections for lower latency
- Bidirectional communication
- Connection state awareness
- Reduced overhead for frequent messages

## Integration with Other Crates

The TAP Node integrates with the TAP ecosystem:

- **tap-agent**: Provides the agent implementation used by the node
- **tap-msg**: Defines the message types and formats
- **tap-caip**: Handles chain-agnostic identifiers used in transactions
- **tap-http**: Can be used to create HTTP endpoints for the node
- **tap-wasm**: Enables WASM compatibility for browser environments

## Performance Considerations

The TAP Node is designed for high performance:

- Use processor pools for concurrent message processing
- Configure worker counts based on your hardware
- Consider message validation trade-offs
- Use appropriate channel capacities for your workload
- Profile your specific use case for optimal settings

## Examples

The package includes several examples:

- `benches/stress_test.rs` - Benchmark of node performance with different message loads
- `examples/http_message_flow.rs` - Example of using HTTP for message delivery
- `examples/websocket_message_flow.rs` - Example of using WebSockets for real-time communication

Run examples with:

```bash
# Run with HTTP support
cargo run --example http_message_flow --features native

# Run with WebSocket support
cargo run --example websocket_message_flow --features websocket
```

## License

This crate is licensed under the terms of the MIT license.
