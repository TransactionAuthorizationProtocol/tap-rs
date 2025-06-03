# TAP Node Per-Agent Storage Implementation

## Overview

This document describes the per-agent storage implementation for TAP Node, which provides:
1. **Agent-isolated storage**: Each agent gets its own dedicated SQLite database
2. **Multi-agent transaction storage**: Transactions are stored in all involved agents' databases
3. **Multi-recipient message delivery**: Messages delivered to ALL recipients in the `to` field
4. **Complete audit trail**: All incoming and outgoing messages logged per agent
5. **Automatic storage initialization**: Agent storage created during registration
6. **DID-based organization**: Storage organized by agent DID with configurable root directory

## Features

- **Agent-Specific Databases**: Each agent has its own SQLite database at `~/.tap/{sanitized_did}/transactions.db`
- **AgentStorageManager**: Centralized management with caching and lazy loading of agent storage
- **Multi-Agent Transaction Distribution**: Transactions automatically stored in all involved agents' databases
- **Multi-Recipient Message Delivery**: Full DIDComm spec compliance for message delivery to all recipients
- **Dual-table design**: Separate tables for transactions and message audit trail (per agent)
- **Append-only design** for auditing and compliance
- **Automatic schema migrations** on startup per agent database
- **WASM compatibility** through feature gates
- **Native async API** using sqlx with connection pooling per agent
- **Configurable TAP root directory** with sensible defaults (`~/.tap`)

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
    
    // Message audit trail operations per agent
    let agent1_msgs = agent1_storage.list_messages(10, 0, None).await?;
    let agent2_incoming = agent2_storage.list_messages(10, 0, Some(MessageDirection::Incoming)).await?;
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
4. WAL mode is enabled for better concurrency
5. Connection pooling supports up to 10 concurrent connections
6. Duplicate messages are silently ignored (no error on re-insertion)