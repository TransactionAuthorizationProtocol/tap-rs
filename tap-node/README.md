# TAP Node

A high-performance, asynchronous node implementation for the Transaction Authorization Protocol (TAP). This crate provides a complete node infrastructure for managing TAP agents, routing messages, and coordinating secure financial transactions.

## Overview

The TAP Node acts as a central hub for TAP communications, managing multiple agents, processing messages, and coordinating the transaction lifecycle. It is designed for high-throughput environments, with support for concurrent message processing, event-driven architecture, and robust error handling.

## Key Features

- **Multi-Agent Management**: Register and manage multiple TAP agents with different roles and capabilities
- **Message Processing Pipeline**: Process messages through configurable middleware chains
- **Travel Rule Compliance**: Automatic IVMS101 data generation and attachment for regulatory compliance
- **Customer Data Management**: Automatic extraction and storage of party information from TAP messages
- **Message Routing**: Intelligently route messages to the appropriate agent based on DID addressing
- **Concurrent Processing**: Scale to high throughput with worker pools for message processing
- **Event Publishing**: Comprehensive event system for monitoring and reacting to node activities
- **Flexible Message Delivery**: Send messages via HTTP or WebSockets with robust error handling
- **Cross-Platform Support**: Native and WASM environments for both HTTP and WebSocket transports
- **DID Resolution**: Resolve DIDs for message verification and routing
- **Configurable Components**: Customize node behavior with pluggable components
- **Thread-Safe Design**: Safely share the node across threads with appropriate synchronization
- **WASM Compatibility**: Optional WASM support for browser environments
- **Persistent Storage**: SQLite-based storage using async SQLx with comprehensive functionality:
  - Transaction tracking for Transfer and Payment messages
  - Complete audit trail of all incoming/outgoing messages
  - **Message delivery tracking** with status monitoring, retry counts, and error logging
  - **Customer profiles** with Schema.org JSON-LD format and IVMS101 caching
  - **Relationship tracking** for TAIP-9 compliance

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
┌─────────────────────────────────────────────────────────────────────┐
│                           TAP Node                                   │
├───────────────┬───────────────┬─────────────────┬───────────────────┤
│ Agent Registry│ Message Router│  Event Bus      │ AgentStorageManager│
├───────────────┼───────────────┼─────────────────┼───────────────────┤
│ Message       │ Processor Pool│  DID Resolver   │ Per-Agent Storage │
│ Processors    │               │                 │    Isolation      │
├───────────────┼───────────────┼─────────────────┼───────────────────┤
│ Travel Rule   │ Customer      │ Event Handlers  │ IVMS101 Data      │
│ Processor     │ Manager       │                 │ Generation        │
└───────────────┴───────────────┴─────────────────┴───────────────────┘
        │               │               │                       │
        ▼               ▼               ▼                       ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐ ┌─────────────────┐
│   TAP Agent   │ │   TAP Agent   │ │   TAP Agent   │ │ ~/.tap/{did}/   │
│    Agent A    │ │    Agent B    │ │    Agent C    │ │ transactions.db │
└───────────────┘ └───────────────┘ └───────────────┘ └─────────────────┘
        │               │               │                       │
        ▼               ▼               ▼                       ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐ ┌─────────────────┐
│   SQLite DB   │ │   SQLite DB   │ │   SQLite DB   │ │ Multi-Recipient │
│   Agent A     │ │   Agent B     │ │   Agent C     │ │ Message Delivery│
└───────────────┘ └───────────────┘ └───────────────┘ └─────────────────┘
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
        storage_path: None, // Uses default path for legacy centralized storage
        #[cfg(feature = "storage")]
        agent_did: None, // Optional: primary agent DID for initialization
        #[cfg(feature = "storage")]
        tap_root: None, // Uses default ~/.tap for per-agent storage
    };

    // Create a new node
    let mut node = TapNode::new(config);

    // Initialize storage (if enabled)
    #[cfg(feature = "storage")]
    node.init_storage().await?;

    // Start processor pool for high throughput
    let pool_config = tap_node::message::processor_pool::ProcessorPoolConfig {
        workers: 4,
        channel_capacity: 100,
        worker_timeout: Duration::from_secs(30),
    };
    node.start(pool_config).await?;

    // Create and register an agent
    let (agent, agent_did) = TapAgent::from_ephemeral_key().await?;
    node.register_agent(Arc::new(agent)).await?;

    // Agent registration automatically initializes per-agent storage
    // at ~/.tap/{sanitized_did}/transactions.db

    // The node is now ready to process messages and deliver them
    // to all recipients specified in each message's 'to' field

    Ok(())
}
```

### Processing Messages

```rust
use tap_msg::didcomm::PlainMessage;
use serde_json::json;

