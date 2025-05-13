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
    Core(#[from] tap_msg::error::Error),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// DID resolution errors
    #[error("DID resolution error: {0}")]
    DidResolution(String),

    /// Error related to invalid DID
    #[error("Invalid DID")]
    InvalidDID,

    /// Error for unsupported DID method
    #[error("Unsupported DID method: {0}")]
    UnsupportedDIDMethod(String),

    /// Error when failed to acquire resolver read lock
    #[error("Failed to acquire resolver read lock")]
    FailedToAcquireResolverReadLock,

    /// Error when failed to acquire resolver write lock
    #[error("Failed to acquire resolver write lock")]
    FailedToAcquireResolverWriteLock,

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

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Feature not implemented
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),

    /// DIDComm specific errors
    #[error("DIDComm error: {0}")]
    DIDComm(#[from] didcomm::error::Error),

    /// DID Resolution error
    #[error("DID Resolution error: {0}")]
    DIDResolution(String),

    /// JavaScript error (WASM)
    #[cfg(target_arch = "wasm32")]
    #[error("JavaScript error: {0}")]
    JsError(String),

    /// JavaScript resolver error (WASM)
    #[cfg(target_arch = "wasm32")]
    #[error("JavaScript resolver error: {0}")]
    JsResolverError(String),

    /// Serde JSON error
    #[error("Serde JSON error: {0}")]
    SerdeError(#[from] serde_json::Error),
}
