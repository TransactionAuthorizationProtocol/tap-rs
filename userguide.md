# TAP-RS User Guide

## Introduction

The **Transaction Authorization Protocol (TAP)** is a decentralized off-chain protocol that allows multiple participants in a blockchain transaction to identify each other and collaboratively authorize or reject the transaction *before* on-chain settlement.

This user guide provides a conceptual overview of the TAP-RS implementation, explaining its architecture, key concepts, and usage patterns. For detailed implementation tutorials, refer to the [docs/tutorials](./docs/tutorials) directory.

## What Problem Does TAP Solve?

Blockchain transactions traditionally follow a "commit first, validate later" approach, where:

1. A transaction is submitted to the blockchain
2. The transaction is confirmed (settled) on-chain
3. Recipients discover the transaction after it's finalized

This approach creates several challenges:

- **No Pre-validation**: Recipients can't approve/reject transactions before they're settled
- **Compliance Challenges**: Travel Rule and KYC/AML requirements are difficult to satisfy
- **Error Risk**: Sending to incorrect addresses or with incorrect parameters is irreversible
- **Limited Context**: Transaction metadata and purpose can't be communicated effectively

TAP adds an **authorization layer** on top of the blockchain's settlement layer, enabling counterparties (originators and beneficiaries, and their service providers) to coordinate safely and privately without modifying on-chain mechanisms.

## Architecture Overview

`tap-rs` is the Rust implementation of TAP, providing developers with a way to create TAP agents and process TAP messages programmatically.

### Core Components

The TAP-RS architecture consists of these key components:

