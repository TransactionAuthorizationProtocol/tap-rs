//! Transaction state machine for TAP Node
//!
//! This module implements a state machine for managing transaction lifecycle,
//! including automatic state transitions and Settle message generation.
//!
//! ## Sub-modules
//!
//! - [`fsm`]: Formal finite state machine with explicit states, transitions,
//!   and decision points for the full transaction lifecycle.

pub mod fsm;

use crate::agent::AgentRegistry;
use crate::error::{Error, Result};
use crate::event::EventBus;
use crate::storage::Storage;
use async_trait::async_trait;
use dashmap::DashMap;
use fsm::{
    AutoApproveHandler, Decision, DecisionHandler, DecisionMode, FsmEvent, LogOnlyHandler,
    TransactionContext, TransactionFsm,
};
use std::sync::Arc;
use tap_agent::Agent;
use tap_msg::didcomm::PlainMessage;
use tap_msg::message::TapMessage;

/// Trait for processing transaction state changes
#[async_trait]
pub trait TransactionStateProcessor: Send + Sync {
    /// Process an incoming message and update transaction state
    async fn process_message(&self, message: &PlainMessage) -> Result<()>;
}

/// Standard transaction state processor
///
/// Routes incoming TAP messages through the FSM and delegates decisions
/// to the configured [`DecisionHandler`].
pub struct StandardTransactionProcessor {
    storage: Arc<Storage>,
    event_bus: Arc<EventBus>,
    agents: Arc<AgentRegistry>,
    /// In-memory FSM contexts keyed by transaction ID.
    contexts: DashMap<String, TransactionContext>,
    /// Handler for FSM decision points.
    decision_handler: Arc<dyn DecisionHandler>,
    /// Whether to auto-act on decisions (send Authorize/Settle messages).
    auto_act: bool,
}

impl StandardTransactionProcessor {
    /// Create a new standard transaction processor with the given decision mode.
    pub fn new(
        storage: Arc<Storage>,
        event_bus: Arc<EventBus>,
        agents: Arc<AgentRegistry>,
        decision_mode: DecisionMode,
    ) -> Self {
        let (decision_handler, auto_act): (Arc<dyn DecisionHandler>, bool) = match decision_mode {
            DecisionMode::AutoApprove => (Arc::new(AutoApproveHandler), true),
            DecisionMode::EventBus => (Arc::new(LogOnlyHandler), false),
            DecisionMode::Custom(handler) => (handler, false),
        };

        Self {
            storage,
            event_bus,
            agents,
            contexts: DashMap::new(),
            decision_handler,
            auto_act,
        }
    }

    /// Extract agents from a Transfer or Payment message.
    /// Returns (agent_did, role) pairs for agents only (not primary parties).
    fn extract_agents_from_tap_message(tap_message: &TapMessage) -> Vec<(String, String)> {
        let agents_list = match tap_message {
            TapMessage::Transfer(t) => &t.agents,
            TapMessage::Payment(p) => &p.agents,
            _ => return Vec::new(),
        };
        agents_list
            .iter()
            .map(|a| {
                let role = match a.role.as_deref() {
                    Some("compliance") => "compliance",
                    _ => "other",
                };
                (a.id.clone(), role.to_string())
            })
            .collect()
    }

    /// Get or create the FSM context for a transaction.
    fn get_or_create_context(
        &self,
        transaction_id: &str,
        agent_dids: Vec<String>,
    ) -> TransactionContext {
        self.contexts
            .entry(transaction_id.to_string())
            .or_insert_with(|| TransactionContext::new(transaction_id.to_string(), agent_dids))
            .clone()
    }

    /// Persist the FSM context back to the in-memory map.
    fn save_context(&self, ctx: &TransactionContext) {
        self.contexts
            .insert(ctx.transaction_id.clone(), ctx.clone());
    }

