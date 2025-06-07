# TAP Node Event System Documentation

## Overview

The TAP Node event system provides a comprehensive event-driven architecture for monitoring and reacting to various activities within the node. It implements a publish-subscribe pattern with support for both callback-based and channel-based subscriptions.

## Event Types

The TAP Node defines the following event categories and types:

### 1. Message Events

#### `PlainMessageReceived`
- **Description**: Triggered when a message is received by the node
- **Data**: 
  - `message`: The received message as a JSON Value
- **Use Cases**:
  - Monitoring and logging received messages
  - Triggering follow-up actions based on message content
  - Auditing message flow through the system

#### `PlainMessageSent`
- **Description**: Triggered when a message is sent from one agent to another
- **Data**:
  - `message`: The sent message as a JSON Value
  - `from`: The DID of the sending agent
  - `to`: The DID of the receiving agent
- **Use Cases**:
  - Tracking message delivery
  - Analyzing communication patterns
  - Generating message delivery receipts

### 2. Agent Lifecycle Events

#### `AgentRegistered`
- **Description**: Triggered when a new agent is registered with the node
- **Data**:
  - `did`: The DID of the registered agent
- **Use Cases**:
  - Tracking agent lifecycle
  - Initializing resources for new agents
  - Notifying other components of new agent availability

#### `AgentUnregistered`
- **Description**: Triggered when an agent is removed from the node
- **Data**:
  - `did`: The DID of the unregistered agent
- **Use Cases**:
  - Cleanup of resources associated with the agent
  - Notifying other components of agent removal
  - Updating routing tables

### 3. Resolution Events

#### `DidResolved`
- **Description**: Triggered when the node attempts to resolve a DID
- **Data**:
  - `did`: The DID that was resolved
  - `success`: Whether the resolution was successful
- **Use Cases**:
  - Monitoring resolution failures
  - Caching resolution results
  - Diagnostics and debugging

### 4. Raw Message Events

#### `AgentPlainMessage`
- **Description**: Contains raw binary message data intended for a specific agent
- **Data**:
  - `did`: The DID of the target agent
  - `message`: The raw binary message data (Vec<u8>)
- **Use Cases**:
  - Direct message delivery to agents
  - Integration with transport-specific mechanisms
  - Binary protocol support

### 5. Message Validation Events

#### `MessageRejected`
- **Description**: Triggered when a message fails validation checks
- **Data**:
  - `message_id`: The ID of the rejected message
  - `reason`: The reason for rejection
  - `from`: The DID of the sender
  - `to`: The DID of the intended recipient
- **Use Cases**:
  - Monitoring validation failures
  - Alerting on suspicious activity
  - Debugging message flow issues

#### `MessageAccepted`
- **Description**: Triggered when a message passes all validation checks
- **Data**:
  - `message_id`: The ID of the accepted message
  - `message_type`: The type of the message
  - `from`: The DID of the sender
  - `to`: The DID of the recipient
- **Use Cases**:
  - Tracking successful message flow
  - Updating message status in database
  - Triggering downstream processing

### 6. Reply Events

#### `ReplyReceived`
- **Description**: Triggered when a reply is received for a previously sent message
- **Data**:
  - `original_message_id`: The ID of the original message
  - `reply_message`: The reply message
  - `original_message`: The original message being replied to
- **Use Cases**:
  - Correlating request/response pairs
  - Tracking conversation flow
  - Implementing timeout handling

### 7. Transaction State Events

#### `TransactionStateChanged`
- **Description**: Triggered when a transaction transitions from one state to another
- **Data**:
  - `transaction_id`: The ID of the transaction
  - `old_state`: The previous state
  - `new_state`: The new state
  - `agent_did`: The DID of the agent that triggered the change (optional)
- **Use Cases**:
  - Monitoring transaction lifecycle
  - Triggering state-specific actions
  - Auditing state transitions

## Event Handlers

### 1. EventLogger

**Purpose**: Provides comprehensive logging of all node events to various destinations.

**Configuration**:
```rust
EventLoggerConfig {
    destination: LogDestination::File {
        path: "./logs/tap-node.log".to_string(),
        max_size: Some(10 * 1024 * 1024), // 10 MB
        rotate: true,
    },
    structured: true,  // JSON format
    log_level: log::Level::Info,
}
```

**Destination Options**:
- `Console`: Logs to standard output
- `File`: Logs to a file with optional rotation
- `Custom`: Custom logging function

**Features**:
- Plain text or structured JSON output
- Automatic file rotation
- Configurable log levels
- Thread-safe operation

### 2. MessageStatusHandler

**Purpose**: Updates message status in the database based on validation results.

