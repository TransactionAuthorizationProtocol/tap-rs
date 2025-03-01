//! Policy handling for TAP Agent
//!
//! This module provides policy-related functionality for the TAP Agent.

use crate::error::Result;
use async_trait::async_trait;
use tap_core::message::TapMessage;

/// Result of policy evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyResult {
    /// Message is allowed
    Allow,
    /// Message is denied with a reason
    Deny(String),
}

/// Handler for policy evaluation
#[async_trait]
pub trait PolicyHandler: Send + Sync {
    /// Evaluates a message against policies
    ///
    /// # Arguments
    /// * `message` - The message to evaluate
    ///
    /// # Returns
    /// * `Ok(PolicyResult)` - The result of policy evaluation
    /// * `Err` - If evaluation fails
    async fn evaluate_message(&self, message: &TapMessage) -> Result<PolicyResult>;
}

/// Default implementation that allows all messages
#[derive(Debug, Default)]
pub struct DefaultPolicyHandler;

impl DefaultPolicyHandler {
    /// Creates a new instance
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PolicyHandler for DefaultPolicyHandler {
    async fn evaluate_message(&self, _message: &TapMessage) -> Result<PolicyResult> {
        // Default implementation just allows all messages
        Ok(PolicyResult::Allow)
    }
}
