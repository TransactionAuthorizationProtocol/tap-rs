# TAP Agent

This crate implements the agent functionality for the Transaction Authorization Protocol (TAP).

## Features

- **Agent Identity Management**: Create and manage agent identities using DIDs
- **Message Processing**: Handle TAP message flows with proper validation
- **DID Resolution**: Resolve DIDs for message routing and key discovery
- **Cryptographic Operations**: Sign, verify, encrypt, and decrypt messages
- **Key Management**: Securely manage cryptographic keys
- **Asynchronous Processing**: Process messages concurrently using Tokio

## Usage

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

// Process an incoming message
let result = agent.process_message("incoming_message").await;
```

## Components

### Agent

The `Agent` trait defines the core interface for a TAP agent, including methods for processing messages, getting configuration, and accessing message packing functionality.

### AgentConfig

Configuration for a TAP agent, including the agent's DID and other settings.

### MessagePacker

Handles the packing and unpacking of DIDComm messages, including encryption/decryption and signing/verification.

### DIDResolver

Resolves DIDs to DID Documents for key discovery and message routing.

### SecretsResolver

Manages cryptographic secrets like private keys for signing and decryption operations.

## Integration with Other Crates

- **tap-msg**: Uses TAP message types defined in tap-msg
- **tap-caip**: Validates chain-agnostic identifiers in messages
- **tap-node**: Integrates with tap-node for multi-agent orchestration
- **tap-http**: Can be used with tap-http for HTTP-based DIDComm messaging

## Security

This crate includes implementations for secure key management and message integrity. By default, the implementation:

- Uses Ed25519 keys for signing with proper conversion to X25519 for encryption
- Supports DIDComm v2 secure messaging patterns
- Implements proper validation of message contents and signatures

For production use, it's recommended to implement a custom `SecretsResolver` that integrates with a secure key management system.

## Examples

See the [examples directory](./examples) for more detailed examples.
