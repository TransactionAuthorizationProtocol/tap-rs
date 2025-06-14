# TAP-RS: Transaction Authorization Protocol in Rust

This repository contains a Rust implementation of the Transaction Authorization Protocol (TAP), a decentralized protocol for securely authorizing blockchain transactions before they are submitted on-chain. TAP-RS targets payment-related use cases, Travel Rule compliance, and secure transaction coordination.

**New in this release**: Full Travel Rule support with IVMS101 data model implementation, automatic customer data extraction, and compliance workflow automation.

## Project Structure

TAP-RS is organized as a Rust workspace with multiple crates:

- **[tap-msg](./tap-msg/README.md)**: Core message processing for TAP with integrated DIDComm support
- **[tap-msg-derive](./tap-msg-derive/README.md)**: Procedural derive macro for automatic TAP message trait implementation
- **[tap-agent](./tap-agent/README.md)**: TAP agent functionality and identity management
- **[tap-caip](./tap-caip/README.md)**: Implementation of Chain Agnostic Identifier Standards
- **[tap-ivms101](./tap-ivms101/README.md)**: Complete IVMS 101.2023 implementation for Travel Rule compliance
- **[tap-node](./tap-node/README.md)**: TAP node orchestration with per-agent storage isolation, multi-recipient message delivery, and Travel Rule processor
- **[tap-http](./tap-http/README.md)**: HTTP DIDComm server implementation
- **[tap-wasm](./tap-wasm/README.md)**: WebAssembly bindings with DIDComm SecretsResolver integration
- **[tap-ts](./tap-ts/README.md)**: TypeScript/WASM wrapper for browser and Node.js environments
- **[tap-mcp](./tap-mcp/README.md)**: Model Context Protocol server for AI/LLM integration

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
- **Scalability**: Per-agent storage isolation with multi-recipient message delivery
- **Compliance**: Full DIDComm specification adherence for multi-party transactions with IVMS101 Travel Rule support
- **Extensibility**: Modular design allowing custom integrations
- **Cross-Platform**: Native support and WebAssembly for browser environments
- **Travel Rule**: Automatic IVMS101 data generation and attachment for regulatory compliance

## Development Status

This project has successfully implemented all core TAP message types and flows as specified in the TAIPs (Transaction Authorization Protocol Improvement Proposals). The codebase is feature-complete for standard TAP use cases.

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

### Installing Command-line Tools

TAP-RS includes several useful command-line tools that can be installed from crates.io or from source:

```bash
# Install tools from crates.io
cargo install tap-agent tap-http

# Or install from the repository
cargo install --path tap-rs/tap-agent
cargo install --path tap-rs/tap-http
```

Available command-line tools:

1. **tap-agent-cli**: Manage DIDs and keys for TAP protocol
   ```bash
   # Generate a new did:key with Ed25519
   tap-agent-cli generate

   # List stored keys
   tap-agent-cli keys list

   # Pack a plaintext DIDComm message (supports signed, authcrypt, and anoncrypt modes)
   tap-agent-cli pack --input message.json --output packed.json --mode signed
   tap-agent-cli pack --input message.json --output packed.json --mode authcrypt --recipient did:key:z6Mk...
   tap-agent-cli pack --input message.json --output packed.json --mode anoncrypt --recipient did:key:z6Mk...

   # Unpack a signed or encrypted DIDComm message
   tap-agent-cli unpack --input packed.json --output unpacked.json

   # Pack/unpack with specific key selection
   tap-agent-cli pack --input message.json --output packed.json --mode signed --key did:key:z6Mk...
   tap-agent-cli unpack --input packed.json --output unpacked.json --key did:key:z6Mk...
   ```

2. **tap-http**: Run a TAP HTTP server for DIDComm messaging
   ```bash
   # Start a server with default settings
   tap-http
   ```

3. **tap-payment-simulator**: Test TAP payment flows against a server
   ```bash
   # Send a test payment flow to a server
   tap-payment-simulator --url http://localhost:8000/didcomm --did <server-agent-did>
   ```

See individual tool READMEs for detailed usage instructions.

## Key Features

