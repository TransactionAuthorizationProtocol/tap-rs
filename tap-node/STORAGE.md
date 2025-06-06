# TAP Node Storage Implementation

## Overview

This document describes the storage implementation for TAP Node, which provides:
1. Persistent storage for Transfer and Payment transactions
2. Complete audit trail of all incoming and outgoing messages
3. **Message delivery tracking** with comprehensive status monitoring
4. DID-based storage organization with configurable root directory

## Features

- **SQLite-based storage** with connection pooling via sqlx
- **Dual-table design**: Separate tables for transactions and message audit trail
- **Append-only design** for auditing and compliance
- **Automatic schema migrations** on startup
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
Audit trail for all DIDComm messages:
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

## Usage

### Configuration

#### Default DID-based Storage
By default, storage uses a DID-based path structure under `~/.tap`:
```rust
let config = NodeConfig {
    agent_did: Some("did:web:example.com".to_string()),
    ..Default::default()
};
// Creates database at ~/.tap/did_web_example.com/transactions.db
```

#### Custom TAP Root Directory
Set a custom root directory via environment variable:
```bash
export TAP_ROOT=/custom/tap/directory
```

Or configure in NodeConfig:
```rust
let config = NodeConfig {
    agent_did: Some("did:web:example.com".to_string()),
    tap_root: Some(PathBuf::from("/custom/tap/root")),
    ..Default::default()
};
// Creates database at /custom/tap/root/did_web_example.com/transactions.db
```

#### Explicit Database Path
For backward compatibility, you can still specify an explicit path:
```bash
export TAP_NODE_DB_PATH=/path/to/database.db
```

Or configure in NodeConfig:
```rust
let config = NodeConfig {
    storage_path: Some(PathBuf::from("/path/to/database.db")),
    ..Default::default()
};
```

### API Examples

```rust
// Create a node with DID-based storage
let config = NodeConfig {
    agent_did: Some("did:web:example.com".to_string()),
    ..Default::default()
};
let mut node = TapNode::new(config);

// Initialize storage (required for async initialization)
node.init_storage().await?;

// Access storage
if let Some(storage) = node.storage() {
    // Transaction operations
    let tx = storage.get_transaction_by_id("msg_123").await?;
    let txs = storage.list_transactions(10, 0).await?;
    
    // Message audit trail operations
    let msg = storage.get_message_by_id("msg_123").await?;
    let all_msgs = storage.list_messages(10, 0, None).await?;
    let incoming_msgs = storage.list_messages(10, 0, Some(MessageDirection::Incoming)).await?;
    
    // Manual message logging (automatic for node operations)
    storage.log_message(&message, MessageDirection::Incoming).await?;
    
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

// Direct storage creation with DID
let storage = Storage::new_with_did("did:web:example.com", None).await?;

// Get default logs directory
let logs_dir = Storage::default_logs_dir(None);
// Returns ~/.tap/logs
```

## Feature Flags

- `storage`: Enables storage functionality (enabled by default)
- Storage is automatically disabled for WASM builds

## Implementation Notes

1. The `reference_id` uses the PlainMessage's `id` field as the unique identifier
2. All incoming and outgoing messages are automatically logged to the messages table
3. Transfer and Payment messages are additionally stored in the transactions table
4. **Delivery tracking is automatic for:**
   - HTTP deliveries when using `HttpPlainMessageSenderWithTracking`
   - Internal agent deliveries within TAP Node
   - Router-based message deliveries
5. WAL mode is enabled for better concurrency
6. Connection pooling supports up to 10 concurrent connections
7. Duplicate messages are silently ignored (no error on re-insertion)
8. Delivery records include full message text for debugging and retry processing

## Delivery Tracking Features

- **Automatic Status Updates**: Delivery status automatically updated from `pending` to `success` or `failed`
- **Error Logging**: HTTP status codes and error messages captured for debugging
- **Retry Processing**: Retry count tracked for future automatic retry implementation
- **Multiple Delivery Types**: Support for `https`, `internal`, `return_path`, and `pickup` delivery types
- **Agent Isolation**: Each agent has separate delivery tracking in DID-specific databases
- **MCP Integration**: Delivery data accessible via MCP `tap://deliveries` resource for monitoring