//! Message Context Pattern for TAP messages.
//!
//! This module provides a declarative way to handle message context,
//! including participant extraction, routing hints, and transaction context.

use crate::message::Participant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Message context providing participants, routing hints, and transaction context
pub trait MessageContext {
    /// Extract all participants from the message
    fn participants(&self) -> Vec<&Participant>;

    /// Get routing hints for message delivery
    fn routing_hints(&self) -> RoutingHints {
        RoutingHints::default()
    }

    /// Get transaction context if applicable
    fn transaction_context(&self) -> Option<TransactionContext> {
        None
    }

    /// Extract all participant DIDs for routing
    fn participant_dids(&self) -> Vec<String> {
        self.participants()
            .into_iter()
            .map(|p| p.id.clone())
            .collect()
    }

    /// Get transaction ID if available
    fn transaction_id(&self) -> Option<String> {
        self.transaction_context().map(|ctx| ctx.transaction_id)
    }
}

/// Routing hints for message delivery
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoutingHints {
    /// Preferred delivery endpoints
    pub preferred_endpoints: Vec<String>,

    /// Priority routing (high, normal, low)
    pub priority: Priority,

    /// Whether to use encryption
    pub require_encryption: bool,

    /// Custom routing metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Priority levels for message routing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Priority {
    High,
    #[default]
    Normal,
    Low,
}

/// Transaction context for messages that are part of a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionContext {
    /// The transaction ID
    pub transaction_id: String,

    /// Parent transaction ID if this is a sub-transaction
    pub parent_transaction_id: Option<String>,

    /// Transaction type (transfer, payment, etc.)
    pub transaction_type: String,

    /// Transaction metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TransactionContext {
    /// Create a new transaction context
    pub fn new(transaction_id: String, transaction_type: String) -> Self {
        Self {
            transaction_id,
            parent_transaction_id: None,
            transaction_type,
            metadata: HashMap::new(),
        }
    }

    /// Set parent transaction ID
    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_transaction_id = Some(parent_id);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// A helper trait for extracting participants using attributes
pub trait ParticipantExtractor {
    /// Extract participants marked with #[tap(participant)]
    fn extract_single_participants(&self) -> Vec<&Participant>;

    /// Extract participants from lists marked with #[tap(participant_list)]
    fn extract_list_participants(&self) -> Vec<&Participant>;

    /// Extract optional participants marked with #[tap(participant)]
    fn extract_optional_participants(&self) -> Vec<&Participant>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Participant;

    #[test]
    fn test_routing_hints_default() {
        let hints = RoutingHints::default();
        assert!(hints.preferred_endpoints.is_empty());
        assert!(matches!(hints.priority, Priority::Normal));
        assert!(!hints.require_encryption);
        assert!(hints.metadata.is_empty());
    }

    #[test]
    fn test_transaction_context() {
        let ctx = TransactionContext::new("tx-123".to_string(), "transfer".to_string())
            .with_parent("parent-tx-456".to_string())
            .with_metadata(
                "key".to_string(),
                serde_json::Value::String("value".to_string()),
            );

        assert_eq!(ctx.transaction_id, "tx-123");
        assert_eq!(ctx.transaction_type, "transfer");
        assert_eq!(ctx.parent_transaction_id, Some("parent-tx-456".to_string()));
        assert_eq!(
            ctx.metadata.get("key").unwrap(),
            &serde_json::Value::String("value".to_string())
        );
    }

    // Mock implementation for testing
    struct TestMessage {
        originator: Participant,
        beneficiary: Option<Participant>,
        agents: Vec<Participant>,
        transaction_id: String,
    }

    impl MessageContext for TestMessage {
        fn participants(&self) -> Vec<&Participant> {
            let mut participants = vec![&self.originator];
            if let Some(ref beneficiary) = self.beneficiary {
                participants.push(beneficiary);
            }
            participants.extend(&self.agents);
            participants
        }

        fn transaction_context(&self) -> Option<TransactionContext> {
            Some(TransactionContext::new(
                self.transaction_id.clone(),
                "test".to_string(),
            ))
        }
    }

    #[test]
    fn test_message_context() {
        let msg = TestMessage {
            originator: Participant {
                id: "did:example:alice".to_string(),
                role: Some("originator".to_string()),
                policies: None,
                leiCode: None,
                name: None,
            },
            beneficiary: Some(Participant {
                id: "did:example:bob".to_string(),
                role: Some("beneficiary".to_string()),
                policies: None,
                leiCode: None,
                name: None,
            }),
            agents: vec![Participant {
                id: "did:example:agent".to_string(),
                role: Some("agent".to_string()),
                policies: None,
                leiCode: None,
                name: None,
            }],
            transaction_id: "tx-123".to_string(),
        };

        let participants = msg.participants();
        assert_eq!(participants.len(), 3);

        let dids = msg.participant_dids();
        assert_eq!(dids.len(), 3);
        assert!(dids.contains(&"did:example:alice".to_string()));
        assert!(dids.contains(&"did:example:bob".to_string()));
        assert!(dids.contains(&"did:example:agent".to_string()));

        assert_eq!(msg.transaction_id(), Some("tx-123".to_string()));
    }
}
