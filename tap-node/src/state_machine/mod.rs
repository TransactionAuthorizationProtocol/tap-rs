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
pub struct StandardTransactionProcessor {
    storage: Arc<Storage>,
    event_bus: Arc<EventBus>,
    agents: Arc<AgentRegistry>,
}

impl StandardTransactionProcessor {
    /// Create a new standard transaction processor
    pub fn new(
        storage: Arc<Storage>,
        event_bus: Arc<EventBus>,
        agents: Arc<AgentRegistry>,
    ) -> Self {
        Self {
            storage,
            event_bus,
            agents,
        }
    }

    /// Extract agents from a Transfer or Payment message
    /// Note: This only extracts actual agents (compliance, etc.), not the primary parties
    /// (originator/beneficiary for Transfer, customer/merchant for Payment)
    async fn extract_agents_from_message(
        &self,
        message: &PlainMessage,
    ) -> Result<Vec<(String, String)>> {
        let tap_message = TapMessage::from_plain_message(message)
            .map_err(|e| Error::InvalidPlainMessage(e.to_string()))?;

        let mut agents = Vec::new();

        match tap_message {
            TapMessage::Transfer(transfer) => {
                // Only add agents from the agents field, not the primary parties
                for agent in &transfer.agents {
                    let role_str = match agent.role.as_deref() {
                        Some("compliance") => "compliance",
                        _ => "other",
                    };
                    agents.push((agent.id.clone(), role_str.to_string()));
                }
            }
            TapMessage::Payment(payment) => {
                // Only add agents from the agents field, not the primary parties
                for agent in &payment.agents {
                    let role_str = match agent.role.as_deref() {
                        Some("compliance") => "compliance",
                        _ => "other",
                    };
                    agents.push((agent.id.clone(), role_str.to_string()));
                }
            }
            _ => {
                // Not a Transfer or Payment
                return Ok(agents);
            }
        }

        Ok(agents)
    }

