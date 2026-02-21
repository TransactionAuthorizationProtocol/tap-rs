//! Decision log handler
//!
//! Implements the `DecisionHandler` trait by writing decisions to the
//! `decision_log` table in the agent's SQLite database. This enables
//! poll-based decision making where an external process (e.g., a separate
//! tap-mcp instance) can query pending decisions and act on them.

use crate::state_machine::fsm::{Decision, DecisionHandler, TransactionContext};
use crate::storage::{DecisionType, Storage};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error};

/// Writes decisions to the decision_log table for external polling
#[derive(Debug)]
pub struct DecisionLogHandler {
    storage: Arc<Storage>,
    agent_dids: Vec<String>,
}

impl DecisionLogHandler {
    /// Create a new decision log handler
    pub fn new(storage: Arc<Storage>, agent_dids: Vec<String>) -> Self {
        Self {
            storage,
            agent_dids,
        }
    }
}

#[async_trait]
impl DecisionHandler for DecisionLogHandler {
    async fn handle_decision(&self, ctx: &TransactionContext, decision: &Decision) {
        let (decision_type, context_json) = match decision {
            Decision::AuthorizationRequired {
                transaction_id,
                pending_agents,
            } => (
                DecisionType::AuthorizationRequired,
                json!({
                    "transaction_state": ctx.state.to_string(),
                    "pending_agents": pending_agents,
                    "transaction_id": transaction_id,
                }),
            ),
            Decision::PolicySatisfactionRequired {
                transaction_id,
                requested_by,
            } => (
                DecisionType::PolicySatisfactionRequired,
                json!({
                    "transaction_state": ctx.state.to_string(),
                    "requested_by": requested_by,
                    "transaction_id": transaction_id,
                }),
            ),
            Decision::SettlementRequired { transaction_id } => (
                DecisionType::SettlementRequired,
                json!({
                    "transaction_state": ctx.state.to_string(),
                    "transaction_id": transaction_id,
                }),
            ),
        };

        let agent_did = self.agent_dids.first().cloned().unwrap_or_default();

        match self
            .storage
            .insert_decision(
                &ctx.transaction_id,
                &agent_did,
                decision_type,
                &context_json,
            )
            .await
        {
            Ok(decision_id) => {
                debug!(
                    "Logged decision {} for transaction {} (poll mode)",
                    decision_id, ctx.transaction_id
                );
            }
            Err(e) => {
                error!(
                    "Failed to log decision for transaction {}: {}",
                    ctx.transaction_id, e
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::fsm::TransactionState;
    use crate::storage::DecisionStatus;

    #[tokio::test]
    async fn test_decision_log_handler_writes_to_db() {
        let storage = Arc::new(Storage::new_in_memory().await.unwrap());
        let handler =
            DecisionLogHandler::new(storage.clone(), vec!["did:key:z6MkAgent1".to_string()]);

        let ctx = TransactionContext {
            transaction_id: "txn-dlh-1".to_string(),
            state: TransactionState::Received,
            agents: Default::default(),
            has_pending_policies: false,
        };

        let decision = Decision::AuthorizationRequired {
            transaction_id: "txn-dlh-1".to_string(),
            pending_agents: vec!["did:key:z6MkAgent1".to_string()],
        };

        handler.handle_decision(&ctx, &decision).await;

        let entries = storage
            .list_decisions(
                Some("did:key:z6MkAgent1"),
                Some(DecisionStatus::Pending),
                None,
                100,
            )
            .await
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].transaction_id, "txn-dlh-1");
        assert_eq!(
            entries[0].decision_type,
            DecisionType::AuthorizationRequired
        );
    }

    #[tokio::test]
    async fn test_decision_log_handler_settlement() {
        let storage = Arc::new(Storage::new_in_memory().await.unwrap());
        let handler =
            DecisionLogHandler::new(storage.clone(), vec!["did:key:z6MkAgent1".to_string()]);

        let ctx = TransactionContext {
            transaction_id: "txn-dlh-2".to_string(),
            state: TransactionState::ReadyToSettle,
            agents: Default::default(),
            has_pending_policies: false,
        };

        let decision = Decision::SettlementRequired {
            transaction_id: "txn-dlh-2".to_string(),
        };

        handler.handle_decision(&ctx, &decision).await;

        let entries = storage
            .list_decisions(
                Some("did:key:z6MkAgent1"),
                Some(DecisionStatus::Pending),
                None,
                100,
            )
            .await
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].decision_type, DecisionType::SettlementRequired);
    }
}