**Database Integration**:
- Updates the `messages` table
- Sets status to "accepted" or "rejected"
- Handles database errors gracefully

**Subscribed Events**:
- `MessageAccepted` → Updates status to "accepted"
- `MessageRejected` → Updates status to "rejected"

### 3. TransactionStateHandler

**Purpose**: Maintains transaction state consistency in the database.

**Database Integration**:
- Updates the `transactions` table
- Maps event states to database statuses:
  - "pending" → pending
  - "confirmed" → confirmed
  - "failed" → failed
  - "cancelled" → cancelled
  - "reverted" → reverted

**Subscribed Events**:
- `TransactionStateChanged`

### 4. TransactionAuditHandler

**Purpose**: Provides detailed audit logging for compliance and debugging.

**Features**:
- Logs all transaction state transitions
- Includes agent DIDs when available
- Provides human-readable audit trail
- Helps with debugging transaction flows

**Subscribed Events**:
- `TransactionStateChanged`
- `MessageAccepted`
- `MessageRejected`
- `ReplyReceived`

### 5. TrustPingResponseHandler

**Purpose**: Handles automatic Trust Ping response delivery.

**Operation**:
1. Monitors `PlainMessageSent` events
2. Filters for Trust Ping response messages
3. Serializes and sends responses via configured sender
4. Logs delivery success/failure

**Integration**:
- Works with `TrustPingProcessor` for automatic responses
- Supports various message senders (HTTP, WebSocket)

## Event Processing Workflows

### Trust Ping Workflow

```
1. Incoming Trust Ping → TrustPingProcessor
2. Generate response → Publish PlainMessageSent event
3. TrustPingResponseHandler catches event
4. Response sent via configured sender
```

### Message Validation Workflow

```
1. Message received → ValidationProcessor
2. Validation passes → MessageAccepted event
   OR
   Validation fails → MessageRejected event
3. MessageStatusHandler updates database
```

### Transaction State Machine Workflow

```
1. Transfer/Payment received → Create transaction
2. Authorize received → TransactionStateChanged event
3. All agents authorized → Generate Settle message
4. TransactionStateHandler updates database
5. TransactionAuditHandler logs transition
```

## Subscription Models

### 1. Callback-based Subscriptions

Implement the `EventSubscriber` trait:

```rust
#[async_trait]
impl EventSubscriber for MyHandler {
    async fn handle_event(&self, event: NodeEvent) {
        match event {
            NodeEvent::MessageReceived { message } => {
                // Handle received message
            },
            _ => {}
        }
    }
}

// Subscribe
event_bus.subscribe(Arc::new(MyHandler)).await;
```

### 2. Channel-based Subscriptions

Use broadcast channels for async processing:

```rust
let mut receiver = event_bus.subscribe_channel();

tokio::spawn(async move {
    while let Ok(event) = receiver.recv().await {
        // Process event
    }
});
```

## Best Practices

### 1. Handler Design
- Keep handlers lightweight and non-blocking
- Spawn separate tasks for long-running operations
- Handle errors gracefully without panicking
- Use appropriate logging levels

### 2. Event Publishing
- Use the provided convenience methods on EventBus
- Include all relevant data in events
- Consider event ordering implications
- Avoid publishing events from within handlers (prevent loops)

### 3. Performance Considerations
- The event bus uses a broadcast channel with capacity 100
- Slow subscribers can cause backpressure
- Consider using channel-based subscriptions for heavy processing
- Monitor event processing latency

### 4. Error Handling
- Handlers should not panic on errors
- Log errors appropriately
- Consider retry logic for transient failures
- Maintain system stability despite handler failures

## Configuration Example

```rust
use tap_node::{NodeConfig, TapNode};
use tap_node::event::logger::{EventLogger, EventLoggerConfig, LogDestination};

// Configure node with event logging
let config = NodeConfig {
    event_logger: Some(EventLoggerConfig {
        destination: LogDestination::File {
            path: "./logs/tap-node.log".to_string(),
            max_size: Some(10 * 1024 * 1024),
            rotate: true,
        },
        structured: true,
        log_level: log::Level::Info,
    }),
    ..Default::default()
};

// Create node - event handlers are automatically set up
let mut node = TapNode::new(config);
node.init_storage().await?;
```

## Extending the Event System

To add new event types:

1. Add variant to `NodeEvent` enum
2. Add publishing method to `EventBus`
3. Update existing handlers if needed
4. Document the new event type

To add new handlers:

1. Implement `EventSubscriber` trait
2. Subscribe to the event bus
3. Handle relevant events
4. Add error handling and logging
5. Test with various event scenarios