//! PlainMessage routing implementation.
//!
//! This module provides message routing capabilities for the TAP Node.

use log::debug;
use std::sync::Arc;
use tap_msg::didcomm::PlainMessage;

use crate::agent::AgentRegistry;
use crate::error::{Error, Result};
use crate::message::PlainMessageRouter;

/// Default implementation of PlainMessageRouter
#[derive(Debug, Clone)]
pub struct DefaultPlainMessageRouter {
    /// Registry of agents
    agents: Option<Arc<AgentRegistry>>,
}

impl Default for DefaultPlainMessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultPlainMessageRouter {
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

impl PlainMessageRouter for DefaultPlainMessageRouter {
    fn route_message_impl(&self, message: &PlainMessage) -> Result<String> {
        // Check if the message has a "to" field
        if !message.to.is_empty() {
            {
                // Check if the first to DID exists in our registry
                let to_did = message.to[0].clone();

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

/// Composite message router that can delegate to multiple routers
#[derive(Debug, Default)]
pub struct CompositePlainMessageRouter {
    /// The routers to use, in order
    routers: Vec<crate::message::PlainMessageRouterType>,
}

impl CompositePlainMessageRouter {
    /// Create a new composite router
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a router to the chain
    pub fn add_router(&mut self, router: crate::message::PlainMessageRouterType) {
        self.routers.push(router);
    }
}

impl PlainMessageRouter for CompositePlainMessageRouter {
    fn route_message_impl(&self, message: &PlainMessage) -> Result<String> {
        // Try each router in sequence
        for router in &self.routers {
            match router {
                crate::message::PlainMessageRouterType::Default(r) => {
                    match r.route_message_impl(message) {
                        Ok(target) => return Ok(target),
                        Err(_) => continue, // Try the next router
                    }
                } // Add other router types here if needed
            }
        }

        // If we get here, no router could handle the message
        Err(Error::Dispatch(format!(
            "No route found for message: {}",
            message.id
        )))
    }
}
