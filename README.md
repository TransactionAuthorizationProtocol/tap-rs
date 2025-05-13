# TAP-RS: Transaction Authorization Protocol in Rust

This repository contains a Rust implementation of the Transaction Authorization Protocol (TAP), a decentralized protocol for securely authorizing blockchain transactions before they are submitted on-chain. TAP-RS targets payment-related use cases, Travel Rule compliance, and secure transaction coordination.

## Project Structure

TAP-RS is organized as a Rust workspace with multiple crates:

- **[tap-msg](./tap-msg/README.md)**: Core message processing for TAP with integrated DIDComm support
- **[tap-agent](./tap-agent/README.md)**: TAP agent functionality and identity management
- **[tap-caip](./tap-caip/README.md)**: Implementation of Chain Agnostic Identifier Standards
- **[tap-node](./tap-node/README.md)**: TAP node orchestration and message routing
- **[tap-http](./tap-http/README.md)**: HTTP DIDComm server implementation
- **[tap-wasm](./tap-wasm/README.md)**: WebAssembly bindings with DIDComm SecretsResolver integration
- **[tap-ts](./tap-ts/README.md)**: TypeScript/WASM wrapper for browser and Node.js environments

## Overview

The Transaction Authorization Protocol (TAP) adds a secure authorization layer to blockchain transactions, enabling participants to:

- Verify transaction details before settlement
- Exchange required compliance information privately
- Prevent sending to wrong addresses or incorrect amounts
- Implement multi-party authorization workflows
- Conduct Travel Rule compliance checks off-chain

TAP-RS implements this protocol with a focus on:

- **Security**: End-to-end encrypted messaging via DIDComm v2
- **Interoperability**: Support for multiple blockchains through CAIP standards
- **Extensibility**: Modular design allowing custom integrations
- **Cross-Platform**: Native support and WebAssembly for browser environments

## Development Status

This project has successfully implemented all core TAP message types and flows as specified in the TAIPs (Transaction Authorization Protocol Improvement Proposals). The codebase is feature-complete for standard TAP use cases.

## Development Guide

### Dependencies

This project has specific dependency version requirements:

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

## Key Features

- **Complete TAP Implementation**: Support for all TAP message types (Transfer, Authorize, Reject, Settle, etc.)
- **DIDComm v2 Integration**: Secure, encrypted messaging with authenticated signatures
- **Chain Agnostic Identifiers**: Implementation of CAIP-2 (ChainID), CAIP-10 (AccountID), and CAIP-19 (AssetID)
- **Multiple DID Methods**: Support for did:key, did:web, did:pkh, and more
- **Command-line Tools**: Utilities for DID generation and key management
- **Modular Agent Architecture**: Flexible identity and cryptography primitives
- **High-Performance Message Routing**: Efficient node implementation for high-throughput environments
- **HTTP and WebSocket Transport**: Multiple communication options with robust error handling
- **WASM Compatibility**: Run in browsers and Node.js via WebAssembly
- **TypeScript API**: Developer-friendly TypeScript wrapper for web integrations
- **Comprehensive Validation**: All messages validated against TAP specifications

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

## DID Generation and Key Management

TAP-RS provides comprehensive tools for DID generation and key management:

### Using the Command-line CLI

```bash
# Generate a did:key with Ed25519 key type
cargo run --bin tap-agent-cli -- generate --method key --key-type ed25519 --output did-document.json --key-output private-key.json

# Generate a did:web for a specific domain
cargo run --bin tap-agent-cli -- generate --method web --domain example.com --output web-did.json
```

### Using the TypeScript API

```typescript
import { TAPAgent, DIDKeyType } from '@taprsvp/tap-agent';

// Create a new agent with auto-generated did:key
const agent = new TAPAgent({
  nickname: "My Agent",
  debug: true
});

// Generate a did:key with specific key type
const edDID = await agent.generateDID(DIDKeyType.Ed25519);
console.log(`Generated DID: ${edDID.did}`);

// Generate a did:web
const webDID = await agent.generateWebDID('example.com', DIDKeyType.P256);
console.log(`Web DID: ${webDID.did}`);
```

For more details, see the [DID Generation Documentation](./tap-ts/DID-GENERATION.md).

## Common Use Cases

- **VASP-to-VASP Transfers**: Exchanges and custodians can coordinate transfers with Travel Rule compliance
- **Self-Custody Verification**: Wallets can verify transaction details before settlement
- **Multi-Party Authorization**: Complex transfers requiring approval from multiple entities
- **Cross-Chain Coordination**: Consistent messaging across different blockchain networks
- **Compliance Automation**: Streamline compliance workflows with secure messaging

## Documentation

Comprehensive documentation for TAP-RS is available in the [docs](./docs) directory:

### Tutorials
- [Getting Started](./docs/tutorials/getting_started.md) - Learn how to set up and start using TAP-RS
- [Implementing TAP Flows](./docs/tutorials/implementing_tap_flows.md) - Guide to implementing various TAP message flows
- [Security Best Practices](./docs/tutorials/security_best_practices.md) - Guidelines for securing your implementation
- [WASM Integration](./docs/tutorials/wasm_integration.md) - Using TAP-RS in browser and Node.js environments

### Examples
- [Complete Transfer Flow](./docs/examples/complete_transfer_flow.md) - End-to-end example integrating multiple TAP-RS components

## Build Commands

The following commands are available for working with the codebase:

```bash
# Build all crates
cargo build

# Run tests for all crates
cargo test

# Run tests for a specific crate
cargo test --package tap-msg

# Run benchmarks
cargo bench

# Format code
cargo fmt

# Lint code
cargo clippy
```

## License

This project is licensed under the MIT License - see the [LICENSE-MIT](./LICENSE-MIT) file for details.

## References

- [TAP Specification](https://tap.rsvp)
- [Transaction Authorization Protocol Improvement Proposals (TAIPs)](https://github.com/TransactionAuthorizationProtocol/TAIPs)
- [DIDComm Messaging Specification](https://identity.foundation/didcomm-messaging/spec/)
- [Chain Agnostic Improvement Proposals (CAIPs)](https://github.com/ChainAgnostic/CAIPs)