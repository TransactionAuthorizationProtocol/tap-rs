//! TAP Node implementation
//!
//! This crate provides a node implementation for the Transaction Authorization Protocol (TAP).
//! The TAP Node is responsible for managing multiple agents, routing messages, and coordinating
//! between different TAP agents.

pub mod agent;
pub mod error;
pub mod event;
pub mod message;
pub mod resolver;

pub use error::{Error, Result};

use std::sync::Arc;

use tap_agent::Agent;
use tap_core::didcomm::Message;

use agent::AgentRegistry;
use event::EventBus;
use message::{
    CompositeMessageProcessor, CompositeMessageRouter, MessageProcessorType, MessageRouterType,
};
use message::processor::{
    DefaultMessageProcessor, LoggingMessageProcessor, MessageProcessor, ValidationMessageProcessor
};
use message::processor_pool::{ProcessorPool, ProcessorPoolConfig};
use message::router::DefaultMessageRouter;
use message::RouterAsyncExt;
use resolver::NodeResolver;

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

/// The TAP Node
///
/// The TAP Node is responsible for coordinating message processing, routing, and delivery
/// to TAP Agents. It serves as a central hub for all TAP communications.
#[derive(Clone)]
pub struct TapNode {
    /// Configuration for the node
    config: NodeConfig,
    /// Registry of agents
    agents: Arc<AgentRegistry>,
    /// Event bus for notifications
    event_bus: Arc<EventBus>,
    /// Router for determining message recipients
    router: CompositeMessageRouter,
    /// Processor for incoming messages
    incoming_processor: CompositeMessageProcessor,
    /// Processor for outgoing messages
    outgoing_processor: CompositeMessageProcessor,
    /// Processor pool for concurrent message processing
    processor_pool: Option<ProcessorPool>,
    /// Node resolver for DID resolution
    resolver: Arc<NodeResolver>,
}

impl TapNode {
    /// Create a new TAP Node with the provided configuration
    pub fn new(config: NodeConfig) -> Self {
        let agents = Arc::new(AgentRegistry::new(config.max_agents));
        let event_bus = Arc::new(EventBus::new());
        let resolver = Arc::new(NodeResolver::new());

        // Create default router
        let default_router = MessageRouterType::Default(DefaultMessageRouter::new(Arc::clone(&agents)));
        let router = CompositeMessageRouter::new(vec![default_router]);

        // Create default processors
        let logging_processor = MessageProcessorType::Logging(LoggingMessageProcessor);
        let validation_processor = MessageProcessorType::Validation(ValidationMessageProcessor);
        let default_processor = MessageProcessorType::Default(DefaultMessageProcessor);

        // Create incoming and outgoing processors
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

        Self {
            config,
            agents,
            event_bus,
            router,
            incoming_processor,
            outgoing_processor,
            processor_pool: None,
            resolver,
        }
    }

    /// Get the node configuration
    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    /// Get a reference to the agent registry
    pub fn agents(&self) -> &AgentRegistry {
        &self.agents
    }

    /// Get a reference to the event bus
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Get a reference to the resolver
    pub fn resolver(&self) -> &NodeResolver {
        &self.resolver
    }

    /// Configure the processor pool for the tap node
    pub fn configure_processor_pool(&mut self, config: ProcessorPoolConfig) -> Result<()> {
        self.processor_pool = Some(ProcessorPool::new(config));
        Ok(())
    }

    /// Set the router for the node
    pub fn set_router(&mut self, router: MessageRouterType) {
        self.router.add_router(router);
    }

    /// Add a processor to the incoming processor chain
    pub fn add_incoming_processor(&mut self, processor: MessageProcessorType) {
        self.incoming_processor.add_processor(processor);
    }

    /// Add a processor to the outgoing processor chain
    pub fn add_outgoing_processor(&mut self, processor: MessageProcessorType) {
        self.outgoing_processor.add_processor(processor);
    }

    /// Submit a message for concurrent processing
    pub async fn submit_message(&self, message: Message) -> Result<()> {
        match &self.processor_pool {
            Some(pool) => {
                pool.submit(message).await?;
                Ok(())
            }
            None => {
                self.process_message(message).await?;
                Ok(())
            }
        }
    }

    /// Process and dispatch a message to an agent
    pub async fn process_message(&self, message: Message) -> Result<()> {
        // Process the incoming message
        let processed_message = match self.incoming_processor.process_incoming(message).await? {
            Some(msg) => msg,
            None => return Ok(()), // Message was dropped
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
        let packed = agent.send_serialized_message(&processed_message, to_did).await?;

        // Publish an event for the sent message
        self.event_bus
            .publish_message_sent(processed_message, from_did.to_string(), to_did.to_string())
            .await;

        Ok(packed)
    }

    /// Register a new agent with the node
    pub async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<()> {
        let agent_did = agent.get_agent_did().to_string();
        self.agents.register_agent(agent_did.clone(), agent).await?;

        // Publish event about agent registration
        self.event_bus.publish_agent_registered(agent_did).await;

        Ok(())
    }

    /// Unregister an agent from the node
    pub async fn unregister_agent(&self, did: &str) -> Result<()> {
        self.agents.unregister_agent(did).await?;

        // Publish event about agent unregistration
        self.event_bus
            .publish_agent_unregistered(did.to_string())
            .await;

        Ok(())
    }
}
