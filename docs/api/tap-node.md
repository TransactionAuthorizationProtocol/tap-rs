# tap-node API Reference

The `tap-node` crate provides a node implementation for routing and processing TAP messages. A TAP node can register multiple agents, route messages between them, and apply various message processors.

## Core Types

### `Node`

The main Node implementation for TAP.

```rust
pub struct Node {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new Node with the given configuration
pub fn new(config: NodeConfig) -> Self;

/// Register an agent with this node
pub async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<(), Error>;

/// Unregister an agent from this node
pub async fn unregister_agent(&self, did: &str) -> Result<(), Error>;

/// Find an agent by DID
pub async fn find_agent(&self, did: &str) -> Option<Arc<dyn Agent>>;

/// Receive a message for processing
pub async fn receive(&self, message: Message) -> Result<(), Error>;

/// Register a message handler for a specific DID
pub async fn register_message_handler<F>(&self, did: String, handler: F) -> Result<(), Error>
where
    F: Fn(Message) -> Result<(), Error> + Send + Sync + 'static;

/// Add a message processor to the node's processing pipeline
pub fn add_processor(&self, processor: MessageProcessorType) -> Result<(), Error>;

/// Add a message router to the node's routing pipeline
pub fn add_router(&self, router: MessageRouterType) -> Result<(), Error>;
```

### `NodeConfig`

Configuration options for creating a Node.

```rust
pub struct NodeConfig {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new default NodeConfig
pub fn new() -> Self;

/// Set the maximum number of agents this node can register
pub fn with_max_agents(mut self, max_agents: usize) -> Self;

/// Enable message logging
pub fn with_logging(mut self, enable: bool) -> Self;

/// Set the default message processor for this node
pub fn with_processor(mut self, processor: MessageProcessorType) -> Self;

/// Set the default message router for this node
pub fn with_router(mut self, router: MessageRouterType) -> Self;

/// Configure the node to use the composite message processor
pub fn with_composite_processor(mut self) -> Self;

/// Configure the node to use the composite message router
pub fn with_composite_router(mut self) -> Self;
```

### `DefaultNode`

The standard implementation of Node.

```rust
pub struct DefaultNode {
    // Internal implementation details
}
```

This type implements the primary node functionality and is the concrete type returned by `Node::new()`.

## Message Processing

### `MessageProcessor`

A trait for processing TAP messages.

```rust
pub trait MessageProcessor: Send + Sync + Clone {
    /// Process an incoming message and optionally return a transformed message
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>, Error>;
    
    /// Process an outgoing message and optionally return a transformed message
    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>, Error>;
}
```

### `MessageProcessorType`

An enum representing different message processor implementations.

```rust
pub enum MessageProcessorType {
    Default(DefaultMessageProcessor),
    Logging(LoggingMessageProcessor),
    Validation(ValidationMessageProcessor),
    Composite(CompositeMessageProcessor),
}
```

### `DefaultMessageProcessor`

The default message processor implementation.

```rust
pub struct DefaultMessageProcessor {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new DefaultMessageProcessor
pub fn new() -> Self;
```

### `LoggingMessageProcessor`

A message processor that logs messages.

```rust
pub struct LoggingMessageProcessor {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new LoggingMessageProcessor
pub fn new() -> Self;
```

### `ValidationMessageProcessor`

A message processor that validates messages.

```rust
pub struct ValidationMessageProcessor {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new ValidationMessageProcessor
pub fn new() -> Self;
```

### `CompositeMessageProcessor`

A message processor that applies multiple processors in sequence.

```rust
pub struct CompositeMessageProcessor {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new empty CompositeMessageProcessor
pub fn new() -> Self;

/// Add a processor to the composite
pub fn add_processor(&mut self, processor: MessageProcessorType) -> Result<(), Error>;
```

## Message Routing

### `MessageRouter`

A trait for routing messages to their destinations.

```rust
pub trait MessageRouter: Send + Sync + Clone {
    /// Route a message to its destination
    async fn route(&self, message: Message) -> Result<(), Error>;
}
```

### `MessageRouterType`

An enum representing different message router implementations.

```rust
pub enum MessageRouterType {
    Default(DefaultMessageRouter),
    Logging(LoggingMessageRouter),
    Http(HttpMessageRouter),
    Composite(CompositeMessageRouter),
}
```

### `DefaultMessageRouter`

The default message router implementation.

```rust
pub struct DefaultMessageRouter {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new DefaultMessageRouter
pub fn new() -> Self;
```

### `LoggingMessageRouter`

A message router that logs routing decisions.

```rust
pub struct LoggingMessageRouter {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new LoggingMessageRouter
pub fn new() -> Self;
```

### `HttpMessageRouter`

A message router that routes messages over HTTP.

```rust
pub struct HttpMessageRouter {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new HttpMessageRouter with the given base URL
pub fn new(base_url: String) -> Self;
```

### `CompositeMessageRouter`

A message router that applies multiple routers.

```rust
pub struct CompositeMessageRouter {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new empty CompositeMessageRouter
pub fn new() -> Self;

/// Add a router to the composite
pub fn add_router(&mut self, router: MessageRouterType) -> Result<(), Error>;
```

