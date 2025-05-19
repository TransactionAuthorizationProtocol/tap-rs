//! PlainMessage processing and routing for TAP Node
//!
//! This module provides functionality for processing and routing TAP messages between agents.

pub mod processor;
pub mod processor_pool;
pub mod router;
pub mod sender;

// Re-export processors, routers, and senders
pub use processor::{
    DefaultPlainMessageProcessor, LoggingPlainMessageProcessor, PlainMessageProcessor,
    ValidationPlainMessageProcessor,
};
pub use processor_pool::{ProcessorPool, ProcessorPoolConfig};
pub use router::DefaultPlainMessageRouter;
pub use sender::{HttpPlainMessageSender, NodePlainMessageSender, PlainMessageSender};

// Import the PlainMessage type from tap-msg
use crate::error::Result;
use async_trait::async_trait;
use tap_msg::didcomm::PlainMessage;

/// Router to determine the destination agent for a message
pub trait PlainMessageRouter: Send + Sync {
    /// Route a message to determine the target agent DID
    fn route_message_impl(&self, message: &PlainMessage) -> Result<String>;
}

/// Async extension trait for the PlainMessageRouter
#[async_trait]
pub trait RouterAsyncExt: PlainMessageRouter {
    /// Route a message asynchronously
    async fn route_message(&self, message: &PlainMessage) -> Result<String>;
}

#[async_trait]
impl<T: PlainMessageRouter + Sync> RouterAsyncExt for T {
    async fn route_message(&self, message: &PlainMessage) -> Result<String> {
        self.route_message_impl(message)
    }
}

/// Processor enum to replace trait objects
#[derive(Clone, Debug)]
pub enum PlainMessageProcessorType {
    Default(DefaultPlainMessageProcessor),
    Logging(LoggingPlainMessageProcessor),
    Validation(ValidationPlainMessageProcessor),
    Composite(CompositePlainMessageProcessor),
}

/// Router enum to replace trait objects
#[derive(Clone, Debug)]
pub enum PlainMessageRouterType {
    Default(DefaultPlainMessageRouter),
}

/// A message processor that applies multiple processors in sequence
#[derive(Clone, Debug)]
pub struct CompositePlainMessageProcessor {
    processors: Vec<PlainMessageProcessorType>,
}

impl CompositePlainMessageProcessor {
    /// Create a new composite message processor
    pub fn new(processors: Vec<PlainMessageProcessorType>) -> Self {
        Self { processors }
    }

    /// Add a processor to the chain
    pub fn add_processor(&mut self, processor: PlainMessageProcessorType) {
        self.processors.push(processor);
    }
}

#[async_trait]
impl PlainMessageProcessor for CompositePlainMessageProcessor {
    async fn process_incoming(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
        let mut current_message = message;

        for processor in &self.processors {
            let processed = match processor {
                PlainMessageProcessorType::Default(p) => {
                    p.process_incoming(current_message).await?
                }
                PlainMessageProcessorType::Logging(p) => {
                    p.process_incoming(current_message).await?
                }
                PlainMessageProcessorType::Validation(p) => {
                    p.process_incoming(current_message).await?
                }
                PlainMessageProcessorType::Composite(p) => {
                    p.process_incoming(current_message).await?
                }
            };

            if let Some(msg) = processed {
                current_message = msg;
            } else {
                // PlainMessage was filtered out
                return Ok(None);
            }
        }

        Ok(Some(current_message))
    }

    async fn process_outgoing(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
        let mut current_message = message;

        for processor in &self.processors {
            let processed = match processor {
                PlainMessageProcessorType::Default(p) => {
                    p.process_outgoing(current_message).await?
                }
                PlainMessageProcessorType::Logging(p) => {
                    p.process_outgoing(current_message).await?
                }
                PlainMessageProcessorType::Validation(p) => {
                    p.process_outgoing(current_message).await?
                }
                PlainMessageProcessorType::Composite(p) => {
                    p.process_outgoing(current_message).await?
                }
            };

            if let Some(msg) = processed {
                current_message = msg;
            } else {
                // PlainMessage was filtered out
                return Ok(None);
            }
        }

        Ok(Some(current_message))
    }
}

/// A composite router that tries multiple routers in sequence
#[derive(Clone)]
pub struct CompositePlainMessageRouter {
    routers: Vec<PlainMessageRouterType>,
}

impl CompositePlainMessageRouter {
    /// Create a new composite router
    pub fn new(routers: Vec<PlainMessageRouterType>) -> Self {
        Self { routers }
    }

    /// Add a router to the chain
    pub fn add_router(&mut self, router: PlainMessageRouterType) {
        self.routers.push(router);
    }
}

impl PlainMessageRouter for CompositePlainMessageRouter {
    fn route_message_impl(&self, message: &PlainMessage) -> Result<String> {
        // Try each router in sequence until one succeeds
        for router in &self.routers {
            let result = match router {
                PlainMessageRouterType::Default(r) => r.route_message_impl(message),
            };

            match result {
                Ok(did) => return Ok(did),
                Err(_) => continue, // Try the next router
            }
        }

        // If we get here, all routers failed
        Err(crate::error::Error::Routing(
            "No router could handle the message".to_string(),
        ))
    }
}
