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
    agents: Option<Arc<AgentRegistry>>,
}

impl DefaultMessageRouter {
    /// Create a new default message router
    pub fn new() -> Self {
        Self { agents: None }
    }
    
    /// Set the agent registry
    pub fn with_agents(mut self, agents: Arc<AgentRegistry>) -> Self {
        self.agents = Some(agents);
        self
    }
}

impl MessageRouter for DefaultMessageRouter {
    fn route_message_impl(&self, message: &Message) -> Result<String> {
        // Check if the message has a "to" field
        if let Some(to) = &message.to {
            if !to.is_empty() {
                // Check if the first to DID exists in our registry
                let to_did = to[0].clone();
                
                // If we have an agent registry, check if the agent exists
                if let Some(agents) = &self.agents {
                    if agents.has_agent(&to_did) {
                        debug!("Routing message to: {}", to_did);
                        return Ok(to_did);
                    }
                } else {
                    // If we don't have an agent registry, just return the first DID
                    debug!("No agent registry available, routing to: {}", to_did);
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
    /// The routers to use, in order
    routers: Vec<Arc<dyn MessageRouter>>,
}

impl Default for CompositeMessageRouter {
    fn default() -> Self {
        Self { routers: Vec::new() }
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
                Ok(target) => return Ok(target),
                Err(_) => continue, // Try the next router
            }
        }

        // If we get here, no router could handle the message
        Err(Error::Dispatch(format!(
            "No route found for message: {}",
            message.id
        )))
    }
}