    /// Convert a TapMessage + PlainMessage into an FsmEvent.
    fn to_fsm_event(tap_message: &TapMessage, plain: &PlainMessage) -> Option<FsmEvent> {
        match tap_message {
            TapMessage::Transfer(_) | TapMessage::Payment(_) => {
                let agent_dids: Vec<String> = Self::extract_agents_from_tap_message(tap_message)
                    .into_iter()
                    .map(|(did, _)| did)
                    .collect();
                Some(FsmEvent::TransactionReceived { agent_dids })
            }
            TapMessage::Authorize(auth) => Some(FsmEvent::AuthorizeReceived {
                agent_did: plain.from.clone(),
                settlement_address: auth.settlement_address.clone(),
                expiry: auth.expiry.clone(),
            }),
            TapMessage::Reject(reject) => Some(FsmEvent::RejectReceived {
                agent_did: plain.from.clone(),
                reason: reject.reason.clone(),
            }),
            TapMessage::Cancel(cancel) => Some(FsmEvent::CancelReceived {
                by_did: plain.from.clone(),
                reason: cancel.reason.clone(),
            }),
            TapMessage::Settle(settle) => Some(FsmEvent::SettleReceived {
                settlement_id: settle.settlement_id.clone(),
                amount: settle.amount.clone(),
            }),
            TapMessage::Revert(revert) => Some(FsmEvent::RevertReceived {
                by_did: plain.from.clone(),
                reason: revert.reason.clone(),
            }),
            TapMessage::AddAgents(add) => Some(FsmEvent::AgentsAdded {
                agent_dids: add.agents.iter().map(|a| a.id.clone()).collect(),
            }),
            TapMessage::UpdatePolicies(_) => Some(FsmEvent::PoliciesReceived {
                from_did: plain.from.clone(),
            }),
            TapMessage::Presentation(_) => Some(FsmEvent::PresentationReceived {
                from_did: plain.from.clone(),
            }),
            _ => None,
        }
    }

    /// Get the transaction_id that a TAP message references.
    fn transaction_id_for(tap_message: &TapMessage, plain: &PlainMessage) -> String {
        match tap_message {
            TapMessage::Transfer(_) | TapMessage::Payment(_) => plain.id.clone(),
            TapMessage::Authorize(a) => a.transaction_id.clone(),
            TapMessage::Reject(r) => r.transaction_id.clone(),
            TapMessage::Cancel(c) => c.transaction_id.clone(),
            TapMessage::Settle(s) => s.transaction_id.clone(),
            TapMessage::Revert(r) => r.transaction_id.clone(),
            TapMessage::AddAgents(a) => a.transaction_id.clone(),
            TapMessage::UpdatePolicies(u) => u.transaction_id.clone(),
            // Presentation uses pthid/thid for threading
            _ => plain.thid.clone().unwrap_or_default(),
        }
    }

    /// Publish a decision to the event bus.
    async fn publish_decision(&self, ctx: &TransactionContext, decision: &Decision) {
        let decision_json = serde_json::to_value(decision).unwrap_or_default();
        self.event_bus
            .publish_decision_required(
                ctx.transaction_id.clone(),
                ctx.state.to_string(),
                decision_json,
                ctx.pending_agents(),
            )
            .await;
    }

    // ---- Auto-act methods (only called in AutoApprove mode) ----

