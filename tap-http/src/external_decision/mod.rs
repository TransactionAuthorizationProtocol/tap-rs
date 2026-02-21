//! External decision executable support
//!
//! This module provides support for delegating TAP transaction decisions
//! to an external long-running process that communicates over stdin/stdout
//! using JSON-RPC 2.0.

pub mod manager;
pub mod protocol;

pub use manager::{ExternalDecisionConfig, ExternalDecisionManager, SubscribeMode};
