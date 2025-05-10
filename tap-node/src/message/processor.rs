//! # Message Processor Implementations for TAP Node
//!
//! This module provides message processing functionality for TAP Node. Message processors
//! serve as middleware in the message handling pipeline, allowing for validation, transformation,
//! and filtering of messages as they flow through the system.
//!
//! ## Message Processing Pipeline
//!
//! The TAP Node uses a pipeline architecture for message processing, where messages pass through
//! a series of processors in sequence. Each processor can:
//!
//! - Pass the message through unchanged
//! - Transform the message in some way
//! - Filter out (drop) messages based on certain criteria
//! - Perform side effects (logging, metrics collection, etc.)
//!
//! ## Processor Types
//!
//! The module provides several built-in processor implementations:
//!
//! - `LoggingMessageProcessor`: Logs information about messages passing through the system
//! - `ValidationMessageProcessor`: Validates message structure and content
//! - `DefaultMessageProcessor`: A simple pass-through processor with minimal functionality
//! - `CompositeMessageProcessor`: Combines multiple processors into a processing chain
//!
//! ## Custom Processors
//!
//! You can create custom processors by implementing the `MessageProcessor` trait. This
//! allows for specialized processing such as:
//!
//! - Message transformation for protocol version compatibility
//! - Content-based filtering and routing
//! - Security scanning and anomaly detection
//! - Metrics collection and performance monitoring
//!
//! ## Processing Modes
//!
//! Each processor implements two key methods:
//!
//! - `process_incoming()`: For messages received by the node
//! - `process_outgoing()`: For messages being sent from the node
//!
//! This separation allows for different processing logic depending on message direction.

use async_trait::async_trait;
use log::{debug, info};
use tap_msg::didcomm::Message;

use crate::error::Result;

/// Trait for processing DIDComm messages in TAP nodes
///
/// The `MessageProcessor` trait defines the interface for message processors
/// that handle DIDComm messages flowing through the TAP node. Processors act
/// as middleware, allowing for validation, transformation, logging, metrics
/// collection, and other operations on messages.
///
/// # Design Patterns
///
/// This trait follows the Chain of Responsibility pattern, where each processor
/// can either:
/// - Pass the message along unchanged
/// - Transform the message before passing it along
/// - Filter out (drop) the message by returning None
/// - Perform side effects during processing (logging, metrics, etc.)
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync + Clone` to ensure they can be
/// safely used in multithreaded environments and composed into processor chains.
///
/// # Implementation Guidelines
///
/// When implementing a custom processor:
/// - Ensure both `process_incoming` and `process_outgoing` are implemented
/// - Be mindful of performance in high-throughput environments
/// - Consider making processors stateless when possible
/// - Use the processor's Clone trait to avoid expensive setup/teardown
/// - Document any side effects or transformations clearly
///
/// # Examples
///
/// ```
/// # use async_trait::async_trait;
/// # use tap_node::error::Result;
/// # use tap_msg::didcomm::Message;
/// # use tap_node::message::processor::MessageProcessor;
/// #
/// #[derive(Clone, Debug)]
/// struct MyCustomProcessor;
///
/// #[async_trait]
/// impl MessageProcessor for MyCustomProcessor {
///     async fn process_incoming(&self, message: Message) -> Result<Option<Message>> {
///         // Process incoming message - e.g., validate fields, log, transform
///         println!("Processing incoming message: {}", message.id);
///         Ok(Some(message))  // Pass message along unchanged
///     }
///
///     async fn process_outgoing(&self, message: Message) -> Result<Option<Message>> {
///         // Process outgoing message
///         println!("Processing outgoing message: {}", message.id);
///         Ok(Some(message))  // Pass message along unchanged
///     }
/// }
/// ```
#[async_trait]
pub trait MessageProcessor: Send + Sync + Clone {
    /// Process an incoming message received by the node
    ///
    /// This method handles messages that are being received by the TAP node from
    /// external sources. Implementations can validate, transform, or filter these
    /// messages before they are routed to their target agents.
    ///
    /// # Parameters
    ///
    /// * `message` - The DIDComm message to process
    ///
    /// # Returns
    ///
    /// * `Ok(Some(message))` - The message to pass to the next processor
    /// * `Ok(None)` - Drop the message (do not process further)
    /// * `Err(e)` - Processing error
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>>;

