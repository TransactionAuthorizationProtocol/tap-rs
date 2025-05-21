# TAP Agent

The `tap-agent` crate implements the agent functionality for the Transaction Authorization Protocol (TAP), providing a secure and extensible framework for handling TAP messages, managing cryptographic operations, and resolving decentralized identifiers (DIDs).

## Overview

The TAP Agent serves as the foundation for secure communication in the TAP ecosystem, enabling entities to:

- **Establish secure identities** using Decentralized Identifiers (DIDs)
- **Exchange authenticated and encrypted messages** with strong cryptographic guarantees
- **Process and validate** TAP protocol messages for compliant transfers and payments
- **Manage cryptographic keys** with support for multiple key types and algorithms
- **Resolve DIDs** across different methods with a pluggable resolver architecture
- **Automatically deliver messages** to service endpoints defined in DID documents
- **Generate, store, and retrieve** cryptographic keys for long-term use

## Architecture

The `tap-agent` crate is designed with a modular architecture that separates concerns and allows for extensibility:

```
tap-agent
├── agent        - Core agent implementation and traits
├── agent_key    - Key abstraction for signing, verification, encryption, and decryption
├── config       - Agent configuration parameters
├── cli          - Command-line interface for DID and key management
├── crypto       - Cryptographic operations and message security
├── did          - DID resolution, generation, and validation
├── error        - Error types and handling
├── key_manager  - Key generation and management
├── local_agent_key - Concrete implementation of AgentKey traits
├── message      - Message formatting and processing
├── message_packing - Message packing and unpacking utilities
├── storage      - Persistent storage for keys and DIDs
```

This architecture follows clean separation of concerns principles:
- Core agent functionality is independent from specific cryptographic implementations
- DID resolution is pluggable with support for multiple methods
- Key management is abstracted to support different security approaches
- CLI tools provide user-friendly access to core functionality

### Key Components

#### Agent

The `Agent` trait defines the core interface for TAP agents, with methods for:

- Retrieving the agent's DID
- Sending messages with appropriate security modes
- Finding service endpoints from DID documents
- Receiving and unpacking messages
- Validating message contents

The `Agent` implementation provides a production-ready implementation of this trait with:
- Multiple creation methods (builder pattern, from stored keys, ephemeral)
- Automatic service endpoint discovery and message delivery
- Configurable timeout and security settings
- Comprehensive logging for debugging
- Support for both native and WASM environments
- Integration with the `KeyManager` for cryptographic operations

#### AgentKey

The `AgentKey` trait hierarchy provides a flexible abstraction for cryptographic keys:

- `AgentKey` - Base trait with core properties (key ID, DID, key type)
- `SigningKey` - Extends `AgentKey` with capabilities for creating JWS signatures
- `VerificationKey` - Trait for verifying signatures (can be implemented by public keys)
- `EncryptionKey` - Extends `AgentKey` with capabilities for creating JWE encryptions
- `DecryptionKey` - Extends `AgentKey` with capabilities for decrypting JWEs

The `LocalAgentKey` implementation provides a concrete implementation that:
- Stores key material locally (in memory or persistent storage)
- Supports multiple cryptographic algorithms (Ed25519, P-256, Secp256k1)
- Implements all the AgentKey-related traits for complete cryptographic functionality
- Works with the JWS/JWE standards for signatures and encryption

This trait-based approach enables:
- Clean separation between key management and cryptographic operations
- Support for different key storage mechanisms (local, HSM, remote, etc.)
- Flexible algorithm selection based on key types
- Simple interface for common cryptographic operations

#### DID Resolution

The DID resolution system supports multiple DID methods through a pluggable architecture:

- `SyncDIDResolver` - A trait for resolving DIDs to DID documents
- `DIDMethodResolver` - A trait for method-specific resolvers
- `KeyResolver` - A resolver for the `did:key` method (Ed25519, P-256, Secp256k1)
- `WebResolver` - A resolver for the `did:web` method with HTTP resolution
- `MultiResolver` - A resolver that manages multiple method-specific resolvers

The system includes advanced features like:
- Automatic conversion between Ed25519 verification keys and X25519 key agreement keys
- Support for JWK, Multibase, and Base58 key formats
- Integration with W3C compliant DID Document formats
- Caching for improved performance

