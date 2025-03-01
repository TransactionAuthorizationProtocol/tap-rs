# TAP-Node

TAP-Node is a Rust implementation of a Transaction Authorization Protocol (TAP) node, responsible for managing multiple agents, handling message routing, and coordinating between different TAP agents.

## Features

- **Agent Management**: Register and manage multiple TAP agents
- **Message Routing**: Forward messages to appropriate agents based on DID
- **Message Processing**: Validate, log, and process messages 
- **Event Handling**: Publish and subscribe to TAP events
- **DID Resolution**: Resolve DIDs using multiple resolver methods
- **Concurrency**: Efficient asynchronous message processing

## Usage

```rust
use std::sync::Arc;
use tap_agent::{Agent, AgentConfig, TapAgent};
use tap_node::{NodeConfig, TapNode};

// Create a TAP Node with default configuration
let node_config = NodeConfig::default();
let node = TapNode::new(node_config);

// Create and register an agent
let agent_config = AgentConfig::new()
    .with_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
let agent = Arc::new(TapAgent::with_config(agent_config).unwrap());

// Register the agent with the node
node.register_agent(agent.clone()).await.unwrap();

// Process a message
let message = tap_core::message::TapMessageBuilder::new()
    .id("test-message-id")
    .message_type(tap_core::message::TapMessageType::TransactionProposal)
    .from_did(Some("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH".to_string()))
    .to_did(Some("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string()))
    .build()
    .unwrap();

// Message will be routed to the appropriate agent
let result = node.process_message(message).await;
```

## Architecture

TAP-Node implements a modular, extensible architecture:

- **Message Router**: Determines which agent should receive a message
- **Message Processor**: Pre-processes messages before routing 
- **Processor Pool**: Concurrently processes messages using a worker pool
- **Event Bus**: Provides publish-subscribe mechanism for TAP events
- **Agent Registry**: Manages registered agents and their capabilities
- **Node Resolver**: Resolves DIDs using various resolver methods

## Performance

The node is designed for high throughput, with benchmarks showing processing capabilities of:
- 166,000+ messages per second for small batches (10 messages)
- 400,000+ messages per second for larger batches (1000 messages)

## Testing

Run tests using:

```bash
cargo test -p tap-node
```

Run benchmarks using:

```bash
cargo bench --bench stress_test -p tap-node
```
