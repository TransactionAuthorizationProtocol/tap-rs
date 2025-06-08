# TAP Node Per-Agent Storage Implementation

## Overview

This document describes the storage implementation for TAP Node, which provides:
1. Persistent storage for Transfer and Payment transactions
2. Complete audit trail of all incoming and outgoing messages
3. **Message delivery tracking** with comprehensive status monitoring
4. DID-based storage organization with configurable root directory

## Features

- **Agent-Specific Databases**: Each agent has its own SQLite database at `~/.tap/{sanitized_did}/transactions.db`
- **AgentStorageManager**: Centralized management with caching and lazy loading of agent storage
- **Multi-Agent Transaction Distribution**: Transactions automatically stored in all involved agents' databases
- **Multi-Recipient Message Delivery**: Full DIDComm spec compliance for message delivery to all recipients
- **Dual-table design**: Separate tables for transactions and message audit trail (per agent)
- **Append-only design** for auditing and compliance
- **Automatic schema migrations** on startup per agent database
- **WASM compatibility** through feature gates
- **Native async API** using sqlx
- **DID-based database paths** for multi-agent support
- **Configurable TAP root directory** with sensible defaults
- **Message delivery tracking** with automatic status updates and retry counting

## Database Schema

### transactions table
Stores Transfer and Payment transactions for business logic processing:
- `id`: Auto-incrementing primary key
- `type`: Transaction type (transfer/payment)
- `reference_id`: Unique message ID
- `from_did`: Sender DID
- `to_did`: Recipient DID
- `thread_id`: DIDComm thread ID
- `message_type`: Full TAP message type URI
- `status`: Transaction status (pending/confirmed/failed/cancelled/reverted)
- `message_json`: Full DIDComm message as JSON
- `created_at`: Creation timestamp
- `updated_at`: Last update timestamp

### messages table
Audit trail for all DIDComm messages (plaintext only):
- `id`: Auto-incrementing primary key
- `message_id`: Unique DIDComm message ID
- `message_type`: TAP message type URI
- `from_did`: Sender DID
- `to_did`: Recipient DID (first in the 'to' array)
- `thread_id`: DIDComm thread ID
- `parent_thread_id`: Parent thread ID for nested conversations
- `direction`: Message direction (incoming/outgoing)
- `message_json`: Full DIDComm message as JSON
- `created_at`: Timestamp when message was logged
- `status`: Message acceptance status (pending/accepted/rejected)
- `raw_message`: **DEPRECATED** - Raw messages are now stored in the `received` table

### received table
Stores raw incoming messages (JWE, JWS, or plain JSON) before processing:
- `id`: Auto-incrementing primary key
- `message_id`: Message ID extracted from raw message (if available)
- `raw_message`: Complete raw message content as received
- `source_type`: Source of the message (`https`, `internal`, `websocket`, `return_path`, `pickup`)
- `source_identifier`: Optional identifier for the source (URL, agent DID, etc.)
- `status`: Processing status (`pending`, `processed`, `failed`)
- `error_message`: Error details if processing failed
- `received_at`: Timestamp when message was received
- `processed_at`: Timestamp when processing completed
- `processed_message_id`: Foreign key to messages table for successfully processed messages

### deliveries table
Message delivery tracking and monitoring:
- `id`: Auto-incrementing primary key
- `message_id`: Message ID being delivered
- `message_text`: Full message content as text
- `recipient_did`: Target recipient DID
- `delivery_url`: Endpoint URL (for HTTP deliveries)
- `delivery_type`: Type of delivery (`https`, `internal`, `return_path`, `pickup`)
- `status`: Delivery status (`pending`, `success`, `failed`)
- `retry_count`: Number of retry attempts
- `last_http_status_code`: HTTP status from last delivery attempt
- `error_message`: Error details for failed deliveries
- `created_at`: When delivery record was created
- `updated_at`: Last status update timestamp
- `delivered_at`: Timestamp of successful delivery completion

## Agent Storage Architecture

### Storage Directory Structure
```
~/.tap/                                    # TAP root directory
├── did_key_z6MkpGuzuD38tpgZKPfm/          # Agent-specific directory (sanitized DID)
│   └── transactions.db                    # SQLite database for this agent
├── did_web_example.com/                   # Another agent's directory
│   └── transactions.db                    # SQLite database for this agent
└── logs/                                  # Shared logs directory
```

### AgentStorageManager

The `AgentStorageManager` provides centralized management of per-agent storage:

```rust
// Get storage for a specific agent
let storage_manager = node.agent_storage_manager().unwrap();
let agent_storage = storage_manager.get_agent_storage("did:example:agent").await?;

// Storage is automatically cached and reused
let same_storage = storage_manager.get_agent_storage("did:example:agent").await?;
```

### Multi-Agent Transaction Storage

When a transaction involves multiple agents, it's automatically stored in all their databases:

```rust
// This Transfer will be stored in databases for:
// - Originator (Alice)
// - Beneficiary (Bob)
// - All agents in the agents array
// - All recipients in the message's 'to' field
let transfer = Transfer {
    originator: Party::new("did:example:alice"),
    beneficiary: Some(Party::new("did:example:bob")),
    agents: vec![Agent::new("did:example:custodian", "Custodian", "did:example:alice")],
    // ...
};
```

