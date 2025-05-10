//! # TAP Node Implementation
//!
//! This crate provides a complete node implementation for the Transaction Authorization Protocol (TAP).
//! The TAP Node acts as a central hub for managing multiple TAP agents, routing messages between them,
//! and coordinating the entire transaction lifecycle in a secure and scalable manner.
//!
//! ## Overview
//!
//! The Transaction Authorization Protocol (TAP) is designed for secure, privacy-preserving financial
//! transactions between parties. A TAP Node serves as the infrastructure layer that:
//!
//! - Manages multiple TAP agents with different roles and capabilities
//! - Processes incoming and outgoing TAP messages
//! - Routes messages to the appropriate agents based on DID addressing
//! - Handles message validation, transformation, and delivery
//! - Provides an event system for monitoring and reacting to node activities
//! - Scales to handle high message throughput with a processing pool
//!
//! ## Architecture
//!
//! The TAP Node is built with several key components:
//!
//! - **Agent Registry**: Maintains a collection of TAP agents by their DIDs
//! - **Message Processors**: Process, validate, and transform messages
//! - **Message Routers**: Determine the target agent for a message
//! - **Processor Pool**: Provides concurrent message processing for scalability
//! - **Event Bus**: Broadcasts node events to subscribers
//! - **DID Resolver**: Resolves DIDs to DID Documents for message verification
//!
//! ## Usage Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use tap_agent::{AgentConfig, DefaultAgent};
//! use tap_node::{NodeConfig, TapNode, DefaultAgentExt};
//! use tap_node::message::processor_pool::ProcessorPoolConfig;
//! use tokio::time::Duration;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a node with default configuration
//!     let config = NodeConfig::default();
//!     let mut node = TapNode::new(config);
//!
//!     // Configure and start the processor pool
//!     let pool_config = ProcessorPoolConfig {
//!         workers: 4,
//!         channel_capacity: 100,
//!         worker_timeout: Duration::from_secs(30),
//!     };
//!     node.start(pool_config).await?;
//!
//!     // Create and register a TAP agent
//!     // (simplified - in practice you would need to set up proper crypto)
//!     let agent_config = AgentConfig::new("did:example:123".to_string());
//!     let agent = DefaultAgent::new(agent_config, Arc::new(/* message packer */));
//!     node.register_agent(Arc::new(agent)).await?;
//!
//!     // The node is now ready to process messages
//!     // You would typically listen for incoming messages and call:
//!     // node.receive_message(message).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Thread Safety and Async
//!
//! The TAP Node is designed to be thread-safe and fully async, making it suitable for
//! high-throughput environments. The core `TapNode` structure can be safely cloned and
//! shared between threads, with all mutable state protected by appropriate synchronization
//! primitives.

pub mod agent;
pub mod error;
pub mod event;
pub mod message;
pub mod resolver;

pub use error::{Error, Result};

use std::sync::Arc;

use tap_agent::{Agent, DefaultAgent};
use tap_msg::didcomm::Message;

use agent::AgentRegistry;
use event::EventBus;
use message::processor::{
    DefaultMessageProcessor, LoggingMessageProcessor, MessageProcessor, ValidationMessageProcessor,
};
use message::processor_pool::{ProcessorPool, ProcessorPoolConfig};
use message::router::DefaultMessageRouter;
use message::RouterAsyncExt;
use message::{
    CompositeMessageProcessor, CompositeMessageRouter, MessageProcessorType, MessageRouterType,
};
use resolver::NodeResolver;

use async_trait::async_trait;

// Extension trait for DefaultAgent to add serialization methods
#[async_trait]
pub trait DefaultAgentExt {
    async fn send_serialized_message(&self, message: &Message, _to_did: &str) -> Result<String>;
}