## Agent Management

### `AgentRegistry`

Manages the agents registered with a node.

```rust
pub struct AgentRegistry {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new AgentRegistry with an optional maximum number of agents
pub fn new(max_agents: Option<usize>) -> Self;

/// Register an agent with this registry
pub fn register(&self, agent: Arc<dyn Agent>) -> Result<(), Error>;

/// Unregister an agent from this registry
pub fn unregister(&self, did: &str) -> Result<(), Error>;

/// Find an agent by DID
pub fn find(&self, did: &str) -> Option<Arc<dyn Agent>>;

/// Get the number of registered agents
pub fn count(&self) -> usize;
```

## Error Handling

```rust
pub enum Error {
    /// Error from the core TAP library
    Core(tap_core::error::Error),
    
    /// Error from the agent library
    Agent(tap_agent::Error),
    
    /// Error related to message processing
    MessageProcessing(String),
    
    /// Error related to message routing
    MessageRouting(String),
    
    /// Error related to agent management
    AgentManagement(String),
    
    /// General error
    General(String),
}
```

## Examples

### Creating a Node

```rust
use tap_node::{Node, NodeConfig};

fn create_node() -> Node {
    // Create a configuration
    let config = NodeConfig::new()
        .with_max_agents(100)
        .with_logging(true)
        .with_composite_processor()
        .with_composite_router();
    
    // Create the node
    let node = Node::new(config);
    
    node
}
```

### Registering Agents with a Node

```rust
use tap_node::Node;
use tap_agent::{Agent, AgentConfig};
use tap_core::did::KeyPair;
use std::sync::Arc;

async fn register_agents(node: &Node) -> Result<(), Box<dyn std::error::Error>> {
    // Create the first agent
    let key_pair1 = KeyPair::generate_ed25519().await?;
    let agent1 = Agent::new(
        AgentConfig::new().with_name("Alice"),
        Arc::new(key_pair1),
    )?;
    
    // Create the second agent
    let key_pair2 = KeyPair::generate_ed25519().await?;
    let agent2 = Agent::new(
        AgentConfig::new().with_name("Bob"),
        Arc::new(key_pair2),
    )?;
    
    // Register the agents with the node
    node.register_agent(Arc::new(agent1)).await?;
    node.register_agent(Arc::new(agent2)).await?;
    
    Ok(())
}
```

### Custom Message Processor

```rust
use tap_node::message::{MessageProcessor, DefaultMessageProcessor};
use didcomm::Message;
use async_trait::async_trait;

#[derive(Clone)]
struct AuditMessageProcessor {
    inner: DefaultMessageProcessor,
}

impl AuditMessageProcessor {
    fn new() -> Self {
        Self {
            inner: DefaultMessageProcessor::new(),
        }
    }
}

#[async_trait]
impl MessageProcessor for AuditMessageProcessor {
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>, tap_node::Error> {
        // Log the incoming message for audit purposes
        println!("AUDIT: Incoming message with ID: {}", message.id);
        
        // Delegate to the inner processor
        self.inner.process_incoming(message).await
    }
    
    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>, tap_node::Error> {
        // Log the outgoing message for audit purposes
        println!("AUDIT: Outgoing message with ID: {}", message.id);
        
        // Delegate to the inner processor
        self.inner.process_outgoing(message).await
    }
}
```

### Processing Messages with a Node

```rust
use tap_node::Node;
use didcomm::Message;

async fn process_message(
    node: &Node,
    message: Message,
) -> Result<(), Box<dyn std::error::Error>> {
    // Process the message through the node
    node.receive(message).await?;
    
    Ok(())
}
```

### Setting Up a Complex Node

```rust
use tap_node::{
    Node, 
    NodeConfig,
    message::{
        MessageProcessorType,
        LoggingMessageProcessor,
        ValidationMessageProcessor,
        CompositeMessageProcessor
    },
    router::{
        MessageRouterType,
        LoggingMessageRouter,
        HttpMessageRouter,
        CompositeMessageRouter
    }
};

fn setup_complex_node() -> Node {
    // Create processor components
    let logging_processor = MessageProcessorType::Logging(LoggingMessageProcessor::new());
    let validation_processor = MessageProcessorType::Validation(ValidationMessageProcessor::new());
    
    // Create router components
    let logging_router = MessageRouterType::Logging(LoggingMessageRouter::new());
    let http_router = MessageRouterType::Http(HttpMessageRouter::new("https://example.com/tap".to_string()));
    
    // Create composite processor
    let mut composite_processor = CompositeMessageProcessor::new();
    composite_processor.add_processor(logging_processor).unwrap();
    composite_processor.add_processor(validation_processor).unwrap();
    
    // Create composite router
    let mut composite_router = CompositeMessageRouter::new();
    composite_router.add_router(logging_router).unwrap();
    composite_router.add_router(http_router).unwrap();
    
    // Create node config
    let config = NodeConfig::new()
        .with_processor(MessageProcessorType::Composite(composite_processor))
        .with_router(MessageRouterType::Composite(composite_router));
    
    // Create the node
    Node::new(config)
}
```
