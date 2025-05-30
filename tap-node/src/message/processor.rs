//! # PlainMessage Processor Implementations for TAP Node
//!
//! This module provides message processing functionality for TAP Node. PlainMessage processors
//! serve as middleware in the message handling pipeline, allowing for validation, transformation,
//! and filtering of messages as they flow through the system.
//!
//! ## PlainMessage Processing Pipeline
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
//! - `LoggingPlainMessageProcessor`: Logs information about messages passing through the system
//! - `ValidationPlainMessageProcessor`: Validates message structure and content
//! - `DefaultPlainMessageProcessor`: A simple pass-through processor with minimal functionality
//! - `CompositePlainMessageProcessor`: Combines multiple processors into a processing chain
//!
//! ## Custom Processors
//!
//! You can create custom processors by implementing the `PlainMessageProcessor` trait. This
//! allows for specialized processing such as:
//!
//! - PlainMessage transformation for protocol version compatibility
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
use tap_msg::didcomm::PlainMessage;

use crate::error::Result;

/// Trait for processing DIDComm messages in TAP nodes
///
/// The `PlainMessageProcessor` trait defines the interface for message processors
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
/// # use tap_msg::didcomm::PlainMessage;
/// # use tap_node::message::processor::PlainMessageProcessor;
/// #
/// #[derive(Clone, Debug)]
/// struct MyCustomProcessor;
///
/// #[async_trait]
/// impl PlainMessageProcessor for MyCustomProcessor {
///     async fn process_incoming(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
///         // Process incoming message - e.g., validate fields, log, transform
///         println!("Processing incoming message: {}", message.id);
///         Ok(Some(message))  // Pass message along unchanged
///     }
///
///     async fn process_outgoing(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
///         // Process outgoing message
///         println!("Processing outgoing message: {}", message.id);
///         Ok(Some(message))  // Pass message along unchanged
///     }
/// }
/// ```
#[async_trait]
pub trait PlainMessageProcessor: Send + Sync + Clone {
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
    async fn process_incoming(&self, message: PlainMessage) -> Result<Option<PlainMessage>>;

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
    async fn process_outgoing(&self, message: PlainMessage) -> Result<Option<PlainMessage>>;
}

/// A message processor that logs messages
#[derive(Debug, Clone)]
pub struct LoggingPlainMessageProcessor;

#[async_trait]
impl PlainMessageProcessor for LoggingPlainMessageProcessor {
    async fn process_incoming(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
        info!("Incoming message: {}", message.id);
        debug!("PlainMessage content: {:?}", message);
        Ok(Some(message))
    }

