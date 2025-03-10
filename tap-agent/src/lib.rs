//! TAP Agent implementation
//!
//! This crate provides an agent implementation for the Transaction Authorization Protocol (TAP).
//! The TAP Agent is responsible for sending and receiving TAP messages, managing keys, and
//! applying policies.

/// Agent implementation
pub mod agent;

/// Agent configuration
pub mod config;

/// Cryptographic utilities
pub mod crypto;

/// DID utilities
pub mod did;

/// Error types
pub mod error;

/// Message types and utilities
pub mod message;

/// A trait for types that can be serialized to JSON in an type-erased way
pub trait ErasedSerialize {
    /// Serialize to JSON string
    fn to_json(&self) -> std::result::Result<String, serde_json::Error>;
}

impl<T: serde::Serialize> ErasedSerialize for T {
    fn to_json(&self) -> std::result::Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

// Re-export key types for convenience
pub use agent::{Agent, DefaultAgent};
pub use config::AgentConfig;
pub use crypto::{BasicSecretResolver, DefaultMessagePacker, MessagePacker};
pub use did::{DIDMethodResolver, KeyResolver, MultiResolver, SyncDIDResolver};
pub use didcomm::did::{DIDDoc, DIDResolver};
pub use error::{Error, Result};
pub use message::{SecurityMode, PRESENTATION_MESSAGE_TYPE};

/// Version of the TAP Agent
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
