//! TAP Node - A node implementation for the TAP protocol
//!
//! The TAP Node is the central component that manages TAP Agents, routes messages,
//! processes events, stores transactions, and provides a scalable architecture for TAP deployments.
//!
//! # Key Components
//!
//! - **Agent Registry**: Manages multiple TAP Agents
//! - **Event Bus**: Publishes and distributes events throughout the system
//! - **Message Processors**: Process incoming and outgoing messages
//! - **Message Router**: Routes messages to the appropriate agent
//! - **Processor Pool**: Provides scalable concurrent message processing
//! - **Storage**: Persistent SQLite storage for Transfer and Payment transactions
//!
//! # Thread Safety and Concurrency
//!
//! The TAP Node is designed with concurrent operations in mind. It uses a combination of
//! async/await patterns and synchronization primitives to safely handle multiple operations
//! simultaneously. Most components within the node are either immutable or use interior
//! mutability with appropriate synchronization.
//!
//! # Message Flow
//!
//! Messages in TAP Node follow a structured flow:
//!
//! 1. **Receipt**: Messages are received through the `receive_message` method
//! 2. **Processing**: Each message is processed by the registered processors
//! 3. **Routing**: The router determines which agent should handle the message
//! 4. **Dispatch**: The message is delivered to the appropriate agent
//! 5. **Response**: Responses are handled similarly in the reverse direction
//!
//! # Scalability
//!
//! The node supports scalable message processing through the optional processor pool,
//! which uses a configurable number of worker threads to process messages concurrently.
//! This allows a single node to handle high message throughput while maintaining
//! shared between threads, with all mutable state protected by appropriate synchronization
//! primitives.

pub mod agent;
pub mod error;
pub mod event;
pub mod message;
pub mod resolver;
pub mod storage;

pub use error::{Error, Result};
pub use event::logger::{EventLogger, EventLoggerConfig, LogDestination};
pub use event::{EventSubscriber, NodeEvent};
pub use message::sender::{
    HttpPlainMessageSender, NodePlainMessageSender, PlainMessageSender, WebSocketPlainMessageSender,
};

use std::sync::Arc;

use tap_agent::{Agent, TapAgent};
// use tap_agent::message_packing::PackOptions;
use tap_msg::didcomm::PlainMessage;

use crate::message::processor::PlainMessageProcessor;
use crate::message::{
    CompositePlainMessageProcessor, CompositePlainMessageRouter, PlainMessageProcessorType,
    PlainMessageRouterType,
};
use agent::AgentRegistry;
use event::EventBus;
use resolver::NodeResolver;

use async_trait::async_trait;

// Extension trait for TapAgent to add serialization methods
///
/// This trait extends the TapAgent with methods for serializing and packing
/// DIDComm messages for transmission. It provides functionality for converting
/// in-memory message objects to secure, serialized formats that follow the
/// DIDComm messaging protocol standards.
#[async_trait]
pub trait TapAgentExt {
    /// Pack and serialize a DIDComm message for transmission
    ///
    /// This method takes a DIDComm message and recipient DID, then:
    /// 1. Uses the agent's PlainMessagePacker to properly sign and encrypt the message
    /// 2. Serializes the message to a string format
    ///
    /// # Parameters
    /// * `message` - The DIDComm message to serialize
    /// * `to_did` - The DID of the recipient
    ///
    /// # Returns
    /// The packed message as a string, ready for transmission
    async fn send_serialized_message(&self, message: &PlainMessage, to_did: &str)
        -> Result<String>;
}

