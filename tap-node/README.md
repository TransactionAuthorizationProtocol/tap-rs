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
- **Persistent Storage**: SQLite-based storage with dual functionality:
  - Transaction tracking for Transfer and Payment messages
  - Complete audit trail of all incoming/outgoing messages

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
tap-node = { path = "../tap-node", features = ["storage"] } # Enable persistent storage (enabled by default)
```

## Architecture

The TAP Node is built with a modular architecture:

```
┌─────────────────────────────────────────────────────────┐
│                      TAP Node                            │
├───────────────┬───────────────┬───────────────┬─────────┤
│ Agent Registry│ Message Router│  Event Bus    │ Storage │
├───────────────┼───────────────┼───────────────┼─────────┤
│ Message       │ Processor Pool│  Resolver     │ SQLite  │
│ Processors    │               │               │   DB    │
└───────────────┴───────────────┴───────────────┴─────────┘
        │               │               │               │
        ▼               ▼               ▼               ▼
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
        #[cfg(feature = "storage")]
        storage_path: None, // Uses default path: ./tap-node.db
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
use tap_msg::PlainMessage;

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

### Event Handling and Logging

The TAP Node includes a powerful event system with configurable logging capabilities:

```rust
use std::sync::Arc;
use async_trait::async_trait;
use tap_node::event::{EventBus, EventSubscriber, NodeEvent};
use tap_node::event::logger::{EventLogger, EventLoggerConfig, LogDestination};
use tap_node::{NodeConfig, TapNode};

// Create a TAP Node with event logging enabled
let mut config = NodeConfig::default();
config.event_logger = Some(EventLoggerConfig {
    destination: LogDestination::File {
        path: "./logs/tap-node.log".to_string(),
        max_size: Some(10 * 1024 * 1024), // 10 MB
        rotate: true,
    },
    structured: true, // Use JSON format
    log_level: log::Level::Info,
});

// Initialize node with event logging
let node = TapNode::new(config);

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

#### Event Logger Configuration

The event logger supports different destinations:

```rust
// Log to console
let config = EventLoggerConfig {
    destination: LogDestination::Console,
    structured: false, // Plain text
    log_level: log::Level::Info,
};

// Log to file with rotation
let config = EventLoggerConfig {
    destination: LogDestination::File {
        path: "./logs/tap-node.log".to_string(),
        max_size: Some(10 * 1024 * 1024), // 10 MB max file size
        rotate: true, // Enable rotation when max size is reached
    },
    structured: true, // JSON format
    log_level: log::Level::Info,
};

// Custom logging function
let custom_logger = Arc::new(|msg: &str| {
    // Custom handling of log messages
    println!("CUSTOM LOG: {}", msg);
    // Or send to a database, etc.
});

let config = EventLoggerConfig {
    destination: LogDestination::Custom(custom_logger),
    structured: true,
    log_level: log::Level::Info,
};
```

## Custom Message Processors

You can create custom message processors to extend the node's capabilities:

```rust
use async_trait::async_trait;
use tap_node::error::Result;
use tap_node::message::processor::MessageProcessor;
use tap_msg::tap_msg::PlainMessage;

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

## Persistent Storage

The TAP Node includes built-in support for persistent storage using SQLite. This feature is enabled by default and provides:

1. **Transaction Storage**: Automatic storage of Transfer and Payment messages for business logic processing
2. **Message Audit Trail**: Complete logging of all incoming and outgoing messages for compliance and debugging

### Storage Configuration

Configure storage when creating the node:

```rust
use tap_node::{NodeConfig, TapNode};
use std::path::PathBuf;

// Use default storage location (./tap-node.db)
let config = NodeConfig {
    #[cfg(feature = "storage")]
    storage_path: None,
    ..Default::default()
};

// Or specify a custom path
let config = NodeConfig {
    #[cfg(feature = "storage")]
    storage_path: Some(PathBuf::from("/path/to/database.db")),
    ..Default::default()
};

// Or use environment variable
std::env::set_var("TAP_NODE_DB_PATH", "/path/to/database.db");
```

### Accessing Stored Data

```rust
use tap_node::storage::MessageDirection;

// Get storage handle from the node
if let Some(storage) = node.storage() {
    // === Transaction Operations ===
    // Retrieve a specific transaction by message ID
    let transaction = storage.get_transaction_by_id("msg_12345").await?;
    
    // List recent transactions with pagination
    let transactions = storage.list_transactions(
        10,  // limit: 10 transactions
        0    // offset: 0 (first page)
    ).await?;
    
    // === Message Audit Trail Operations ===
    // Retrieve any message by ID
    let message = storage.get_message_by_id("msg_12345").await?;
    
    // List all messages
    let all_messages = storage.list_messages(
        20,   // limit
        0,    // offset
        None  // no direction filter
    ).await?;
    
    // List only incoming messages
    let incoming = storage.list_messages(
        10,
        0,
        Some(MessageDirection::Incoming)
    ).await?;
    
    // List only outgoing messages
    let outgoing = storage.list_messages(
        10,
        0,
        Some(MessageDirection::Outgoing)
    ).await?;
    
    // Examine message details
    for msg in all_messages {
        println!("Message: {} - Type: {} - Direction: {:?} - From: {:?} - To: {:?}", 
            msg.message_id, 
            msg.message_type,
            msg.direction,
            msg.from_did,
            msg.to_did
        );
    }
}
```

### Storage Features

- **Automatic Migration**: Database schema is automatically created and migrated on startup
- **Dual-Table Design**: Separate tables for transactions and message audit trail
- **Append-Only Design**: All data is immutable for compliance and auditing
- **SQLite WAL Mode**: Optimized for concurrent reads and writes
- **Connection Pooling**: Up to 10 concurrent database connections
- **WASM Compatibility**: Storage is automatically disabled in WASM builds
- **Duplicate Handling**: Duplicate messages are silently ignored (idempotent)

### Database Schema

The storage system maintains two tables:

#### `transactions` Table
Business logic for Transfer and Payment messages:
- Transaction ID and type (Transfer/Payment)
- Sender and recipient DIDs
- Thread ID for conversation tracking
- Full message content as JSON
- Status tracking (pending/confirmed/failed/cancelled/reverted)
- Timestamps for creation and updates

#### `messages` Table
Complete audit trail of all messages:
- Message ID and type (all TAP message types)
- Direction (incoming/outgoing)
- Sender and recipient DIDs
- Thread IDs (including parent threads)
- Full message content as JSON
- Creation timestamp

### Disabling Storage

To disable storage (for example, in memory-only deployments):

```toml
[dependencies]
tap-node = { path = "../tap-node", default-features = false, features = ["native"] }
```

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
