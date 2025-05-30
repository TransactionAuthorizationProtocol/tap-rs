//! Event handlers for updating message and transaction statuses
//!
//! This module provides event handlers that respond to message acceptance/rejection events
//! and update the corresponding database records.

use super::{EventSubscriber, NodeEvent};
use crate::storage::Storage;
use async_trait::async_trait;
use std::sync::Arc;

/// Event handler for updating message status in the database
///
/// This handler listens for MessageAccepted and MessageRejected events
/// and updates the corresponding message status in the database.
pub struct MessageStatusHandler {
    storage: Arc<Storage>,
}

impl MessageStatusHandler {
    /// Create a new message status handler
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// Update message status in the database
    async fn update_message_status(&self, message_id: &str, status: &str) {
        if let Err(e) = self.storage.update_message_status(message_id, status).await {
            log::error!("Failed to update message status for {}: {}", message_id, e);
        }
    }
}

#[async_trait]
impl EventSubscriber for MessageStatusHandler {
    async fn handle_event(&self, event: NodeEvent) {
        match event {
            NodeEvent::MessageAccepted { message_id, .. } => {
                self.update_message_status(&message_id, "accepted").await;
            }
            NodeEvent::MessageRejected { message_id, .. } => {
                self.update_message_status(&message_id, "rejected").await;
            }
            _ => {} // Ignore other events
        }
    }
}

/// Event handler for updating transaction state in the database
///
/// This handler listens for TransactionStateChanged events
/// and updates the corresponding transaction status in the database.
pub struct TransactionStateHandler {
    storage: Arc<Storage>,
}

impl TransactionStateHandler {
    /// Create a new transaction state handler
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// Update transaction status in the database
    async fn update_transaction_status(&self, transaction_id: &str, status: &str) {
        if let Err(e) = self
            .storage
            .update_transaction_status(transaction_id, status)
            .await
        {
            log::error!(
                "Failed to update transaction status for {}: {}",
                transaction_id,
                e
            );
        }
    }
}

#[async_trait]
impl EventSubscriber for TransactionStateHandler {
    async fn handle_event(&self, event: NodeEvent) {
        match event {
            NodeEvent::TransactionStateChanged {
                transaction_id,
                new_state,
                ..
            } => {
                self.update_transaction_status(&transaction_id, &new_state)
                    .await;
            }
            _ => {} // Ignore other events
        }
    }
}

/// Event handler for logging transaction state transitions
///
/// This handler provides detailed logging of transaction state changes
/// for debugging and auditing purposes.
pub struct TransactionAuditHandler;

impl TransactionAuditHandler {
    /// Create a new transaction audit handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for TransactionAuditHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventSubscriber for TransactionAuditHandler {
    async fn handle_event(&self, event: NodeEvent) {
        match event {
            NodeEvent::TransactionStateChanged {
                transaction_id,
                old_state,
                new_state,
                agent_did,
            } => match agent_did {
                Some(did) => {
                    log::info!(
                        "Transaction {} state changed from '{}' to '{}' by agent {}",
                        transaction_id,
                        old_state,
                        new_state,
                        did
                    );
                }
                None => {
                    log::info!(
                        "Transaction {} state changed from '{}' to '{}'",
                        transaction_id,
                        old_state,
                        new_state
                    );
                }
            },
            NodeEvent::MessageAccepted {
                message_id,
                message_type,
                from,
                to,
            } => {
                log::info!(
                    "Message {} of type {} accepted from {} to {}",
                    message_id,
                    message_type,
                    from,
                    to
                );
            }
            NodeEvent::MessageRejected {
                message_id,
                reason,
                from,
                to,
            } => {
                log::warn!(
                    "Message {} rejected from {} to {}: {}",
                    message_id,
                    from,
                    to,
                    reason
                );
            }
            NodeEvent::ReplyReceived {
                original_message_id,
                ..
            } => {
                log::info!("Reply received for message {}", original_message_id);
            }
            _ => {} // Ignore other events
        }
    }
}
