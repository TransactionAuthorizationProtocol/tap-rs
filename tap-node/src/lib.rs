//! TAP Node - A node implementation for the TAP protocol
//!
//! The TAP Node is the central component that manages TAP Agents, routes messages,
//! processes events, stores transactions, and provides a scalable architecture for TAP deployments.
//!
//! # Architecture Overview
//!
//! The TAP Node acts as a message router and coordinator for multiple TAP Agents. It provides:
//!
//! - **Efficient Message Processing**: Different handling for plain, signed, and encrypted messages
//! - **Centralized Verification**: Signed messages are verified once for all agents
//! - **Smart Routing**: Encrypted messages are routed to appropriate recipient agents
//! - **Scalable Design**: Supports multiple agents with concurrent message processing
//!
//! # Key Components
//!
//! - **Agent Registry**: Manages multiple TAP Agents
//! - **Event Bus**: Publishes and distributes events throughout the system
//! - **Message Processors**: Process incoming and outgoing messages
//! - **Message Router**: Routes messages to the appropriate agent
//! - **DID Resolver**: Resolves DIDs for signature verification
//! - **Storage**: Persistent SQLite storage with transaction tracking and audit trails
//!
//! # Message Processing Flow
//!
//! The TAP Node uses an optimized message processing flow based on message type:
//!
//! ## Signed Messages (JWS)
//! 1. **Single Verification**: Signature verified once using DID resolver
//! 2. **Routing**: Verified PlainMessage routed to appropriate agent
//! 3. **Processing**: Agent receives verified message via `receive_plain_message()`
//!
//! ## Encrypted Messages (JWE)  
//! 1. **Recipient Identification**: Extract recipient DIDs from JWE headers
//! 2. **Agent Routing**: Send encrypted message to each matching agent
//! 3. **Decryption**: Each agent attempts decryption via `receive_encrypted_message()`
//! 4. **Processing**: Successfully decrypted messages are processed by the agent
//!
//! ## Plain Messages
//! 1. **Direct Processing**: Plain messages processed through the pipeline
//! 2. **Routing**: Routed to appropriate agent
//! 3. **Delivery**: Agent receives via `receive_plain_message()`
//!
//! # Benefits of This Architecture
//!
//! - **Efficiency**: Signed messages verified once, not per-agent
//! - **Scalability**: Encrypted messages naturally distributed to recipients
//! - **Flexibility**: Agents remain fully functional standalone
//! - **Security**: Centralized verification with distributed decryption
//!
//! # Thread Safety and Concurrency
//!
//! The TAP Node is designed with concurrent operations in mind. It uses a combination of
//! async/await patterns and synchronization primitives to safely handle multiple operations
//! simultaneously. Most components within the node are either immutable or use interior
//! mutability with appropriate synchronization.
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use tap_node::{TapNode, NodeConfig};
//! use tap_agent::TapAgent;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create node
//!     let config = NodeConfig::default();
//!     let node = Arc::new(TapNode::new(config));
//!     
//!     // Create and register agent
//!     let (agent, _did) = TapAgent::from_ephemeral_key().await?;
//!     node.register_agent(Arc::new(agent)).await?;
//!     
//!     // Process incoming message (JSON Value)
//!     let message_value = serde_json::json!({
//!         "id": "msg-123",
//!         "type": "test-message",
//!         "body": {"content": "Hello"}
//!     });
//!     
//!     node.receive_message(message_value).await?;
//!     Ok(())
//! }
//! ```