    async fn process_outgoing(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
        info!("Outgoing message: {}", message.id);
        debug!("PlainMessage content: {:?}", message);
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
/// # PlainMessage Flow
///
/// The validator sits in the message processor pipeline and can filter out invalid
/// messages by returning Ok(None), or let valid messages continue through the
/// pipeline by returning Ok(Some(message)).
#[derive(Debug, Clone)]
pub struct ValidationPlainMessageProcessor;

#[async_trait]
impl PlainMessageProcessor for ValidationPlainMessageProcessor {
    async fn process_incoming(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
        debug!("Validating incoming message: {}", message.id);

        // Basic validation - ID and type should not be empty
        if message.id.is_empty() {
            info!("PlainMessage has empty ID, rejecting");
            return Ok(None);
        }

        if message.typ.is_empty() {
            info!("PlainMessage has empty type, rejecting");
            return Ok(None);
        }

        // Validate DID format if present
        if !message.from.is_empty() && !message.from.starts_with("did:") {
            info!("Invalid 'from' DID format: {}", message.from);
            return Ok(None);
        }

        // Validate recipient DIDs
        if !message.to.is_empty() {
            // All DIDs should have valid format
            for recipient in &message.to {
                if !recipient.starts_with("did:") {
                    info!("Invalid recipient DID format: {}", recipient);
                    return Ok(None);
                }
            }
        }

        // Validate body
        if message.body == serde_json::json!(null) {
            info!("PlainMessage has null body, rejecting");
            return Ok(None);
        }

        // Validate pthid if present
        if let Some(pthid) = &message.pthid {
            if pthid.is_empty() {
                info!("PlainMessage has empty parent thread ID, rejecting");
                return Ok(None);
            }
        }

        // Validate timestamp
        if let Some(created_time) = message.created_time {
            let now = chrono::Utc::now().timestamp() as u64;
            // Check if the timestamp is more than 5 minutes in the future
            if created_time > now + 300 {
                info!("PlainMessage has future timestamp, rejecting");
                return Ok(None);
            }
        }

        // Protocol-specific validation based on message type
        let typ = &message.typ;

        // Validate TAP messages
        if typ.starts_with("https://tap.rsvp/schema/") {
            // TAP-specific validations
            // Check that it's a valid TAP message type
            if !typ.contains("Transfer")
                && !typ.contains("Authorize")
                && !typ.contains("Reject")
                && !typ.contains("Settle")
                && !typ.contains("Payment")
                && !typ.contains("Connect")
                && !typ.contains("Cancel")
                && !typ.contains("Revert")
                && !typ.contains("AddAgents")
                && !typ.contains("ReplaceAgent")
                && !typ.contains("RemoveAgent")
                && !typ.contains("UpdateParty")
                && !typ.contains("UpdatePolicies")
                && !typ.contains("ConfirmRelationship")
                && !typ.contains("OutOfBand")
                && !typ.contains("AuthorizationRequired")
                && !typ.contains("RequestPresentation")
                && !typ.contains("Presentation")
                && !typ.contains("Error")
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

        // PlainMessage passed validation
        Ok(Some(message))
    }

    async fn process_outgoing(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
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
        if !message.from.is_empty() && !message.from.starts_with("did:") {
            info!(
                "Invalid 'from' DID format in outgoing message: {}",
                message.from
            );
            return Ok(None);
        }

        // Validate recipient DIDs
        if !message.to.is_empty() {
            // All DIDs should have valid format
            for recipient in &message.to {
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

        // PlainMessage passed validation
        Ok(Some(message))
    }
}

/// Default message processor with core functionality
#[derive(Debug, Clone)]
pub struct DefaultPlainMessageProcessor;

#[async_trait]
impl PlainMessageProcessor for DefaultPlainMessageProcessor {
    async fn process_incoming(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
        // By default, we just pass the message through
        Ok(Some(message))
    }

    async fn process_outgoing(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
        // By default, we just pass the message through
        Ok(Some(message))
    }
}

/// Default message processor that logs and validates messages
#[derive(Clone, Debug)]
pub struct DefaultPlainMessageProcessorImpl {
    /// The internal processor
    processor: crate::message::PlainMessageProcessorType,
}

impl Default for DefaultPlainMessageProcessorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultPlainMessageProcessorImpl {
    /// Create a new default message processor
    pub fn new() -> Self {
        let logging_processor =
            crate::message::PlainMessageProcessorType::Logging(LoggingPlainMessageProcessor);
        let validation_processor =
            crate::message::PlainMessageProcessorType::Validation(ValidationPlainMessageProcessor);

        let mut processor = crate::message::CompositePlainMessageProcessor::new(Vec::new());
        processor.add_processor(validation_processor);
        processor.add_processor(logging_processor);

        let processor = crate::message::PlainMessageProcessorType::Composite(processor);

        Self { processor }
    }
}

#[async_trait]
impl PlainMessageProcessor for DefaultPlainMessageProcessorImpl {
    async fn process_incoming(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
        match &self.processor {
            crate::message::PlainMessageProcessorType::Default(p) => {
                p.process_incoming(message).await
            }
            crate::message::PlainMessageProcessorType::Logging(p) => {
                p.process_incoming(message).await
            }
            crate::message::PlainMessageProcessorType::Validation(p) => {
                p.process_incoming(message).await
            }
            crate::message::PlainMessageProcessorType::Composite(p) => {
                p.process_incoming(message).await
            }
        }
    }

    async fn process_outgoing(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
        match &self.processor {
            crate::message::PlainMessageProcessorType::Default(p) => {
                p.process_outgoing(message).await
            }
            crate::message::PlainMessageProcessorType::Logging(p) => {
                p.process_outgoing(message).await
            }
            crate::message::PlainMessageProcessorType::Validation(p) => {
                p.process_outgoing(message).await
            }
            crate::message::PlainMessageProcessorType::Composite(p) => {
                p.process_outgoing(message).await
            }
        }
    }
}
