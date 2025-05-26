# TAP Node Storage Implementation

## Overview

This document describes the storage implementation for TAP Node, which provides persistent storage for Transfer and Payment transactions.

## Features

- **SQLite-based storage** with connection pooling via r2d2
- **Append-only design** for transaction auditing
- **Automatic schema migrations** on startup
- **WASM compatibility** through feature gates
- **Async-friendly API** using tokio spawn_blocking

## Database Schema

### transactions table
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
    // Get transaction by ID
    let tx = storage.get_transaction_by_id("msg_123").await?;
    
    // List recent transactions
    let txs = storage.list_transactions(10, 0).await?;
}
```

## Feature Flags

- `storage`: Enables storage functionality (enabled by default)
- Storage is automatically disabled for WASM builds

## Implementation Notes

1. The `reference_id` uses the PlainMessage's `id` field as the unique identifier
2. Transactions are automatically stored when Transfer or Payment messages are processed
3. WAL mode is enabled for better concurrency
4. Connection pooling supports up to 10 concurrent connections