#### Cryptographic Operations

The cryptographic system provides:

- `MessagePacker` - A trait for packing and unpacking secure messages
- `DefaultMessagePacker` - Standards-compliant implementation of JWS/JWE formats
- `BasicSecretResolver` - A simple in-memory implementation for development
- `KeyManager` - A component for generating and managing cryptographic keys
- `KeyStorage` - Persistent storage for cryptographic keys and metadata

#### Security Modes

The agent supports different security modes for messages:

- `Plain` - No security (for testing only)
- `Signed` - Messages are digitally signed but not encrypted (integrity protection)
- `AuthCrypt` - Messages are authenticated and encrypted (confidentiality + integrity)

Each security mode uses standards-compliant cryptographic approaches:
- Signing uses JSON Web Signatures (JWS) with algorithm selection based on key type
- Encryption uses JSON Web Encryption (JWE) with AES-GCM and ECDH-ES key agreement
- Message formats are compatible with broader DIDComm ecosystem standards

## Features

- **Secure Identity Management**: Create and manage agent identities using DIDs
- **Message Processing**: Handle TAP message flows with proper validation
- **DID Resolution**: Resolve DIDs for message routing, key discovery, and service endpoints
- **Cryptographic Operations**: Sign, verify, encrypt, and decrypt messages
- **Key Management**: Generate, store, and retrieve cryptographic keys
- **Message Delivery**: Automatically deliver messages to service endpoints
- **DID Generation CLI**: Command-line tools for creating and managing DIDs and keys
- **Ephemeral DIDs**: Create temporary DIDs for testing or short-lived processes
- **Asynchronous Processing**: Process messages concurrently using Tokio
- **WASM Support**: Run in browser environments with WebAssembly
- **Extensible DID Methods**: Support for did:key and did:web, with architecture for adding more methods
- **Performance Optimized**: Benchmarked for high-throughput scenarios
- **Native Cryptography**: Direct implementation of cryptographic operations without external DIDComm dependencies
- **Persistent Storage**: Store keys securely for long-term use
- **Multiple Key Types**: Support for Ed25519, P-256, and Secp256k1 keys
- **Standards-Compliant**: Implementation follows W3C DID and IETF JWS/JWE standards

## Usage Examples

### Agent Creation

The TAP Agent can be created in multiple ways depending on your needs. Here are the main approaches:

#### 1. Using Agent Builder (Recommended)

The builder pattern provides a clean, fluent interface for creating agents:

```rust
use tap_agent::{Agent, AgentBuilder, DefaultKeyManager};
use std::sync::Arc;

// Create a key manager
let key_manager = Arc::new(DefaultKeyManager::new());

// Generate a new key or load an existing one
let key = key_manager.generate_key(DIDGenerationOptions {
    key_type: KeyType::Ed25519,
})?;

// Build the agent with the generated key
let agent = AgentBuilder::new(key.did)
    .with_debug(true)
    .with_timeout(30)
    .with_security_mode("SIGNED".to_string())
    .build(key_manager);
```

#### 2. Using Stored Keys

Load keys from the default storage location (~/.tap/keys.json):

```rust
use tap_agent::Agent;

// Use a stored key with a specific DID
let agent = Agent::from_stored_keys(
    Some("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string()),
    true
).await?;

// Or use the default key from storage
let agent = Agent::from_stored_keys(None, true).await?;
```

#### 3. Using Ephemeral Keys

Create an agent with a temporary key that is not persisted:

```rust
use tap_agent::{DefaultKeyManager, AgentBuilder};
use std::sync::Arc;

// Create a key manager
let key_manager = Arc::new(DefaultKeyManager::new());

// Generate a random key
let key = key_manager.generate_key(DIDGenerationOptions {
    key_type: KeyType::Ed25519,
})?;

// Create an agent with the ephemeral key
let agent = AgentBuilder::new(key.did.clone())
    .with_debug(true)
    .build(key_manager);

println!("Created ephemeral agent with DID: {}", key.did);
```

#### 4. Manual Creation (Advanced)

For complete control over the agent configuration:

