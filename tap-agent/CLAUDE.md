# tap-agent Crate

Core TAP agent implementation providing cryptographic key management, message packing/unpacking, and DID operations.

## Purpose

The `tap-agent` crate provides:
- TAP agent implementation with key management
- Message packing and unpacking with encryption/signing
- DID generation and resolution
- Payment link creation and processing
- Out-of-band message handling
- Key storage and persistence

## Key Components

- `agent.rs` - Main TapAgent implementation
- `agent_key_manager.rs` - Key management for agents
- `did.rs` - DID operations and utilities
- `message_packing.rs` - Message encryption/decryption
- `payment_link.rs` - Payment link functionality
- `storage.rs` - Key storage abstraction
- `verification.rs` - Message verification

## Build Commands

```bash
# Build the crate
cargo build -p tap-agent

# Run tests
cargo test -p tap-agent

# Run specific test
cargo test -p tap-agent test_name

# Run benchmarks
cargo bench -p tap-agent

# Run agent benchmark
cargo bench --bench agent_benchmark

# Build with WASM support
cargo build -p tap-agent --features wasm

# Build with test utilities
cargo build -p tap-agent --features test-utils

# Build with examples
cargo build -p tap-agent --features examples
```

## Development Guidelines

### Agent Implementation
- Use `TapAgent` as the primary interface for TAP operations
- Implement proper error handling with custom error types
- Ensure thread safety for multi-threaded environments
- Support both ephemeral and persistent key storage

### Key Management
- Use `AgentKeyManager` for cryptographic operations
- Support multiple key types (Ed25519, secp256k1, P-256)
- Implement secure key storage with optional persistence
- Include key rotation and backup capabilities

### Message Processing
- All messages must be properly encrypted and signed
- Use DIDComm v2 for message transport
- Implement proper nonce handling to prevent replay attacks
- Support both direct and routed message delivery

### Testing
- Create comprehensive test suites for all agent operations
- Test key generation, storage, and recovery
- Include integration tests with other crates
- Use property-based testing for cryptographic operations

## Features

- `native` (default) - Includes reqwest for HTTP transport
- `wasm` - WebAssembly support with browser APIs
- `test-utils` - Testing utilities and temporary storage
- `examples` - Example code compilation

## Examples

The crate includes several examples:
- `transfer_flow.rs` - Basic transfer workflow
- `payment_flow.rs` - Payment request handling
- `key_management.rs` - Key operations
- `multi_agent_flow.rs` - Multi-agent interactions

Run examples with:
```bash
cargo run --example transfer_flow --features examples
```

## Cryptographic Operations

The agent supports multiple cryptographic curves:
- **Ed25519** - Primary signing algorithm for DIDs
- **secp256k1** - Bitcoin/Ethereum compatibility
- **P-256** - NIST standard curve

All cryptographic operations use industry-standard libraries and follow best practices for key generation, storage, and usage.

## Storage Options

- **Ephemeral** - In-memory key storage (testing/temporary use)
- **Persistent** - File-based key storage with encryption
- **Custom** - Pluggable storage backend support

## DID Support

- **DID:key** - Built-in support for key-based DIDs
- **DID Resolution** - Pluggable resolver interface
- **Key Recovery** - DID reconstruction from stored keys

## WASM Compatibility

When built with the `wasm` feature, the agent:
- Uses browser APIs for HTTP requests
- Implements WASM-compatible random number generation
- Provides JavaScript-compatible async interfaces
- Supports browser storage APIs

## Testing

Run the full test suite with:
```bash
cargo test -p tap-agent --all-features
```

For integration testing with other components:
```bash
cargo test --workspace
```