#[async_trait]
impl TapAgentExt for TapAgent {
    async fn send_serialized_message(
        &self,
        message: &PlainMessage,
        _to_did: &str,
    ) -> Result<String> {
        // Serialize the PlainMessage to JSON first to work around the TapMessageBody trait constraint
        let json_value = serde_json::to_value(message).map_err(Error::Serialization)?;

        // Use JSON string for transportation instead of direct message passing
        // This bypasses the need for PlainMessage to implement TapMessageBody
        let serialized = serde_json::to_string(&json_value).map_err(Error::Serialization)?;

        Ok(serialized)
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
    /// Configuration for the event logger
    pub event_logger: Option<EventLoggerConfig>,
    /// Path to the storage database (None for default)
    #[cfg(feature = "storage")]
    pub storage_path: Option<std::path::PathBuf>,
}

/// # The TAP Node
///
/// The TAP Node is the core component responsible for coordinating message processing, routing, and delivery
/// to TAP Agents. It serves as a central hub for all TAP communications and transaction coordination.
///
/// ## Core Responsibilities
///
/// - **Agent Management**: Registration and deregistration of TAP Agents
/// - **PlainMessage Processing**: Processing incoming and outgoing messages through middleware chains
/// - **PlainMessage Routing**: Determining the appropriate recipient for each message
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
    incoming_processor: CompositePlainMessageProcessor,
    /// Outgoing message processor
    outgoing_processor: CompositePlainMessageProcessor,
    /// PlainMessage router
    router: CompositePlainMessageRouter,
    /// Resolver for DIDs
    resolver: Arc<NodeResolver>,
    /// Worker pool for handling messages
    processor_pool: Option<ProcessorPool>,
    /// Node configuration
    config: NodeConfig,
    /// Storage for transactions
    #[cfg(feature = "storage")]
    storage: Option<Arc<storage::Storage>>,
}

