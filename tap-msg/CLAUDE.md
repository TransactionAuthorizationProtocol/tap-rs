# tap-msg Crate

Core message processing library for the Transaction Authorization Protocol (TAP). This crate defines all TAP message types, serialization, validation, and DIDComm integration.

## Purpose

The `tap-msg` crate provides:
- TAP message type definitions and enums
- Message validation and serialization
- DIDComm message wrapping and unwrapping
- Message context and threading support
- Policy and constraint definitions

## Key Components

- `message/` - All TAP message type definitions
  - `transfer.rs` - Transfer message implementation
  - `payment.rs` - Payment request messages
  - `authorize.rs` - Authorization messages
  - `settle.rs` - Settlement messages
  - `cancel.rs` - Cancellation messages
  - `agent_management.rs` - Agent management messages
  - And more...
- `didcomm.rs` - DIDComm message wrapper
- `utils/` - Utility functions for message processing

## Build Commands

```bash
# Build the crate
cargo build -p tap-msg

# Run tests
cargo test -p tap-msg

# Run specific test
cargo test -p tap-msg test_name

# Run benchmarks
cargo bench -p tap-msg

# Run message benchmark
cargo bench --bench message_benchmark

# Build with WASM support
cargo build -p tap-msg --features wasm

# Test with examples
cargo test -p tap-msg --features examples
```

## Development Guidelines

### Message Implementation
- Always use `#[derive(TapMessage)]` from tap-msg-derive for new messages
- Implement the `Validation` trait for validatable message components
- Use typed structs over raw JSON for message bodies
- Include proper error handling with thiserror
- Add comprehensive doc comments with examples

### Testing
- Create test vectors for each message type in `tests/`
- Test both valid and invalid message scenarios
- Include round-trip serialization tests
- Test WASM compatibility when using `wasm` feature

### Message Types
All TAP messages should follow these patterns:
- Use snake_case for field names
- Include proper validation
- Support both JSON and binary serialization
- Be WASM-compatible when the feature is enabled
- Include appropriate threading support for multi-message flows

## Features

- `wasm` - Enables WebAssembly support with console error hooks and getrandom JS
- `examples` - Enables example code compilation

## Dependencies

Key dependencies include:
- `serde` for serialization
- `tap-caip` for CAIP identifier validation
- `tap-msg-derive` for message derive macros
- `chrono` for timestamp handling
- `uuid` for message IDs
- Optional WASM support with `wasm-bindgen`

## Examples

Message examples are available in `examples/` and can be run with:
```bash
cargo run --example message_example --features examples
```

## Testing

The crate includes comprehensive test suites:
- Unit tests for individual messages
- Integration tests for message flows
- Fuzz testing for robustness
- Test vector validation against TAIPs specifications

Run all tests with: `cargo test -p tap-msg`