// Receive and process an incoming message
async fn handle_message(node: &TapNode, message: serde_json::Value) -> Result<(), tap_node::Error> {
    // Process through the node's pipeline
    // If the message has multiple recipients in the 'to' field,
    // it will be delivered to ALL of them
    node.receive_message(message).await?;
    Ok(())
}

// Send a message from one agent to multiple recipients
async fn send_message(node: &TapNode, from_did: &str, to_did: &str, message: PlainMessage) -> Result<String, tap_node::Error> {
    // Process and dispatch the message, returns the packed message
    // Storage will be logged to all involved agents' databases
    let packed = node.send_message(from_did, to_did, message).await?;
    Ok(packed)
}

// Example: Creating a multi-recipient message
async fn send_to_multiple_recipients(node: &TapNode) -> Result<(), tap_node::Error> {
    let message = PlainMessage {
        id: "msg-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "basic-message".to_string(),
        from: "did:example:sender".to_string(),
        to: vec![
            "did:example:recipient1".to_string(),
            "did:example:recipient2".to_string(),
            "did:example:recipient3".to_string(),
        ],
        body: json!({"content": "Hello everyone!"}),
        // ... other fields
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        // ... rest of fields
    };

    // This message will be delivered to all three recipients
    // and logged in each of their storage databases
    node.receive_message(serde_json::to_value(message)?).await?;
    Ok(())
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

TAP Node provides multiple options for sending messages between nodes with optional delivery tracking:

### HTTP Message Sender

For standard request-response communication patterns:

```rust
use tap_node::{HttpPlainMessageSender, PlainMessageSender};

// Create an HTTP sender with default settings
let sender = HttpPlainMessageSender::new("https://recipient-endpoint.example.com".to_string());

// Create with custom settings (timeout and retries)
let sender = HttpPlainMessageSender::with_options(
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

### HTTP Message Sender with Delivery Tracking

For HTTP delivery with comprehensive tracking and monitoring:

```rust
use tap_node::{HttpPlainMessageSenderWithTracking, PlainMessageSender, Storage};
use std::sync::Arc;

// Create storage for tracking deliveries
let storage = Arc::new(Storage::new(None).await?);

// Create a sender with delivery tracking
let sender = HttpPlainMessageSenderWithTracking::new(
    "https://recipient-endpoint.example.com".to_string(),
    storage
);

// Send message - delivery will be tracked automatically
sender.send(
    packed_message,
    vec!["did:example:recipient".to_string()]
).await?;

// Check delivery status later using storage queries
```

This sender automatically:
- Creates delivery records before sending with status `pending`
- Updates status to `success` or `failed` after delivery attempts
- Records HTTP status codes and error messages
- Tracks retry counts for future automatic retry processing

### WebSocket Message Sender

For real-time bidirectional communication:

```rust
use tap_node::{WebSocketPlainMessageSender, PlainMessageSender};

// Create a WebSocket sender with default settings
let sender = WebSocketPlainMessageSender::new("https://recipient-endpoint.example.com".to_string());

// Create with custom settings
let sender = WebSocketPlainMessageSender::with_options(
    "https://recipient-endpoint.example.com".to_string()
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

## Per-Agent Storage Architecture

The TAP Node features a sophisticated per-agent storage system using SQLite databases. This architecture provides complete data isolation between agents while ensuring all involved parties receive relevant transaction data.

1. **Transaction Storage**: Automatic storage of Transfer and Payment messages for business logic processing
2. **Message Audit Trail**: Complete logging of all incoming and outgoing messages for compliance and debugging
3. **Message Delivery Tracking**: Comprehensive tracking of message delivery attempts with status monitoring

## Message Delivery Tracking

TAP Node provides comprehensive delivery tracking for both external HTTP deliveries and internal agent-to-agent message routing. This feature enables monitoring, debugging, and automatic retry processing.

### Delivery Types

The system tracks four types of message delivery:

- **`https`**: HTTP/HTTPS delivery to external endpoints
- **`internal`**: Delivery to agents within the same TAP Node
- **`return_path`**: Return path delivery (future implementation)
- **`pickup`**: Pickup delivery (future implementation)

### Automatic Tracking

Delivery tracking is automatic for:

1. **HTTP External Deliveries**: When using `HttpPlainMessageSenderWithTracking`
2. **Internal Agent Deliveries**: When messages are processed by the node and delivered to registered agents
3. **Router-based Deliveries**: When messages are routed through the default router to fallback agents

### Tracking Information

Each delivery record includes:

- **Message identification**: Message ID and full message text
- **Recipient details**: Target DID and delivery URL (for HTTP)
- **Status tracking**: `pending`, `success`, or `failed`
- **Error information**: HTTP status codes and error messages
- **Retry tracking**: Count for automatic retry processing
- **Timestamps**: Creation, update, and delivery completion times

### Querying Delivery Status

```rust
use tap_node::storage::{DeliveryStatus, DeliveryType};

// Get storage from the node (agent-specific storage)
if let Some(storage_manager) = node.agent_storage_manager() {
    let agent_storage = storage_manager.get_agent_storage("did:example:agent").await?;

    // Get all deliveries for a specific message
    let deliveries = agent_storage.get_deliveries_for_message("msg_12345").await?;

    // Get pending deliveries for retry processing
    let pending = agent_storage.get_pending_deliveries(
        10,  // max retry count
        50   // limit
    ).await?;

    // Get failed deliveries for a recipient
    let failed = agent_storage.get_failed_deliveries_for_recipient(
        "did:example:recipient",
        20,  // limit
        0    // offset
    ).await?;

    // Get specific delivery by ID
    let delivery = agent_storage.get_delivery_by_id(123).await?;

    // Examine delivery details
    for delivery in deliveries {
        println!("Delivery {} to {} - Status: {:?} - Type: {:?}",
            delivery.id,
            delivery.recipient_did,
            delivery.status,
            delivery.delivery_type
        );

        if let Some(error) = delivery.error_message {
            println!("Error: {}", error);
        }

        if let Some(http_status) = delivery.last_http_status_code {
            println!("HTTP Status: {}", http_status);
        }
    }
}

### Storage Configuration

Configure per-agent storage when creating the node:

```rust
use tap_node::{NodeConfig, TapNode};
use std::path::PathBuf;

// Use default TAP root (~/.tap) for per-agent storage
let config = NodeConfig {
    #[cfg(feature = "storage")]
    tap_root: None, // Uses ~/.tap/
    #[cfg(feature = "storage")]
    agent_did: None, // Primary agent DID (optional)
    #[cfg(feature = "storage")]
    storage_path: None, // Legacy centralized storage (optional)
    ..Default::default()
};

// Or specify a custom TAP root directory
let config = NodeConfig {
    #[cfg(feature = "storage")]
    tap_root: Some(PathBuf::from("/custom/tap/root")),
    #[cfg(feature = "storage")]
    agent_did: Some("did:example:primary-agent".to_string()),
    ..Default::default()
};

// Agent storage locations:
// - Default: ~/.tap/{sanitized_did}/transactions.db
// - Custom: /custom/tap/root/{sanitized_did}/transactions.db
```

### Accessing Stored Data

```rust
use tap_node::storage::MessageDirection;

// Access agent-specific storage
if let Some(storage_manager) = node.agent_storage_manager() {
    // Get storage for a specific agent
    let agent_storage = storage_manager.get_agent_storage("did:example:agent").await?;

    // === Transaction Operations ===
    // Retrieve a specific transaction by message ID
    let transaction = agent_storage.get_transaction_by_id("msg_12345").await?;

    // List recent transactions with pagination
    let transactions = agent_storage.list_transactions(
        10,  // limit: 10 transactions
        0    // offset: 0 (first page)
    ).await?;

    // === Message Audit Trail Operations ===
    // Retrieve any message by ID
    let message = agent_storage.get_message_by_id("msg_12345").await?;

    // List all messages for this agent
    let all_messages = agent_storage.list_messages(
        20,   // limit
        0,    // offset
        None  // no direction filter (shows both incoming and outgoing)
    ).await?;

    // List only incoming messages
    let incoming = agent_storage.list_messages(
        10,
        0,
        Some(MessageDirection::Incoming)
    ).await?;

    // List only outgoing messages
    let outgoing = agent_storage.list_messages(
        10,
        0,
        Some(MessageDirection::Outgoing)
    ).await?;

    // Examine message details
    for msg in all_messages {
        println!("Agent Storage - Message: {} - Type: {} - Direction: {:?} - From: {:?} - To: {:?}",
            msg.message_id,
            msg.message_type,
            msg.direction,
            msg.from_did,
            msg.to_did
        );
    }
}

// Access legacy centralized storage (if available)
if let Some(storage) = node.storage() {
    // Same API as above, but accesses centralized storage
    let transactions = storage.list_transactions(10, 0).await?;
}
```

### Storage Features

- **Agent Isolation**: Each agent has its own dedicated SQLite database
- **Multi-Agent Transactions**: Transactions automatically stored in all involved agents' databases
- **Multi-Recipient Delivery**: Messages delivered to ALL recipients in the `to` field
- **Async Database Operations**: Built on SQLx for native async support
- **Automatic Migration**: Database schema is automatically created and migrated on startup
- **Dual-Table Design**: Separate tables for transactions and message audit trail per agent
- **Append-Only Design**: All data is immutable for compliance and auditing
- **SQLite WAL Mode**: Optimized for concurrent reads and writes
- **Connection Pooling**: SQLx connection pool for efficient database access per agent
- **JSON Column Support**: Message content stored as validated JSON
- **WASM Compatibility**: Storage is automatically disabled in WASM builds
- **Duplicate Handling**: Duplicate messages are silently ignored (idempotent)
- **Directory Management**: Automatic creation of agent-specific directories

### Database Schema

The storage system maintains six tables:

#### `transactions` Table
Business logic for Transfer and Payment messages:
- Transaction ID and type (Transfer/Payment)
- Sender and recipient DIDs
- Thread ID for conversation tracking
- Full message content stored in JSON column type
- Status tracking (pending/confirmed/failed/cancelled/reverted)
- Timestamps for creation and updates

#### `messages` Table
Complete audit trail of all messages:
- Message ID and type (all TAP message types)
- Direction (incoming/outgoing)
- Sender and recipient DIDs
- Thread IDs (including parent threads)
- Full message content stored in JSON column type
- Creation timestamp

#### `deliveries` Table
Message delivery tracking and monitoring:
- Delivery ID (auto-incrementing primary key)
- Message ID and full message text
- Recipient DID and delivery URL (for HTTP)
- Delivery type (`https`, `internal`, `return_path`, `pickup`)
- Status (`pending`, `success`, `failed`)
- Retry count for automatic retry processing
- HTTP status code and error message for debugging
- Timestamps for creation, updates, and delivery completion

#### `customers` Table
Customer profiles with Schema.org JSON-LD data:
- Customer ID (UUID or DID)
- Agent DID (owner of the customer record)
- Schema type (Person/Organization)
- Name fields (given, family, display, legal)
- Address fields (country, locality, postal, street)
- Profile data (full Schema.org JSON-LD)
- IVMS101 data (cached compliance data)
- Verification and timestamps

#### `customer_identifiers` Table
Multiple identifiers per customer:
- Identifier (DID, email, phone, URL, etc.)
- Customer ID reference
- Identifier type classification
- Verification status and method
- Creation timestamp

#### `customer_relationships` Table
TAIP-9 compliant relationship tracking:
- Relationship ID
- Customer ID reference
- Relationship type (controls, owns, manages)
- Related identifier
- Proof of relationship (JSON)
- Confirmation timestamp

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

## Travel Rule Support

The TAP Node includes comprehensive Travel Rule support through the Travel Rule Processor and Customer Manager:

### Automatic IVMS101 Attachment

When sending Transfer messages, the node automatically:
- Checks if IVMS101 data should be attached based on policies
- Generates IVMS101 data from customer profiles
- Attaches data as Verifiable Presentations
- Handles policy-based presentation requests

See [TRAVEL-RULE.md](./TRAVEL-RULE.md) for detailed documentation.

### Customer Data Management

The Customer Manager automatically:
- Extracts party information from TAP messages
- Creates/updates customer profiles
- Manages multiple identifiers per customer
- Tracks relationships between parties
- Generates IVMS101-compliant data

See [CUSTOMER-MANAGEMENT.md](./CUSTOMER-MANAGEMENT.md) for detailed documentation.

## Examples

The package includes several examples:

- `benches/stress_test.rs` - Benchmark of node performance with different message loads
- `examples/http_message_flow.rs` - Example of using HTTP for message delivery
- `examples/websocket_message_flow.rs` - Example of using WebSockets for real-time communication
- `examples/travel_rule_flow.rs` - Complete Travel Rule compliance example with IVMS101

Run examples with:

```bash
# Run with HTTP support
cargo run --example http_message_flow --features native

# Run with WebSocket support
cargo run --example websocket_message_flow --features websocket

# Run Travel Rule example
cargo run --example travel_rule_flow --features native
```

## License

This crate is licensed under the terms of the MIT license.
