# TAP-RS: Transaction Authorization Protocol in Rust

This repository contains a Rust implementation of the Transaction Authorization Protocol (TAP), targeting payment-related use cases and Travel Rule messaging.

## Project Structure

TAP-RS is organized as a Rust workspace with multiple crates:

- **[tap-msg](./tap-msg/README.md)**: Core message processing for TAP with integrated DIDComm support
- **[tap-agent](./tap-agent/README.md)**: TAP agent functionality and identity management
- **caip**: Implementation of Chain Agnostic Identifier Standards
- **tap-node**: TAP node orchestration and message routing
- **tap-server**: HTTP DIDComm server implementation
- **[tap-wasm](./tap-wasm/README.md)**: WebAssembly bindings with DIDComm SecretsResolver integration
- **[tap-ts](./tap-ts/README.md)**: TypeScript/WASM wrapper for browser and Node.js environments

## Development Status

This project is under active development. See the [PRD](./prds/v1.md) for the complete roadmap and status.

## Getting Started

### Prerequisites

- Rust 1.71.0 or later
- Cargo

### Building

```bash
# Clone the repository
git clone https://github.com/notabene/tap-rs.git
cd tap-rs

# Build all crates
cargo build

# Run tests
cargo test
```

## Features

- Direct DIDComm v2 integration in TAP message types for secure, encrypted messaging
- Support for all TAP message types
- Implementation of Chain Agnostic Standards (CAIP-2, CAIP-10, CAIP-19)
- Multiple DID method support (did:key, did:web, did:pkh)
- WASM compatibility for browser environments
- Secure key management with DIDComm SecretsResolver implementation
- Message signing and verification with Ed25519 and other key types

## Getting Started with tap-msg

```rust
use tap_msg::message::{TapMessage, TapMessageType};
use serde_json::json;

// Create a message using builder pattern
let message = TapMessage::new()
    .with_message_type(TapMessageType::TransactionProposal)
    .with_body(json!({
        "transaction": {
            "amount": "100.00",
            "currency": "USD"
        }
    }))
    .build();
```

See the [tap-msg README](./tap-msg/README.md) for more detailed examples.

## Getting Started with tap-agent

```rust
use tap_agent::{Agent, AgentConfig, TapAgent};

// Configure and create an agent
let config = AgentConfig::new()
    .with_did("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
    .with_name("My TAP Agent");

let agent = TapAgent::with_defaults(
    config,
    "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
    Some("My TAP Agent".to_string()),
).unwrap();
```

See the [tap-agent README](./tap-agent/README.md) for more detailed examples.

## Documentation

Comprehensive documentation for TAP-RS is available in the [docs](./docs) directory:

### Tutorials
- [Getting Started](./docs/tutorials/getting_started.md) - Learn how to set up and start using TAP-RS
- [Implementing TAP Flows](./docs/tutorials/implementing_tap_flows.md) - Guide to implementing various TAP message flows
- [Security Best Practices](./docs/tutorials/security_best_practices.md) - Guidelines for securing your implementation
- [WASM Integration](./docs/tutorials/wasm_integration.md) - Using TAP-RS in browser and Node.js environments

### API Reference
- [API Documentation](./docs/api/index.md) - Complete API reference for all TAP-RS crates

### Examples
- [Complete Transfer Flow](./docs/examples/complete_transfer_flow.md) - End-to-end example integrating multiple TAP-RS components

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## References

- [TAP Specification](https://tap.rsvp)
- [Transaction Authorization Protocol Improvement Proposals (TAIPs)](https://github.com/TransactionAuthorizationProtocol/TAIPs)
- [DIDComm Messaging Specification](https://identity.foundation/didcomm-messaging/spec/)
- [Chain Agnostic Improvement Proposals (CAIPs)](https://github.com/ChainAgnostic/CAIPs)