```rust
use tap_agent::{Agent, AgentConfig, DefaultKeyManager, KeyManagerPacking, Secret, SecretMaterial, SecretType};
use std::sync::Arc;

// Create agent configuration
let config = AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());

// Set up a key manager
let mut key_manager = DefaultKeyManager::new();

// Add a secret to the key manager
let secret = Secret {
    id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
    type_: SecretType::JsonWebKey2020,
    secret_material: SecretMaterial::JWK {
        private_key_jwk: serde_json::json!({
            "kty": "OKP",
            "crv": "Ed25519",
            "x": "11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo",
            "d": "nWGxne_9WmC6hEr-BQh-uDpW6n7dZsN4c4C9rFfIz3Yh",
            "kid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#keys-1"
        }),
    },
};
key_manager.add_secret(&secret.id, secret)?;

// Create the agent
let agent = Agent::new(config, Arc::new(key_manager));
```

### Sending Messages

The agent provides a flexible API for sending messages to one or more recipients:

```rust
use tap_agent::Agent;
use tap_msg::message::Transfer;
use tap_caip::AssetId;
use tap_msg::Participant;
use std::str::FromStr;
use std::collections::HashMap;

// Create a transfer message
let transfer = Transfer {
    transaction_id: uuid::Uuid::new_v4().to_string(),
    asset: AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
    originator: Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    },
    beneficiary: Some(Participant {
        id: "did:key:z6MkhFvVnYxkqLNEiWQmUwhQuVpXiCfNmRUVi5yZ4Cg9w15k".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    }),
    amount: "100.0".to_string(),
    agents: vec![],
    settlement_id: None,
    memo: Some("Test transfer".to_string()),
    metadata: HashMap::new(),
};

// Option 1: Pack a message without delivery (for manual handling)
let recipient_did = "did:key:z6MkhFvVnYxkqLNEiWQmUwhQuVpXiCfNmRUVi5yZ4Cg9w15k";
let (packed_message, _) = agent.send_message(&transfer, vec![recipient_did], false).await?;

// You can now manually send the packed message however you want

// Option 2: Automatic delivery to service endpoints
let (packed_message, delivery_results) = agent.send_message(&transfer, vec![recipient_did], true).await?;

// Option 3: Send to multiple recipients
let recipients = vec![
    "did:key:z6MkhFvVnYxkqLNEiWQmUwhQuVpXiCfNmRUVi5yZ4Cg9w15k",
    "did:web:example.com"
];
let (packed_message, delivery_results) = agent.send_message(&transfer, recipients, true).await?;

// Check delivery results
for result in delivery_results {
    if let Some(status) = result.status {
        println!("Delivery status: {}", status);
    } else if let Some(error) = &result.error {
        println!("Error delivering to {}: {}", result.did, error);
    }
}

// You can also look up service endpoints manually
if let Ok(Some(endpoint)) = agent.get_service_endpoint(recipient_did).await {
    println!("Found service endpoint: {}", endpoint);
    // Use the endpoint for custom delivery logic
}
```

The `send_message` method provides flexibility with three parameters:
1. The message to send (any type implementing `TapMessageBody`)
2. A list of recipient DIDs
3. A boolean indicating whether to attempt automatic delivery

The method returns:
1. The packed message as a string (ready for transport)
2. A vector of delivery results (when automatic delivery is requested)

### Receiving Messages

The agent provides a simple API for unpacking and validating received messages:

```rust
use tap_agent::Agent;
use tap_msg::message::Transfer;

// Receive a message from some transport mechanism
let packed_message = "..."; // Received from HTTP, WebSockets, etc.

// Unpack and validate the message, converting to the expected type
let transfer: Transfer = agent.receive_message(packed_message).await?;

// Now you can access the typed message content
println!("Received transfer:");
println!("  Transaction ID: {}", transfer.transaction_id);
println!("  Asset: {}", transfer.asset);
println!("  Amount: {}", transfer.amount);
println!("  From: {}", transfer.originator.id);
if let Some(beneficiary) = &transfer.beneficiary {
    println!("  To: {}", beneficiary.id);
}

// The agent automatically:
// 1. Verifies signatures (if the message is signed)
// 2. Decrypts content (if the message is encrypted)
// 3. Verifies the message is valid according to its type
// 4. Converts the message to the requested type
```

The agent handles the entire verification and decryption process, allowing you to focus on processing the message content rather than worrying about cryptographic details.

