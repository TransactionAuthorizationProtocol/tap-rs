//! Message handling for TAP Node
//!
//! This module provides message handling utilities for TAP Node.

mod processor;
mod processor_pool;
mod router;

pub use processor::*;
pub use processor_pool::ProcessorPool;
pub use processor_pool::ProcessorPoolConfig;
pub use router::*;

use async_trait::async_trait;
use tap_core::message::TapMessage;

use crate::error::Result;

/// Message processor for intercepting and processing messages
#[async_trait]
pub trait MessageProcessor: Send + Sync {
    /// Process an incoming message before it's dispatched to an agent
    ///
    /// Returns the processed message or None if the message should be dropped
    async fn process_incoming(&self, message: TapMessage) -> Result<Option<TapMessage>>;

    /// Process an outgoing message before it's sent
    ///
    /// Returns the processed message or None if the message should be dropped
    async fn process_outgoing(&self, message: TapMessage) -> Result<Option<TapMessage>>;
}

/// Router to determine the destination agent for a message
#[async_trait]
pub trait MessageRouter: Send + Sync {
    /// Determine the agent DID that should receive this message
    ///
    /// Returns the DID of the agent that should receive the message
    async fn route_message(&self, message: &TapMessage) -> Result<String>;
}

/// A message processor that applies multiple processors in sequence
#[derive(Default)]
pub struct CompositeMessageProcessor {
    /// The processors to apply in sequence
    processors: Vec<Box<dyn MessageProcessor>>,
}

impl CompositeMessageProcessor {
    /// Create a new composite message processor
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    /// Add a processor to the chain
    pub fn add_processor(&mut self, processor: Box<dyn MessageProcessor>) {
        self.processors.push(processor);
    }
}

#[async_trait]
impl MessageProcessor for CompositeMessageProcessor {
    async fn process_incoming(&self, message: TapMessage) -> Result<Option<TapMessage>> {
        let mut current_message = Some(message);

        // Apply each processor in sequence
        for processor in &self.processors {
            if let Some(msg) = current_message {
                current_message = processor.process_incoming(msg).await?;

                // If a processor returns None, stop processing
                if current_message.is_none() {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(current_message)
    }

    async fn process_outgoing(&self, message: TapMessage) -> Result<Option<TapMessage>> {
        let mut current_message = Some(message);

        // Apply each processor in sequence
        for processor in &self.processors {
            if let Some(msg) = current_message {
                current_message = processor.process_outgoing(msg).await?;

                // If a processor returns None, stop processing
                if current_message.is_none() {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(current_message)
    }
}
