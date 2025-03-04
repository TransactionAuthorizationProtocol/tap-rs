//! Message processor implementations for TAP Node
//!
//! This module provides message processing functionality for TAP Node messages.

use async_trait::async_trait;
use log::{debug, info};
use tap_core::didcomm::Message;

use crate::error::Result;

/// Trait for processing messages
#[async_trait]
pub trait MessageProcessor: Send + Sync + Clone {
    /// Process an incoming message
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>>;

    /// Process an outgoing message
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
#[derive(Debug, Clone)]
pub struct ValidationMessageProcessor;

#[async_trait]
impl MessageProcessor for ValidationMessageProcessor {
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>> {
        // TODO: Implement validation
        debug!("Validating incoming message: {}", message.id);
        Ok(Some(message))
    }

    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>> {
        // TODO: Implement validation
        debug!("Validating outgoing message: {}", message.id);
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
        let logging_processor = crate::message::MessageProcessorType::Logging(LoggingMessageProcessor);
        let validation_processor = crate::message::MessageProcessorType::Validation(ValidationMessageProcessor);

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
            crate::message::MessageProcessorType::Validation(p) => p.process_incoming(message).await,
            crate::message::MessageProcessorType::Composite(p) => p.process_incoming(message).await,
        }
    }

    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>> {
        match &self.processor {
            crate::message::MessageProcessorType::Default(p) => p.process_outgoing(message).await,
            crate::message::MessageProcessorType::Logging(p) => p.process_outgoing(message).await,
            crate::message::MessageProcessorType::Validation(p) => p.process_outgoing(message).await,
            crate::message::MessageProcessorType::Composite(p) => p.process_outgoing(message).await,
        }
    }
}
