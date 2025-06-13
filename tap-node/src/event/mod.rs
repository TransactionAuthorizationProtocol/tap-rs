//! # Event System for TAP Node
//!
//! This module provides a comprehensive event handling and subscription system for TAP Node.
//! The event system allows components to publish and subscribe to various events that occur
//! within the node, enabling loose coupling between components and reactive programming patterns.
//!
//! ## Event Types
//!
//! The `NodeEvent` enum defines all the possible events that can be emitted by the TAP Node:
//!
//! - **PlainMessageReceived**: When a message is received by an agent
//! - **PlainMessageSent**: When a message is sent from an agent to another
//! - **AgentRegistered**: When a new agent is registered with the node
//! - **AgentUnregistered**: When an agent is removed from the node
//! - **DidResolved**: When a DID is resolved (successfully or not)
//! - **AgentPlainMessage**: Raw message data intended for an agent
//!
//! ## Subscription Models
//!
//! The event system supports two subscription models:
//!
//! 1. **Callback-based**: Implementing the `EventSubscriber` trait to receive events via callbacks
//! 2. **Channel-based**: Using `tokio::sync::broadcast` channels to receive events asynchronously
//!
//! ## Built-in Event Handlers
//!
//! The event system includes several built-in event handlers:
//!
//! - **EventLogger**: Logs all events to a configurable destination (console, file, or custom handler)
//!
//! ## Usage Examples
//!
//! ### Callback-based Subscription
//!
//! ```
//! use std::sync::Arc;
//! use async_trait::async_trait;
//! use tap_node::event::{EventBus, EventSubscriber, NodeEvent};
//!
//! // Create a custom event handler
//! struct LoggingEventHandler;
//!
//! #[async_trait]
//! impl EventSubscriber for LoggingEventHandler {
//!     async fn handle_event(&self, event: NodeEvent) {
//!         match event {
//!             NodeEvent::PlainMessageReceived { message } => {
//!                 println!("PlainMessage received: {:?}", message);
//!             },
//!             NodeEvent::AgentRegistered { did } => {
//!                 println!("Agent registered: {}", did);
//!             },
//!             // Handle other event types...
//!             _ => {}
//!         }
//!     }
//! }
//!
//! // Later, subscribe to events
//! async fn subscribe_events(event_bus: &EventBus) {
//!     let handler = Arc::new(LoggingEventHandler);
//!     event_bus.subscribe(handler).await;
//! }
//! ```
//!
//! ### Channel-based Subscription
//!
//! ```
//! use tap_node::event::{EventBus, NodeEvent};
//! use tokio::spawn;
//!
//! async fn monitor_events(event_bus: &EventBus) {
//!     // Get a receiver for the events
//!     let mut receiver = event_bus.subscribe_channel();
//!
//!     // Process events in a separate task
//!     spawn(async move {
//!         while let Ok(event) = receiver.recv().await {
//!             match event {
//!                 NodeEvent::PlainMessageSent { message, from, to } => {
//!                     println!("PlainMessage sent from {} to {}", from, to);
//!                 },
//!                 // Handle other events...
//!                 _ => {}
//!             }
//!         }
//!     });
//! }
//! ```
//!
//! ### Using the Event Logger
//!
//! ```no_run
//! use std::sync::Arc;
//! use tap_node::{NodeConfig, TapNode};
//! use tap_node::event::logger::{EventLogger, EventLoggerConfig, LogDestination};
//!
//! async fn example() {
//!     // Create a new TAP node
//!     let node = TapNode::new(NodeConfig::default());
//!     
//!     // Configure the event logger
//!     let logger_config = EventLoggerConfig {
//!         destination: LogDestination::File {
//!             path: "/var/log/tap-node/events.log".to_string(),
//!             max_size: Some(10 * 1024 * 1024), // 10 MB
//!             rotate: true,
//!         },
//!         structured: true, // Use JSON format
//!         log_level: log::Level::Info,
//!     };
//!     
//!     // Create and subscribe the event logger
//!     let event_logger = Arc::new(EventLogger::new(logger_config));
//!     node.event_bus().subscribe(event_logger).await;
//! }
//! ```
//!
//! ## Thread Safety
//!
//! The event system is designed to be thread-safe, with all mutable state protected
//! by appropriate synchronization primitives. The `EventBus` can be safely shared
//! across threads using `Arc<EventBus>`.

#[cfg(feature = "storage")]
pub mod customer_handler;
pub mod handlers;
pub mod logger;
pub mod trust_ping_handler;

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tap_msg::didcomm::PlainMessage;
use tokio::sync::{broadcast, RwLock};

