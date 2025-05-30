//! PlainMessage routing implementation.
//!
//! This module provides message routing capabilities for the TAP Node.

use log::debug;
use std::sync::Arc;
use tap_agent::Agent;
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
                }
                crate::message::PlainMessageRouterType::IntraNode(r) => {
                    match r.route_message_impl(message) {
                        Ok(target) => return Ok(target),
                        Err(_) => continue, // Try the next router
                    }
                }
            }
        }

        // If we get here, no router could handle the message
        Err(Error::Dispatch(format!(
            "No route found for message: {}",
            message.id
        )))
    }
}

/// Intra-node router that handles routing between local agents
#[derive(Debug, Clone)]
pub struct IntraNodePlainMessageRouter {
    /// Registry of agents
    agents: Option<Arc<AgentRegistry>>,
}

impl Default for IntraNodePlainMessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl IntraNodePlainMessageRouter {
    /// Create a new intra-node router
    pub fn new() -> Self {
        Self { agents: None }
    }

    /// Set the agent registry
    pub fn with_agents(mut self, agents: Arc<AgentRegistry>) -> Self {
        self.agents = Some(agents);
        self
    }

    /// Route a message to local agents if possible
    async fn route_to_local_agents(&self, message: &PlainMessage) -> Result<Vec<String>> {
        let mut local_recipients = Vec::new();

        if let Some(agents) = &self.agents {
            // Check each recipient to see if they're local
            for recipient in &message.to {
                if agents.has_agent(recipient) {
                    local_recipients.push(recipient.clone());
                }
            }
        }

        Ok(local_recipients)
    }

    /// Send message to local agents
    pub async fn deliver_to_local_agents(&self, message: &PlainMessage) -> Result<()> {
        if let Some(agents) = &self.agents {
            let local_recipients = self.route_to_local_agents(message).await?;

            for recipient_did in local_recipients {
                if let Ok(agent) = agents.get_agent(&recipient_did).await {
                    // Send message to local agent
                    if let Err(e) = agent.receive_plain_message(message.clone()).await {
                        log::warn!(
                            "Failed to deliver message {} to local agent {}: {}",
                            message.id,
                            recipient_did,
                            e
                        );
                    } else {
                        log::debug!(
                            "Delivered message {} to local agent {}",
                            message.id,
                            recipient_did
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

impl PlainMessageRouter for IntraNodePlainMessageRouter {
    fn route_message_impl(&self, message: &PlainMessage) -> Result<String> {
        // This router prioritizes local agents
        if let Some(agents) = &self.agents {
            for recipient in &message.to {
                if agents.has_agent(recipient) {
                    debug!("Routing message to local agent: {}", recipient);
                    return Ok(recipient.clone());
                }
            }
        }

        // If no local agents found, fall back to external routing
        Err(Error::Dispatch(format!(
            "No local agents found for message: {}",
            message.id
        )))
    }
}