- **Complete TAP Implementation**: Support for all TAP message types (Transfer, Authorize, Reject, Settle, Complete, etc.)
- **DIDComm v2 Integration**: Secure, encrypted messaging with authenticated signatures
- **Chain Agnostic Identifiers**: Implementation of CAIP-2 (ChainID), CAIP-10 (AccountID), and CAIP-19 (AssetID)
- **Multiple DID Methods**: Support for did:key, did:web, did:pkh, and more
- **Travel Rule Compliance**: Full IVMS 101.2023 implementation with automatic data attachment
- **Customer Data Management**: Automatic extraction and storage of party information from TAP messages
- **Command-line Tools**: Utilities for DID generation, resolution, and key management
- **Modular Agent Architecture**: Flexible identity and cryptography primitives
- **High-Performance Message Routing**: Efficient node implementation for high-throughput environments
- **HTTP and WebSocket Transport**: Multiple communication options with robust error handling
- **WASM Compatibility**: Run in browsers and Node.js via WebAssembly
- **TypeScript API**: Developer-friendly TypeScript wrapper for web integrations
- **Comprehensive Validation**: All messages validated against TAP specifications
- **Generic Typed Messages**: Compile-time type safety with `PlainMessage<Transfer>` while maintaining backward compatibility
- **Derive Macro**: Automatic implementation of `TapMessage` and `MessageContext` traits with `#[derive(TapMessage)]`
- **Persistent Storage**: SQLite-based storage with automatic migrations providing:
  - Transaction tracking for Transfer and Payment messages
  - Complete audit trail of all messages for compliance and debugging
  - **Message delivery tracking** with status monitoring, retry counts, and error logging
  - **Customer profiles** with Schema.org JSON-LD format and IVMS101 caching

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

## Typed Messages for Type Safety

TAP-RS now supports generic typed messages for compile-time type safety while maintaining 100% backward compatibility:

```rust
use tap_msg::{PlainMessage, Transfer, Participant};
use tap_agent::{Agent, TapAgent};

// Create a strongly-typed message
let transfer = Transfer {
    asset: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".parse()?,
    originator: Participant::new("did:example:alice"),
    beneficiary: Some(Participant::new("did:example:bob")),
    amount: "100".to_string(),
    // ... other fields
};

// Type-safe message construction
let typed_msg = PlainMessage::new_typed(transfer, "did:example:alice")
    .with_recipient("did:example:bob")
    .with_thread_id(Some("payment-123".to_string()));

// Send with compile-time type checking
let (packed, results) = agent.send_typed(typed_msg, true).await?;

// Receive with type safety
let received: PlainMessage<Transfer> = agent.receive_typed(&packed).await?;
println!("Amount: {}", received.body.amount);

// Backward compatibility - existing code unchanged
let plain_msg: PlainMessage = serde_json::from_str(json_data)?;
// This is now PlainMessage<Value> due to default type parameter
```

See [GENERIC_PLAINMESSAGE.md](./GENERIC_PLAINMESSAGE.md) for complete documentation.

## Derive Macro for TAP Messages

TAP-RS provides a procedural derive macro that automatically implements `TapMessage` and `MessageContext` traits:

```rust
use tap_msg::TapMessage;
use tap_msg::message::{Participant, TapMessageBody};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
pub struct CustomMessage {
    #[tap(participant)]
    pub sender: Participant,

    #[tap(participant)]
    pub receiver: Option<Participant>,

    #[tap(participant_list)]
    pub validators: Vec<Participant>,

    #[tap(transaction_id)]
    pub transaction_id: String,

    pub data: String,
}

// The macro automatically provides:
// - thread_id() -> transaction_id
// - get_all_participants() -> extracts all participant DIDs
// - participants() -> returns &Participant references
// - transaction_context() -> creates context with ID and type
```

