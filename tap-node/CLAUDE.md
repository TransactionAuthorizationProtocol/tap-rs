# tap-node Crate

TAP node implementation providing message routing, processing, storage, and state management for the Transaction Authorization Protocol.

## Purpose

The `tap-node` crate provides:
- Message routing and processing infrastructure
- Persistent storage for transactions and messages
- Event handling and logging
- State machine management for transaction flows
- Customer data management and travel rule compliance
- Multi-agent message delivery tracking

## Key Components

- `message/` - Message processing and routing
  - `processor.rs` - Core message processing logic
  - `router.rs` - Message routing between agents
  - `sender.rs` - Message delivery mechanisms
- `storage/` - Data persistence layer
  - `db.rs` - Database operations and queries
  - `models.rs` - Data models and structures
  - `agent_storage_manager.rs` - Per-agent storage management
- `event/` - Event handling system
  - `handlers.rs` - Event processing logic
  - `logger.rs` - Event logging and auditing
  - `customer_handler.rs` - Customer data extraction
- `state_machine/` - Transaction state management
- `validation/` - Message and data validation

## Build Commands

```bash
# Build the crate (with default features: native + storage)
cargo build -p tap-node

# Run tests
cargo test -p tap-node

# Run specific test
cargo test -p tap-node test_name

# Run benchmarks
cargo bench -p tap-node

# Run stress tests
cargo bench --bench stress_test

# Build with WebSocket support
cargo build -p tap-node --features native-with-websocket

# Build for WASM
cargo build -p tap-node --features wasm

# Build WASM with WebSocket
cargo build -p tap-node --features wasm-with-websocket
```

## Development Guidelines

### Message Processing
- All messages flow through the processor pipeline
- Implement validation before processing
- Use event handlers for side effects
- Maintain transaction state consistency
- Support both synchronous and asynchronous processing

### Storage Operations
- Use migrations for database schema changes (see `migrations/`)
- Implement proper transaction isolation
- Support per-agent data separation
- Include comprehensive error handling
- Test with both in-memory and persistent storage

### Event Handling
- Create event handlers for business logic
- Log all significant events for audit trails
- Support pluggable event handler registration
- Include customer data extraction for compliance

### State Management
- Use state machines for complex transaction flows
- Ensure state transitions are atomic
- Support rollback and recovery mechanisms
- Maintain consistency across distributed operations

## Features

- `native` (default) - Full native features with Tokio runtime and reqwest
- `storage` (default) - SQLite database storage with migrations
- `websocket` - WebSocket transport support
- `native-with-websocket` - Native features plus WebSocket
- `wasm` - WebAssembly support with browser APIs
- `wasm-with-websocket` - WASM with WebSocket support

## Database Schema

The node uses SQLite with the following main tables:
- `transactions` - TAP transaction records
- `messages` - Message storage and tracking
- `transaction_agents` - Agent-transaction relationships
- `deliveries` - Message delivery tracking
- `received` - Raw received message storage
- `customers` - Customer/party information

Database migrations are in `migrations/` and run automatically on startup.

## Examples

The crate includes practical examples:
- `event_logger_demo.rs` - Event logging setup
- `delivery_tracking_demo.rs` - Message delivery tracking
- `travel_rule_flow.rs` - Travel rule compliance
- `websocket_message_flow.rs` - WebSocket transport

Run examples with:
```bash
cargo run --example event_logger_demo -p tap-node
```

## Storage Features

### Per-Agent Storage
- Each agent maintains separate data isolation
- Automatic agent discovery and registration  
- Consistent storage interface across agents

### Transaction Management
- Full transaction lifecycle tracking
- Message threading and correlation
- Delivery status monitoring
- Customer data extraction and hashing

### Compliance Features
- Travel rule data extraction
- Customer information management
- IVMS101 data structure support
- PII hashing for privacy protection

## WebSocket Support

With WebSocket features enabled:
- Real-time message delivery
- Bidirectional communication
- Connection management and reconnection
- Both native and WASM WebSocket implementations

## WASM Compatibility

When built for WASM:
- Browser-compatible storage (no SQLite)
- In-memory message processing
- WebSocket support via browser APIs
- Compatible with web workers

## Testing

Comprehensive test coverage including:
- Unit tests for individual components
- Integration tests for message flows
- Storage layer tests with migrations
- Multi-agent interaction tests
- Performance and stress tests

Run the full test suite:
```bash
cargo test -p tap-node --all-features
```

## Performance

The node is designed for high throughput:
- Async message processing with Tokio
- Concurrent storage operations
- Connection pooling for database access
- Efficient message routing algorithms

Benchmark performance with:
```bash
cargo bench -p tap-node --bench stress_test
```

## Related Tools

- [tap-cli](../tap-cli/README.md) — Command-line interface that wraps tap-node for terminal use
- [tap-mcp](../tap-mcp/README.md) — MCP server exposing tap-node to AI assistants
- [tap-http](../tap-http/README.md) — HTTP server using tap-node for DIDComm transport