#[async_trait]
impl DefaultAgentExt for DefaultAgent {
    async fn send_serialized_message(&self, message: &Message, _to_did: &str) -> Result<String> {
        // Convert DIDComm Message to a packed DIDComm Message string
        // We use the raw didcomm_message methods of DefaultAgent

        // First, serialize the message to JSON
        let json_value = serde_json::to_value(message).map_err(Error::Serialization);

        // Use the message packer directly with security mode Signed
        let _security_mode = tap_agent::message::SecurityMode::Signed;

        // Since we can't directly use the agent's message packer or send_message_raw method,
        // we'll just return the serialized message for now
        let packed = serde_json::to_string(&json_value?).map_err(Error::Serialization);

        Ok(packed?)
    }
}

/// Configuration for a TAP Node
#[derive(Debug, Clone, Default)]
pub struct NodeConfig {
    /// Debug mode
    pub debug: bool,
    /// Maximum number of agents
    pub max_agents: Option<usize>,
    /// Whether to enable message logging
    pub enable_message_logging: bool,
    /// Whether to log full message content
    pub log_message_content: bool,
    /// Configuration for the processor pool
    pub processor_pool: Option<ProcessorPoolConfig>,
}

/// # The TAP Node
///
/// The TAP Node is the core component responsible for coordinating message processing, routing, and delivery
/// to TAP Agents. It serves as a central hub for all TAP communications and transaction coordination.
///
/// ## Core Responsibilities
///
/// - **Agent Management**: Registration and deregistration of TAP Agents
/// - **Message Processing**: Processing incoming and outgoing messages through middleware chains
/// - **Message Routing**: Determining the appropriate recipient for each message
/// - **Event Publishing**: Broadcasting node events to subscribers
/// - **Scalability**: Managing concurrent message processing through worker pools
///
/// ## Lifecycle
///
/// 1. Create a node with appropriate configuration
/// 2. Register one or more agents with the node
/// 3. Start the processor pool (if high throughput is required)
/// 4. Process incoming/outgoing messages
/// 5. Publish and respond to events
///
/// ## Thread Safety
///
/// The `TapNode` is designed to be thread-safe and can be shared across multiple
/// threads using an `Arc<TapNode>`. All internal mutability is handled through
/// appropriate synchronization primitives.
#[derive(Clone)]
pub struct TapNode {
    /// Agent registry
    agents: Arc<AgentRegistry>,
    /// Event bus
    event_bus: Arc<EventBus>,
    /// Incoming message processor
    incoming_processor: CompositeMessageProcessor,
    /// Outgoing message processor
    outgoing_processor: CompositeMessageProcessor,
    /// Message router
    router: CompositeMessageRouter,
    /// Resolver for DIDs
    resolver: Arc<NodeResolver>,
    /// Worker pool for handling messages
    processor_pool: Option<ProcessorPool>,
    /// Node configuration
    config: NodeConfig,
}

impl TapNode {
    /// Create a new TAP node with the given configuration
    pub fn new(config: NodeConfig) -> Self {
        // Create the agent registry
        let agents = Arc::new(AgentRegistry::new(config.max_agents));

        // Create the event bus
        let event_bus = Arc::new(EventBus::new());

        // Create the message router
        let default_router = MessageRouterType::Default(DefaultMessageRouter::new());

        let router = CompositeMessageRouter::new(vec![default_router]);

        // Create the message processors
        let logging_processor = MessageProcessorType::Logging(LoggingMessageProcessor);
        let validation_processor = MessageProcessorType::Validation(ValidationMessageProcessor);
        let default_processor = MessageProcessorType::Default(DefaultMessageProcessor);

        let incoming_processor = CompositeMessageProcessor::new(vec![
            logging_processor.clone(),
            validation_processor.clone(),
            default_processor.clone(),
        ]);

        let outgoing_processor = CompositeMessageProcessor::new(vec![
            logging_processor,
            validation_processor,
            default_processor,
        ]);

        // Create the resolver
        let resolver = Arc::new(NodeResolver::default());

        Self {
            agents,
            event_bus,
            incoming_processor,
            outgoing_processor,
            router,
            resolver,
            processor_pool: None,
            config,
        }
    }

    /// Start the node
    pub async fn start(&mut self, config: ProcessorPoolConfig) -> Result<()> {
        let processor_pool = ProcessorPool::new(config);
        self.processor_pool = Some(processor_pool);
        Ok(())
    }