/// Event types that can be emitted by the TAP Node
///
/// The `NodeEvent` enum represents all the possible events that can occur
/// within a TAP Node. These events can be subscribed to using the `EventBus`
/// to enable reactive programming patterns and loose coupling between components.
///
/// # Event Categories
///
/// Events are broadly categorized into:
///
/// - **PlainMessage Events**: Related to message processing and delivery (PlainMessageReceived, PlainMessageSent)
/// - **Agent Events**: Related to agent lifecycle management (AgentRegistered, AgentUnregistered)
/// - **Resolution Events**: Related to DID resolution (DidResolved)
/// - **Raw PlainMessage Events**: Raw binary messages for agents (AgentPlainMessage)
///
/// # Usage
///
/// Events are typically consumed by matching on the event type and taking appropriate action:
///
/// ```
/// use tap_node::event::NodeEvent;
///
/// fn process_event(event: NodeEvent) {
///     match event {
///         NodeEvent::PlainMessageReceived { message } => {
///             println!("PlainMessage received: {:?}", message);
///         },
///         NodeEvent::AgentRegistered { did } => {
///             println!("Agent registered: {}", did);
///         },
///         // Handle other event types...
///         _ => {}
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum NodeEvent {
    /// A DIDComm message was received by the node
    ///
    /// This event is triggered after a message has been successfully processed by
    /// the node's incoming message processors. It contains the deserialized message
    /// content as a JSON Value.
    ///
    /// # Parameters
    ///
    /// - `message`: The received message as a JSON Value
    ///
    /// # Example Use Cases
    ///
    /// - Monitoring and logging received messages
    /// - Triggering follow-up actions based on message content
    /// - Auditing message flow through the system
    PlainMessageReceived {
        /// The received message as a JSON Value
        message: Value,
    },

    /// A DIDComm message was sent from one agent to another
    ///
    /// This event is triggered after a message has been successfully processed by
    /// the node's outgoing message processors and prepared for delivery.
    ///
    /// # Parameters
    ///
    /// - `message`: The sent message as a JSON Value
    /// - `from`: The DID of the sending agent
    /// - `to`: The DID of the receiving agent
    ///
    /// # Example Use Cases
    ///
    /// - Tracking message delivery
    /// - Analyzing communication patterns
    /// - Generating message delivery receipts
    PlainMessageSent {
        /// The sent message as a JSON Value
        message: Value,
        /// The DID of the sending agent
        from: String,
        /// The DID of the receiving agent
        to: String,
    },

    /// A new agent was registered with the node
    ///
    /// This event is triggered when an agent is successfully registered with the
    /// node's agent registry. It contains the DID of the registered agent.
    ///
    /// # Parameters
    ///
    /// - `did`: The DID of the registered agent
    ///
    /// # Example Use Cases
    ///
    /// - Tracking agent lifecycle
    /// - Initializing resources for new agents
    /// - Notifying other components of new agent availability
    AgentRegistered {
        /// The DID of the registered agent
        did: String,
    },

    /// An agent was unregistered from the node
    ///
    /// This event is triggered when an agent is removed from the node's agent
    /// registry. It contains the DID of the unregistered agent.
    ///
    /// # Parameters
    ///
    /// - `did`: The DID of the unregistered agent
    ///
    /// # Example Use Cases
    ///
    /// - Cleanup of resources associated with the agent
    /// - Notifying other components of agent removal
    /// - Updating routing tables
    AgentUnregistered {
        /// The DID of the unregistered agent
        did: String,
    },

    /// A DID was resolved by the node's resolver
    ///
    /// This event is triggered when the node attempts to resolve a DID. It includes
    /// both the DID being resolved and whether the resolution was successful.
    ///
    /// # Parameters
    ///
    /// - `did`: The DID that was resolved
    /// - `success`: Whether the resolution was successful
    ///
    /// # Example Use Cases
    ///
    /// - Monitoring resolution failures
    /// - Caching resolution results
    /// - Diagnostics and debugging
    DidResolved {
        /// The DID that was resolved
        did: String,
        /// Whether the resolution was successful
        success: bool,
    },

    /// A raw message event for an agent
    ///
    /// This event contains raw binary message data intended for a specific agent.
    /// It is typically used for low-level message delivery mechanisms.
    ///
    /// # Parameters
    ///
    /// - `did`: The DID of the target agent
    /// - `message`: The raw binary message data
    ///
    /// # Example Use Cases
    ///
    /// - Direct message delivery to agents
    /// - Integration with transport-specific mechanisms
    /// - Binary protocol support
    AgentPlainMessage {
        /// The DID of the target agent
        did: String,
        /// The raw binary message data
        message: Vec<u8>,
    },

    /// A message was rejected by validation
    ///
    /// This event is triggered when a message fails validation checks and is rejected.
    /// It contains information about why the message was rejected.
    ///
    /// # Parameters
    ///
    /// - `message_id`: The ID of the rejected message
    /// - `reason`: The reason for rejection
    /// - `from`: The DID of the sender
    /// - `to`: The DID of the intended recipient
    ///
    /// # Example Use Cases
    ///
    /// - Monitoring validation failures
    /// - Alerting on suspicious activity
    /// - Debugging message flow issues
    MessageRejected {
        /// The ID of the rejected message
        message_id: String,
        /// The reason for rejection
        reason: String,
        /// The DID of the sender
        from: String,
        /// The DID of the intended recipient
        to: String,
    },

    /// A message was accepted and processed
    ///
    /// This event is triggered when a message passes all validation checks and is accepted
    /// for processing. It indicates successful message reception and validation.
    ///
    /// # Parameters
    ///
    /// - `message_id`: The ID of the accepted message
    /// - `message_type`: The type of the message
    /// - `from`: The DID of the sender
    /// - `to`: The DID of the recipient
    ///
    /// # Example Use Cases
    ///
    /// - Tracking successful message flow
    /// - Updating message status in database
    /// - Triggering downstream processing
    MessageAccepted {
        /// The ID of the accepted message
        message_id: String,
        /// The type of the message
        message_type: String,
        /// The DID of the sender
        from: String,
        /// The DID of the recipient
        to: String,
    },

    /// A reply was received for a previous message
    ///
    /// This event is triggered when a message is received that is a reply to a previously
    /// sent message. It includes both the original message and the reply for context.
    ///
    /// # Parameters
    ///
    /// - `original_message_id`: The ID of the original message
    /// - `reply_message`: The reply message
    /// - `original_message`: The original message being replied to
    ///
    /// # Example Use Cases
    ///
    /// - Correlating request/response pairs
    /// - Tracking conversation flow
    /// - Implementing timeout handling
    ReplyReceived {
        /// The ID of the original message
        original_message_id: String,
        /// The reply message
        reply_message: PlainMessage,
        /// The original message being replied to
        original_message: Box<PlainMessage>,
    },

    /// A transaction's state has changed
    ///
    /// This event is triggered when a transaction transitions from one state to another.
    /// It includes information about the state transition and optionally the agent that
    /// triggered the change.
    ///
    /// # Parameters
    ///
    /// - `transaction_id`: The ID of the transaction
    /// - `old_state`: The previous state
    /// - `new_state`: The new state
    /// - `agent_did`: The DID of the agent that triggered the change (if applicable)
    ///
    /// # Example Use Cases
    ///
    /// - Monitoring transaction lifecycle
    /// - Triggering state-specific actions
    /// - Auditing state transitions
    TransactionStateChanged {
        /// The ID of the transaction
        transaction_id: String,
        /// The previous state
        old_state: String,
        /// The new state
        new_state: String,
        /// The DID of the agent that triggered the change
        agent_did: Option<String>,
    },

    /// New events for customer extraction and compliance

    /// A message was received from a source
    MessageReceived {
        /// The received message
        message: PlainMessage,
        /// The source of the message
        source: String,
    },

    /// A message was sent to a destination
    MessageSent {
        /// The sent message
        message: PlainMessage,
        /// The destination of the message
        destination: String,
    },

    /// A new transaction was created
    TransactionCreated {
        /// The transaction data
        transaction: crate::storage::Transaction,
        /// The agent that created the transaction
        agent_did: String,
    },

    /// A customer record was created or updated
    CustomerUpdated {
        /// The customer ID
        customer_id: String,
        /// The agent that owns the customer
        agent_did: String,
        /// The type of update (created, updated, verified)
        update_type: String,
    },
}

