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

use agent::AgentRegistry;
use event::EventBus;
use message::{
    CompositeMessageProcessor, DefaultMessageRouter, LoggingMessageProcessor,
    MessageProcessor, MessageRouter, ProcessorPool, ProcessorPoolConfig, ValidationMessageProcessor,
};
use resolver::NodeResolver;

use std::sync::Arc;
use tap_agent::Agent;
use tap_core::message::TapMessage;

/// Version of the TAP Node
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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

/// Main TAP Node implementation
pub struct TapNode {
    /// Configuration for the node
    config: NodeConfig,
    /// Registry of agents
    agents: Arc<AgentRegistry>,
    /// DID resolver for the node
    resolver: Arc<NodeResolver>,
    /// Event bus for node events
    event_bus: Arc<EventBus>,
    /// Message router
    router: Arc<dyn MessageRouter>,
    /// Message processor for incoming messages
    incoming_processor: Arc<dyn MessageProcessor>,
    /// Message processor for outgoing messages
    outgoing_processor: Arc<dyn MessageProcessor>,
    /// Processor pool for concurrent message processing
    processor_pool: Option<ProcessorPool<CompositeMessageProcessor>>,
}

impl TapNode {
    /// Create a new TAP Node with the provided configuration
    pub fn new(config: NodeConfig) -> Self {
        let agents = Arc::new(AgentRegistry::new(config.max_agents));
        let resolver = Arc::new(NodeResolver::new());
        let event_bus = Arc::new(EventBus::new());

        // Set up the router
        let router = Arc::new(DefaultMessageRouter::new(agents.clone()));

        // Set up the processors
        let mut incoming_processor = CompositeMessageProcessor::new();
        let mut outgoing_processor = CompositeMessageProcessor::new();

        // Add validation processor
        incoming_processor.add_processor(Box::new(ValidationMessageProcessor::new()));
        outgoing_processor.add_processor(Box::new(ValidationMessageProcessor::new()));

        // Add logging processor if enabled
        if config.enable_message_logging {
            incoming_processor.add_processor(Box::new(LoggingMessageProcessor::new(
                config.log_message_content,
            )));
            outgoing_processor.add_processor(Box::new(LoggingMessageProcessor::new(
                config.log_message_content,
            )));
        }

        let incoming_processor_arc = Arc::new(incoming_processor);
        let outgoing_processor_arc = Arc::new(outgoing_processor);

        // Set up the processor pool if configured
        let processor_pool = config.processor_pool.as_ref().map(|pool_config| {
            ProcessorPool::new(incoming_processor_arc.clone(), pool_config.clone())
        });

        Self {
            config,
            agents,
            resolver,
            event_bus,
            router: router as Arc<dyn MessageRouter>,
            incoming_processor: incoming_processor_arc,
            outgoing_processor: outgoing_processor_arc,
            processor_pool,
        }
    }

    /// Get the node configuration
    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    /// Get access to the agent registry
    pub fn agents(&self) -> Arc<AgentRegistry> {
        self.agents.clone()
    }

    /// Get access to the node resolver
    pub fn resolver(&self) -> Arc<NodeResolver> {
        self.resolver.clone()
    }

    /// Get access to the event bus
    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    /// Set a custom message router
    pub fn set_router(&mut self, router: Arc<dyn MessageRouter>) {
        self.router = router;
    }

    /// Set a custom incoming message processor
    pub fn set_incoming_processor(&mut self, processor: Arc<dyn MessageProcessor>) {
        self.incoming_processor = processor;
    }

    /// Set a custom outgoing message processor
    pub fn set_outgoing_processor(&mut self, processor: Arc<dyn MessageProcessor>) {
        self.outgoing_processor = processor;
    }

    /// Submit a message for concurrent processing
    pub async fn submit_message(&self, message: TapMessage) -> Result<()> {
        match &self.processor_pool {
            Some(pool) => pool.submit(message).await,
            None => {
                // If no processor pool is configured, process synchronously
                self.process_message(message).await?;
                Ok(())
            }
        }
    }

    /// Register a new agent with the node
    pub async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<()> {
        let agent_did = agent.did().to_string();
        self.agents.register_agent(agent_did.clone(), agent).await?;
        
        // Publish event about agent registration
        self.event_bus.publish_agent_registered(agent_did).await;
        
        Ok(())
    }

    /// Unregister an agent from the node
    pub async fn unregister_agent(&self, did: &str) -> Result<()> {
        self.agents.unregister_agent(did).await?;
        
        // Publish event about agent unregistration
        self.event_bus.publish_agent_unregistered(did.to_string()).await;
        
        Ok(())
    }

    /// Process and dispatch a message to an agent
    pub async fn process_message(&self, message: TapMessage) -> Result<()> {
        // Process the incoming message
        let processed_message = match self.incoming_processor.process_incoming(message).await? {
            Some(msg) => msg,
            None => return Ok(()), // Message was filtered out
        };

        // Route the message to determine the target agent
        let target_did = self.router.route_message(&processed_message).await?;

        // Dispatch the message to the agent
        self.dispatch_message(&target_did, processed_message).await
    }

    /// Dispatch a message to an agent by DID
    pub async fn dispatch_message(&self, target_did: &str, message: TapMessage) -> Result<()> {
        let agent = self.agents.get_agent(target_did).await?;
        
        // Convert the message to a packed format for transport
        let packed_message = serde_json::to_string(&message)
            .map_err(Error::Serialization)?;
        
        // Have the agent process the message
        let received = agent.receive_message(&packed_message).await
            .map_err(|e| Error::Agent(e.to_string()))?;
        
        // Publish event about received message
        self.event_bus.publish_message_received(received).await;
        
        Ok(())
    }

    /// Send a message from one agent to another
    pub async fn send_message(
        &self,
        from_did: &str,
        to_did: &str,
        message: TapMessage,
    ) -> Result<String> {
        // Process the outgoing message
        let processed_message = match self.outgoing_processor.process_outgoing(message).await? {
            Some(msg) => msg,
            None => return Err(Error::Dispatch("Message was filtered out".to_string())),
        };

        // Get the sender agent
        let sender = self.agents.get_agent(from_did).await?;
        
        // Pack and send the message
        let packed_message = sender.send_message(&processed_message, to_did).await
            .map_err(|e| Error::Agent(e.to_string()))?;
        
        // Publish event about sent message
        self.event_bus.publish_message_sent(processed_message, from_did.to_string(), to_did.to_string()).await;
        
        Ok(packed_message)
    }
}
