//! Agent authorization validation for transaction responses

use super::{MessageValidator, ValidationResult};
use crate::storage::Storage;
use async_trait::async_trait;
use std::sync::Arc;
use tap_msg::didcomm::PlainMessage;
use tap_msg::message::TapMessage;

/// Validator that ensures only authorized agents can respond to transactions
///
/// This validator checks that messages responding to a transaction (like Authorize,
/// Cancel, Reject) are only accepted from agents that are part of the transaction.
pub struct AgentAuthorizationValidator {
    storage: Arc<Storage>,
}

impl AgentAuthorizationValidator {
    /// Create a new agent authorization validator
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// Check if a message is a response to a transaction
    fn is_transaction_response(message: &PlainMessage) -> bool {
        // Messages that are responses to transactions typically have these types
        matches!(
            message.type_.as_str(),
            "https://tap.rsvp/schema/1.0#Authorize"
                | "https://tap.rsvp/schema/1.0#Cancel"
                | "https://tap.rsvp/schema/1.0#Reject"
                | "https://tap.rsvp/schema/1.0#Settle"
                | "https://tap.rsvp/schema/1.0#Revert"
                | "https://tap.rsvp/schema/1.0#AddAgents"
                | "https://tap.rsvp/schema/1.0#RemoveAgent"
                | "https://tap.rsvp/schema/1.0#ReplaceAgent"
                | "https://tap.rsvp/schema/1.0#UpdatePolicies"
        )
    }

    /// Extract transaction ID from message
    async fn get_transaction_id(&self, message: &PlainMessage) -> Option<String> {
        // First try to get it from thread_id
        if let Some(thread_id) = &message.thid {
            // Look up the original transaction by thread ID
            if let Ok(Some(transaction)) =
                self.storage.get_transaction_by_thread_id(thread_id).await
            {
                return Some(transaction.reference_id);
            }
        }

        // Try to parse the message and extract transaction_id from specific message types
        if let Ok(tap_message) = TapMessage::from_plain_message(message) {
            match tap_message {
                TapMessage::Authorize(auth) => Some(auth.transaction_id),
                TapMessage::Cancel(cancel) => Some(cancel.transaction_id),
                TapMessage::Reject(reject) => Some(reject.transaction_id),
                TapMessage::Settle(settle) => Some(settle.transaction_id),
                TapMessage::Revert(revert) => Some(revert.transaction_id),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[async_trait]
impl MessageValidator for AgentAuthorizationValidator {
    async fn validate(&self, message: &PlainMessage) -> ValidationResult {
        // Only validate transaction response messages
        if !Self::is_transaction_response(message) {
            return ValidationResult::Accept;
        }

        // Get the transaction ID
        let transaction_id = match self.get_transaction_id(message).await {
            Some(id) => id,
            None => {
                // Can't find transaction ID - this might be a new transaction
                // or a message type we don't need to validate
                return ValidationResult::Accept;
            }
        };

        // Check if the sender is authorized for this transaction
        match self
            .storage
            .is_agent_authorized_for_transaction(&transaction_id, &message.from)
            .await
        {
            Ok(true) => ValidationResult::Accept,
            Ok(false) => ValidationResult::Reject(format!(
                "Agent {} is not authorized to respond to transaction {}",
                message.from, transaction_id
            )),
            Err(e) => {
                ValidationResult::Reject(format!("Unable to verify agent authorization: {}", e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tap_msg::message::Authorize;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_non_transaction_response_accepted() {
        let dir = tempdir().unwrap();
        let storage = Arc::new(
            Storage::new(Some(dir.path().join("test.db")))
                .await
                .unwrap(),
        );
        let validator = AgentAuthorizationValidator::new(storage);

        // A Connect message is not a transaction response
        let message = PlainMessage::new(
            "test_msg_1".to_string(),
            "https://tap.rsvp/schema/1.0#Connect".to_string(),
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
    async fn test_authorize_for_new_transaction_rejected() {
        let dir = tempdir().unwrap();
        let storage = Arc::new(
            Storage::new(Some(dir.path().join("test.db")))
                .await
                .unwrap(),
        );
        let validator = AgentAuthorizationValidator::new(storage);

        // An Authorize message with a transaction_id that doesn't exist in storage
        // should be rejected because the sender is not authorized
        let authorize = Authorize {
            transaction_id: "new_transaction_123".to_string(),
            settlement_address: None,
            expiry: None,
        };

        let message = PlainMessage::new(
            "test_msg_2".to_string(),
            "https://tap.rsvp/schema/1.0#Authorize".to_string(),
            serde_json::to_value(&authorize).unwrap(),
            "did:example:sender".to_string(),
        )
        .with_recipient("did:example:receiver");

        match validator.validate(&message).await {
            ValidationResult::Accept => panic!("Expected reject, got accept"),
            ValidationResult::Reject(reason) => {
                assert!(reason.contains("not authorized"));
            }
        }
    }
}