    /// Automatically send Authorize message for incoming Transfer or Payment messages
    async fn auto_authorize_transaction(&self, message: &PlainMessage) -> Result<()> {
        let tap_message = TapMessage::from_plain_message(message)
            .map_err(|e| Error::InvalidPlainMessage(e.to_string()))?;

        // Only auto-authorize Transfer and Payment messages
        let transaction_id = match &tap_message {
            TapMessage::Transfer(transfer) => &transfer.transaction_id,
            TapMessage::Payment(payment) => &payment.transaction_id,
            _ => return Ok(()), // Not a transaction message
        };

        // Find agents in our registry that are involved in this transaction
        let our_agents = self.agents.get_all_dids();
        let transaction_agents = self.extract_agents_from_message(message).await?;

        // Check if any of our agents are involved in this transaction
        for (agent_did, _role) in transaction_agents {
            if our_agents.contains(&agent_did) {
                // Get the agent from registry
                if let Ok(agent) = self.agents.get_agent(&agent_did).await {
                    // Create an Authorize message using the Authorizable trait
                    use tap_msg::message::tap_message_trait::Authorizable;
                    let authorize_message = match &tap_message {
                        TapMessage::Transfer(transfer) => {
                            transfer.authorize(&agent_did, None, None)
                        }
                        TapMessage::Payment(payment) => payment.authorize(&agent_did, None, None),
                        _ => continue, // Should not happen due to earlier check
                    };

                    // Convert to body for sending
                    let auth_body = authorize_message.body;

                    // Send to the original sender
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

    /// Check if we should send a Settle message
    async fn check_and_send_settle(&self, transaction_id: &str) -> Result<()> {
        // Get the transaction
        let transaction = self
            .storage
            .get_transaction_by_id(transaction_id)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?
            .ok_or_else(|| Error::Storage(format!("Transaction {} not found", transaction_id)))?;

        // Check if we're the sender (originator)
        let our_agents = self.agents.get_all_dids();
        let is_sender = transaction
            .from_did
            .as_ref()
            .map(|did| our_agents.contains(did))
            .unwrap_or(false);

        if !is_sender {
            return Ok(()); // Only sender sends Settle
        }

        // Check if all agents have authorized
        let all_authorized = self
            .storage
            .are_all_agents_authorized(transaction_id)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        if !all_authorized {
            return Ok(()); // Not all agents have authorized yet
        }

        // Check if transaction is already in 'confirmed' status
        if transaction.status.to_string() == "confirmed" {
            return Ok(()); // Already settled
        }

        // Create and send Settle message
        log::info!(
            "All agents authorized for transaction {}, sending Settle message",
            transaction_id
        );

        // Get the sender agent
        let sender_did = transaction
            .from_did
            .as_ref()
            .ok_or_else(|| Error::Processing("Transaction missing from_did".to_string()))?;

        let agent = self
            .agents
            .get_agent(sender_did)
            .await
            .map_err(|e| Error::Agent(e.to_string()))?;

        // Create Settle message using the Transaction trait's settle() method
        // This is the proper way to create settlement messages using the TAP framework
        let settlement_id = format!("settle_{}", transaction_id);

        // Parse the original transaction message to use the Transaction trait
        let transaction_message: PlainMessage =
            serde_json::from_value(transaction.message_json.clone()).map_err(|e| {
                Error::Serialization(format!("Failed to parse transaction message: {}", e))
            })?;

        let tap_message = TapMessage::from_plain_message(&transaction_message)
            .map_err(|e| Error::InvalidPlainMessage(e.to_string()))?;

        // Use the Transaction trait to create the Settle message
        use tap_msg::message::tap_message_trait::Transaction;

        // Send Settle message only to agents (not primary parties)
        // Get all agents for this transaction
        let agents = self
            .extract_agents_from_message(&transaction_message)
            .await?;

        if agents.is_empty() {
            log::debug!(
                "No agents to send Settle message to for transaction {}",
                transaction_id
            );
            return Ok(());
        }

        // Send settle message to all agents
        for (agent_did, _role) in agents {
            if agent_did != *sender_did {
                // Use the Transaction trait to create a proper Settle message for this agent
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

                // Convert to body for sending
                let settle_body = settle_message.body;

                // Send message using the Agent trait's send_message method for proper signing
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

        // Update transaction status to 'confirmed'
        self.storage
            .update_transaction_status(transaction_id, "confirmed")
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        // Emit state change event
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

        match tap_message {
            TapMessage::Transfer(_) | TapMessage::Payment(_) => {
                // First, store the transaction itself
                let transaction_id = &message.id;
                if let Err(e) = self.storage.insert_transaction(message).await {
                    log::warn!("Failed to insert transaction {}: {}", transaction_id, e);
                }

                // Extract and store agents for new transaction
                let agents = self.extract_agents_from_message(message).await?;

                for (agent_did, role) in agents {
                    if let Err(e) = self
                        .storage
                        .insert_transaction_agent(transaction_id, &agent_did, &role)
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

                // Automatically send Authorize messages for our agents involved in this transaction
                if let Err(e) = self.auto_authorize_transaction(message).await {
                    log::warn!(
                        "Failed to auto-authorize transaction {}: {}",
                        transaction_id,
                        e
                    );
                }
            }
            TapMessage::Authorize(auth) => {
                let transaction_id = &auth.transaction_id;
                let agent_did = &message.from;

                // Update agent status to 'authorized'
                if let Err(e) = self
                    .storage
                    .update_transaction_agent_status(transaction_id, agent_did, "authorized")
                    .await
                {
                    log::warn!(
                        "Failed to update agent {} status for transaction {}: {}",
                        agent_did,
                        transaction_id,
                        e
                    );
                } else {
                    // Emit state change event
                    self.event_bus
                        .publish_transaction_state_changed(
                            transaction_id.clone(),
                            "pending".to_string(),
                            "pending".to_string(), // Individual agent authorized, but transaction still pending
                            Some(agent_did.clone()),
                        )
                        .await;

                    // Check if we should send Settle
                    if let Err(e) = self.check_and_send_settle(transaction_id).await {
                        log::warn!(
                            "Failed to check/send settle for transaction {}: {}",
                            transaction_id,
                            e
                        );
                    }
                }
            }
            TapMessage::Cancel(cancel) => {
                let transaction_id = &cancel.transaction_id;
                let agent_did = &message.from;

                // Update agent status to 'cancelled'
                if let Err(e) = self
                    .storage
                    .update_transaction_agent_status(transaction_id, agent_did, "cancelled")
                    .await
                {
                    log::warn!(
                        "Failed to update agent {} status for transaction {}: {}",
                        agent_did,
                        transaction_id,
                        e
                    );
                }

                // Update transaction status to 'cancelled'
                if let Err(e) = self
                    .storage
                    .update_transaction_status(transaction_id, "cancelled")
                    .await
                {
                    log::warn!(
                        "Failed to update transaction {} status: {}",
                        transaction_id,
                        e
                    );
                } else {
                    // Emit state change event
                    self.event_bus
                        .publish_transaction_state_changed(
                            transaction_id.clone(),
                            "pending".to_string(),
                            "cancelled".to_string(),
                            Some(agent_did.clone()),
                        )
                        .await;
                }
            }
            TapMessage::Reject(reject) => {
                let transaction_id = &reject.transaction_id;
                let agent_did = &message.from;

                // Update agent status to 'rejected'
                if let Err(e) = self
                    .storage
                    .update_transaction_agent_status(transaction_id, agent_did, "rejected")
                    .await
                {
                    log::warn!(
                        "Failed to update agent {} status for transaction {}: {}",
                        agent_did,
                        transaction_id,
                        e
                    );
                }

                // Update transaction status to 'failed'
                if let Err(e) = self
                    .storage
                    .update_transaction_status(transaction_id, "failed")
                    .await
                {
                    log::warn!(
                        "Failed to update transaction {} status: {}",
                        transaction_id,
                        e
                    );
                } else {
                    // Emit state change event
                    self.event_bus
                        .publish_transaction_state_changed(
                            transaction_id.clone(),
                            "pending".to_string(),
                            "failed".to_string(),
                            Some(agent_did.clone()),
                        )
                        .await;
                }
            }
            TapMessage::Settle(settle) => {
                let transaction_id = &settle.transaction_id;

                // Update transaction status to 'confirmed'
                if let Err(e) = self
                    .storage
                    .update_transaction_status(transaction_id, "confirmed")
                    .await
                {
                    log::warn!(
                        "Failed to update transaction {} status: {}",
                        transaction_id,
                        e
                    );
                } else {
                    // Emit state change event
                    self.event_bus
                        .publish_transaction_state_changed(
                            transaction_id.clone(),
                            "pending".to_string(),
                            "confirmed".to_string(),
                            Some(message.from.clone()),
                        )
                        .await;
                }
            }
            TapMessage::Revert(revert) => {
                let transaction_id = &revert.transaction_id;

                // Update transaction status to 'reverted'
                if let Err(e) = self
                    .storage
                    .update_transaction_status(transaction_id, "reverted")
                    .await
                {
                    log::warn!(
                        "Failed to update transaction {} status: {}",
                        transaction_id,
                        e
                    );
                } else {
                    // Emit state change event
                    self.event_bus
                        .publish_transaction_state_changed(
                            transaction_id.clone(),
                            "confirmed".to_string(),
                            "reverted".to_string(),
                            Some(message.from.clone()),
                        )
                        .await;
                }
            }
            TapMessage::AddAgents(add) => {
                // Update agents based on TAIP-5
                let transaction_id = &add.transaction_id;

                for agent in &add.agents {
                    let role_str = match agent.role.as_deref() {
                        Some("compliance") => "compliance",
                        _ => "other",
                    };

                    if let Err(e) = self
                        .storage
                        .insert_transaction_agent(transaction_id, &agent.id, role_str)
                        .await
                    {
                        log::warn!(
                            "Failed to add agent {} to transaction {}: {}",
                            agent.id,
                            transaction_id,
                            e
                        );
                    }
                }
            }
            TapMessage::UpdatePolicies(_) => {
                // Update policies based on TAIP-7
                // This would update transaction metadata, but we don't have a specific
                // field for policies in our current schema, so we'll skip for now
                log::debug!("UpdatePolicies message received, but policy storage not implemented");
            }
            _ => {
                // Other message types don't affect transaction state
            }
        }

        Ok(())
    }
}
