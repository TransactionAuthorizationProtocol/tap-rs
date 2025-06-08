# TAP Node Event System Documentation

## Overview

The TAP Node event system provides a comprehensive event-driven architecture for monitoring and reacting to various activities within the node. It implements a publish-subscribe pattern with support for both callback-based and channel-based subscriptions.

## Event Types

The TAP Node defines the following event categories and types:

### 1. Message Events

#### `PlainMessageReceived` (Deprecated)
- **Description**: Legacy event for backward compatibility
- **Data**: 
  - `message`: The received message as a JSON Value
- **Note**: Use `MessageReceived` instead for new implementations

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

### 8. Enhanced Message Events

#### `MessageReceived`
- **Description**: Triggered when a message is received from a specific source
- **Data**:
  - `message`: The received PlainMessage object
  - `source`: The source identifier (e.g., "https", "internal", "websocket")
- **Use Cases**:
  - Source-aware message processing
  - Transport-specific handling
  - Security auditing by source

#### `MessageSent`
- **Description**: Triggered when a message is sent to a specific destination
- **Data**:
  - `message`: The sent PlainMessage object
  - `destination`: The destination identifier
- **Use Cases**:
  - Destination-aware delivery tracking
  - Transport selection
  - Delivery confirmation

### 9. Transaction Management Events

#### `TransactionCreated`
- **Description**: Triggered when a new transaction is created in the system
- **Data**:
  - `transaction`: The complete transaction data from storage
  - `agent_did`: The DID of the agent that created the transaction
- **Use Cases**:
  - Customer data extraction
  - Compliance reporting
  - Transaction monitoring

### 10. Customer Management Events

#### `CustomerUpdated`
- **Description**: Triggered when customer information is created or updated
- **Data**:
  - `customer_id`: The unique customer identifier
  - `agent_did`: The DID of the agent that owns the customer
  - `update_type`: The type of update ("created", "updated", "verified")
- **Use Cases**:
  - Customer lifecycle tracking
  - KYC/AML monitoring
  - Data synchronization

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

### 6. CustomerEventHandler

**Purpose**: Automatically extracts and manages customer data from TAP messages.

**Features**:
- Extracts party information from Transfer messages
- Updates customer records from UpdateParty messages
- Manages relationships from ConfirmRelationship messages
- Generates IVMS101-compatible data structures
- **Automatically registered for each agent** when the agent is registered with the node

**Subscribed Events**:
- `MessageReceived` → Processes incoming Transfer, UpdateParty, and ConfirmRelationship messages
- `MessageSent` → Processes outgoing messages for customer data
- `TransactionCreated` → Extracts customer data from new transactions

**Database Integration**:
- Creates/updates customer records
- Manages customer relationships
- Stores IVMS101 compliance data
- Maintains customer metadata
- Uses agent-specific storage for data isolation

**Operation**:
1. Automatically created when an agent is registered with `node.register_agent()`
2. Monitors message events for relevant TAP message types
3. Extracts party information (originator, beneficiary, agents)
4. Creates or updates customer records with extracted data
5. Establishes relationships between customers and agents
6. Stores compliance-relevant information for reporting in agent-specific database

**Automatic Registration**:
The CustomerEventHandler is now automatically registered for each agent during the agent registration process. This ensures that:
- Each agent has its own customer data handler
- Customer data is properly isolated in agent-specific databases
- No manual configuration is needed for customer data extraction

## Event Processing Workflows

### Trust Ping Workflow

```
1. Incoming Trust Ping → TrustPingProcessor
2. Generate response → Publish PlainMessageSent event
3. TrustPingResponseHandler catches event
4. Response sent via configured sender
```

### Customer Data Extraction Workflow

```
1. Transfer/Payment message received → MessageReceived event
2. CustomerEventHandler processes message
3. Extract party information (originator, beneficiary)
4. Create/update customer records
5. CustomerUpdated event published
6. Relationships established between parties
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
2. TransactionCreated event published
3. CustomerEventHandler extracts customer data
4. Authorize received → TransactionStateChanged event
5. All agents authorized → Generate Settle message
6. TransactionStateHandler updates database
7. TransactionAuditHandler logs transition
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
- Consider data privacy when logging customer information

### 2. Event Publishing
- Use the provided convenience methods on EventBus
- Include all relevant data in events
- Consider event ordering implications
- Avoid publishing events from within handlers (prevent loops)
- Use specific event types (MessageReceived vs PlainMessageReceived) for clarity

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

// Note: CustomerEventHandler is now automatically registered for each agent
// when the agent is registered with the node, so manual registration is no longer needed
```

## Extending the Event System

To add new event types:

1. Add variant to `NodeEvent` enum
2. Add publishing method to `EventBus`
3. Update existing handlers if needed
4. Document the new event type
5. Consider backward compatibility with legacy events

To add new handlers:

1. Implement `EventSubscriber` trait
2. Subscribe to the event bus
3. Handle relevant events
4. Add error handling and logging
5. Test with various event scenarios
6. Consider performance impact for high-frequency events

## Event Type Migration Guide

The TAP Node event system has evolved to provide more specific and feature-rich events:

- `PlainMessageReceived` → `MessageReceived` (includes source information)
- `PlainMessageSent` → `MessageSent` (includes destination information)

Legacy events are maintained for backward compatibility but new implementations should use the enhanced event types for better tracking and monitoring capabilities.