Both `TapAgent` and `DefaultAgent` implement the `Agent` trait, so the receiving API is the same regardless of which agent implementation you use.

### Using DID Resolvers

The agent provides flexible DID resolution capabilities:

```rust
use tap_agent::did::MultiResolver;
use std::sync::Arc;

// Create a resolver with built-in support for did:key and did:web
let resolver = MultiResolver::default();
let resolver = Arc::new(resolver);

// Resolve any supported DID to its DID document
let did_doc = resolver.resolve("did:web:example.com").await?;

if let Some(doc) = did_doc {
    println!("Resolved DID: {}", doc.id);

    // Check verification methods
    for vm in &doc.verification_method {
        println!("Verification method: {}", vm.id);
        println!("  Type: {:?}", vm.type_);
        println!("  Controller: {}", vm.controller);
    }

    // Check authentication methods
    if !doc.authentication.is_empty() {
        println!("Authentication methods:");
        for auth in &doc.authentication {
            println!("  {}", auth);
        }
    }

    // Check key agreement methods
    if !doc.key_agreement.is_empty() {
        println!("Key agreement methods:");
        for ka in &doc.key_agreement {
            println!("  {}", ka);
        }
    }

    // Check service endpoints
    if !doc.service.is_empty() {
        println!("Service endpoints:");
        for (i, service) in doc.service.iter().enumerate() {
            println!("  [{}] ID: {}", i+1, service.id);
            println!("      Type: {}", service.type_);
            println!("      Endpoint: {}", service.service_endpoint);
        }
    }
}
```

#### Supported DID Methods

The default resolver supports:

1. **did:key** - Self-contained DIDs with embedded public keys
   ```
   did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
   ```

2. **did:web** - DIDs associated with web domains
   ```
   did:web:example.com
   did:web:example.com:path:to:resource
   ```

You can extend the resolver with custom DID methods as shown in the next section.

### Custom DID Method Resolver

You can implement and register custom DID method resolvers to extend the agent's capabilities:

```rust
use tap_agent::did::{DIDMethodResolver, MultiResolver, DIDDoc, VerificationMethod};
use tap_agent::did::{VerificationMethodType, VerificationMaterial};
use async_trait::async_trait;
use tap_agent::error::Result;

#[derive(Debug)]
struct CustomResolver;

impl CustomResolver {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DIDMethodResolver for CustomResolver {
    fn method(&self) -> &str {
        "example" // For did:example:123
    }

    async fn resolve_method(&self, did: &str) -> Result<Option<DIDDoc>> {
        // Validate DID format
        if !did.starts_with("did:example:") {
            return Ok(None);
        }

        // Extract ID portion
        let id_part = &did[12..]; // Skip "did:example:"

        // Create a simple verification method
        let vm_id = format!("{}#keys-1", did);
        let vm = VerificationMethod {
            id: vm_id.clone(),
            type_: VerificationMethodType::Ed25519VerificationKey2018,
            controller: did.to_string(),
            verification_material: VerificationMaterial::Base58 {
                public_key_base58: format!("custom-key-for-{}", id_part),
            },
        };

        // Create a DID document
        let doc = DIDDoc {
            id: did.to_string(),
            verification_method: vec![vm],
            authentication: vec![vm_id.clone()],
            key_agreement: vec![],
            assertion_method: vec![],
            capability_invocation: vec![],
            capability_delegation: vec![],
            service: vec![],
        };

        Ok(Some(doc))
    }
}

// Create a resolver with the default resolvers
let mut resolver = MultiResolver::default();

// Register the custom resolver
resolver.register_method("example", CustomResolver::new());

// Now you can resolve did:example: DIDs
let doc = resolver.resolve("did:example:123").await?;
```

You can implement resolvers for any DID method, including:
- Blockchain-based DIDs (did:ethr, did:sol, etc.)
- Registry-based DIDs (did:ion, did:factom, etc.)
- Custom protocol DIDs for specific use cases

The resolver will automatically route DID resolution requests to the appropriate method resolver based on the DID prefix.

## Security Considerations

The `tap-agent` crate implements several security features:

- **Message Integrity**: All messages can be digitally signed to ensure integrity
- **Message Confidentiality**: Messages can be encrypted for confidentiality
- **Key Management**: Proper key handling with separation of concerns
- **DID Verification**: Validation of DIDs and DID documents
- **Secure Defaults**: Secure defaults for message security modes
- **Native Cryptography**: Direct implementation of cryptographic algorithms using well-tested libraries