## Usage

### Configuration

#### Default Per-Agent Storage
By default, each agent gets its own storage under `~/.tap`:
```rust
let config = NodeConfig {
    tap_root: None, // Uses ~/.tap
    ..Default::default()
};

// When registering agents, each gets their own database:
// ~/.tap/did_key_z6MkpGuzuD38tpgZKPfm/transactions.db
// ~/.tap/did_web_example.com/transactions.db
```

#### Custom TAP Root Directory
```rust
let config = NodeConfig {
    tap_root: Some(PathBuf::from("/custom/tap/root")),
    ..Default::default()
};
// Agent databases at: /custom/tap/root/{sanitized_did}/transactions.db
```

#### Legacy Centralized Storage (Optional)
For backward compatibility, you can still use centralized storage:
```rust
let config = NodeConfig {
    storage_path: Some(PathBuf::from("/path/to/centralized.db")),
    ..Default::default()
};
```

### API Examples

```rust
// Create a node with per-agent storage
let config = NodeConfig {
    tap_root: None, // Uses ~/.tap for per-agent storage
    ..Default::default()
};
let mut node = TapNode::new(config);

// Initialize storage (required for async initialization)
node.init_storage().await?;

// Register agents (automatically initializes their storage)
let (agent1, did1) = TapAgent::from_ephemeral_key().await?;
let (agent2, did2) = TapAgent::from_ephemeral_key().await?;
node.register_agent(Arc::new(agent1)).await?;
node.register_agent(Arc::new(agent2)).await?;

// Access agent-specific storage
if let Some(storage_manager) = node.agent_storage_manager() {
    // Get storage for a specific agent
    let agent1_storage = storage_manager.get_agent_storage(&did1).await?;
    let agent2_storage = storage_manager.get_agent_storage(&did2).await?;

    // Each agent has their own transaction and message data
    let agent1_txs = agent1_storage.list_transactions(10, 0).await?;
    let agent2_txs = agent2_storage.list_transactions(10, 0).await?;

    // Manual message logging (automatic for node operations)
    storage.log_message(&message, MessageDirection::Incoming).await?;

    // Receive raw messages (automatic when using receive_message_from_source)
    let received_id = storage.create_received(
        raw_message_str,
        SourceType::Https,
        Some("https://example.com/sender")
    ).await?;

    // Update processing status
    storage.update_received_status(
        received_id,
        ReceivedStatus::Processed,
        Some("msg_123"), // processed message ID
        None
    ).await?;

    // Query received messages
    let pending = storage.get_pending_received(100).await?;
    let all_received = storage.list_received(50, 0, None, None).await?;

    // Delivery tracking operations
    let deliveries = storage.get_deliveries_for_message("msg_123").await?;
    let pending = storage.get_pending_deliveries(10, 50).await?;
    let failed = storage.get_failed_deliveries_for_recipient("did:example:recipient", 20, 0).await?;

    // Create and update delivery records (automatic for HTTP senders with tracking)
    let delivery_id = storage.create_delivery(
        "msg_123",
        &message_text,
        "did:example:recipient",
        Some("https://example.com/endpoint"),
        DeliveryType::Https
    ).await?;

    storage.update_delivery_status(
        delivery_id,
        DeliveryStatus::Success,
        Some(200),
        None
    ).await?;
}

// Legacy centralized storage access (if configured)
if let Some(storage) = node.storage() {
    let txs = storage.list_transactions(10, 0).await?;
    let msgs = storage.list_messages(10, 0, None).await?;
}

// Direct storage creation for specific agent
let agent_storage = Storage::new_with_did("did:web:example.com", None).await?;
// Creates database at ~/.tap/did_web_example.com/transactions.db
```

## Feature Flags

- `storage`: Enables storage functionality (enabled by default)
- Storage is automatically disabled for WASM builds

## Implementation Notes

1. The `reference_id` uses the PlainMessage's `id` field as the unique identifier
2. All incoming and outgoing messages are automatically logged to the messages table
3. Transfer and Payment messages are additionally stored in the transactions table
4. **Raw message storage:**
   - All incoming messages (JWE, JWS, or plain) are stored in the `received` table
   - The `raw_message` column in `messages` table is deprecated
   - Use `receive_message_from_source()` to specify message source information
5. **Delivery tracking is automatic for:**
   - HTTP deliveries when using `HttpPlainMessageSenderWithTracking`
   - Internal agent deliveries within TAP Node
   - Router-based message deliveries
6. WAL mode is enabled for better concurrency
7. Connection pooling supports up to 10 concurrent connections
8. Duplicate messages are silently ignored (no error on re-insertion)
9. Delivery records include full message text for debugging and retry processing

## Delivery Tracking Features

- **Automatic Status Updates**: Delivery status automatically updated from `pending` to `success` or `failed`
- **Error Logging**: HTTP status codes and error messages captured for debugging
- **Retry Processing**: Retry count tracked for future automatic retry implementation
- **Multiple Delivery Types**: Support for `https`, `internal`, `return_path`, and `pickup` delivery types
- **Agent Isolation**: Each agent has separate delivery tracking in DID-specific databases
- **MCP Integration**: Delivery data accessible via MCP `tap://deliveries` resource for monitoring