    /// Automatically send Authorize for our registered agents.
    async fn auto_authorize_transaction(&self, message: &PlainMessage) -> Result<()> {
        let tap_message = TapMessage::from_plain_message(message)
            .map_err(|e| Error::InvalidPlainMessage(e.to_string()))?;

        let transaction_id = match &tap_message {
            TapMessage::Transfer(transfer) => &transfer.transaction_id,
            TapMessage::Payment(payment) => &payment.transaction_id,
            _ => return Ok(()),
        };

        let our_agents = self.agents.get_all_dids();
        let transaction_agents = Self::extract_agents_from_tap_message(&tap_message);

        for (agent_did, _role) in transaction_agents {
            if our_agents.contains(&agent_did) {
                if let Ok(agent) = self.agents.get_agent(&agent_did).await {
                    use tap_msg::message::tap_message_trait::Authorizable;
                    let authorize_message = match &tap_message {
                        TapMessage::Transfer(transfer) => {
                            transfer.authorize(&agent_did, None, None)
                        }
                        TapMessage::Payment(payment) => payment.authorize(&agent_did, None, None),
                        _ => continue,
                    };

                    let auth_body = authorize_message.body;
                    let recipients_list = vec![message.from.as_str()];

                    log::info!(
                        "Auto-authorizing transaction {:?} from agent {}",
                        transaction_id,
                        agent_did
                    );

                    if let Err(e) = agent.send_message(&auth_body, recipients_list, true).await {
                        log::warn!(
                            "Failed to send auto-Authorize for transaction {:?} from agent {}: {}",
                            transaction_id,
                            agent_did,
                            e
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if all agents authorized and send Settle if so.
    async fn check_and_send_settle(&self, transaction_id: &str) -> Result<()> {
        let transaction = self
            .storage
            .get_transaction_by_id(transaction_id)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?
            .ok_or_else(|| Error::Storage(format!("Transaction {} not found", transaction_id)))?;

        let our_agents = self.agents.get_all_dids();
        let is_sender = transaction
            .from_did
            .as_ref()
            .map(|did| our_agents.contains(did))
            .unwrap_or(false);

        if !is_sender {
            return Ok(());
        }

        let all_authorized = self
            .storage
            .are_all_agents_authorized(transaction_id)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        if !all_authorized {
            return Ok(());
        }

        if transaction.status.to_string() == "confirmed" {
            return Ok(());
        }

        log::info!(
            "All agents authorized for transaction {}, sending Settle message",
            transaction_id
        );

        let sender_did = transaction
            .from_did
            .as_ref()
            .ok_or_else(|| Error::Processing("Transaction missing from_did".to_string()))?;

        let agent = self
            .agents
            .get_agent(sender_did)
            .await
            .map_err(|e| Error::Agent(e.to_string()))?;

        let settlement_id = format!("settle_{}", transaction_id);

        let transaction_message: PlainMessage =
            serde_json::from_value(transaction.message_json.clone()).map_err(|e| {
                Error::Serialization(format!("Failed to parse transaction message: {}", e))
            })?;

        let tap_message = TapMessage::from_plain_message(&transaction_message)
            .map_err(|e| Error::InvalidPlainMessage(e.to_string()))?;

        use tap_msg::message::tap_message_trait::Transaction;

        let agents = Self::extract_agents_from_tap_message(&tap_message);

        if agents.is_empty() {
            log::debug!(
                "No agents to send Settle message to for transaction {}",
                transaction_id
            );
            return Ok(());
        }

        for (agent_did, _role) in agents {
            if agent_did != *sender_did {
                let settle_message = match &tap_message {
                    TapMessage::Transfer(transfer) => {
                        transfer.settle(sender_did, &settlement_id, None)
                    }
                    TapMessage::Payment(payment) => {
                        payment.settle(sender_did, &settlement_id, None)
                    }
                    _ => {
                        log::warn!(
                            "Unexpected message type for settlement: {}",
                            transaction.message_type
                        );
                        continue;
                    }
                };

                let settle_body = settle_message.body;
                let recipients_list = vec![agent_did.as_str()];
                let _ = agent
                    .send_message(&settle_body, recipients_list, true)
                    .await
                    .map_err(|e| {
                        log::warn!("Failed to send Settle to {}: {}", agent_did, e);
                        e
                    });
            }
        }

        self.storage
            .update_transaction_status(transaction_id, "confirmed")
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        self.event_bus
            .publish_transaction_state_changed(
                transaction_id.to_string(),
                "pending".to_string(),
                "confirmed".to_string(),
                Some(sender_did.clone()),
            )
            .await;

        Ok(())
    }
}

#[async_trait]
impl TransactionStateProcessor for StandardTransactionProcessor {
    async fn process_message(&self, message: &PlainMessage) -> Result<()> {
        let tap_message = TapMessage::from_plain_message(message)
            .map_err(|e| Error::InvalidPlainMessage(e.to_string()))?;

        let transaction_id = Self::transaction_id_for(&tap_message, message);

        // Convert message to FSM event
        let fsm_event = Self::to_fsm_event(&tap_message, message);

        // --- Storage operations (always run regardless of decision mode) ---
        match &tap_message {
            TapMessage::Transfer(_) | TapMessage::Payment(_) => {
                if let Err(e) = self.storage.insert_transaction(message).await {
                    log::warn!("Failed to insert transaction {}: {}", transaction_id, e);
                }
                let agents = Self::extract_agents_from_tap_message(&tap_message);
                for (agent_did, role) in &agents {
                    if let Err(e) = self
                        .storage
                        .insert_transaction_agent(&transaction_id, agent_did, role)
                        .await
                    {
                        log::warn!(
                            "Failed to insert agent {} for transaction {}: {}",
                            agent_did,
                            transaction_id,
                            e
                        );
                    }
                }
            }
            TapMessage::Authorize(_) => {
                if let Err(e) = self
                    .storage
                    .update_transaction_agent_status(&transaction_id, &message.from, "authorized")
                    .await
                {
                    log::warn!(
                        "Failed to update agent {} status for transaction {}: {}",
                        message.from,
                        transaction_id,
                        e
                    );
                }
            }
            TapMessage::Reject(_) => {
                let _ = self
                    .storage
                    .update_transaction_agent_status(&transaction_id, &message.from, "rejected")
                    .await;
                let _ = self
                    .storage
                    .update_transaction_status(&transaction_id, "failed")
                    .await;
            }
            TapMessage::Cancel(_) => {
                let _ = self
                    .storage
                    .update_transaction_agent_status(&transaction_id, &message.from, "cancelled")
                    .await;
                let _ = self
                    .storage
                    .update_transaction_status(&transaction_id, "cancelled")
                    .await;
            }
            TapMessage::Settle(_) => {
                let _ = self
                    .storage
                    .update_transaction_status(&transaction_id, "confirmed")
                    .await;
            }
            TapMessage::Revert(_) => {
                let _ = self
                    .storage
                    .update_transaction_status(&transaction_id, "reverted")
                    .await;
            }
            TapMessage::AddAgents(add) => {
                for agent in &add.agents {
                    let role_str = match agent.role.as_deref() {
                        Some("compliance") => "compliance",
                        _ => "other",
                    };
                    let _ = self
                        .storage
                        .insert_transaction_agent(&transaction_id, &agent.id, role_str)
                        .await;
                }
            }
            _ => {}
        }

        // --- FSM transition ---
        if let Some(event) = fsm_event {
            let agent_dids: Vec<String> = Self::extract_agents_from_tap_message(&tap_message)
                .into_iter()
                .map(|(did, _)| did)
                .collect();

            let mut ctx = self.get_or_create_context(&transaction_id, agent_dids);
            let old_state = ctx.state.to_string();

            match TransactionFsm::apply(&mut ctx, event) {
                Ok(transition) => {
                    let new_state = transition.to_state.to_string();

                    // Persist FSM context
                    self.save_context(&ctx);

                    // Publish state change if it actually changed
                    if old_state != new_state {
                        self.event_bus
                            .publish_transaction_state_changed(
                                transaction_id.clone(),
                                old_state,
                                new_state,
                                Some(message.from.clone()),
                            )
                            .await;
                    }

                    // Handle decision if one was produced
                    if let Some(ref decision) = transition.decision {
                        // Always notify the decision handler
                        self.decision_handler.handle_decision(&ctx, decision).await;

                        // Always publish to event bus for observability
                        self.publish_decision(&ctx, decision).await;

                        // Auto-act if configured
                        if self.auto_act {
                            match decision {
                                Decision::AuthorizationRequired { .. } => {
                                    if let Err(e) = self.auto_authorize_transaction(message).await {
                                        log::warn!(
                                            "Failed to auto-authorize transaction {}: {}",
                                            transaction_id,
                                            e
                                        );
                                    }
                                }
                                Decision::SettlementRequired { transaction_id } => {
                                    if let Err(e) = self.check_and_send_settle(transaction_id).await
                                    {
                                        log::warn!(
                                            "Failed to check/send settle for transaction {}: {}",
                                            transaction_id,
                                            e
                                        );
                                    }
                                }
                                Decision::PolicySatisfactionRequired { .. } => {
                                    log::debug!(
                                        "Policy satisfaction required for transaction {} — no auto-action available",
                                        transaction_id
                                    );
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!(
                        "FSM transition error for transaction {}: {}",
                        transaction_id,
                        e
                    );
                }
            }
        }

        Ok(())
    }
}
