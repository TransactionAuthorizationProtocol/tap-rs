//! Message processor implementations for TAP Node
//!
//! This module provides message processing functionality for TAP Node messages.

use async_trait::async_trait;
use log::{debug, info};
use std::sync::Arc;
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

/// A composite message processor that chains multiple processors together
#[derive(Debug, Clone)]
pub struct CompositeMessageProcessor {
    /// The processors to use, in order
    processors: Vec<Arc<dyn MessageProcessor>>,
}

impl CompositeMessageProcessor {
    /// Create a new composite message processor
    pub fn new(processors: Vec<Arc<dyn MessageProcessor>>) -> Self {
        Self { processors }
    }

    /// Add a processor to the chain
    pub fn add_processor(&mut self, processor: Arc<dyn MessageProcessor>) {
        self.processors.push(processor);
    }
}

#[async_trait]
impl MessageProcessor for CompositeMessageProcessor {
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>> {
        let mut current_message = Some(message);

        for processor in &self.processors {
            if let Some(msg) = current_message {
                current_message = processor.process_incoming(msg).await?;
            } else {
                // Message was filtered out by a previous processor
                break;
            }
        }

        Ok(current_message)
    }

    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>> {
        let mut current_message = Some(message);

        for processor in &self.processors {
            if let Some(msg) = current_message {
                current_message = processor.process_outgoing(msg).await?;
            } else {
                // Message was filtered out by a previous processor
                break;
            }
        }

        Ok(current_message)
    }
}

/// Default message processor that logs and validates messages
pub struct DefaultMessageProcessorImpl {
    /// The internal composite processor
    processor: CompositeMessageProcessor,
}

impl DefaultMessageProcessorImpl {
    /// Create a new default message processor
    pub fn new() -> Self {
        let logging_processor = Arc::new(LoggingMessageProcessor);
        let validation_processor = Arc::new(ValidationMessageProcessor);

        let processor = CompositeMessageProcessor::new(vec![
            logging_processor,
            validation_processor,
        ]);

        Self { processor }
    }
}

#[async_trait]
impl MessageProcessor for DefaultMessageProcessorImpl {
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>> {
        self.processor.process_incoming(message).await
    }

    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>> {
        self.processor.process_outgoing(message).await
    }
}
