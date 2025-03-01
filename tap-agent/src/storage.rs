//! Message storage for TAP Agent
//!
//! This module provides message storage functionality for the TAP Agent.

use crate::error::{Error, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use tap_core::message::{TapMessage, TapMessageType};

/// Query parameters for filtering messages
#[derive(Debug, Default, Clone)]
pub struct MessageQuery {
    /// Filter by message ID
    pub id: Option<String>,
    /// Filter by message type
    pub message_type: Option<TapMessageType>,
    /// Filter by sender DID
    pub from: Option<String>,
    /// Filter by recipient DID
    pub to: Option<String>,
    /// Filter by thread ID
    pub thread_id: Option<String>,
    /// Limit the number of results
    pub limit: Option<usize>,
}

/// Trait for message storage
#[async_trait]
pub trait MessageStore: Send + Sync {
    /// Stores a message
    async fn store_message(&self, message: &TapMessage) -> Result<()>;

    /// Retrieves a message by ID
    async fn get_message(&self, id: &str) -> Result<Option<TapMessage>>;

    /// Queries for messages matching the criteria
    async fn query_messages(&self, query: MessageQuery) -> Result<Vec<TapMessage>>;
}

/// In-memory implementation of MessageStore
#[derive(Debug, Default)]
pub struct InMemoryMessageStore {
    /// Internal storage using a map of message ID to message
    messages: Mutex<HashMap<String, TapMessage>>,
}

impl InMemoryMessageStore {
    /// Creates a new empty in-memory message store
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl MessageStore for InMemoryMessageStore {
    async fn store_message(&self, message: &TapMessage) -> Result<()> {
        let mut messages = self
            .messages
            .lock()
            .map_err(|e| Error::Storage(format!("Failed to acquire lock: {}", e)))?;

        messages.insert(message.id.clone(), message.clone());
        Ok(())
    }

    async fn get_message(&self, id: &str) -> Result<Option<TapMessage>> {
        let messages = self
            .messages
            .lock()
            .map_err(|e| Error::Storage(format!("Failed to acquire lock: {}", e)))?;

        Ok(messages.get(id).cloned())
    }

    async fn query_messages(&self, query: MessageQuery) -> Result<Vec<TapMessage>> {
        let messages = self
            .messages
            .lock()
            .map_err(|e| Error::Storage(format!("Failed to acquire lock: {}", e)))?;

        let mut results: Vec<TapMessage> = messages
            .values()
            .filter(|msg| {
                // Match ID if specified
                if let Some(id) = &query.id {
                    if &msg.id != id {
                        return false;
                    }
                }

                // Match message type if specified
                if let Some(msg_type) = &query.message_type {
                    if &msg.message_type != msg_type {
                        return false;
                    }
                }

                // Match sender if specified
                if let Some(from) = &query.from {
                    // Check in metadata since TapMessage doesn't have a direct 'from' field
                    match msg.metadata.get("from") {
                        Some(val) => {
                            if let Some(sender) = val.as_str() {
                                if sender != from {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }

                // Match recipient if specified
                if let Some(to) = &query.to {
                    // Check in metadata since TapMessage doesn't have a direct 'to' field
                    match msg.metadata.get("to") {
                        Some(val) => {
                            if let Some(recipient) = val.as_str() {
                                if recipient != to {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }

                // Match thread ID if specified
                if let Some(thread_id) = &query.thread_id {
                    // Check in metadata since TapMessage doesn't have a direct 'thread_id' field
                    match msg.metadata.get("thread_id") {
                        Some(val) => {
                            if let Some(thread) = val.as_str() {
                                if thread != thread_id {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Apply limit if specified
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }
}
