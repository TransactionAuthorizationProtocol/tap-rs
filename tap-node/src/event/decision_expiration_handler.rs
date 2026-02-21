//! Decision expiration handler
//!
//! Listens for `TransactionStateChanged` events and expires pending/delivered
//! decisions when a transaction reaches a terminal state (Rejected, Cancelled,
//! Reverted). This prevents external decision processes from acting on stale
//! decisions after a transaction has already been resolved.

use super::{EventSubscriber, NodeEvent};
use crate::state_machine::fsm::TransactionState;
use crate::storage::Storage;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error};

/// Expires pending decisions when transactions reach terminal states
pub struct DecisionExpirationHandler {
    storage: Arc<Storage>,
}

impl DecisionExpirationHandler {
    /// Create a new decision expiration handler
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl EventSubscriber for DecisionExpirationHandler {
    async fn handle_event(&self, event: NodeEvent) {
        if let NodeEvent::TransactionStateChanged {
            transaction_id,
            new_state,
            ..
        } = event
        {
            // Parse the new state and check if it's terminal
            if let Ok(state) = new_state.parse::<TransactionState>() {
                if state.is_terminal() {
                    debug!(
                        "Transaction {} reached terminal state {}, expiring pending decisions",
                        transaction_id, new_state
                    );
                    match self
                        .storage
                        .expire_decisions_for_transaction(&transaction_id)
                        .await
                    {
                        Ok(count) => {
                            if count > 0 {
                                debug!(
                                    "Expired {} decisions for transaction {}",
                                    count, transaction_id
                                );
                            }
                        }
                        Err(e) => {
                            error!(
                                "Failed to expire decisions for transaction {}: {}",
                                transaction_id, e
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{DecisionStatus, DecisionType};
    use serde_json::json;

    #[tokio::test]
    async fn test_expires_pending_decisions_on_terminal_state() {
        let storage = Arc::new(Storage::new_in_memory().await.unwrap());
        let handler = DecisionExpirationHandler::new(storage.clone());

        let context = json!({"info": "test"});

        // Insert a pending decision
        let id = storage
            .insert_decision(
                "txn-100",
                "did:key:z6MkAgent1",
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        // Simulate transaction reaching Rejected state
        handler
            .handle_event(NodeEvent::TransactionStateChanged {
                transaction_id: "txn-100".to_string(),
                old_state: "received".to_string(),
                new_state: "rejected".to_string(),
                agent_did: Some("did:key:z6MkOther".to_string()),
            })
            .await;

        let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
        assert_eq!(entry.status, DecisionStatus::Expired);
    }

    #[tokio::test]
    async fn test_expires_delivered_decisions_on_terminal_state() {
        let storage = Arc::new(Storage::new_in_memory().await.unwrap());
        let handler = DecisionExpirationHandler::new(storage.clone());

        let context = json!({"info": "test"});

        let id = storage
            .insert_decision(
                "txn-101",
                "did:key:z6MkAgent1",
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        // Mark as delivered
        storage
            .update_decision_status(id, DecisionStatus::Delivered, None, None)
            .await
            .unwrap();

        // Transaction cancelled
        handler
            .handle_event(NodeEvent::TransactionStateChanged {
                transaction_id: "txn-101".to_string(),
                old_state: "received".to_string(),
                new_state: "cancelled".to_string(),
                agent_did: None,
            })
            .await;

        let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
        assert_eq!(entry.status, DecisionStatus::Expired);
    }

    #[tokio::test]
    async fn test_does_not_expire_resolved_decisions() {
        let storage = Arc::new(Storage::new_in_memory().await.unwrap());
        let handler = DecisionExpirationHandler::new(storage.clone());

        let context = json!({"info": "test"});

        let id = storage
            .insert_decision(
                "txn-102",
                "did:key:z6MkAgent1",
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        // Resolve the decision
        storage
            .update_decision_status(id, DecisionStatus::Resolved, Some("authorize"), None)
            .await
            .unwrap();

        // Transaction reverted
        handler
            .handle_event(NodeEvent::TransactionStateChanged {
                transaction_id: "txn-102".to_string(),
                old_state: "settled".to_string(),
                new_state: "reverted".to_string(),
                agent_did: None,
            })
            .await;

        // Resolved decision should NOT be expired
        let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
        assert_eq!(entry.status, DecisionStatus::Resolved);
    }

    #[tokio::test]
    async fn test_ignores_non_terminal_states() {
        let storage = Arc::new(Storage::new_in_memory().await.unwrap());
        let handler = DecisionExpirationHandler::new(storage.clone());

        let context = json!({"info": "test"});

        let id = storage
            .insert_decision(
                "txn-103",
                "did:key:z6MkAgent1",
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        // Transition to non-terminal state
        handler
            .handle_event(NodeEvent::TransactionStateChanged {
                transaction_id: "txn-103".to_string(),
                old_state: "received".to_string(),
                new_state: "partially_authorized".to_string(),
                agent_did: Some("did:key:z6MkAgent2".to_string()),
            })
            .await;

        // Decision should still be pending
        let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
        assert_eq!(entry.status, DecisionStatus::Pending);
    }

    #[tokio::test]
    async fn test_ignores_non_state_change_events() {
        let storage = Arc::new(Storage::new_in_memory().await.unwrap());
        let handler = DecisionExpirationHandler::new(storage.clone());

        let context = json!({"info": "test"});

        let id = storage
            .insert_decision(
                "txn-104",
                "did:key:z6MkAgent1",
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        // Send a completely different event
        handler
            .handle_event(NodeEvent::AgentRegistered {
                did: "did:key:z6MkNew".to_string(),
            })
            .await;

        // Decision should still be pending
        let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
        assert_eq!(entry.status, DecisionStatus::Pending);
    }
}
