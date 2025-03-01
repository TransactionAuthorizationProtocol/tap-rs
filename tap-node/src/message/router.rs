//! Message routing for TAP Node
//!
//! This module provides message routing capabilities for the TAP Node.

use async_trait::async_trait;
use std::sync::Arc;
use tap_core::message::TapMessage;

use crate::agent::AgentRegistry;
use crate::error::{Error, Result};
use crate::message::MessageRouter;

/// Default router that routes messages based on the recipient DID
pub struct DefaultMessageRouter {
    /// Registry of agents
    agent_registry: Arc<AgentRegistry>,
}

impl DefaultMessageRouter {
    /// Create a new default message router
    pub fn new(agent_registry: Arc<AgentRegistry>) -> Self {
        Self { agent_registry }
    }
}

#[async_trait]
impl MessageRouter for DefaultMessageRouter {
    async fn route_message(&self, message: &TapMessage) -> Result<String> {
        // Get the to field from the message
        let to = message.to_did.as_ref().ok_or_else(|| {
            Error::Dispatch("Message missing recipient (to_did) field".to_string())
        })?;

        // Check if we have an agent for this DID
        if !self.agent_registry.has_agent(to) {
            return Err(Error::Dispatch(format!(
                "No agent registered for DID: {}",
                to
            )));
        }

        Ok(to.clone())
    }
}

/// A message router that combines multiple routers
#[derive(Clone)]
pub struct CompositeMessageRouter {
    routers: Vec<Arc<dyn MessageRouter>>,
}

impl Default for CompositeMessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeMessageRouter {
    /// Create a new composite router
    pub fn new() -> Self {
        Self {
            routers: Vec::new(),
        }
    }

    /// Add a router to the chain
    pub fn add_router(&mut self, router: Arc<dyn MessageRouter>) {
        self.routers.push(router);
    }
}

#[async_trait]
impl MessageRouter for CompositeMessageRouter {
    async fn route_message(&self, message: &TapMessage) -> Result<String> {
        // Try each router in sequence
        let mut last_error = None;

        for router in &self.routers {
            match router.route_message(message).await {
                Ok(did) => return Ok(did),
                Err(err) => last_error = Some(err),
            }
        }

        // If we reach here, all routers failed
        Err(last_error.unwrap_or_else(|| {
            Error::Dispatch("No routers available to route message".to_string())
        }))
    }
}
