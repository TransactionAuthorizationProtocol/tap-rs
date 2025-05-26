# TAP Node Storage Implementation

## Overview

This document describes the storage implementation for TAP Node, which provides:
1. Persistent storage for Transfer and Payment transactions
2. Complete audit trail of all incoming and outgoing messages

## Features

- **SQLite-based storage** with connection pooling via r2d2
- **Dual-table design**: Separate tables for transactions and message audit trail
- **Append-only design** for auditing and compliance
- **Automatic schema migrations** on startup
- **WASM compatibility** through feature gates
- **Async-friendly API** using tokio spawn_blocking

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

## Usage

### Configuration

Set the database path via environment variable:
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
// Storage is automatically initialized when creating a TapNode
let node = TapNode::new(config);

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
}
```

## Feature Flags

- `storage`: Enables storage functionality (enabled by default)
- Storage is automatically disabled for WASM builds

## Implementation Notes

1. The `reference_id` uses the PlainMessage's `id` field as the unique identifier
2. All incoming and outgoing messages are automatically logged to the messages table
3. Transfer and Payment messages are additionally stored in the transactions table
4. WAL mode is enabled for better concurrency
5. Connection pooling supports up to 10 concurrent connections
6. Duplicate messages are silently ignored (no error on re-insertion)