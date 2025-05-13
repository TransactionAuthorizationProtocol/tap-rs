# TAP-RS User Guide

## Overview

The **Transaction Authorization Protocol (TAP)** is a decentralized off-chain protocol that allows multiple participants in a blockchain transaction to identify each other and collaboratively authorize or reject the transaction *before* on-chain settlement. 

TAP adds an **authorization layer** on top of the blockchain's settlement layer, enabling counterparties (originators and beneficiaries, and their service providers) to coordinate safely and privately without modifying on-chain mechanisms. This approach helps solve real-world challenges like regulatory compliance and fraud prevention while preserving the trustless nature of blockchain transactions.

`tap-rs` is the Rust implementation of TAP, providing developers with a way to create TAP agents and process TAP messages programmatically. It implements the TAP messaging flows using **DIDComm v2** – a secure messaging standard based on decentralized identifiers (DIDs). All TAP messages in `tap-rs` are DIDComm v2 compliant, meaning each message is a JSON envelope with standard fields (`id`, `type`, `from`, `to`, `body`, etc.) and can be cryptographically signed and encrypted as needed.

## Core Components

TAP-RS is organized as a workspace of Rust crates, each providing specific functionality:

- **tap-msg**: Core message types and processing
- **tap-agent**: Agent implementation with DID resolution and key management
- **tap-caip**: Chain Agnostic Identifier support (CAIP-2, CAIP-10, CAIP-19)
- **tap-node**: Node implementation for message routing
- **tap-http**: HTTP server for exposing a TAP agent endpoint
- **tap-wasm**: WebAssembly bindings for browser environments
- **tap-ts**: TypeScript wrapper for web developers

## Basic Usage Flow

The typical usage flow for TAP-RS involves:

1. **Setting up an Agent**: Creating a TAP agent with a DID and cryptographic keys
2. **Creating Messages**: Generating TAP messages like Transfer, Authorize, etc.
3. **Signing and Packing**: Cryptographically signing and optionally encrypting messages
4. **Sending**: Transmitting messages to counterparties (via HTTP or other transport)
5. **Receiving**: Processing incoming messages, verifying signatures
6. **Responding**: Generating appropriate response messages based on business logic

## DID Generation and Key Management

TAP-RS provides comprehensive tools for DID generation and key management:

### Using the Command-line CLI

The project includes a dedicated command-line tool for generating and managing DIDs:

```bash
# Generate a did:key with Ed25519 key type
cargo run --bin tap-agent-cli -- generate --method key --key-type ed25519 --output did-document.json --key-output private-key.json

# Generate a did:web for a specific domain
cargo run --bin tap-agent-cli -- generate --method web --domain example.com --output web-did.json
```

### Using the Rust API

```rust
use tap_agent::did::{DIDGenerationOptions, DIDKeyGenerator, KeyType};

// Create a generator
let generator = DIDKeyGenerator::new();

// Generate a DID using Ed25519 keys
let options = DIDGenerationOptions { key_type: KeyType::Ed25519 };
let generated_key = generator.generate_did(options)?;

println!("Generated DID: {}", generated_key.did);
```

### Using the TypeScript API

When working with the TypeScript bindings, you can auto-generate DIDs or create them explicitly:

```typescript
import { TAPAgent, DIDKeyType } from '@taprsvp/tap-agent';

// Create a new agent - an Ed25519 did:key will be automatically generated
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

## Example: Transfer Authorization Flow

Here's a simplified example of how two parties might coordinate a transfer:

1. **Originator** creates a Transfer message describing the intended transaction
2. **Originator's Agent** signs and sends the Transfer to the Beneficiary's Agent
3. **Beneficiary's Agent** verifies and processes the Transfer 
4. **Beneficiary's Agent** creates an Authorize message if the transfer is acceptable
5. **Originator's Agent** receives the Authorize, verifies it is valid
6. **Originator** submits the transaction on-chain
7. **Originator's Agent** sends a Settle message with the transaction details

## Key Concepts

### Message Types

TAP-RS supports all standard TAP message types:

- **Transfer**: Initiates a transaction request
- **Authorize**: Approves a transaction
- **Reject**: Declines a transaction
- **Settle**: Confirms on-chain settlement
- **Cancel**: Cancels an in-progress transaction
- **Presentation**: Provides requested credentials
- **PaymentRequest**: Requests payment with specified details
- **Invoice**: Detailed invoice structure with line items

See [tap-messages.md](./tap-messages.md) for a complete list of message types.

### DIDComm Integration

TAP uses DIDComm v2 for secure messaging, with features including:

- **Signed Messages**: Digital signatures provide authenticity
- **Encrypted Messages**: End-to-end encryption for confidentiality
- **Message Threading**: Messages are linked through thread IDs
- **Transport Independence**: Messages can be sent via HTTP, WebSockets, etc.

### Chain Agnostic Identifiers

TAP uses Chain Agnostic Identifiers (CAIP) for blockchain references:

- **CAIP-2 Chain ID**: Identifies blockchain networks (e.g., `eip155:1` for Ethereum mainnet)
- **CAIP-10 Account ID**: Identifies blockchain accounts (e.g., `eip155:1:0x123...`)
- **CAIP-19 Asset ID**: Identifies blockchain assets (e.g., `eip155:1/erc20:0xA0b...`)

## WebAssembly Integration

TAP-RS includes comprehensive WebAssembly support for integration in browser and Node.js environments:

### Browser Usage

```javascript
import init, { 
  init_tap_wasm, 
  Message, 
  TapAgent, 
  MessageType 
} from 'tap-wasm';

async function main() {
  // Initialize the WASM module
  await init();
  init_tap_wasm();

  // Create a new agent with auto-generated DID
  const agent = new TapAgent({
    nickname: "Test Agent",
    debug: true
  });
  
  // Generate and process TAP messages
  const message = new Message('msg_123', 'Transfer', '1.0');
  
  // Set the transfer message body
  message.set_transfer_body({
    asset: "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7",
    originator: {
      id: agent.get_did(),
      role: "originator"
    },
    beneficiary: {
      id: "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL",
      role: "beneficiary"
    },
    amount: "100.0",
    agents: [],
    memo: "Test transfer"
  });

  // Sign the message
  agent.sign_message(message);
}
```

See the [tap-wasm README](./tap-wasm/README.md) for more detailed information on WASM bindings.

## Key Features

- **Security**: End-to-end encrypted and signed messages
- **Interoperability**: Support for multiple blockchains and assets
- **Extensibility**: Flexible message structure with metadata support
- **Compliance**: Travel Rule support with secure information exchange
- **Cross-Platform**: Native Rust and WebAssembly for browser environments

## Build and Test Commands

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

## Further Reading

- For detailed API examples, see the individual crate README files
- [TAP Specification](https://tap.rsvp)
- [Transaction Authorization Protocol Improvement Proposals (TAIPs)](https://github.com/TransactionAuthorizationProtocol/TAIPs)
- [DIDComm Messaging Specification](https://identity.foundation/didcomm-messaging/spec/)
- [Chain Agnostic Improvement Proposals (CAIPs)](https://github.com/ChainAgnostic/CAIPs)