//! Event handling for TAP Node
//!
//! This module provides event handling and subscription functionality for TAP Node events.

use async_trait::async_trait;
use tap_msg::didcomm::Message;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Event types that can be emitted by the TAP Node
#[derive(Debug, Clone)]
pub enum NodeEvent {
    /// A message was received by an agent
    MessageReceived {
        /// The received message
        message: Value,
    },
    /// A message was sent by an agent
    MessageSent {
        /// The sent message
        message: Value,
        /// The sender DID
        from: String,
        /// The recipient DID
        to: String,
    },
    /// An agent was registered with the node
    AgentRegistered {
        /// The registered agent's DID
        did: String,
    },
    /// An agent was unregistered from the node
    AgentUnregistered {
        /// The unregistered agent's DID
        did: String,
    },
    /// A DID was resolved
    DidResolved {
        /// The resolved DID
        did: String,
        /// Whether the resolution was successful
        success: bool,
    },
    /// An agent message event
    AgentMessage {
        /// The agent's DID
        did: String,
        /// The message
        message: Vec<u8>,
    },
}

/// Event subscriber trait for receiving node events
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Handle a node event
    async fn handle_event(&self, event: NodeEvent);
}

/// Event bus for publishing and subscribing to node events
pub struct EventBus {
    /// Sender for events
    sender: broadcast::Sender<NodeEvent>,
    /// Subscribers
    subscribers: RwLock<Vec<Arc<dyn EventSubscriber>>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            subscribers: RwLock::new(Vec::new()),
        }
    }
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        // Create a channel with capacity for 100 events
        let (sender, _) = broadcast::channel(100);

        Self {
            sender,
            subscribers: RwLock::new(Vec::new()),
        }
    }

    /// Subscribe to node events
    pub async fn subscribe(&self, subscriber: Arc<dyn EventSubscriber>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(subscriber);
    }

    /// Get a receiver for node events
    pub fn subscribe_channel(&self) -> broadcast::Receiver<NodeEvent> {
        self.sender.subscribe()
    }

    /// Remove a subscriber from the event bus
    pub async fn unsubscribe(&self, subscriber: &Arc<dyn EventSubscriber>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.retain(|s| !Arc::ptr_eq(s, subscriber));
    }

    /// Publish a message received event
    pub async fn publish_message_received(&self, message: Message) {
        let event = NodeEvent::MessageReceived { message: serde_json::to_value(message).unwrap() };
        self.publish_event(event).await;
    }

    /// Publish a message sent event
    pub async fn publish_message_sent(&self, message: Message, from: String, to: String) {
        let event = NodeEvent::MessageSent { message: serde_json::to_value(message).unwrap(), from, to };
        self.publish_event(event).await;
    }

    /// Publish an agent registered event
    pub async fn publish_agent_registered(&self, did: String) {
        let event = NodeEvent::AgentRegistered { did };
        self.publish_event(event).await;
    }

    /// Publish an agent unregistered event
    pub async fn publish_agent_unregistered(&self, did: String) {
        let event = NodeEvent::AgentUnregistered { did };
        self.publish_event(event).await;
    }

    /// Publish an agent message event
    pub async fn publish_agent_message(&self, did: String, message: Vec<u8>) {
        let event = NodeEvent::AgentMessage { did, message };
        self.publish_event(event).await;
    }

    /// Publish a DID resolved event
    pub async fn publish_did_resolved(&self, did: String, success: bool) {
        let event = NodeEvent::DidResolved { did, success };
        self.publish_event(event).await;
    }

    /// Publish an event to all subscribers
    async fn publish_event(&self, event: NodeEvent) {
        // Send to channel
        let _ = self.sender.send(event.clone());

        // Notify subscribers
        for subscriber in self.subscribers.read().await.iter() {
            subscriber.handle_event(event.clone()).await;
        }
    }
}