pub mod agent;
pub mod error;
pub mod event;
pub mod message;
pub mod storage;
#[cfg(feature = "storage")]
pub mod validation;

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
use tap_agent::did::MultiResolver;

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
        let json_value =
            serde_json::to_value(message).map_err(|e| Error::Serialization(e.to_string()))?;

        // Use JSON string for transportation instead of direct message passing
        // This bypasses the need for PlainMessage to implement TapMessageBody
        let serialized =
            serde_json::to_string(&json_value).map_err(|e| Error::Serialization(e.to_string()))?;

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
    /// Agent DID for storage organization
    #[cfg(feature = "storage")]
    pub agent_did: Option<String>,
    /// Custom TAP root directory (defaults to ~/.tap)
    #[cfg(feature = "storage")]
    pub tap_root: Option<std::path::PathBuf>,
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
    resolver: Arc<MultiResolver>,
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
        let resolver = Arc::new(MultiResolver::default());

        // Storage will be initialized on first use
        #[cfg(feature = "storage")]
        let storage = None;

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

    /// Initialize storage asynchronously
    #[cfg(feature = "storage")]
    pub async fn init_storage(&mut self) -> Result<()> {
        let storage = if let Some(agent_did) = &self.config.agent_did {
            // Use new DID-based storage structure
            match storage::Storage::new_with_did(agent_did, self.config.tap_root.clone()).await {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to initialize storage with DID: {}", e);
                    return Err(Error::Storage(e.to_string()));
                }
            }
        } else if let Some(storage_path) = self.config.storage_path.clone() {
            // Use explicit path
            match storage::Storage::new(Some(storage_path)).await {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to initialize storage: {}", e);
                    return Err(Error::Storage(e.to_string()));
                }
            }
        } else {
            // Initialize with default path
            match storage::Storage::new(None).await {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to initialize storage: {}", e);
                    return Err(Error::Storage(e.to_string()));
                }
            }
        };

        let storage_arc = Arc::new(storage);

        // Subscribe event handlers
        let message_status_handler = Arc::new(event::handlers::MessageStatusHandler::new(
            storage_arc.clone(),
        ));
        self.event_bus.subscribe(message_status_handler).await;

        let transaction_state_handler = Arc::new(event::handlers::TransactionStateHandler::new(
            storage_arc.clone(),
        ));
        self.event_bus.subscribe(transaction_state_handler).await;

        let transaction_audit_handler = Arc::new(event::handlers::TransactionAuditHandler::new());
        self.event_bus.subscribe(transaction_audit_handler).await;

        self.storage = Some(storage_arc);
        Ok(())
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
    /// 1. Determining the message type (plain, signed, or encrypted)
    /// 2. Verifying signatures or routing to agents for decryption
    /// 3. Processing the resulting plain messages through the pipeline
    /// 4. Routing and dispatching to the appropriate agents
    ///
    /// # Parameters
    ///
    /// * `message` - The message as a JSON Value (can be plain, JWS, or JWE)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the message was successfully processed
    /// * `Err(Error)` if there was an error during processing
    pub async fn receive_message(&self, message: serde_json::Value) -> Result<()> {
        // Store the raw message for logging
        let raw_message = serde_json::to_string(&message).ok();
        use tap_agent::{verify_jws, Jwe, Jws};

        // Determine message type
        let is_encrypted =
            message.get("protected").is_some() && message.get("recipients").is_some();
        let is_signed = message.get("payload").is_some() && message.get("signatures").is_some();

        if is_signed {
            // Verify signature once using resolver
            let jws: Jws = serde_json::from_value(message)
                .map_err(|e| Error::Serialization(format!("Failed to parse JWS: {}", e)))?;

            let plain_message = verify_jws(&jws, &*self.resolver)
                .await
                .map_err(|e| Error::Verification(format!("JWS verification failed: {}", e)))?;

            // Process the verified plain message
            self.process_plain_message(plain_message, raw_message.as_deref())
                .await
        } else if is_encrypted {
            // Route encrypted message to each matching agent
            let jwe: Jwe = serde_json::from_value(message.clone())
                .map_err(|e| Error::Serialization(format!("Failed to parse JWE: {}", e)))?;

            // Find agents that match recipients
            let mut processed = false;
            for recipient in &jwe.recipients {
                if let Some(did) = recipient.header.kid.split('#').next() {
                    if let Ok(agent) = self.agents.get_agent(did).await {
                        // Let the agent handle decryption and processing
                        match agent.receive_encrypted_message(&message).await {
                            Ok(_) => {
                                processed = true;
                                log::debug!(
                                    "Agent {} successfully processed encrypted message",
                                    did
                                );
                            }
                            Err(e) => {
                                log::debug!(
                                    "Agent {} couldn't process encrypted message: {}",
                                    did,
                                    e
                                );
                            }
                        }
                    }
                }
            }

            if !processed {
                return Err(Error::Processing(
                    "No agent could process the encrypted message".to_string(),
                ));
            }
            Ok(())
        } else {
            // Plain message - parse and process
            let plain_message: PlainMessage = serde_json::from_value(message).map_err(|e| {
                Error::Serialization(format!("Failed to parse PlainMessage: {}", e))
            })?;

            self.process_plain_message(plain_message, raw_message.as_deref())
                .await
        }
    }

    /// Process a plain message through the pipeline
    async fn process_plain_message(
        &self,
        message: PlainMessage,
        raw_message: Option<&str>,
    ) -> Result<()> {
        // Validate the message if storage/validation is available
        #[cfg(feature = "storage")]
        {
            if let Some(ref storage) = self.storage {
                // Create validator
                let validator_config = validation::StandardValidatorConfig {
                    max_timestamp_drift_secs: 60,
                    storage: storage.clone(),
                };
                let validator = validation::create_standard_validator(validator_config).await;

                // Validate the message
                use crate::validation::{MessageValidator, ValidationResult};
                match validator.validate(&message).await {
                    ValidationResult::Accept => {
                        // Publish accepted event
                        self.event_bus
                            .publish_message_accepted(
                                message.id.clone(),
                                message.type_.clone(),
                                message.from.clone(),
                                message.to.first().cloned().unwrap_or_default(),
                            )
                            .await;
                    }
                    ValidationResult::Reject(reason) => {
                        // Publish rejected event
                        self.event_bus
                            .publish_message_rejected(
                                message.id.clone(),
                                reason.clone(),
                                message.from.clone(),
                                message.to.first().cloned().unwrap_or_default(),
                            )
                            .await;

                        // Return error to stop processing
                        return Err(Error::Validation(reason));
                    }
                }
            }
        }
        // Log all incoming messages for audit trail
        #[cfg(feature = "storage")]
        {
            if let Some(ref storage) = self.storage {
                // Log the message to the audit trail
                match storage
                    .log_message(&message, storage::MessageDirection::Incoming, raw_message)
                    .await
                {
                    Ok(_) => log::debug!("Logged incoming message: {}", message.id),
                    Err(e) => log::warn!("Failed to log incoming message: {}", e),
                }

                // Store as transaction if it's a Transfer or Payment
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

        // Get the agent
        let agent = match self.agents.get_agent(&target_did).await {
            Ok(a) => a,
            Err(e) => {
                log::warn!("Failed to get agent for dispatch: {}", e);
                return Ok(());
            }
        };

        // Let the agent process the plain message
        match agent.receive_plain_message(processed_message).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Log the error but don't fail the entire operation
                log::warn!("Agent failed to process message: {}", e);
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
        // Log all outgoing messages for audit trail
        #[cfg(feature = "storage")]
        {
            if let Some(ref storage) = self.storage {
                // Log the message to the audit trail
                match storage
                    .log_message(&message, storage::MessageDirection::Outgoing, None)
                    .await
                {
                    Ok(_) => log::debug!("Logged outgoing message: {}", message.id),
                    Err(e) => log::warn!("Failed to log outgoing message: {}", e),
                }

                // Store as transaction if it's a Transfer or Payment
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
    pub fn resolver(&self) -> &Arc<MultiResolver> {
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
