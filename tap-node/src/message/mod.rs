//! Message processing and routing for TAP Node
//!
//! This module provides functionality for processing and routing TAP messages between agents.

pub mod processor;
pub mod processor_pool;
pub mod router;
pub mod sender;

// Re-export processors, routers, and senders
pub use processor::{DefaultMessageProcessor, LoggingMessageProcessor, ValidationMessageProcessor, MessageProcessor};
pub use processor_pool::{ProcessorPool, ProcessorPoolConfig};
pub use router::{DefaultMessageRouter};
pub use sender::{HttpMessageSender, NodeMessageSender};

// Import the Message type from tap-core
use tap_core::didcomm::Message;
use crate::error::{Result};
use async_trait::async_trait;

/// Router to determine the destination agent for a message
pub trait MessageRouter: Send + Sync {
    /// Route a message to determine the target agent DID
    fn route_message_impl(&self, message: &Message) -> Result<String>;
}

/// Async extension trait for the MessageRouter
#[async_trait]
pub trait RouterAsyncExt: MessageRouter {
    /// Route a message asynchronously
    async fn route_message(&self, message: &Message) -> Result<String>;
}

#[async_trait]
impl<T: MessageRouter + Sync> RouterAsyncExt for T {
    async fn route_message(&self, message: &Message) -> Result<String> {
        self.route_message_impl(message)
    }
}

/// Processor enum to replace trait objects
#[derive(Clone, Debug)]
pub enum MessageProcessorType {
    Default(DefaultMessageProcessor),
    Logging(LoggingMessageProcessor),
    Validation(ValidationMessageProcessor),
    Composite(CompositeMessageProcessor),
}

/// Router enum to replace trait objects
#[derive(Clone, Debug)]
pub enum MessageRouterType {
    Default(DefaultMessageRouter),
}

/// A message processor that applies multiple processors in sequence
#[derive(Clone, Debug)]
pub struct CompositeMessageProcessor {
    processors: Vec<MessageProcessorType>,
}

impl CompositeMessageProcessor {
    /// Create a new composite message processor
    pub fn new(processors: Vec<MessageProcessorType>) -> Self {
        Self { processors }
    }

    /// Add a processor to the chain
    pub fn add_processor(&mut self, processor: MessageProcessorType) {
        self.processors.push(processor);
    }
}

#[async_trait]
impl MessageProcessor for CompositeMessageProcessor {
    async fn process_incoming(&self, message: Message) -> Result<Option<Message>> {
        let mut current_message = message;

        for processor in &self.processors {
            let processed = match processor {
                MessageProcessorType::Default(p) => p.process_incoming(current_message).await?,
                MessageProcessorType::Logging(p) => p.process_incoming(current_message).await?,
                MessageProcessorType::Validation(p) => p.process_incoming(current_message).await?,
                MessageProcessorType::Composite(p) => p.process_incoming(current_message).await?,
            };

            if let Some(msg) = processed {
                current_message = msg;
            } else {
                // Message was filtered out
                return Ok(None);
            }
        }

        Ok(Some(current_message))
    }

    async fn process_outgoing(&self, message: Message) -> Result<Option<Message>> {
        let mut current_message = message;

        for processor in &self.processors {
            let processed = match processor {
                MessageProcessorType::Default(p) => p.process_outgoing(current_message).await?,
                MessageProcessorType::Logging(p) => p.process_outgoing(current_message).await?,
                MessageProcessorType::Validation(p) => p.process_outgoing(current_message).await?,
                MessageProcessorType::Composite(p) => p.process_outgoing(current_message).await?,
            };

            if let Some(msg) = processed {
                current_message = msg;
            } else {
                // Message was filtered out
                return Ok(None);
            }
        }

        Ok(Some(current_message))
    }
}

/// A composite router that tries multiple routers in sequence
#[derive(Clone)]
pub struct CompositeMessageRouter {
    routers: Vec<MessageRouterType>,
}

impl CompositeMessageRouter {
    /// Create a new composite router
    pub fn new(routers: Vec<MessageRouterType>) -> Self {
        Self { routers }
    }

    /// Add a router to the chain
    pub fn add_router(&mut self, router: MessageRouterType) {
        self.routers.push(router);
    }
}

impl MessageRouter for CompositeMessageRouter {
    fn route_message_impl(&self, message: &Message) -> Result<String> {
        // Try each router in sequence until one succeeds
        for router in &self.routers {
            let result = match router {
                MessageRouterType::Default(r) => r.route_message_impl(message),
            };

            match result {
                Ok(did) => return Ok(did),
                Err(_) => continue, // Try the next router
            }
        }

        // If we get here, all routers failed
        Err(crate::error::Error::Routing("No router could handle the message".to_string()))
    }
}