See the [tap-msg README](./tap-msg/README.md#derive-macro-for-tap-messages) for detailed documentation.

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
# Install the tap-agent CLI
cargo install tap-agent

# Generate a did:key with Ed25519 key type
tap-agent-cli generate --method key --key-type ed25519 --output did-document.json --key-output private-key.json

# Generate a did:web for a specific domain
tap-agent-cli generate --method web --domain example.com --output web-did.json

# Look up and resolve a DID to its DID Document
tap-agent-cli lookup did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

# Look up a DID and save the result to a file
tap-agent-cli lookup did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK --output did-document.json

# Key management operations
tap-agent-cli keys list
tap-agent-cli keys view did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
tap-agent-cli keys set-default did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
```

For TypeScript and WebAssembly bindings, see the [tap-ts README](./tap-ts/README.md).

## Common Use Cases

- **VASP-to-VASP Transfers**: Exchanges and custodians can coordinate transfers with automatic Travel Rule compliance
- **Travel Rule Compliance**: Automatic IVMS101 data generation and exchange for regulatory requirements
- **Self-Custody Verification**: Wallets can verify transaction details before settlement
- **Multi-Party Authorization**: Complex transfers requiring approval from multiple entities
- **Cross-Chain Coordination**: Consistent messaging across different blockchain networks
- **Compliance Automation**: Streamline compliance workflows with secure messaging and automatic data extraction
- **Customer Data Management**: Automatic extraction and storage of party information for KYC/AML

## Documentation

Comprehensive documentation for TAP-RS is available in the [docs](./docs) directory:

### Tutorials
- [Getting Started](./docs/tutorials/getting_started.md) - Learn how to set up and start using TAP-RS
- [Implementing TAP Flows](./docs/tutorials/implementing_tap_flows.md) - Guide to implementing various TAP message flows
- [Security Best Practices](./docs/tutorials/security_best_practices.md) - Guidelines for securing your implementation
- [WASM Integration](./docs/tutorials/wasm_integration.md) - Using TAP-RS in browser and Node.js environments

### Travel Rule Documentation
- [Travel Rule Implementation](./tap-node/TRAVEL-RULE.md) - Complete guide to Travel Rule compliance in TAP Node
- [Customer Management](./tap-node/CUSTOMER-MANAGEMENT.md) - Automatic customer data extraction and management
- [IVMS101 Data Model](./tap-ivms101/README.md) - Full IVMS 101.2023 implementation

### Examples
- [Complete Transfer Flow](./docs/examples/complete_transfer_flow.md) - End-to-end example integrating multiple TAP-RS components
- [Travel Rule Flow](./tap-node/examples/travel_rule_flow.rs) - Complete example of Travel Rule compliance with IVMS101

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

# Install command-line tools
cargo install --path tap-agent
cargo install --path tap-http
```

## CLI Tools Reference

### DIDComm Message Packing and Unpacking

The `tap-agent-cli` tool provides commands for packing and unpacking DIDComm messages:

```bash
# Install the tap-agent CLI
cargo install tap-agent

# Pack a plaintext message to a signed DIDComm message
tap-agent-cli pack --input message.json --output packed.json --mode signed

# Pack using authenticated encryption (requires recipient DID)
tap-agent-cli pack --input message.json --output packed.json --mode authcrypt --recipient did:key:z6Mk...

# Pack using anonymous encryption (requires recipient DID)
tap-agent-cli pack --input message.json --output packed.json --mode anoncrypt --recipient did:key:z6Mk...

# Use a specific key for packing (otherwise the default key is used)
tap-agent-cli pack --input message.json --output packed.json --mode signed --key did:key:z6Mk...

# Unpack a DIDComm message (works with signed, authcrypt, or anoncrypt messages)
tap-agent-cli unpack --input packed.json --output unpacked.json

# Unpack using a specific key (otherwise all available keys are tried)
tap-agent-cli unpack --input packed.json --output unpacked.json --key did:key:z6Mk...
```

The input message.json should be a plain JSON object following the DIDComm message format:

```json
{
  "id": "1234567890",
  "type": "https://tap.rsvp/schema/1.0#Transfer",
  "from": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "to": ["did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL"],
  "body": {
    "asset": "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7",
    "originator": {
      "@id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "role": "originator"
    },
    "beneficiary": {
      "@id": "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL",
      "role": "beneficiary"
    },
    "amount": "100.0",
    "agents": []
  }
}
```

## Running TAP HTTP Server

The TAP HTTP server provides a DIDComm messaging endpoint for the TAP protocol:

```bash
# Install the server
cargo install tap-http

# Run with default settings (creates ephemeral agent)
tap-http

# Run with custom settings
tap-http --host 0.0.0.0 --port 8080 --endpoint /didcomm --logs-dir /var/log/tap

# Use a stored key from tap-agent-cli
tap-http --use-stored-key
```

You can test the server using the payment simulator:

```bash
# Install the simulator
cargo install tap-http

# Run a test payment flow (using the DID printed when starting the server)
tap-payment-simulator --url http://localhost:8000/didcomm --did did:key:z6Mk...
```

## License

This project is licensed under the MIT License - see the [LICENSE-MIT](./LICENSE-MIT) file for details.

## References

- [TAP Specification](https://tap.rsvp)
- [Transaction Authorization Protocol Improvement Proposals (TAIPs)](https://github.com/TransactionAuthorizationProtocol/TAIPs)
- [DIDComm Messaging Specification](https://identity.foundation/didcomm-messaging/spec/)
- [Chain Agnostic Improvement Proposals (CAIPs)](https://github.com/ChainAgnostic/CAIPs)