For production use, it's recommended to:

1. Implement a custom `DebugSecretsResolver` that integrates with a secure key management system
2. Use proper key rotation and management practices
3. Ensure secure transport for message exchange
4. Regularly update dependencies to incorporate security fixes

## Integration with Other TAP Components

The `tap-agent` crate integrates with other components in the TAP ecosystem:

- **tap-msg**: Uses message types and validation from tap-msg
- **tap-caip**: Validates chain-agnostic identifiers in messages
- **tap-node**: Provides the agent functionality for tap-node
- **tap-http**: Can be used with tap-http for HTTP-based secure messaging
- **tap-wasm**: Supports WASM bindings for browser environments
- **tap-ts**: Provides TypeScript bindings for the agent functionality

## Performance

The `tap-agent` crate is designed for high performance, with benchmarks showing:

- Message packing/unpacking: Thousands of operations per second
- DID resolution: Fast caching of resolved DIDs
- Cryptographic operations: Optimized for common use cases

Benchmarks can be run with:

```bash
cargo bench --bench agent_benchmark
```

## DID Generation CLI

The `tap-agent` crate includes a command-line interface (CLI) for generating and managing DIDs and keys. This makes it easy to create DIDs for testing, development, or production use.

### Installation

The CLI tool can be installed in several ways:

```bash
# From crates.io (recommended for most users)
cargo install tap-agent

# From the repository (if you have it cloned)
cargo install --path tap-agent

# Build without installing
cargo build --package tap-agent
```

After installation, the following commands will be available:
- `tap-agent-cli` - Command-line tool for DID and key management

### Command Reference

After installation, you can use the `tap-agent-cli` command to manage DIDs and keys. Here's a complete reference of available commands:

#### Generate Command

The `generate` command creates new DIDs with different key types and methods:

```bash
# Generate a did:key with Ed25519 (default)
tap-agent-cli generate

# Specify method and key type
tap-agent-cli generate --method key --key-type ed25519
tap-agent-cli generate --method key --key-type p256
tap-agent-cli generate --method key --key-type secp256k1

# Generate a did:web for a domain
tap-agent-cli generate --method web --domain example.com

# Save outputs to files
tap-agent-cli generate --output did.json --key-output key.json

# Save key to default storage (~/.tap/keys.json) and set as default
tap-agent-cli generate --save --default
```

#### Lookup Command

The `lookup` command resolves DIDs to their DID documents:

```bash
# Look up a DID and display its DID document
tap-agent-cli lookup did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

# Look up a DID and save the document to a file
tap-agent-cli lookup did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK --output did-document.json

# Look up WebDIDs
tap-agent-cli lookup did:web:example.com
tap-agent-cli lookup did:web:example.com:path:to:resource
```

The resolver supports the following DID methods by default:
- `did:key` - Resolves DIDs based on public keys
- `did:web` - Resolves DIDs from web domains

#### Keys Command

The `keys` command manages stored keys:

```bash
# List all stored keys
tap-agent-cli keys list

# View details for a specific key
tap-agent-cli keys view did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

# Set a key as the default
tap-agent-cli keys set-default did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

# Delete a key (with confirmation)
tap-agent-cli keys delete did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

# Delete a key (without confirmation)
tap-agent-cli keys delete did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK --force
```

Keys are stored in `~/.tap/keys.json` by default. This storage location is shared with other TAP tools like `tap-http` for consistent key management.

#### Import Command

The `import` command imports existing keys:

```bash
# Import a key from a file
tap-agent-cli import key.json

# Import and set as default
tap-agent-cli import key.json --default
```

### Help and Documentation

```bash
# Display general help
tap-agent-cli --help

# Display help for a specific command
tap-agent-cli generate --help
tap-agent-cli lookup --help
tap-agent-cli keys --help
tap-agent-cli import --help

# Display help for a subcommand
tap-agent-cli keys delete --help
```

### Using Generated DIDs

For `did:web`, you'll need to:
1. Generate the DID using the CLI: `tap-agent-cli generate --method web --domain yourdomain.com --output did.json`
2. Place the generated DID document at:
   - For `did:web:example.com`: `https://example.com/.well-known/did.json`
   - For `did:web:example.com:path:to:resource`: `https://example.com/path/to/resource/did.json`