![TAP-RS Architecture](https://mermaid.ink/img/pako:eNptkc9OwzAMxl8l8gVG2Q49bJqENHECiQvXKJicZv0DcVJlLVLfd4JSNgSHKvb3_fyS2UMjpcIcGs0sLgq5Z-R1aTlV7dY-KGbdQytFI5UD1QnWQN4VcnlXdA5_7MSPFsJ_aP6Cxgb0UiJoaOVL5c8xOD5C8GE_FI8gT2nwNLAj9lGYSDj4T5SdqL5aXIyxdZkMmBkUkmJ8N9tBbZ9jijYAYZxhxKDHjUQ2VIX0IXppjY_x42bUcOUTlz7PKiJ9RPlK-r3CeOWzq3NIHTHZZx2xj6Q_lNlAjKeUDVEVLvSQoZaMW4yMpZcZrKGzFnPYwjvCw9RM4Jmt8IZeDz2KaUcH2Qlu-VdxyI-JD25Qyxzqn8QlbB4?type=png)

1. **Messages (tap-msg)**:
   - Core message types (Transfer, Authorize, etc.)
   - Message validation and serialization
   - DIDComm integration for secure messaging

2. **Agents (tap-agent)**:
   - Identity management with DIDs
   - Cryptographic operations (signing, encryption)
   - Message handling and routing

3. **Identifiers (tap-caip)**:
   - Chain Agnostic Identifiers for assets and accounts
   - Cross-chain compatibility

4. **Node (tap-node)**:
   - Message routing and event handling
   - Connection management
   - API endpoints

5. **Transport (tap-http)**:
   - HTTP server for DIDComm messaging
   - Endpoint management

6. **Cross-Platform Support**:
   - **tap-wasm**: WebAssembly bindings
   - **tap-ts**: TypeScript wrapper

### Message Flow

TAP messages flow between participants in a structured way:

1. **Originator** sends a Transfer or Payment message to **Beneficiary**
2. **Beneficiary** reviews and responds with Authorize or Reject
3. **Originator** (if authorized) performs the on-chain transaction
4. **Originator** sends a Settle message confirming settlement
5. Optional additional messages (Presentation for compliance, etc.)

## Key Concepts

### Decentralized Identifiers (DIDs)

TAP uses DIDs to identify participants, ensuring secure and verifiable identities:

- **did:key**: Simple cryptographic identifiers (e.g., `did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK`)
- **did:web**: Domain-based identifiers (e.g., `did:web:example.com`)
- **did:pkh**: Blockchain address-based identifiers (e.g., `did:pkh:eip155:1:0x123...`)

### DIDComm Messaging

All TAP messages use DIDComm v2 for secure transport:

- **Signed Messages**: Guarantee message authenticity
- **Encrypted Messages**: Provide confidentiality with authentication (authcrypt) or anonymity (anoncrypt)
- **Standardized Format**: JSON-based envelope structure

### Chain Agnostic Identifiers

TAP uses the CAIP standards for cross-chain compatibility:

- **CAIP-2 Chain ID**: `eip155:1` (Ethereum mainnet)
- **CAIP-10 Account ID**: `eip155:1:0x123...` (Ethereum account)
- **CAIP-19 Asset ID**: `eip155:1/erc20:0xA0b...` (ERC-20 token)

### Message Types

TAP defines a comprehensive set of message types for different scenarios:

| Message Type | Purpose |
|--------------|---------|
| Transfer | Initiate a transfer request |
| Authorize | Approve a transfer |
| Reject | Decline a transfer |
| Settle | Confirm on-chain settlement |
| Cancel | Cancel a pending transfer |
| Presentation | Provide compliance information |
| Payment | Request a payment |
| Invoice | Detailed payment information |
| Connect | Establish a relationship |

For a complete list, see [tap-messages.md](./tap-messages.md).

## Use Cases

### Travel Rule Compliance

Financial institutions can use TAP to:
- Exchange required beneficiary/originator information
- Document compliance checks
- Maintain auditable records of authorization

### Wallet Security

Self-custody wallets can integrate TAP to:
- Verify recipients before sending funds
- Receive transfer context and metadata
- Implement approval workflows for high-value transfers

### Multi-Party Authorization

Complex flows requiring multiple approvals:
- Multi-sig wallet coordination
- Corporate treasury management
- Regulated transaction approval

### Cross-Chain Transactions

TAP's chain-agnostic approach enables:
- Coordinating transfers across different blockchains
- Maintaining consistent messaging regardless of underlying chain
- Implementing cross-chain compliance

## How It Works: A Practical Overview

### Basic Components in Your Application

When integrating TAP-RS, you'll work with these components:

1. **Agent**: Your identity in the TAP network
2. **Messages**: Structured data you send/receive
3. **Node** (optional): For routing messages to multiple agents

### Typical Implementation Flow

```
┌─────────────────────┐    ┌────────────────────┐    ┌─────────────────────┐
│ 1. Set Up Agent     │───►│ 2. Create Messages │───►│ 3. Process Messages │
└─────────────────────┘    └────────────────────┘    └─────────────────────┘
       │                           ▲                          │
       │                           │                          │
       └───────────────────────────┴──────────────────────────┘
                        Message Exchange
```

#### 1. Set Up Agent

Create a TAP agent with a DID and cryptographic keys:
- Generate a DID or use existing one
- Configure resolvers and key management
- Set up communication endpoints

#### 2. Create Messages

Generate TAP messages based on your needs:
- Construct the message body (Transfer, etc.)
- Set appropriate metadata
- Sign and optionally encrypt

#### 3. Process Messages

Handle incoming messages:
- Verify signatures and decrypt
- Validate message content
- Generate appropriate responses
- Update internal state

For detailed implementation steps, see the [Getting Started tutorial](./docs/tutorials/getting_started.md).

## Security Considerations

When implementing TAP, consider these security aspects:

1. **Key Management**: Properly secure private keys
2. **Message Validation**: Thoroughly validate all incoming messages
3. **Transport Security**: Use TLS for HTTP connections
4. **Authorization**: Verify that senders are authorized to send specific message types
5. **Replay Protection**: Implement nonce handling or timestamps

For detailed security guidelines, see the [Security Best Practices](./docs/tutorials/security_best_practices.md) tutorial.

## Configuration Options

TAP-RS provides various configuration options:

- **Agent Configuration**: DID settings, key management, etc.
- **Node Configuration**: Routing, processors, etc.
- **HTTP Server Configuration**: Endpoints, TLS, etc.

Example of agent configuration:

```rust
let config = AgentConfig::new(did)
    .with_default_endpoint("https://example.com/didcomm")
    .with_debug(true);
```

## Cross-Platform Support

TAP-RS includes WebAssembly support for integration in browser and Node.js environments through the `tap-wasm` crate and TypeScript bindings in the `tap-ts` package.

For details on the WASM integration, see the [tap-wasm README](./tap-wasm/README.md) and [tap-ts README](./tap-ts/README.md).

## API Documentation Structure

The TAP-RS API documentation is organized as follows:

- **Crate READMEs**: Each crate has its own README with API details
- **Code Documentation**: Generated from doc comments in the code
- **Implementation Tutorials**: In the [docs/tutorials](./docs/tutorials) directory

## Next Steps

Now that you understand the TAP-RS concepts, explore these resources:

- [Getting Started Tutorial](./docs/tutorials/getting_started.md) - Step-by-step guide to your first TAP implementation
- [Implementing TAP Flows](./docs/tutorials/implementing_tap_flows.md) - Guide to common message flows
- [Security Best Practices](./docs/tutorials/security_best_practices.md) - Secure your TAP implementation
- [Complete Transfer Flow Example](./docs/examples/complete_transfer_flow.md) - End-to-end example 

For questions or support, please open an issue on the [TAP-RS GitHub repository](https://github.com/notabene/tap-rs).

## References

- [TAP Specification](https://tap.rsvp)
- [Transaction Authorization Protocol Improvement Proposals (TAIPs)](https://github.com/TransactionAuthorizationProtocol/TAIPs)
- [DIDComm Messaging Specification](https://identity.foundation/didcomm-messaging/spec/)
- [Chain Agnostic Improvement Proposals (CAIPs)](https://github.com/ChainAgnostic/CAIPs)