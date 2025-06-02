//! Message validation framework for TAP Node
//!
//! This module provides a comprehensive validation system for incoming TAP messages.
//! It includes validators for:
//! - Message uniqueness (preventing duplicate messages)
//! - Timestamp validation (messages not too far in future/past)
//! - Agent authorization (only authorized agents can respond to transactions)
//! - Message expiry validation

use crate::storage::Storage;
use async_trait::async_trait;
use std::sync::Arc;
use tap_msg::didcomm::PlainMessage;

pub mod agent_validator;
pub mod timestamp_validator;
pub mod uniqueness_validator;

/// Result of message validation
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Message passed validation
    Accept,
    /// Message failed validation with reason
    Reject(String),
}

/// Trait for message validators
#[async_trait]
pub trait MessageValidator: Send + Sync {
    /// Validate a message
    ///
    /// Returns Accept if the message passes validation,
    /// or Reject with a reason if it fails.
    async fn validate(&self, message: &PlainMessage) -> ValidationResult;
}

/// Composite validator that runs multiple validators
pub struct CompositeValidator {
    validators: Vec<Box<dyn MessageValidator>>,
}

impl CompositeValidator {
    /// Create a new composite validator
    pub fn new(validators: Vec<Box<dyn MessageValidator>>) -> Self {
        Self { validators }
    }
}

#[async_trait]
impl MessageValidator for CompositeValidator {
    async fn validate(&self, message: &PlainMessage) -> ValidationResult {
        for validator in &self.validators {
            match validator.validate(message).await {
                ValidationResult::Accept => continue,
                ValidationResult::Reject(reason) => return ValidationResult::Reject(reason),
            }
        }
        ValidationResult::Accept
    }
}

/// Standard validator configuration
pub struct StandardValidatorConfig {
    /// Maximum allowed timestamp drift in seconds
    pub max_timestamp_drift_secs: i64,
    /// Storage for uniqueness and agent checks
    pub storage: Arc<Storage>,
}

// Note: StandardValidatorConfig doesn't have a Default implementation
// because storage must be provided by the caller

/// Create a standard validator with all recommended validators
pub async fn create_standard_validator(config: StandardValidatorConfig) -> CompositeValidator {
    let validators: Vec<Box<dyn MessageValidator>> = vec![
        Box::new(timestamp_validator::TimestampValidator::new(
            config.max_timestamp_drift_secs,
        )),
        Box::new(uniqueness_validator::UniquenessValidator::new(
            config.storage.clone(),
        )),
        Box::new(agent_validator::AgentAuthorizationValidator::new(
            config.storage.clone(),
        )),
    ];

    CompositeValidator::new(validators)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysAcceptValidator;

    #[async_trait]
    impl MessageValidator for AlwaysAcceptValidator {
        async fn validate(&self, _message: &PlainMessage) -> ValidationResult {
            ValidationResult::Accept
        }
    }

    struct AlwaysRejectValidator {
        reason: String,
    }

    #[async_trait]
    impl MessageValidator for AlwaysRejectValidator {
        async fn validate(&self, _message: &PlainMessage) -> ValidationResult {
            ValidationResult::Reject(self.reason.clone())
        }
    }

    #[tokio::test]
    async fn test_composite_validator_all_accept() {
        let validators: Vec<Box<dyn MessageValidator>> = vec![
            Box::new(AlwaysAcceptValidator),
            Box::new(AlwaysAcceptValidator),
        ];

        let composite = CompositeValidator::new(validators);
        let message = PlainMessage::new(
            "test_msg_1".to_string(),
            "test_type".to_string(),
            serde_json::json!({}),
            "did:example:sender".to_string(),
        )
        .with_recipient("did:example:receiver");

        match composite.validate(&message).await {
            ValidationResult::Accept => {} // Expected
            ValidationResult::Reject(reason) => panic!("Expected accept, got reject: {}", reason),
        }
    }

    #[tokio::test]
    async fn test_composite_validator_one_reject() {
        let validators: Vec<Box<dyn MessageValidator>> = vec![
            Box::new(AlwaysAcceptValidator),
            Box::new(AlwaysRejectValidator {
                reason: "Test rejection".to_string(),
            }),
            Box::new(AlwaysAcceptValidator),
        ];

        let composite = CompositeValidator::new(validators);
        let message = PlainMessage::new(
            "test_msg_1".to_string(),
            "test_type".to_string(),
            serde_json::json!({}),
            "did:example:sender".to_string(),
        )
        .with_recipient("did:example:receiver");

        match composite.validate(&message).await {
            ValidationResult::Accept => panic!("Expected reject, got accept"),
            ValidationResult::Reject(reason) => assert_eq!(reason, "Test rejection"),
        }
    }
}
