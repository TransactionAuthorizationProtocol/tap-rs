# TAP-RS: Transaction Authorization Protocol in Rust

This repository contains a Rust implementation of the Transaction Authorization Protocol (TAP), targeting payment-related use cases and Travel Rule messaging.

## Project Structure

TAP-RS is organized as a Rust workspace with multiple crates:

- **[tap-msg](./tap-msg/README.md)**: Core message processing for TAP with integrated DIDComm support
- **[tap-agent](./tap-agent/README.md)**: TAP agent functionality and identity management
- **[tap-caip](./tap-caip/README.md)**: Implementation of Chain Agnostic Identifier Standards
- **[tap-node](./tap-node/README.md)**: TAP node orchestration and message routing
- **[tap-http](./tap-http/README.md)**: HTTP DIDComm server implementation
- **[tap-wasm](./tap-wasm/README.md)**: WebAssembly bindings with DIDComm SecretsResolver integration
- **[tap-ts](./tap-ts/README.md)**: TypeScript/WASM wrapper for browser and Node.js environments

## Development Status

This project has successfully implemented all items from the [PRD](./prds/v1.md). The codebase is feature-complete as per the initial requirements.

## Development Guide

### Dependencies

This project has some specific dependency version requirements:

- **UUID v0.8.2**: Required for compatibility with the didcomm crate. Do not upgrade! See [DEPENDENCIES.md](./DEPENDENCIES.md) for details.
- **WASM Support**: Several dependencies require special features for WebAssembly compatibility.

Please review the [DEPENDENCIES.md](./DEPENDENCIES.md) file before updating any dependencies or adding new crates to the workspace.

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
- Support for all TAP message types with proper validation
- Implementation of Chain Agnostic Standards (CAIP-2, CAIP-10, CAIP-19)
- Multiple DID method support (did:key, did:web, did:pkh)
- Participant-based message flows (replacing Agent terminology in some contexts)
- WASM compatibility for browser environments
- Secure key management with DIDComm SecretsResolver implementation
- Message signing and verification with Ed25519 and other key types
- Proper Ed25519 to X25519 key conversion for encryption

## Getting Started with tap-msg

```rust
use tap_msg::message::types::{Transfer, Participant};
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_caip::AssetId;
use std::collections::HashMap;
use std::str::FromStr;

// Create a Transfer message body
let asset = AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7")?;

let originator = Participant {
    id: "did:example:sender".to_string(),
    role: Some("originator".to_string()),
};

let beneficiary = Participant {
    id: "did:example:receiver".to_string(),
    role: Some("beneficiary".to_string()),
};

let transfer = Transfer {
    asset,
    originator,
    beneficiary: Some(beneficiary),
    amount: "100.0".to_string(),
    agents: vec![],
    settlement_id: None,
    memo: Some("Test transfer".to_string()),
    metadata: HashMap::new(),
};

// Create a DIDComm message from the transfer
let message = transfer.to_didcomm_with_route(
    Some("did:example:sender"), 
    ["did:example:receiver"].iter().copied()
)?;
```

See the [tap-msg README](./tap-msg/README.md) for more detailed examples.

## Getting Started with tap-agent

```rust
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{DefaultMessagePacker, BasicSecretResolver};
use tap_agent::did::DefaultDIDResolver;
use std::sync::Arc;

// Configure the agent
let config = AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());

// Set up components
let did_resolver = Arc::new(DefaultDIDResolver::new());
let secret_resolver = Arc::new(BasicSecretResolver::new());
let message_packer = Arc::new(DefaultMessagePacker::new(did_resolver, secret_resolver));

// Create the agent
let agent = DefaultAgent::new(config, message_packer);
```

See the [tap-agent README](./tap-agent/README.md) for more detailed examples.

## Documentation

Comprehensive documentation for TAP-RS is available in the [docs](./docs) directory:

### Tutorials
- [Getting Started](./docs/tutorials/getting_started.md) - Learn how to set up and start using TAP-RS
- [Implementing TAP Flows](./docs/tutorials/implementing_tap_flows.md) - Guide to implementing various TAP message flows
- [Security Best Practices](./docs/tutorials/security_best_practices.md) - Guidelines for securing your implementation
- [WASM Integration](./docs/tutorials/wasm_integration.md) - Using TAP-RS in browser and Node.js environments

### Examples
- [Complete Transfer Flow](./docs/examples/complete_transfer_flow.md) - End-to-end example integrating multiple TAP-RS components

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## References

- [TAP Specification](https://tap.rsvp)
- [Transaction Authorization Protocol Improvement Proposals (TAIPs)](https://github.com/TransactionAuthorizationProtocol/TAIPs)
- [DIDComm Messaging Specification](https://identity.foundation/didcomm-messaging/spec/)
- [Chain Agnostic Improvement Proposals (CAIPs)](https://github.com/ChainAgnostic/CAIPs)
