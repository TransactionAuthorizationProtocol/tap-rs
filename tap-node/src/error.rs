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
    InvalidPlainMessage(String),

    /// Error from agent
    #[error("Agent error: {0}")]
    Agent(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// PlainMessage dispatch error
    #[error("PlainMessage dispatch error: {0}")]
    Dispatch(String),

    /// PlainMessage processing error
    #[error("PlainMessage processing error: {0}")]
    Processing(String),

    /// PlainMessage routing error
    #[error("PlainMessage routing error: {0}")]
    Routing(String),

    /// Resolver error
    #[error("Resolver error: {0}")]
    Resolver(String),

    /// DID resolution error
    #[error("DID resolution error: {0}")]
    DidResolution(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Message dropped during processing
    #[error("Message dropped: {0}")]
    MessageDropped(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Verification error
    #[error("Verification error: {0}")]
    Verification(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type for TAP Node
pub type Result<T> = std::result::Result<T, Error>;

/// Convert from tap_agent::Error to Error
impl From<tap_agent::Error> for Error {
    fn from(err: tap_agent::Error) -> Self {
        Error::Agent(err.to_string())
    }
}
