//! Error types for the tap-msg crate.

use std::result;
use thiserror::Error;

/// Core TAP error types.
#[derive(Debug, Error)]
pub enum Error {
    /// Error related to DIDComm operations.
    #[error("DIDComm error: {0}")]
    DIDComm(String),

    /// Error related to serialization/deserialization.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Error related to parsing operations.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Error related to validation failures.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Error related to mismatched message types.
    #[error("Invalid message type: {0}")]
    InvalidMessageType(String),

    /// Error related to CAIP validation.
    #[error("CAIP error: {0}")]
    CaipError(#[from] tap_caip::error::Error),
}

/// Custom Result type for TAP Core operations.
pub type Result<T> = result::Result<T, Error>;
