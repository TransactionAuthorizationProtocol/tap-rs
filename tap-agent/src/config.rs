//! Configuration for the TAP Agent

use crate::error::Result;

/// Configuration options for a TAP Agent
#[derive(Clone, Debug)]
pub struct AgentConfig {
    /// Endpoint URL where the agent can receive messages
    pub endpoint: Option<String>,

    /// Agent name
    pub name: Option<String>,

    /// Agent DID
    pub did: Option<String>,

    /// Whether to verify sender's DID in incoming messages
    pub verify_sender: bool,

    /// Whether to use authenticated encryption (authcrypt) by default
    /// If false, anonymous encryption (anoncrypt) will be used
    pub use_authcrypt: bool,
}

impl AgentConfig {
    /// Creates a new agent configuration with default values
    pub fn new() -> Self {
        Self {
            endpoint: None,
            name: None,
            did: None,
            verify_sender: true,
            use_authcrypt: true,
        }
    }

    /// Creates a new agent configuration with the specified DID
    pub fn new_with_did(did: impl Into<String>) -> Self {
        Self {
            endpoint: None,
            name: None,
            did: Some(did.into()),
            verify_sender: true,
            use_authcrypt: true,
        }
    }

    /// Sets the agent's endpoint
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Sets the agent's name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the agent's DID
    pub fn with_did(mut self, did: impl Into<String>) -> Self {
        self.did = Some(did.into());
        self
    }

    /// Sets whether to verify sender's DID in incoming messages
    pub fn with_verify_sender(mut self, verify: bool) -> Self {
        self.verify_sender = verify;
        self
    }

    /// Sets whether to use authenticated encryption (authcrypt) by default
    pub fn with_authcrypt(mut self, use_authcrypt: bool) -> Self {
        self.use_authcrypt = use_authcrypt;
        self
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<()> {
        // Add validation logic as needed
        Ok(())
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self::new()
    }
}
