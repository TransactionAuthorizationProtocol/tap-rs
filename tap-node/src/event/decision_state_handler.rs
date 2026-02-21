//! Decision state handler
//!
//! Listens for `TransactionStateChanged` events and manages decision lifecycle:
//! - Expires pending/delivered decisions when a transaction reaches a terminal state
//!   (Rejected, Cancelled, Reverted)
//! - Resolves pending/delivered decisions when the corresponding action is observed
//!   (e.g., authorization_required decisions resolved when state becomes Authorized)

use super::{EventSubscriber, NodeEvent};
use crate::state_machine::fsm::TransactionState;
use crate::storage::{DecisionType, Storage};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error};

/// Manages decision lifecycle based on transaction state changes
pub struct DecisionStateHandler {
    storage: Arc<Storage>,
}

impl DecisionStateHandler {
    /// Create a new decision state handler
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl EventSubscriber for DecisionStateHandler {
    async fn handle_event(&self, event: NodeEvent) {
        if let NodeEvent::TransactionStateChanged {
            transaction_id,
            new_state,
            ..
        } = event
        {
            let state = match new_state.parse::<TransactionState>() {
                Ok(s) => s,
                Err(_) => return,
            };

            if state.is_terminal() {
                // Terminal states: expire all pending/delivered decisions
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
            } else {
                // Non-terminal state changes: resolve matching decisions
                let resolution = match state {
                    TransactionState::PartiallyAuthorized | TransactionState::ReadyToSettle => {
                        Some(("authorize", Some(DecisionType::AuthorizationRequired)))
                    }
                    TransactionState::Settled => {
                        Some(("settle", Some(DecisionType::SettlementRequired)))
                    }
                    _ => None,
                };

                if let Some((action, decision_type)) = resolution {
                    debug!(
                        "Transaction {} reached state {}, resolving {} decisions",
                        transaction_id, new_state, action
                    );
                    match self
                        .storage
                        .resolve_decisions_for_transaction(&transaction_id, action, decision_type)
                        .await
                    {
                        Ok(count) => {
                            if count > 0 {
                                debug!(
                                    "Resolved {} decisions for transaction {} with action: {}",
                                    count, transaction_id, action
                                );
                            }
                        }
                        Err(e) => {
                            error!(
                                "Failed to resolve decisions for transaction {}: {}",
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
    use crate::storage::DecisionStatus;
    use serde_json::json;

    #[tokio::test]
    async fn test_expires_pending_decisions_on_terminal_state() {
        let storage = Arc::new(Storage::new_in_memory().await.unwrap());
        let handler = DecisionStateHandler::new(storage.clone());

        let context = json!({"info": "test"});

        let id = storage
            .insert_decision(
                "txn-100",
                "did:key:z6MkAgent1",
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

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
        let handler = DecisionStateHandler::new(storage.clone());

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

        storage
            .update_decision_status(id, DecisionStatus::Delivered, None, None)
            .await
            .unwrap();

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
        let handler = DecisionStateHandler::new(storage.clone());

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

        storage
            .update_decision_status(id, DecisionStatus::Resolved, Some("authorize"), None)
            .await
            .unwrap();

        handler
            .handle_event(NodeEvent::TransactionStateChanged {
                transaction_id: "txn-102".to_string(),
                old_state: "settled".to_string(),
                new_state: "reverted".to_string(),
                agent_did: None,
            })
            .await;

        let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
        assert_eq!(entry.status, DecisionStatus::Resolved);
    }

    #[tokio::test]
    async fn test_resolves_authorization_on_ready_to_settle_state() {
        let storage = Arc::new(Storage::new_in_memory().await.unwrap());
        let handler = DecisionStateHandler::new(storage.clone());

        let context = json!({"info": "test"});

        let id = storage
            .insert_decision(
                "txn-200",
                "did:key:z6MkAgent1",
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        handler
            .handle_event(NodeEvent::TransactionStateChanged {
                transaction_id: "txn-200".to_string(),
                old_state: "received".to_string(),
                new_state: "ready_to_settle".to_string(),
                agent_did: Some("did:key:z6MkAgent1".to_string()),
            })
            .await;

        let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
        assert_eq!(entry.status, DecisionStatus::Resolved);
        assert_eq!(entry.resolution.as_deref(), Some("authorize"));
    }

    #[tokio::test]
    async fn test_resolves_settlement_on_settled_state() {
        let storage = Arc::new(Storage::new_in_memory().await.unwrap());
        let handler = DecisionStateHandler::new(storage.clone());

        let context = json!({"info": "test"});

        let id = storage
            .insert_decision(
                "txn-201",
                "did:key:z6MkAgent1",
                DecisionType::SettlementRequired,
                &context,
            )
            .await
            .unwrap();

        handler
            .handle_event(NodeEvent::TransactionStateChanged {
                transaction_id: "txn-201".to_string(),
                old_state: "ready_to_settle".to_string(),
                new_state: "settled".to_string(),
                agent_did: Some("did:key:z6MkAgent1".to_string()),
            })
            .await;

        let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
        assert_eq!(entry.status, DecisionStatus::Resolved);
        assert_eq!(entry.resolution.as_deref(), Some("settle"));
    }

    #[tokio::test]
    async fn test_does_not_resolve_unrelated_decision_types() {
        let storage = Arc::new(Storage::new_in_memory().await.unwrap());
        let handler = DecisionStateHandler::new(storage.clone());

        let context = json!({"info": "test"});

        // Insert a settlement decision, but trigger a ready_to_settle state
        let id = storage
            .insert_decision(
                "txn-202",
                "did:key:z6MkAgent1",
                DecisionType::SettlementRequired,
                &context,
            )
            .await
            .unwrap();

        handler
            .handle_event(NodeEvent::TransactionStateChanged {
                transaction_id: "txn-202".to_string(),
                old_state: "received".to_string(),
                new_state: "ready_to_settle".to_string(),
                agent_did: Some("did:key:z6MkAgent1".to_string()),
            })
            .await;

        // Settlement decision should still be pending (authorize resolves auth decisions, not settlement)
        let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
        assert_eq!(entry.status, DecisionStatus::Pending);
    }

    #[tokio::test]
    async fn test_ignores_non_state_change_events() {
        let storage = Arc::new(Storage::new_in_memory().await.unwrap());
        let handler = DecisionStateHandler::new(storage.clone());

        let context = json!({"info": "test"});

        let id = storage
            .insert_decision(
                "txn-203",
                "did:key:z6MkAgent1",
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();

        handler
            .handle_event(NodeEvent::AgentRegistered {
                did: "did:key:z6MkNew".to_string(),
            })
            .await;

        let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
        assert_eq!(entry.status, DecisionStatus::Pending);
    }
}
