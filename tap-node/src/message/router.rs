//! Message routing implementation.
//!
//! This module provides message routing capabilities for the TAP Node.

use log::debug;
use std::sync::Arc;
use tap_core::didcomm::Message;

use crate::agent::AgentRegistry;
use crate::error::{Error, Result};
use crate::message::MessageRouter;

/// Default message router using the "to" field in messages
#[derive(Clone)]
pub struct DefaultMessageRouter {
    /// Registry of agents
    agents: Arc<AgentRegistry>,
}

impl DefaultMessageRouter {
    /// Create a new default message router
    pub fn new(agents: Arc<AgentRegistry>) -> Self {
        Self { agents }
    }
}

impl MessageRouter for DefaultMessageRouter {
    fn route_message_impl(&self, message: &Message) -> Result<String> {
        // Check if the message has a "to" field
        if let Some(to) = &message.to {
            if !to.is_empty() {
                // Check if the first to DID exists in our registry
                let to_did = to[0].clone();
                if self.agents.has_agent(&to_did) {
                    debug!("Routing message to: {}", to_did);
                    return Ok(to_did);
                }
            }
        }

        // If we get here, we couldn't route the message
        Err(Error::Dispatch(format!(
            "No route found for message: {}",
            message.id
        )))
    }
}

/// A message router that combines multiple routers
#[derive(Clone)]
pub struct CompositeMessageRouter {
    routers: Vec<Arc<dyn MessageRouter>>,
}

impl Default for CompositeMessageRouter {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl CompositeMessageRouter {
    /// Create a new composite router
    pub fn new(routers: Vec<Arc<dyn MessageRouter>>) -> Self {
        Self { routers }
    }

    /// Add a router to the chain
    pub fn add_router(&mut self, router: Arc<dyn MessageRouter>) {
        self.routers.push(router);
    }
}

impl MessageRouter for CompositeMessageRouter {
    fn route_message_impl(&self, message: &Message) -> Result<String> {
        // Try each router in sequence
        for router in &self.routers {
            match router.route_message_impl(message) {
                Ok(did) => return Ok(did),
                Err(_) => continue, // Try the next router
            }
        }

        // If we get here, none of the routers worked
        Err(Error::Dispatch("No suitable route found for message".into()))
    }
}