    /// Process an outgoing message being sent from the node
    ///
    /// This method handles messages that are being sent from the TAP node to
    /// external recipients. Implementations can transform these messages for
    /// compatibility, add headers, perform logging, or filter messages before
    /// they are delivered.
    ///
    /// # Parameters
    ///
    /// * `message` - The DIDComm message to process
    ///
    /// # Returns
    ///
    /// * `Ok(Some(message))` - The message to pass to the next processor
    /// * `Ok(None)` - Drop the message (do not process further)
    /// * `Err(e)` - Processing error
    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>>;
}

/// A message processor that logs messages
#[derive(Debug, Clone)]
pub struct LoggingMessageProcessor;

#[async_trait]
impl MessageProcessor for LoggingMessageProcessor {
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>> {
        info!("Incoming message: {}", message.id);
        debug!("Message content: {:?}", message);
        Ok(Some(message))
    }

    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>> {
        info!("Outgoing message: {}", message.id);
        debug!("Message content: {:?}", message);
        Ok(Some(message))
    }
}

/// A message processor that validates messages
///
/// This processor validates incoming and outgoing DIDComm messages to ensure they
/// conform to the expected structure and protocol requirements.
///
/// In a production implementation, this would perform comprehensive validation including:
/// - Field validation (required fields, format, values)
/// - Protocol compliance checks for each message type
/// - Signature verification
/// - Timestamp and expiration checks
/// - Security and authorization checks
///
/// # Implementation
///
/// Currently, this implementation validates:
/// - The message ID is not empty
/// - The message type is not empty
/// - Any 'from' or 'to' DIDs follow the 'did:' prefix format
/// - Basic protocol-specific requirements based on message type
///
/// # Message Flow
///
/// The validator sits in the message processor pipeline and can filter out invalid
/// messages by returning Ok(None), or let valid messages continue through the
/// pipeline by returning Ok(Some(message)).
#[derive(Debug, Clone)]
pub struct ValidationMessageProcessor;

#[async_trait]
impl MessageProcessor for ValidationMessageProcessor {
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>> {
        debug!("Validating incoming message: {}", message.id);

        // Basic validation - ID and type should not be empty
        if message.id.is_empty() {
            info!("Message has empty ID, rejecting");
            return Ok(None);
        }

        if message.typ.is_empty() {
            info!("Message has empty type, rejecting");
            return Ok(None);
        }

        // Validate DID format if present
        if let Some(from) = &message.from {
            if !from.starts_with("did:") {
                info!("Invalid 'from' DID format: {}", from);
                return Ok(None);
            }
        }

        // Validate recipient DIDs
        if let Some(to) = &message.to {
            if to.is_empty() {
                info!("Message has empty 'to' field");
                return Ok(None);
            }

            // All DIDs should have valid format
            for recipient in to {
                if !recipient.starts_with("did:") {
                    info!("Invalid recipient DID format: {}", recipient);
                    return Ok(None);
                }
            }
        }

        // Validate body
        if message.body == serde_json::json!(null) {
            info!("Message has null body, rejecting");
            return Ok(None);
        }

        // Validate pthid if present
        if let Some(pthid) = &message.pthid {
            if pthid.is_empty() {
                info!("Message has empty parent thread ID, rejecting");
                return Ok(None);
            }
        }

        // Validate timestamp
        if let Some(created_time) = message.created_time {
            let now = chrono::Utc::now().timestamp() as u64;
            // Check if the timestamp is more than 5 minutes in the future
            if created_time > now + 300 {
                info!("Message has future timestamp, rejecting");
                return Ok(None);
            }
        }

        // Protocol-specific validation based on message type
        let typ = &message.typ;

        // Validate TAP messages
        if typ.starts_with("https://tap.rsvp/schema/") {
            // TAP-specific validations
            // Check that it's a valid TAP message type
            if !typ.contains("transfer")
                && !typ.contains("authorize")
                && !typ.contains("reject")
                && !typ.contains("settle")
            {
                info!("Unknown TAP message type: {}", typ);
                return Ok(None);
            }
        }
        // Validate DIDComm messages
        else if typ.starts_with("https://didcomm.org/") {
            // DIDComm-specific validations
            // Add more specific DIDComm validations here
        }
        // Unknown message type protocol
        else if !typ.starts_with("https://tap.rsvp/schema/")
            && !typ.starts_with("https://didcomm.org/")
        {
            info!("Unknown message protocol: {}", typ);
            // Reject unknown message protocols
            return Ok(None);
        }

        // Message passed validation
        Ok(Some(message))
    }

    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>> {
        debug!("Validating outgoing message: {}", message.id);

        // For outgoing messages, apply the same validations as incoming messages
        // In a production system, there might be different validations for outgoing vs incoming

        // Basic validation - ID and type should not be empty
        if message.id.is_empty() {
            info!("Outgoing message has empty ID, rejecting");
            return Ok(None);
        }

        if message.typ.is_empty() {
            info!("Outgoing message has empty type, rejecting");
            return Ok(None);
        }

        // Validate DID format if present
        if let Some(from) = &message.from {
            if !from.starts_with("did:") {
                info!("Invalid 'from' DID format in outgoing message: {}", from);
                return Ok(None);
            }
        }

        // Validate recipient DIDs
        if let Some(to) = &message.to {
            if to.is_empty() {
                info!("Outgoing message has empty 'to' field");
                return Ok(None);
            }

            // All DIDs should have valid format
            for recipient in to {
                if !recipient.starts_with("did:") {
                    info!(
                        "Invalid recipient DID format in outgoing message: {}",
                        recipient
                    );
                    return Ok(None);
                }
            }
        }

        // Validate body
        if message.body == serde_json::json!(null) {
            info!("Outgoing message has null body, rejecting");
            return Ok(None);
        }

        // Validate pthid if present
        if let Some(pthid) = &message.pthid {
            if pthid.is_empty() {
                info!("Outgoing message has empty parent thread ID, rejecting");
                return Ok(None);
            }
        }

        // Validate timestamp
        if let Some(created_time) = message.created_time {
            let now = chrono::Utc::now().timestamp() as u64;
            // Check if the timestamp is more than 5 minutes in the future
            if created_time > now + 300 {
                info!("Outgoing message has future timestamp, rejecting");
                return Ok(None);
            }
        }

        // Protocol-specific validation based on message type
        let typ = &message.typ;

        // Validate TAP messages
        if typ.starts_with("https://tap.rsvp/schema/") {
            // TAP-specific validations
            // Check that it's a valid TAP message type
            if !typ.contains("transfer")
                && !typ.contains("authorize")
                && !typ.contains("reject")
                && !typ.contains("settle")
            {
                info!("Unknown TAP message type in outgoing message: {}", typ);
                return Ok(None);
            }
        }
        // Validate DIDComm messages
        else if typ.starts_with("https://didcomm.org/") {
            // DIDComm-specific validations
            // Add more specific DIDComm validations here
        }
        // Unknown message type protocol
        else if !typ.starts_with("https://tap.rsvp/schema/")
            && !typ.starts_with("https://didcomm.org/")
        {
            info!("Unknown message protocol in outgoing message: {}", typ);
            // Reject unknown message protocols
            return Ok(None);
        }

        // Message passed validation
        Ok(Some(message))
    }
}

