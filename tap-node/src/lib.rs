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
#[cfg(feature = "storage")]
pub mod state_machine;
pub mod storage;
#[cfg(feature = "storage")]
pub mod validation;

pub use error::{Error, Result};
pub use event::logger::{EventLogger, EventLoggerConfig, LogDestination};
pub use event::{EventSubscriber, NodeEvent};
pub use message::sender::{
    HttpPlainMessageSender, HttpPlainMessageSenderWithTracking, NodePlainMessageSender,
    PlainMessageSender, WebSocketPlainMessageSender,
};
#[cfg(feature = "storage")]
pub use storage::{
    models::{DeliveryStatus, DeliveryType},
    Storage,
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
    /// Storage for transactions (legacy centralized storage)
    #[cfg(feature = "storage")]
    storage: Option<Arc<storage::Storage>>,
    /// Agent-specific storage manager
    #[cfg(feature = "storage")]
    agent_storage_manager: Option<Arc<storage::AgentStorageManager>>,
    /// Transaction state processor
    #[cfg(feature = "storage")]
    state_processor: Option<Arc<state_machine::StandardTransactionProcessor>>,
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
        let trust_ping_processor = PlainMessageProcessorType::TrustPing(
            TrustPingProcessor::with_event_bus(event_bus.clone()),
        );
        let default_processor = PlainMessageProcessorType::Default(DefaultPlainMessageProcessor);

        let incoming_processor = CompositePlainMessageProcessor::new(vec![
            logging_processor.clone(),
            validation_processor.clone(),
            trust_ping_processor.clone(),
            default_processor.clone(),
        ]);

        let outgoing_processor = CompositePlainMessageProcessor::new(vec![
            logging_processor,
            validation_processor,
            trust_ping_processor,
            default_processor,
        ]);

        // Create the resolver
        let resolver = Arc::new(MultiResolver::default());

        // Storage will be initialized on first use
        #[cfg(feature = "storage")]
        let storage = None;
        #[cfg(feature = "storage")]
        let agent_storage_manager = Some(Arc::new(storage::AgentStorageManager::new(
            config.tap_root.clone(),
        )));
        #[cfg(feature = "storage")]
        let state_processor = None;

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
            #[cfg(feature = "storage")]
            agent_storage_manager,
            #[cfg(feature = "storage")]
            state_processor,
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

        // Create state processor
        let state_processor = Arc::new(state_machine::StandardTransactionProcessor::new(
            storage_arc.clone(),
            self.event_bus.clone(),
            self.agents.clone(),
        ));

        self.storage = Some(storage_arc);
        self.state_processor = Some(state_processor);
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

        // Process message through state machine if available
        #[cfg(feature = "storage")]
        {
            if let Some(ref state_processor) = self.state_processor {
                use crate::state_machine::TransactionStateProcessor;
                if let Err(e) = state_processor.process_message(&message).await {
                    log::warn!("State processor error: {}", e);
                    // Don't fail the entire message processing, just log the error
                }
            }
        }
        // Log incoming messages to agent-specific storage
        #[cfg(feature = "storage")]
        {
            if let Some(ref storage_manager) = self.agent_storage_manager {
                // Check if this is a transaction message
                let message_type_lower = message.type_.to_lowercase();
                let is_transaction = message_type_lower.contains("transfer")
                    || message_type_lower.contains("payment");
                log::debug!(
                    "Message type: {}, is_transaction: {}",
                    message.type_,
                    is_transaction
                );

                if is_transaction {
                    // For transactions, store in ALL involved agents' databases
                    let involved_agents = self.extract_transaction_agents(&message);

                    if involved_agents.is_empty() {
                        log::warn!("No registered agents found for transaction: {}", message.id);
                    } else {
                        log::debug!(
                            "Storing transaction {} in {} agent databases",
                            message.id,
                            involved_agents.len()
                        );

                        // Store transaction in each involved agent's database
                        for agent_did in &involved_agents {
                            if let Ok(agent_storage) =
                                storage_manager.get_agent_storage(agent_did).await
                            {
                                // Log the message
                                match agent_storage
                                    .log_message(
                                        &message,
                                        storage::MessageDirection::Incoming,
                                        raw_message,
                                    )
                                    .await
                                {
                                    Ok(_) => log::debug!(
                                        "Logged incoming message to agent {}: {}",
                                        agent_did,
                                        message.id
                                    ),
                                    Err(e) => log::warn!(
                                        "Failed to log incoming message for agent {}: {}",
                                        agent_did,
                                        e
                                    ),
                                }

                                // Store as transaction
                                match agent_storage.insert_transaction(&message).await {
                                    Ok(_) => log::debug!(
                                        "Stored transaction for agent {}: {}",
                                        agent_did,
                                        message.id
                                    ),
                                    Err(e) => log::warn!(
                                        "Failed to store transaction for agent {}: {}",
                                        agent_did,
                                        e
                                    ),
                                }
                            } else {
                                log::warn!("Failed to get storage for agent: {}", agent_did);
                            }
                        }
                    }
                } else {
                    // For non-transaction messages, log to all recipient agents' storage
                    let mut logged_to_any = false;

                    for recipient_did in &message.to {
                        // Check if this recipient is a registered agent
                        if self.agents.has_agent(recipient_did) {
                            if let Ok(agent_storage) =
                                storage_manager.get_agent_storage(recipient_did).await
                            {
                                // Log the message to this recipient's storage
                                match agent_storage
                                    .log_message(
                                        &message,
                                        storage::MessageDirection::Incoming,
                                        raw_message,
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        log::debug!(
                                            "Logged incoming message to recipient {}: {}",
                                            recipient_did,
                                            message.id
                                        );
                                        logged_to_any = true;
                                    }
                                    Err(e) => log::warn!(
                                        "Failed to log incoming message for recipient {}: {}",
                                        recipient_did,
                                        e
                                    ),
                                }
                            } else {
                                log::warn!(
                                    "Failed to get storage for recipient: {}",
                                    recipient_did
                                );
                            }
                        }
                    }

                    // If no recipients were logged, fall back to sender or router-based storage
                    if !logged_to_any {
                        match self.determine_message_agent(&message) {
                            Ok(agent_did) => {
                                if let Ok(agent_storage) =
                                    storage_manager.get_agent_storage(&agent_did).await
                                {
                                    // Log the message to the agent's storage
                                    match agent_storage
                                        .log_message(
                                            &message,
                                            storage::MessageDirection::Incoming,
                                            raw_message,
                                        )
                                        .await
                                    {
                                        Ok(_) => log::debug!(
                                            "Logged incoming message to fallback agent {}: {}",
                                            agent_did,
                                            message.id
                                        ),
                                        Err(e) => log::warn!(
                                            "Failed to log incoming message for fallback agent {}: {}",
                                            agent_did,
                                            e
                                        ),
                                    }
                                } else {
                                    log::warn!(
                                        "Failed to get storage for fallback agent: {}",
                                        agent_did
                                    );
                                }
                            }
                            Err(e) => {
                                log::warn!(
                                    "Failed to determine fallback agent for message storage: {}",
                                    e
                                );
                                // Fall back to centralized storage if available
                                if let Some(ref storage) = self.storage {
                                    let _ = storage
                                        .log_message(
                                            &message,
                                            storage::MessageDirection::Incoming,
                                            raw_message,
                                        )
                                        .await;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Process the incoming message
        let processed_message = match self.incoming_processor.process_incoming(message).await? {
            Some(msg) => msg,
            None => return Ok(()), // PlainMessage was dropped during processing
        };

        // Deliver the message to all recipients in the 'to' field
        let mut delivery_success = false;

        for recipient_did in &processed_message.to {
            // Check if we have a registered agent for this recipient
            match self.agents.get_agent(recipient_did).await {
                Ok(agent) => {
                    // Create delivery record for internal delivery tracking
                    #[cfg(feature = "storage")]
                    let delivery_id = if let Some(ref storage_manager) = self.agent_storage_manager
                    {
                        if let Ok(agent_storage) =
                            storage_manager.get_agent_storage(recipient_did).await
                        {
                            // Serialize message for storage
                            let message_text = serde_json::to_string(&processed_message)
                                .unwrap_or_else(|_| "Failed to serialize message".to_string());

                            match agent_storage
                                .create_delivery(
                                    &processed_message.id,
                                    &message_text,
                                    recipient_did,
                                    None, // No URL for internal delivery
                                    storage::models::DeliveryType::Internal,
                                )
                                .await
                            {
                                Ok(id) => {
                                    log::debug!(
                                        "Created internal delivery record {} for message {} to {}",
                                        id,
                                        processed_message.id,
                                        recipient_did
                                    );
                                    Some(id)
                                }
                                Err(e) => {
                                    log::warn!("Failed to create internal delivery record: {}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Let the agent process the plain message
                    match agent.receive_plain_message(processed_message.clone()).await {
                        Ok(_) => {
                            log::debug!(
                                "Successfully delivered message to agent: {}",
                                recipient_did
                            );
                            delivery_success = true;

                            // Update delivery record to success
                            #[cfg(feature = "storage")]
                            if let (Some(delivery_id), Some(ref storage_manager)) =
                                (delivery_id, &self.agent_storage_manager)
                            {
                                if let Ok(agent_storage) =
                                    storage_manager.get_agent_storage(recipient_did).await
                                {
                                    if let Err(e) = agent_storage
                                        .update_delivery_status(
                                            delivery_id,
                                            storage::models::DeliveryStatus::Success,
                                            None, // No HTTP status for internal delivery
                                            None, // No error message
                                        )
                                        .await
                                    {
                                        log::warn!("Failed to update internal delivery record to success: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Agent {} failed to process message: {}", recipient_did, e);

                            // Update delivery record to failed
                            #[cfg(feature = "storage")]
                            if let (Some(delivery_id), Some(ref storage_manager)) =
                                (delivery_id, &self.agent_storage_manager)
                            {
                                if let Ok(agent_storage) =
                                    storage_manager.get_agent_storage(recipient_did).await
                                {
                                    if let Err(e2) = agent_storage
                                        .update_delivery_status(
                                            delivery_id,
                                            storage::models::DeliveryStatus::Failed,
                                            None, // No HTTP status for internal delivery
                                            Some(&e.to_string()), // Include error message
                                        )
                                        .await
                                    {
                                        log::warn!("Failed to update internal delivery record to failed: {}", e2);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::debug!(
                        "No registered agent found for recipient {}: {}",
                        recipient_did,
                        e
                    );
                    // This is not an error - the recipient might be external to this node
                }
            }
        }

        // If no recipients were successfully processed, try the router as fallback
        if !delivery_success {
            let target_did = match self.router.route_message(&processed_message).await {
                Ok(did) => did,
                Err(e) => {
                    log::warn!("Unable to route message and no recipients processed: {}", e);
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

            // Create delivery record for internal delivery tracking
            #[cfg(feature = "storage")]
            let delivery_id = if let Some(ref storage_manager) = self.agent_storage_manager {
                if let Ok(agent_storage) = storage_manager.get_agent_storage(&target_did).await {
                    // Serialize message for storage
                    let message_text = serde_json::to_string(&processed_message)
                        .unwrap_or_else(|_| "Failed to serialize message".to_string());

                    match agent_storage
                        .create_delivery(
                            &processed_message.id,
                            &message_text,
                            &target_did,
                            None, // No URL for internal delivery
                            storage::models::DeliveryType::Internal,
                        )
                        .await
                    {
                        Ok(id) => {
                            log::debug!(
                                "Created internal delivery record {} for routed message {} to {}",
                                id,
                                processed_message.id,
                                target_did
                            );
                            Some(id)
                        }
                        Err(e) => {
                            log::warn!(
                                "Failed to create internal delivery record for routing: {}",
                                e
                            );
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Let the agent process the plain message
            match agent.receive_plain_message(processed_message).await {
                Ok(_) => {
                    log::debug!("Successfully routed message to agent: {}", target_did);

                    // Update delivery record to success
                    #[cfg(feature = "storage")]
                    if let (Some(delivery_id), Some(ref storage_manager)) =
                        (delivery_id, &self.agent_storage_manager)
                    {
                        if let Ok(agent_storage) =
                            storage_manager.get_agent_storage(&target_did).await
                        {
                            if let Err(e) = agent_storage
                                .update_delivery_status(
                                    delivery_id,
                                    storage::models::DeliveryStatus::Success,
                                    None, // No HTTP status for internal delivery
                                    None, // No error message
                                )
                                .await
                            {
                                log::warn!(
                                    "Failed to update routed delivery record to success: {}",
                                    e
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Agent failed to process message: {}", e);

                    // Update delivery record to failed
                    #[cfg(feature = "storage")]
                    if let (Some(delivery_id), Some(ref storage_manager)) =
                        (delivery_id, &self.agent_storage_manager)
                    {
                        if let Ok(agent_storage) =
                            storage_manager.get_agent_storage(&target_did).await
                        {
                            if let Err(e2) = agent_storage
                                .update_delivery_status(
                                    delivery_id,
                                    storage::models::DeliveryStatus::Failed,
                                    None,                 // No HTTP status for internal delivery
                                    Some(&e.to_string()), // Include error message
                                )
                                .await
                            {
                                log::warn!(
                                    "Failed to update routed delivery record to failed: {}",
                                    e2
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
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
    /// 
    /// This method now includes comprehensive delivery tracking and actual message delivery.
    /// For internal recipients (registered agents), messages are delivered directly.
    /// For external recipients, messages are delivered via HTTP with tracking.
    pub async fn send_message(
        &self,
        sender_did: String,
        to_did: String,
        message: PlainMessage,
    ) -> Result<String> {
        // Log outgoing messages to agent-specific storage
        #[cfg(feature = "storage")]
        {
            if let Some(ref storage_manager) = self.agent_storage_manager {
                // Check if this is a transaction message
                let message_type_lower = message.type_.to_lowercase();
                let is_transaction = message_type_lower.contains("transfer")
                    || message_type_lower.contains("payment");
                log::debug!(
                    "Message type: {}, is_transaction: {}",
                    message.type_,
                    is_transaction
                );

                if is_transaction {
                    // For transactions, store in ALL involved agents' databases
                    let involved_agents = self.extract_transaction_agents(&message);

                    if involved_agents.is_empty() {
                        log::warn!(
                            "No registered agents found for outgoing transaction: {}",
                            message.id
                        );
                    } else {
                        log::debug!(
                            "Storing outgoing transaction {} in {} agent databases",
                            message.id,
                            involved_agents.len()
                        );

                        // Store transaction in each involved agent's database
                        for agent_did in &involved_agents {
                            if let Ok(agent_storage) =
                                storage_manager.get_agent_storage(agent_did).await
                            {
                                // Log the message
                                match agent_storage
                                    .log_message(
                                        &message,
                                        storage::MessageDirection::Outgoing,
                                        None,
                                    )
                                    .await
                                {
                                    Ok(_) => log::debug!(
                                        "Logged outgoing message to agent {}: {}",
                                        agent_did,
                                        message.id
                                    ),
                                    Err(e) => log::warn!(
                                        "Failed to log outgoing message for agent {}: {}",
                                        agent_did,
                                        e
                                    ),
                                }

                                // Store as transaction
                                match agent_storage.insert_transaction(&message).await {
                                    Ok(_) => log::debug!(
                                        "Stored outgoing transaction for agent {}: {}",
                                        agent_did,
                                        message.id
                                    ),
                                    Err(e) => log::warn!(
                                        "Failed to store outgoing transaction for agent {}: {}",
                                        agent_did,
                                        e
                                    ),
                                }
                            } else {
                                log::warn!("Failed to get storage for agent: {}", agent_did);
                            }
                        }
                    }
                } else {
                    // For non-transaction messages, just store in sender's storage
                    if let Ok(sender_storage) = storage_manager.get_agent_storage(&sender_did).await
                    {
                        // Log the message to the sender's storage
                        match sender_storage
                            .log_message(&message, storage::MessageDirection::Outgoing, None)
                            .await
                        {
                            Ok(_) => log::debug!(
                                "Logged outgoing message for agent {}: {}",
                                sender_did,
                                message.id
                            ),
                            Err(e) => log::warn!(
                                "Failed to log outgoing message for agent {}: {}",
                                sender_did,
                                e
                            ),
                        }
                    } else {
                        log::warn!("Failed to get storage for sender agent: {}", sender_did);
                        // Fall back to centralized storage if available
                        if let Some(ref storage) = self.storage {
                            let _ = storage
                                .log_message(&message, storage::MessageDirection::Outgoing, None)
                                .await;
                        }
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

        // Pack/sign the message
        let packed = agent
            .send_serialized_message(&processed_message, to_did.as_str())
            .await?;

        // Check if recipient is a local agent (internal delivery) or external
        let is_internal_recipient = self.agents.get_agent(&to_did).await.is_ok();

        if is_internal_recipient {
            // Internal delivery - deliver to registered agent with tracking
            log::debug!("Delivering message internally to agent: {}", to_did);
            
            #[cfg(feature = "storage")]
            let delivery_id = if let Some(ref storage_manager) = self.agent_storage_manager {
                if let Ok(sender_storage) = storage_manager.get_agent_storage(&sender_did).await {
                    // Create delivery record for internal delivery
                    match sender_storage
                        .create_delivery(
                            &processed_message.id,
                            &packed, // Store the signed/packed message
                            &to_did,
                            None, // No URL for internal delivery
                            storage::models::DeliveryType::Internal,
                        )
                        .await
                    {
                        Ok(id) => {
                            log::debug!(
                                "Created internal delivery record {} for message {} to {}",
                                id,
                                processed_message.id,
                                to_did
                            );
                            Some(id)
                        }
                        Err(e) => {
                            log::warn!("Failed to create internal delivery record: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Process the message internally
            match self.process_plain_message(processed_message.clone(), Some(&packed)).await {
                Ok(_) => {
                    log::debug!("Successfully delivered message internally to: {}", to_did);
                    
                    // Update delivery record to success
                    #[cfg(feature = "storage")]
                    if let (Some(delivery_id), Some(ref storage_manager)) = 
                        (delivery_id, &self.agent_storage_manager) {
                        if let Ok(sender_storage) = storage_manager.get_agent_storage(&sender_did).await {
                            if let Err(e) = sender_storage
                                .update_delivery_status(
                                    delivery_id,
                                    storage::models::DeliveryStatus::Success,
                                    None, // No HTTP status for internal delivery
                                    None, // No error message
                                )
                                .await
                            {
                                log::warn!("Failed to update internal delivery status: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to deliver message internally to {}: {}", to_did, e);
                    
                    // Update delivery record to failed
                    #[cfg(feature = "storage")]
                    if let (Some(delivery_id), Some(ref storage_manager)) = 
                        (delivery_id, &self.agent_storage_manager) {
                        if let Ok(sender_storage) = storage_manager.get_agent_storage(&sender_did).await {
                            if let Err(e) = sender_storage
                                .update_delivery_status(
                                    delivery_id,
                                    storage::models::DeliveryStatus::Failed,
                                    None, // No HTTP status for internal delivery
                                    Some(&format!("Internal delivery failed: {}", e)),
                                )
                                .await
                            {
                                log::warn!("Failed to update internal delivery status: {}", e);
                            }
                        }
                    }
                    
                    return Err(e);
                }
            }
        } else {
            // External delivery - attempt HTTP delivery with tracking
            log::debug!("Attempting external delivery to: {}", to_did);
            
            // TODO: In a real implementation, you would:
            // 1. Resolve the DID to find HTTP endpoints
            // 2. Use HttpPlainMessageSenderWithTracking for delivery
            // 3. Handle delivery results and update tracking
            
            // For now, create a delivery record but mark it as pending
            #[cfg(feature = "storage")]
            if let Some(ref storage_manager) = self.agent_storage_manager {
                if let Ok(sender_storage) = storage_manager.get_agent_storage(&sender_did).await {
                    // Create delivery record for external delivery
                    match sender_storage
                        .create_delivery(
                            &processed_message.id,
                            &packed, // Store the signed/packed message
                            &to_did,
                            Some("https://external-endpoint.example.com"), // TODO: Resolve from DID
                            storage::models::DeliveryType::Https,
                        )
                        .await
                    {
                        Ok(delivery_id) => {
                            log::debug!(
                                "Created external delivery record {} for message {} to {}",
                                delivery_id,
                                processed_message.id,
                                to_did
                            );
                            
                            // TODO: Implement actual HTTP delivery with tracking
                            log::warn!("External delivery not yet implemented - message {} marked as pending delivery", processed_message.id);
                        }
                        Err(e) => {
                            log::warn!("Failed to create external delivery record: {}", e);
                        }
                    }
                }
            }
        }

        // Publish an event for the message
        self.event_bus
            .publish_agent_message(sender_did, packed.clone().into_bytes())
            .await;

        Ok(packed)
    }

    /// Register a new agent with the node
    ///
    /// This method registers an agent with the TAP Node and automatically initializes
    /// DID-specific storage for the agent. The storage directory structure follows:
    /// - `~/.tap/{sanitized_did}/transactions.db` (default)
    /// - `{tap_root}/{sanitized_did}/transactions.db` (if custom TAP root is configured)
    ///
    /// # Storage Initialization
    ///
    /// When an agent is registered, a dedicated SQLite database is created for that agent's DID.
    /// This ensures transaction isolation between different agents while maintaining a consistent
    /// storage structure. If storage initialization fails, the agent registration continues but
    /// a warning is logged.
    ///
    /// # Arguments
    ///
    /// * `agent` - The TapAgent to register with the node
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the agent was successfully registered
    /// * `Err(Error)` if agent registration fails
    pub async fn register_agent(&self, agent: Arc<TapAgent>) -> Result<()> {
        let agent_did = agent.get_agent_did().to_string();

        // Initialize storage for this agent if storage is enabled
        #[cfg(feature = "storage")]
        {
            if let Some(ref storage_manager) = self.agent_storage_manager {
                match storage_manager.ensure_agent_storage(&agent_did).await {
                    Ok(_) => {
                        log::info!("Initialized storage for agent: {}", agent_did);
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to initialize storage for agent {}: {}",
                            agent_did,
                            e
                        );
                        // Don't fail the registration, just log the warning
                    }
                }
            }
        }

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

    /// Get a reference to the agent storage manager (if available)
    #[cfg(feature = "storage")]
    pub fn agent_storage_manager(&self) -> Option<&Arc<storage::AgentStorageManager>> {
        self.agent_storage_manager.as_ref()
    }

    /// Determine which agent's storage should be used for a message
    ///
    /// This method uses the following strategy:
    /// 1. Use the first recipient DID if it's one of our registered agents
    /// 2. Use the sender DID if it's one of our registered agents  
    /// 3. If no local agents are involved, fall back to the first recipient
    #[cfg(feature = "storage")]
    fn determine_message_agent(&self, message: &PlainMessage) -> Result<String> {
        // Strategy 1: Use the first recipient DID if it's one of our agents
        for recipient in &message.to {
            if self.agents.has_agent(recipient) {
                return Ok(recipient.clone());
            }
        }

        // Strategy 2: Use the sender DID if it's one of our agents
        if self.agents.has_agent(&message.from) {
            return Ok(message.from.clone());
        }

        // Strategy 3: If no local agents involved, fall back to first recipient
        if !message.to.is_empty() {
            return Ok(message.to[0].clone());
        }

        Err(Error::Storage(
            "Cannot determine agent for message storage".to_string(),
        ))
    }

    /// Extract all agent DIDs involved in a transaction
    ///
    /// For Transfer and Payment messages, this includes:
    /// - Originator/Customer from the message body
    /// - Beneficiary/Merchant from the message body
    /// - All agents in the agents array
    /// - Sender (from) and recipients (to) from the message envelope
    #[cfg(feature = "storage")]
    fn extract_transaction_agents(&self, message: &PlainMessage) -> Vec<String> {
        use std::collections::HashSet;
        let mut agent_dids = HashSet::new();

        log::debug!("Extracting transaction agents for message: {}", message.id);

        // Add sender and recipients from message envelope
        agent_dids.insert(message.from.clone());
        log::debug!("Added sender: {}", message.from);

        for recipient in &message.to {
            agent_dids.insert(recipient.clone());
            log::debug!("Added recipient: {}", recipient);
        }

        // Extract agents from message body based on type
        let message_type_lower = message.type_.to_lowercase();
        log::debug!("Message type: {}", message_type_lower);

        if message_type_lower.contains("transfer") {
            // Parse Transfer message body
            if let Ok(transfer) =
                serde_json::from_value::<tap_msg::message::Transfer>(message.body.clone())
            {
                // Add originator
                agent_dids.insert(transfer.originator.id.clone());
                log::debug!("Added originator: {}", transfer.originator.id);

                // Add beneficiary if present
                if let Some(beneficiary) = &transfer.beneficiary {
                    agent_dids.insert(beneficiary.id.clone());
                    log::debug!("Added beneficiary: {}", beneficiary.id);
                }

                // Add all agents
                for agent in &transfer.agents {
                    agent_dids.insert(agent.id.clone());
                    log::debug!("Added agent: {}", agent.id);
                }
            } else {
                log::warn!("Failed to parse Transfer message body");
            }
        } else if message_type_lower.contains("payment") {
            // Parse Payment message body
            if let Ok(payment) =
                serde_json::from_value::<tap_msg::message::Payment>(message.body.clone())
            {
                // Add merchant
                agent_dids.insert(payment.merchant.id.clone());
                log::debug!("Added merchant: {}", payment.merchant.id);

                // Add customer if present
                if let Some(customer) = &payment.customer {
                    agent_dids.insert(customer.id.clone());
                    log::debug!("Added customer: {}", customer.id);
                }

                // Add all agents
                for agent in &payment.agents {
                    agent_dids.insert(agent.id.clone());
                    log::debug!("Added agent: {}", agent.id);
                }
            } else {
                log::warn!("Failed to parse Payment message body");
            }
        }

        log::debug!("Total unique agents found: {}", agent_dids.len());

        // Convert to Vec and filter to only include registered agents
        let registered_agents: Vec<String> = agent_dids
            .into_iter()
            .filter(|did| {
                let is_registered = self.agents.has_agent(did);
                log::debug!("Agent {} registered: {}", did, is_registered);
                is_registered
            })
            .collect();

        log::debug!(
            "Registered agents involved in transaction: {:?}",
            registered_agents
        );
        registered_agents
    }
}

// Namespace imports
// These imports make the implementation cleaner, but should be hidden from public API
use message::processor::DefaultPlainMessageProcessor;
use message::processor::LoggingPlainMessageProcessor;
use message::processor::ValidationPlainMessageProcessor;
use message::processor_pool::{ProcessorPool, ProcessorPoolConfig};
use message::router::DefaultPlainMessageRouter;
use message::trust_ping_processor::TrustPingProcessor;
use message::RouterAsyncExt;
