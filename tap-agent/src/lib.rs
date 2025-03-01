//! TAP Agent implementation
//!
//! This crate provides an agent implementation for the Transaction Authorization Protocol (TAP).
//! The TAP Agent is responsible for sending, receiving, and processing TAP messages.

mod agent;
mod config;
mod crypto;
mod did;
mod error;
mod policy;

pub use agent::{Agent, TapAgent};
pub use config::AgentConfig;
pub use crypto::MessagePacker;
pub use did::{DidDoc, DidResolver};
pub use error::{Error, Result};
pub use policy::{PolicyHandler, PolicyResult};

/// Version of the TAP Agent
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
