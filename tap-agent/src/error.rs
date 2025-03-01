//! Error handling for TAP Agent
//!
//! This module provides error types and utilities for the TAP Agent.

use thiserror::Error;

/// Type alias for Results with TAP Agent errors
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for TAP Agent
#[derive(Error, Debug)]
pub enum Error {
    /// Error related to configuration
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Error related to DID resolution
    #[error("DID resolution error: {0}")]
    DidResolution(String),

    /// Error related to cryptographic operations
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Error related to message validation
    #[error("Validation error: {0}")]
    Validation(String),

    /// Error related to message processing
    #[error("Processing error: {0}")]
    Processing(String),

    /// Error related to invalid data
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Error from TAP Core
    #[error("TAP Core error: {0}")]
    Core(#[from] tap_core::error::Error),

    /// Other generic errors
    #[error("Other error: {0}")]
    Other(String),
}
