//! Configuration for the TAP Agent

use crate::error::Result;
use std::collections::HashMap;

/// Configuration options for a TAP Agent
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Agent DID
    pub agent_did: String,

    /// Security mode for messages
    pub security_mode: Option<String>,

    /// Additional configuration parameters
    pub parameters: HashMap<String, String>,
}

impl AgentConfig {
    /// Creates a new AgentConfig with the specified DID
    pub fn new(did: String) -> Self {
        Self {
            agent_did: did,
            security_mode: Some("PLAIN".to_string()),
            parameters: HashMap::new(),
        }
    }

    /// Sets a configuration parameter
    pub fn set_parameter(&mut self, key: &str, value: &str) {
        self.parameters.insert(key.to_string(), value.to_string());
    }

    /// Gets a configuration parameter
    pub fn get_parameter(&self, key: &str) -> Option<&String> {
        self.parameters.get(key)
    }

    /// Sets the security mode
    pub fn with_security_mode(mut self, mode: &str) -> Self {
        self.security_mode = Some(mode.to_string());
        self
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self::new("default_did".to_string())
    }
}

/// Validates an Agent configuration for required fields
pub fn validate(_config: &AgentConfig) -> Result<()> {
    // TODO: Add validation logic here
    Ok(())
}
