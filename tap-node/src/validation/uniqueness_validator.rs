//! Message uniqueness validation

use super::{MessageValidator, ValidationResult};
use crate::storage::Storage;
use async_trait::async_trait;
use std::sync::Arc;
use tap_msg::didcomm::PlainMessage;

/// Validator that ensures message uniqueness
///
/// This validator checks that we haven't already received a message
/// with the same ID, preventing replay attacks and duplicate processing.
pub struct UniquenessValidator {
    storage: Arc<Storage>,
}

impl UniquenessValidator {
    /// Create a new uniqueness validator
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl MessageValidator for UniquenessValidator {
    async fn validate(&self, message: &PlainMessage) -> ValidationResult {
        // Check if message already exists in storage
        match self.storage.get_message_by_id(&message.id).await {
            Ok(Some(_)) => {
                // Message already exists
                ValidationResult::Reject(format!(
                    "Duplicate message: message with ID {} already processed",
                    message.id
                ))
            }
            Ok(None) => {
                // Message is unique
                ValidationResult::Accept
            }
            Err(e) => {
                // Storage error - reject to be safe
                ValidationResult::Reject(format!("Unable to verify message uniqueness: {}", e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MessageDirection;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_unique_message() {
        let dir = tempdir().unwrap();
        let storage = Arc::new(
            Storage::new(Some(dir.path().join("test.db")))
                .await
                .unwrap(),
        );
        let validator = UniquenessValidator::new(storage);

        let message = PlainMessage::new(
            "test_msg_1".to_string(),
            "test_type".to_string(),
            serde_json::json!({}),
            "did:example:sender".to_string(),
        )
        .with_recipient("did:example:receiver");

        match validator.validate(&message).await {
            ValidationResult::Accept => {} // Expected
            ValidationResult::Reject(reason) => panic!("Expected accept, got reject: {}", reason),
        }
    }

    #[tokio::test]
    async fn test_duplicate_message() {
        let dir = tempdir().unwrap();
        let storage = Arc::new(
            Storage::new(Some(dir.path().join("test.db")))
                .await
                .unwrap(),
        );
        let validator = UniquenessValidator::new(storage.clone());

        let message = PlainMessage::new(
            "test_msg_1".to_string(),
            "test_type".to_string(),
            serde_json::json!({}),
            "did:example:sender".to_string(),
        )
        .with_recipient("did:example:receiver");

        // Store the message first
        storage
            .log_message(&message, MessageDirection::Incoming, None)
            .await
            .unwrap();

        // Now validate - should be rejected as duplicate
        match validator.validate(&message).await {
            ValidationResult::Accept => panic!("Expected reject, got accept"),
            ValidationResult::Reject(reason) => {
                assert!(reason.contains("Duplicate message"));
                assert!(reason.contains(&message.id));
            }
        }
    }
}
