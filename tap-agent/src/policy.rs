//! Policy handling for TAP Agent
//!
//! This module provides policy-related functionality for the TAP Agent.

use async_trait::async_trait;
use std::fmt::Debug;

use crate::error::Result;

/// Result of a policy evaluation
#[derive(Debug, Clone)]
pub struct PolicyResult {
    /// Whether the message is allowed
    pub allowed: bool,
    /// Reason for the decision
    pub reason: Option<String>,
}

/// Trait for policy handlers that evaluate messages against policies
#[async_trait]
pub trait PolicyHandler: Send + Sync + Debug {
    /// Evaluates an outgoing message against policies
    async fn evaluate_outgoing(&self, message: &(dyn erased_serde::Serialize + Sync))
        -> Result<()>;

    /// Evaluates an incoming message against policies
    async fn evaluate_incoming(&self, message: &serde_json::Value) -> Result<()>;
}

/// A policy handler that does not enforce any policies
#[derive(Debug)]
pub struct DefaultPolicyHandler;

impl Default for DefaultPolicyHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultPolicyHandler {
    /// Creates a new DefaultPolicyHandler
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PolicyHandler for DefaultPolicyHandler {
    async fn evaluate_outgoing(
        &self,
        _message: &(dyn erased_serde::Serialize + Sync),
    ) -> Result<()> {
        // Default implementation allows all outgoing messages
        Ok(())
    }

    async fn evaluate_incoming(&self, _message: &serde_json::Value) -> Result<()> {
        // Default implementation allows all incoming messages
        Ok(())
    }
}
