# TAP-RS: Transaction Authorization Protocol in Rust

This repository contains a Rust implementation of the Transaction Authorization Protocol (TAP), targeting payment-related use cases and Travel Rule messaging.

## Project Structure

TAP-RS is organized as a Rust workspace with multiple crates:

- **[tap-core](./tap-core/README.md)**: Core message processing for TAP
- **[tap-agent](./tap-agent/README.md)**: TAP agent functionality and identity management
- **caip**: Implementation of Chain Agnostic Identifier Standards
- **tap-node**: TAP node orchestration and message routing
- **tap-server**: HTTP DIDComm server implementation
- **tap-ts**: TypeScript/WASM wrapper for browser and Node.js environments

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

- DIDComm v2 integration for secure, encrypted messaging
- Support for all TAP message types
- Implementation of Chain Agnostic Standards (CAIP-2, CAIP-10, CAIP-19)
- Multiple DID method support (did:key, did:web, did:pkh)
- WASM compatibility for browser environments

## Getting Started with tap-core

```rust
use tap_core::message::{TapMessage, TapMessageType};
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

See the [tap-core README](./tap-core/README.md) for more detailed examples.

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

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## References

- [TAP Specification](https://tap.rsvp)
- [Transaction Authorization Protocol Improvement Proposals (TAIPs)](https://github.com/TransactionAuthorizationProtocol/TAIPs)
- [DIDComm Messaging Specification](https://identity.foundation/didcomm-messaging/spec/)
- [Chain Agnostic Improvement Proposals (CAIPs)](https://github.com/ChainAgnostic/CAIPs)
