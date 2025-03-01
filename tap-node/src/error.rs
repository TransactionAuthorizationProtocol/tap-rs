//! Error handling for TAP Node

use thiserror::Error;

/// Error types for TAP Node
#[derive(Error, Debug)]
pub enum Error {
    /// Agent not found
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// Agent registration error
    #[error("Agent registration error: {0}")]
    AgentRegistration(String),

    /// Invalid TAP message
    #[error("Invalid TAP message: {0}")]
    InvalidMessage(String),

    /// Error from agent
    #[error("Agent error: {0}")]
    Agent(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(serde_json::Error),

    /// Message dispatch error
    #[error("Message dispatch error: {0}")]
    Dispatch(String),

    /// Resolver error
    #[error("Resolver error: {0}")]
    Resolver(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Result type for TAP Node
pub type Result<T> = std::result::Result<T, Error>;