/// Event subscriber trait for receiving node events
///
/// This trait defines the interface for components that want to receive
/// node events via callbacks. Implementers must define the `handle_event`
/// method to process events as they occur.
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` to ensure they can be safely
/// used in multithreaded environments.
///
/// # Usage
///
/// ```
/// use std::sync::Arc;
/// use async_trait::async_trait;
/// use tap_node::event::{EventSubscriber, NodeEvent, EventBus};
///
/// #[derive(Debug)]
/// struct MyEventHandler {
///     name: String,
/// }
///
/// #[async_trait]
/// impl EventSubscriber for MyEventHandler {
///     async fn handle_event(&self, event: NodeEvent) {
///         println!("Handler '{}' received event: {:?}", self.name, event);
///     }
/// }
///
/// async fn example(event_bus: &EventBus) {
///     let handler = Arc::new(MyEventHandler { name: "Logger".to_string() });
///     event_bus.subscribe(handler).await;
/// }
/// ```
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Handle a node event
    ///
    /// This method is called whenever an event is published to the event bus.
    /// Implementations should process the event appropriately based on its type.
    ///
    /// # Parameters
    ///
    /// - `event`: The NodeEvent to handle
    ///
    /// # Note
    ///
    /// - This method should return quickly to avoid blocking the event bus
    /// - For long-running operations, spawn a separate task
    /// - Handle errors gracefully, as exceptions may disrupt the event system
    async fn handle_event(&self, event: NodeEvent);
}

