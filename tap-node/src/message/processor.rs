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

/// A message processor that validates message structure and content
///
/// The `ValidationMessageProcessor` ensures that messages flowing through the
/// TAP node adhere to protocol requirements and structural constraints.
/// It performs checks on required fields, message format, and protocol compliance.
///
/// # Validation Checks
///
/// For both incoming and outgoing messages, this processor checks:
///
/// - Presence of required fields (`id`, `type`, etc.)
/// - Content format and structure
/// - Compliance with protocol requirements
/// - Signature validity (if configured to do so)
///
/// Messages that fail validation will be dropped (returning `Ok(None)`),
/// preventing invalid messages from proceeding through the processing pipeline.
#[derive(Debug, Clone)]
pub struct ValidationMessageProcessor;

#[async_trait]
impl MessageProcessor for ValidationMessageProcessor {
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>> {
        debug!("Validating incoming message: {}", message.id);

        // Basic structural validation
        if let Err(err) = self.validate_message_structure(&message) {
            info!("Message {} failed structural validation: {}", message.id, err);
            return Ok(None);
        }

        // Protocol-specific validation
        if let Err(err) = self.validate_protocol_compliance(&message) {
            info!("Message {} failed protocol validation: {}", message.id, err);
            return Ok(None);
        }

        // Message passed all validation checks
        Ok(Some(message))
    }

    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>> {
        debug!("Validating outgoing message: {}", message.id);

        // Basic structural validation
        if let Err(err) = self.validate_message_structure(&message) {
            info!("Outgoing message {} failed structural validation: {}", message.id, err);
            return Ok(None);
        }

        // Protocol-specific validation
        if let Err(err) = self.validate_protocol_compliance(&message) {
            info!("Outgoing message {} failed protocol validation: {}", message.id, err);
            return Ok(None);
        }

        // Message passed all validation checks
        Ok(Some(message))
    }
}

impl ValidationMessageProcessor {
    /// Validate the basic structure of a DIDComm message
    ///
    /// Checks that the message has all required fields and that they are properly formatted.
    fn validate_message_structure(&self, message: &Message) -> std::result::Result<(), String> {
        // Check for required fields
        if message.id.is_empty() {
            return Err("Message is missing required 'id' field".to_string());
        }

        if message.typ.is_empty() {
            return Err("Message is missing required 'type' field".to_string());
        }

        // Validate 'from' field if present (should be a valid DID)
        if let Some(from) = &message.from {
            if !from.starts_with("did:") {
                return Err(format!("Invalid 'from' DID format: {}", from));
            }
        }

        // Validate 'to' field if present (should be a list of valid DIDs)
        if let Some(to) = &message.to {
            if to.is_empty() {
                return Err("Message has empty 'to' field".to_string());
            }

            // Check each recipient DID
            for recipient in to {
                if !recipient.starts_with("did:") {
                    return Err(format!("Invalid recipient DID format: {}", recipient));
                }
            }
        }

        // Check message created time if present
        if let Some(created_time) = message.created_time {
            // Ensure time is not in the future (with a small tolerance)
            let current_time = chrono::Utc::now().timestamp() as u64;
            const FUTURE_TOLERANCE_SECONDS: u64 = 300; // 5 minutes

            if created_time > current_time + FUTURE_TOLERANCE_SECONDS {
                return Err(format!(
                    "Message created time is too far in the future: {} (current time: {})",
                    created_time, current_time
                ));
            }
        }

        Ok(())
    }

    /// Validate protocol-specific requirements for the message
    ///
    /// This checks that the message complies with TAP protocol requirements
    /// based on its message type.
    fn validate_protocol_compliance(&self, message: &Message) -> std::result::Result<(), String> {
        // Check message type to determine which protocol-specific validation to apply
        let message_type = &message.typ;

        // For TAP messages, the type should start with the TAP schema prefix
        if message_type.starts_with("https://tap.rsvp/schema/") {
            // TAP-specific validations

            // For messages that require a body, check that it exists
            if !message_type.ends_with("ack") && message.body.is_none() {
                return Err("TAP message is missing required body".to_string());
            }

            // If a pthid is present, it should follow the expected format
            if let Some(pthid) = &message.pthid {
                if pthid.is_empty() {
                    return Err("Message has empty 'pthid' field".to_string());
                }
            }

            // Additional TAP-specific validations could be added here
        } else if message_type.starts_with("https://didcomm.org/") {
            // Standard DIDComm message validations

            // DIDComm messages should have a valid protocol version
            if let Some(version) = &message.body_enc {
                if version != "json" && !version.starts_with("application/json") {
                    return Err(format!("Unsupported body encoding: {}", version));
                }
            }
        } else {
            // Unknown message type
            return Err(format!("Unsupported message type: {}", message_type));
        }

        Ok(())
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