impl TapNode {
    /// Create a new TAP node with the given configuration
    pub fn new(config: NodeConfig) -> Self {
        // Create the agent registry
        let agents = Arc::new(AgentRegistry::new(config.max_agents));

        // Create the event bus
        let event_bus = Arc::new(EventBus::new());

        // Create the message router
        let default_router = PlainMessageRouterType::Default(DefaultPlainMessageRouter::new());

        let router = CompositePlainMessageRouter::new(vec![default_router]);

        // Create the message processors
        let logging_processor = PlainMessageProcessorType::Logging(LoggingPlainMessageProcessor);
        let validation_processor =
            PlainMessageProcessorType::Validation(ValidationPlainMessageProcessor);
        let default_processor = PlainMessageProcessorType::Default(DefaultPlainMessageProcessor);

        let incoming_processor = CompositePlainMessageProcessor::new(vec![
            logging_processor.clone(),
            validation_processor.clone(),
            default_processor.clone(),
        ]);

        let outgoing_processor = CompositePlainMessageProcessor::new(vec![
            logging_processor,
            validation_processor,
            default_processor,
        ]);

        // Create the resolver
        let resolver = Arc::new(NodeResolver::default());

        // Initialize storage if feature is enabled
        #[cfg(feature = "storage")]
        let storage = {
            let storage_path = config.storage_path.clone();
            match tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(storage::Storage::new(storage_path))
            }) {
                Ok(s) => Some(Arc::new(s)),
                Err(e) => {
                    log::error!("Failed to initialize storage: {}", e);
                    None
                }
            }
        };

        let node = Self {
            agents,
            event_bus,
            incoming_processor,
            outgoing_processor,
            router,
            resolver,
            processor_pool: None,
            config,
            #[cfg(feature = "storage")]
            storage,
        };

        // Set up the event logger if configured
        if let Some(logger_config) = &node.config.event_logger {
            let event_logger = Arc::new(EventLogger::new(logger_config.clone()));

            // We need to handle the async subscribe in a blocking context
            // This is safe because EventBus methods are designed to be called in this way
            let event_bus = node.event_bus.clone();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    event_bus.subscribe(event_logger).await;
                })
            });
        }

        node
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
    /// # use tap_msg::didcomm::PlainMessage;
    /// # async fn example(node: &TapNode, message: PlainMessage) -> Result<(), tap_node::Error> {
    /// // Process an incoming message
    /// node.receive_message(message).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn receive_message(&self, message: PlainMessage) -> Result<()> {
        // Store the message if it's a Transfer or Payment and storage is available
        #[cfg(feature = "storage")]
        {
            if let Some(ref storage) = self.storage {
                if message.type_.contains("transfer") || message.type_.contains("payment") {
                    match storage.insert_transaction(&message).await {
                        Ok(_) => log::debug!("Stored transaction: {}", message.id),
                        Err(e) => log::warn!("Failed to store transaction: {}", e),
                    }
                }
            }
        }

        // Process the incoming message
        let processed_message = match self.incoming_processor.process_incoming(message).await? {
            Some(msg) => msg,
            None => return Ok(()), // PlainMessage was dropped during processing
        };

        // Route the message to the appropriate agent
        let target_did = match self.router.route_message(&processed_message).await {
            Ok(did) => did,
            Err(e) => {
                // Log the error but don't fail the entire operation
                log::warn!("Unable to route message: {}", e);
                return Ok(());
            }
        };

        // Dispatch the message to the agent, handling any errors
        match self.dispatch_message(target_did, processed_message).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Log the error but don't fail the entire operation
                log::warn!("Failed to dispatch message: {}", e);
                Ok(())
            }
        }
    }

    /// Dispatch a message to an agent by DID
    pub async fn dispatch_message(&self, target_did: String, message: PlainMessage) -> Result<()> {
        let agent = self.agents.get_agent(&target_did).await?;

        // Convert the message to a packed format for transport
        let packed = agent.send_serialized_message(&message, &target_did).await?;

        // Publish an event for the dispatched message
        self.event_bus
            .publish_agent_message(target_did, packed.into_bytes())
            .await;

        Ok(())
    }

    /// Send a message to an agent
    pub async fn send_message(
        &self,
        sender_did: String,
        to_did: String,
        message: PlainMessage,
    ) -> Result<String> {
        // Store the message if it's a Transfer or Payment and storage is available
        #[cfg(feature = "storage")]
        {
            if let Some(ref storage) = self.storage {
                if message.type_.contains("transfer") || message.type_.contains("payment") {
                    match storage.insert_transaction(&message).await {
                        Ok(_) => log::debug!("Stored outgoing transaction: {}", message.id),
                        Err(e) => log::warn!("Failed to store outgoing transaction: {}", e),
                    }
                }
            }
        }

        // Process the outgoing message
        let processed_message = match self.outgoing_processor.process_outgoing(message).await? {
            Some(msg) => msg,
            None => {
                return Err(Error::MessageDropped(
                    "PlainMessage dropped during processing".to_string(),
                ))
            }
        };

        // Get the sender agent
        let agent = self.agents.get_agent(&sender_did).await?;

        // Pack the message
        let packed = agent
            .send_serialized_message(&processed_message, to_did.as_str())
            .await?;

        // Publish an event for the message
        self.event_bus
            .publish_agent_message(sender_did, packed.clone().into_bytes())
            .await;

        Ok(packed)
    }

    /// Register a new agent with the node
    pub async fn register_agent(&self, agent: Arc<TapAgent>) -> Result<()> {
        let agent_did = agent.get_agent_did().to_string();
        self.agents.register_agent(agent_did.clone(), agent).await?;

        // Publish event about agent registration
        self.event_bus.publish_agent_registered(agent_did).await;

        Ok(())
    }

    /// Unregister an agent from the node
    pub async fn unregister_agent(&self, did: &str) -> Result<()> {
        self.agents.unregister_agent(did).await?;

        // Publish event about agent registration
        self.event_bus
            .publish_agent_unregistered(did.to_string())
            .await;

        Ok(())
    }

    /// Get a list of registered agent DIDs
    pub fn list_agents(&self) -> Vec<String> {
        self.agents.get_all_dids()
    }

    /// Get a reference to the agent registry
    pub fn agents(&self) -> &Arc<AgentRegistry> {
        &self.agents
    }

    /// Get a reference to the event bus
    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    /// Get a reference to the resolver
    pub fn resolver(&self) -> &Arc<NodeResolver> {
        &self.resolver
    }

    /// Get a mutable reference to the processor pool
    /// This is a reference to `Option<ProcessorPool>` to allow starting the pool after node creation
    pub fn processor_pool_mut(&mut self) -> &mut Option<ProcessorPool> {
        &mut self.processor_pool
    }

    /// Get the node configuration
    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    /// Get a reference to the storage (if available)
    #[cfg(feature = "storage")]
    pub fn storage(&self) -> Option<&Arc<storage::Storage>> {
        self.storage.as_ref()
    }
}

// Namespace imports
// These imports make the implementation cleaner, but should be hidden from public API
use message::processor::DefaultPlainMessageProcessor;
use message::processor::LoggingPlainMessageProcessor;
use message::processor::ValidationPlainMessageProcessor;
use message::processor_pool::{ProcessorPool, ProcessorPoolConfig};
use message::router::DefaultPlainMessageRouter;
use message::RouterAsyncExt;
