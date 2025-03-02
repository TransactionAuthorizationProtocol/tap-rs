//! Error handling for TAP Agent
//!
//! This module provides error types and utilities for the TAP Agent.

use thiserror::Error;

/// Type alias for Results with TAP Agent errors
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for TAP Agent
#[derive(Error, Debug)]
pub enum Error {
    /// Core TAP errors
    #[error("Core error: {0}")]
    Core(#[from] tap_core::error::Error),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// DID resolution errors
    #[error("DID resolution error: {0}")]
    DidResolution(String),

    /// Error related to DID not found
    #[error("DID not found: {0}")]
    DidNotFound(String),

    /// Error related to missing configuration
    #[error("Missing configuration: {0}")]
    MissingConfig(String),

    /// Error related to cryptographic operations
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Error related to message processing
    #[error("Message error: {0}")]
    Message(String),

    /// Error related to policy evaluation
    #[error("Policy error: {0}")]
    Policy(String),

    /// Error related to storage
    #[error("Storage error: {0}")]
    Storage(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Feature not implemented
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
