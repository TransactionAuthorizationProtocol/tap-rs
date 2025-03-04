//! Agent management for TAP Node
//!
//! This module provides utilities for managing multiple TAP agents within a TAP Node.

use dashmap::DashMap;
use std::sync::Arc;

use crate::error::{Error, Result};
use tap_agent::DefaultAgent;

/// Registry of TAP agents
#[derive(Debug)]
pub struct AgentRegistry {
    /// Maximum number of agents that can be registered
    max_agents: Option<usize>,
    /// Mapping of agent DIDs to agent instances
    agents: DashMap<String, Arc<DefaultAgent>>,
}

impl AgentRegistry {
    /// Create a new agent registry with the specified maximum number of agents
    pub fn new(max_agents: Option<usize>) -> Self {
        Self {
            max_agents,
            agents: DashMap::new(),
        }
    }

    /// Get the current number of registered agents
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Check if the registry has an agent with the given DID
    pub fn has_agent(&self, did: &str) -> bool {
        self.agents.contains_key(did)
    }

    /// Get an agent by DID
    pub async fn get_agent(&self, did: &str) -> Result<Arc<DefaultAgent>> {
        self.agents
            .get(did)
            .map(|agent| agent.clone())
            .ok_or_else(|| Error::AgentNotFound(did.to_string()))
    }

    /// Register a new agent
    pub async fn register_agent(&self, did: String, agent: Arc<DefaultAgent>) -> Result<()> {
        // Check if we've reached the maximum number of agents
        if let Some(max) = self.max_agents {
            if self.agent_count() >= max {
                return Err(Error::AgentRegistration(format!(
                    "Maximum number of agents ({}) reached",
                    max
                )));
            }
        }

        // Check if the agent is already registered
        if self.has_agent(&did) {
            return Err(Error::AgentRegistration(format!(
                "Agent with DID {} is already registered",
                did
            )));
        }

        // Register the agent
        self.agents.insert(did, agent);

        Ok(())
    }

    /// Unregister an agent
    pub async fn unregister_agent(&self, did: &str) -> Result<()> {
        // Check if the agent exists
        if !self.has_agent(did) {
            return Err(Error::AgentNotFound(did.to_string()));
        }

        // Unregister the agent
        self.agents.remove(did);

        Ok(())
    }

    /// Get all registered agent DIDs
    pub fn get_all_dids(&self) -> Vec<String> {
        self.agents
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}