/// Default message processor with core functionality
#[derive(Debug, Clone)]
pub struct DefaultMessageProcessor;

#[async_trait]
impl MessageProcessor for DefaultMessageProcessor {
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>> {
        // By default, we just pass the message through
        Ok(Some(message))
    }

    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>> {
        // By default, we just pass the message through
        Ok(Some(message))
    }
}

/// Default message processor that logs and validates messages
#[derive(Clone, Debug)]
pub struct DefaultMessageProcessorImpl {
    /// The internal processor
    processor: crate::message::MessageProcessorType,
}

impl Default for DefaultMessageProcessorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultMessageProcessorImpl {
    /// Create a new default message processor
    pub fn new() -> Self {
        let logging_processor =
            crate::message::MessageProcessorType::Logging(LoggingMessageProcessor);
        let validation_processor =
            crate::message::MessageProcessorType::Validation(ValidationMessageProcessor);

        let mut processor = crate::message::CompositeMessageProcessor::new(Vec::new());
        processor.add_processor(validation_processor);
        processor.add_processor(logging_processor);

        let processor = crate::message::MessageProcessorType::Composite(processor);

        Self { processor }
    }
}

#[async_trait]
impl MessageProcessor for DefaultMessageProcessorImpl {
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>> {
        match &self.processor {
            crate::message::MessageProcessorType::Default(p) => p.process_incoming(message).await,
            crate::message::MessageProcessorType::Logging(p) => p.process_incoming(message).await,
            crate::message::MessageProcessorType::Validation(p) => {
                p.process_incoming(message).await
            }
            crate::message::MessageProcessorType::Composite(p) => p.process_incoming(message).await,
        }
    }

    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>> {
        match &self.processor {
            crate::message::MessageProcessorType::Default(p) => p.process_outgoing(message).await,
            crate::message::MessageProcessorType::Logging(p) => p.process_outgoing(message).await,
            crate::message::MessageProcessorType::Validation(p) => {
                p.process_outgoing(message).await
            }
            crate::message::MessageProcessorType::Composite(p) => p.process_outgoing(message).await,
        }
    }
}