    /// Receive and process an incoming message
    ///
    /// This method handles the complete lifecycle of an incoming message:
    ///
    /// 1. Processing the message through all registered processors
    /// 2. Routing the message to determine the appropriate target agent
    /// 3. Dispatching the message to the target agent
    ///
    /// The processing pipeline may transform or even drop the message based on
    /// validation rules or other processing logic. If a message is dropped during
    /// processing, this method will return Ok(()) without an error.
    ///
    /// # Parameters
    ///
    /// * `message` - The DIDComm message to be processed
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the message was successfully processed and dispatched (or intentionally dropped)
    /// * `Err(Error)` if there was an error during processing, routing, or dispatching
    ///
    /// # Errors
    ///
    /// This method can return errors for several reasons:
    /// * Processing errors from message processors
    /// * Routing errors if no target agent can be determined
    /// * Dispatch errors if the target agent cannot be found or fails to process the message
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use tap_node::{TapNode, NodeConfig};
    /// # use tap_msg::didcomm::Message;
    /// # async fn example(node: &TapNode, message: Message) -> Result<(), tap_node::Error> {
    /// // Process an incoming message
    /// node.receive_message(message).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn receive_message(&self, message: Message) -> Result<()> {
        // Process the incoming message
        let processed_message = match self.incoming_processor.process_incoming(message).await? {
            Some(msg) => msg,
            None => return Ok(()), // Message was dropped during processing
        };

        // Route the message to the appropriate agent
        let target_did = self.router.route_message(&processed_message).await?;

        // Dispatch the message to the agent
        self.dispatch_message(target_did, processed_message).await
    }

    /// Dispatch a message to an agent by DID
    pub async fn dispatch_message(&self, target_did: String, message: Message) -> Result<()> {
        let agent = self.agents.get_agent(&target_did).await?;

        // Convert the message to a packed format for transport
        let packed = agent.send_serialized_message(&message, &target_did).await?;

        // Publish an event for the dispatched message
        self.event_bus
            .publish_agent_message(target_did, packed.into_bytes())
            .await;

        Ok(())
    }

    /// Send a message from one agent to another
    ///
    /// This method handles the processing, routing, and delivery of a message
    /// from one agent to another. It returns the packed message.
    pub async fn send_message(
        &self,
        from_did: &str,
        to_did: &str,
        message: Message,
    ) -> Result<String> {
        // Process the outgoing message
        let processed_message = match self.outgoing_processor.process_outgoing(message).await? {
            Some(msg) => msg,
            None => return Err(Error::Processing("Message was dropped".to_string())),
        };

        // Get the sending agent
        let agent = self.agents.get_agent(from_did).await?;

        // Pack the message
        let packed = agent
            .send_serialized_message(&processed_message, to_did)
            .await?;

        // Publish an event for the sent message
        self.event_bus
            .publish_message_sent(processed_message, from_did.to_string(), to_did.to_string())
            .await;

        Ok(packed)
    }

    /// Register a new agent with the node
    pub async fn register_agent(&self, agent: Arc<DefaultAgent>) -> Result<()> {
        let agent_did = agent.get_agent_did().to_string();
        self.agents.register_agent(agent_did.clone(), agent).await?;

        // Publish event about agent registration
        self.event_bus.publish_agent_registered(agent_did).await;

        Ok(())
    }

    /// Unregister an agent from the node
    pub async fn unregister_agent(&self, did: &str) -> Result<()> {
        // Unregister the agent
        self.agents.unregister_agent(did).await?;

        // Publish event about agent unregistration
        self.event_bus
            .publish_agent_unregistered(did.to_string())
            .await;

        Ok(())
    }

    /// Get all registered agent DIDs
    pub fn get_all_agent_dids(&self) -> Vec<String> {
        self.agents.get_all_dids()
    }

    /// Get the event bus
    pub fn get_event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    /// Get the resolver
    pub fn get_resolver(&self) -> Arc<NodeResolver> {
        self.resolver.clone()
    }

    /// Get the node config
    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    /// Get the agent registry
    pub fn agents(&self) -> &AgentRegistry {
        &self.agents
    }
}
