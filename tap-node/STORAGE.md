# TAP Node Storage Implementation

## Overview

This document describes the storage implementation for TAP Node, which provides:
1. Persistent storage for Transfer and Payment transactions
2. Complete audit trail of all incoming and outgoing messages
3. DID-based storage organization with configurable root directory

## Features

- **SQLite-based storage** with connection pooling via sqlx
- **Dual-table design**: Separate tables for transactions and message audit trail
- **Append-only design** for auditing and compliance
- **Automatic schema migrations** on startup
- **WASM compatibility** through feature gates
- **Native async API** using sqlx
- **DID-based database paths** for multi-agent support
- **Configurable TAP root directory** with sensible defaults

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
4. WAL mode is enabled for better concurrency
5. Connection pooling supports up to 10 concurrent connections
6. Duplicate messages are silently ignored (no error on re-insertion)