/// Event bus for publishing and subscribing to node events
///
/// The `EventBus` is the central coordination point for the event system. It allows
/// components to publish events and provides two mechanisms for subscribing to events:
///
/// 1. Callback-based: Register an `EventSubscriber` to receive events via callbacks
/// 2. Channel-based: Get a `broadcast::Receiver<NodeEvent>` for async event processing
///
/// # Thread Safety
///
/// The `EventBus` is designed to be thread-safe, with all mutable state protected
/// by appropriate synchronization primitives. It can be safely shared across threads
/// using `Arc<EventBus>`.
///
/// # Example Usage
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use tap_node::event::{EventBus, NodeEvent};
///
/// async fn example() {
///     // Create a new event bus
///     let event_bus = Arc::new(EventBus::new());
///
///     // Subscribe to events using a channel
///     let mut receiver = event_bus.subscribe_channel();
///
///     // Publish an event using public methods
///     let did = "did:example:123".to_string();
///     event_bus.publish_agent_registered(did).await;
///
///     // Process events from the channel
///     tokio::spawn(async move {
///         while let Ok(event) = receiver.recv().await {
///             println!("Received event: {:?}", event);
///         }
///     });
/// }
/// ```
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
    pub async fn publish_message_received(&self, message: PlainMessage) {
        let event = NodeEvent::PlainMessageReceived {
            message: serde_json::to_value(message).unwrap(),
        };
        self.publish_event(event).await;
    }

    /// Publish a message sent event
    pub async fn publish_message_sent(&self, message: PlainMessage, from: String, to: String) {
        let event = NodeEvent::PlainMessageSent {
            message: serde_json::to_value(message).unwrap(),
            from,
            to,
        };
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
        let event = NodeEvent::AgentPlainMessage { did, message };
        self.publish_event(event).await;
    }

    /// Publish a DID resolved event
    pub async fn publish_did_resolved(&self, did: String, success: bool) {
        let event = NodeEvent::DidResolved { did, success };
        self.publish_event(event).await;
    }

    /// Publish a message rejected event
    pub async fn publish_message_rejected(
        &self,
        message_id: String,
        reason: String,
        from: String,
        to: String,
    ) {
        let event = NodeEvent::MessageRejected {
            message_id,
            reason,
            from,
            to,
        };
        self.publish_event(event).await;
    }

    /// Publish a message accepted event
    pub async fn publish_message_accepted(
        &self,
        message_id: String,
        message_type: String,
        from: String,
        to: String,
    ) {
        let event = NodeEvent::MessageAccepted {
            message_id,
            message_type,
            from,
            to,
        };
        self.publish_event(event).await;
    }

    /// Publish a reply received event
    pub async fn publish_reply_received(
        &self,
        original_message_id: String,
        reply_message: PlainMessage,
        original_message: PlainMessage,
    ) {
        let event = NodeEvent::ReplyReceived {
            original_message_id,
            reply_message,
            original_message: Box::new(original_message),
        };
        self.publish_event(event).await;
    }

    /// Publish a transaction state changed event
    pub async fn publish_transaction_state_changed(
        &self,
        transaction_id: String,
        old_state: String,
        new_state: String,
        agent_did: Option<String>,
    ) {
        let event = NodeEvent::TransactionStateChanged {
            transaction_id,
            old_state,
            new_state,
            agent_did,
        };
        self.publish_event(event).await;
    }

    /// Publish an event to all subscribers
    pub async fn publish_event(&self, event: NodeEvent) {
        // Send to channel
        let _ = self.sender.send(event.clone());

        // Notify subscribers
        for subscriber in self.subscribers.read().await.iter() {
            subscriber.handle_event(event.clone()).await;
        }
    }
}