## Working with Service Endpoints

Service endpoints in DID documents provide URLs where the DID subject can receive messages. TAP Agents can automatically discover and use these endpoints for message delivery.

```rust
// Example: Find and use a service endpoint for a DID
async fn work_with_service_endpoint(agent: &DefaultAgent, did: &str, message: &Transfer) -> Result<()> {
    // Method 1: Get the service endpoint URL directly
    match agent.get_service_endpoint(did).await? {
        Some(endpoint) => {
            println!("Found service endpoint for {}: {}", did, endpoint);

            // Pack the message first (if you want to handle delivery manually)
            let (packed_message, _) = agent.send_message(message, vec![did], false).await?;

            // Now you can manually send to the endpoint:
            // let client = reqwest::Client::new();
            // let response = client.post(endpoint)
            //     .header("Content-Type", "application/didcomm-encrypted+json")
            //     .body(packed_message)
            //     .send()
            //     .await?;
        },
        None => println!("No service endpoint found for {}", did),
    }

    // Method 2: Let the agent handle delivery automatically
    // The boolean parameter (true) tells the agent to attempt automatic delivery
    let (_, delivery_results) = agent.send_message(message, vec![did], true).await?;

    // Check the delivery results
    for result in delivery_results {
        if let Some(status) = result.status {
            println!("Delivery to {} resulted in status: {}", result.did, status);
        } else if let Some(error) = &result.error {
            println!("Delivery to {} failed: {}", result.did, error);
        }
    }

    Ok(())
}
```

### Service Endpoint Types and Message Delivery

The TAP Agent can work with different types of service endpoints defined in DID documents:

1. **DIDCommMessaging** endpoints (prioritized) - Specifically designed for secure message exchange:
   ```json
   {
     "service": [{
       "id": "did:example:123#didcomm",
       "type": "DIDCommMessaging",
       "serviceEndpoint": {
         "uri": "https://example.com/didcomm",
         "accept": ["didcomm/v2"],
         "routingKeys": []
       }
     }]
   }
   ```

2. **Other** endpoint types - Any service endpoint type that provides a URL:
   ```json
   {
     "service": [{
       "id": "did:example:123#agent",
       "type": "TapAgent",
       "serviceEndpoint": "https://agent.example.com/tap"
     }]
   }
   ```

3. **Simple string endpoints** - Basic URL string endpoints:
   ```json
   {
     "service": [{
       "id": "did:example:123#messaging",
       "type": "MessagingService",
       "serviceEndpoint": "https://example.com/messages"
     }]
   }
   ```

The agent supports multiple endpoint formats and follows these resolution rules:

1. Look for a service with `type = "DIDCommMessaging"` first
2. Fall back to the first available service endpoint if no DIDCommMessaging endpoint is found
3. Handle both object-style and string-style service endpoints

### Automatic Message Delivery

The TAP Agent provides a seamless way to automatically deliver messages to service endpoints:

```rust
// Example 1: Send a message to a single recipient with automatic delivery
let transfer = Transfer {
    asset: AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
    originator: create_originator(),
    beneficiary: Some(create_beneficiary()),
    amount: "100.0".to_string(),
    // Other fields...
    transaction_id: uuid::Uuid::new_v4().to_string(),
};

// The third parameter (true) enables automatic delivery
let (packed_message, delivery_results) =
    agent.send_message(&transfer, vec![recipient_did], true).await?;

// Example 2: Send to multiple recipients
let recipients = vec!["did:web:example.com", "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"];
let (packed_message, delivery_results) =
    agent.send_message(&transfer, recipients, true).await?;

// Process delivery results
for result in &delivery_results {
    if let Some(status) = result.status {
        if status >= 200 && status < 300 {
            println!("✓ Successfully delivered to {} (status {})", result.did, status);
        } else {
            println!("! Delivery to {} failed with status {}", result.did, status);
        }
    } else if let Some(error) = &result.error {
        println!("✗ Error delivering to {}: {}", result.did, error);
    }
}
```

#### Delivery Process

When automatic delivery is enabled, the agent follows this process:

1. **Endpoint Resolution**: Resolves each recipient's DID document to find service endpoints
2. **Message Packing**: Packs the message with appropriate security for all recipients
3. **Delivery Attempts**: For each recipient with a service endpoint:
   - Sends the packed message via HTTP POST with `Content-Type: application/didcomm-encrypted+json`
   - Records success (with HTTP status) or failure (with error message)
4. **Result Collection**: Returns a vector of `DeliveryResult` structures containing:
   - `did`: The recipient DID
   - `endpoint`: The service endpoint URL used
   - `status`: HTTP status code (if successful)
   - `error`: Error message (if failed)

The delivery mechanism is designed to be resilient - failures with one recipient won't prevent delivery attempts to others, and the operation won't fail even if no service endpoints are found.

## Creating Ephemeral DIDs

For testing or short-lived processes, you can create ephemeral DIDs that exist only in memory:

```rust
// Option 1: Create an ephemeral agent with TapAgent (recommended, async API)
let (agent, did) = TapAgent::from_ephemeral_key().await?;
println!("TapAgent DID: {}", did);

// Option 2: Create an ephemeral agent with DefaultAgent
let (agent, did) = DefaultAgent::new_ephemeral()?;
println!("Agent DID: {}", did);

// This agent has a fully functional private key and can be used immediately
// for sending and receiving messages without any persistence
let (packed_message, _) = agent
    .send_message(&transfer, vec!["did:example:recipient"], false)
    .await?;
```

Ephemeral DIDs are useful for:
- Test environments where persistence isn't needed
- Short-lived agent instances that don't need to maintain state
- Situations where you want to minimize key management overhead
- Temporary identities for one-time operations

## Feature Flags

The crate provides several feature flags to customize functionality:

- **native** (default): Enables native platform features including:
  - Tokio runtime for async functionality
  - HTTP support for service endpoint delivery
  - Complete DID resolution with HTTP requests for did:web
  - File system access for key storage

- **wasm**: Enables WebAssembly support for browser environments:
  - Browser-compatible cryptography
  - JavaScript integration
  - Web-based service endpoint delivery
  - Browser storage for keys

When working with different environments:
- For server applications: Use the default **native** feature
- For browser applications: Use the **wasm** feature
- For CLI tools: Use the default **native** feature

Note that did:web resolution requires the **native** feature to be enabled, as it depends on HTTP requests to fetch DID documents.

## Cryptographic Support

The `tap-agent` crate implements comprehensive cryptographic operations with support for:

### Key Abstraction

- **AgentKey Trait Hierarchy** - Modular and extensible key capabilities:
  - `AgentKey` - Core trait with key ID, DID, and key type properties
  - `SigningKey` - For creating digital signatures (JWS)
  - `VerificationKey` - For verifying signatures
  - `EncryptionKey` - For encrypting data (JWE)
  - `DecryptionKey` - For decrypting data

- **LocalAgentKey Implementation** - Concrete implementation of the AgentKey traits:
  - Stores key material in memory or persistent storage
  - Implements all cryptographic operations locally
  - Supports multiple key types and algorithms
  - Compatible with JWS and JWE standards

### Key Types
- **Ed25519** - Fast digital signatures with small signatures (128 bits of security)
- **P-256** - NIST standardized elliptic curve for ECDSA signatures and ECDH
- **secp256k1** - Blockchain-compatible ECDSA signatures (used in Ethereum, Bitcoin)

### Encryption & Signing
- **JWS (JSON Web Signatures)** - Standards-compliant digital signatures:
  - EdDSA algorithm for Ed25519 keys
  - ES256 algorithm for P-256 keys
  - ES256K algorithm for secp256k1 keys

- **JWE (JSON Web Encryption)** - Standards-compliant encryption:
  - AES-GCM (A256GCM) for authenticated encryption
  - ECDH-ES+A256KW for key agreement and key wrapping
  - Per-recipient encrypted content encryption keys (CEKs)

### Security Modes
- **Plain** - No security (for testing only)
- **Signed** - Digital signatures without encryption (integrity protection)
- **AuthCrypt** - Authenticated encryption (confidentiality + integrity)

The cryptographic implementations align with industry standards, allowing interoperability with other systems that support JWS and JWE formats.

## License

This crate is licensed under the [MIT License](LICENSE).
