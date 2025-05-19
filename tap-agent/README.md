# TAP Agent

The `tap-agent` crate implements the agent functionality for the Transaction Authorization Protocol (TAP), providing a secure and extensible framework for handling TAP messages, managing cryptographic operations, and resolving decentralized identifiers (DIDs).

## Overview

The TAP Agent serves as the foundation for secure communication in the TAP ecosystem, enabling entities to:

- Establish and verify identities using DIDs
- Exchange secure messages with cryptographic guarantees
- Process and validate TAP protocol messages
- Manage cryptographic keys and operations
- Integrate with various DID methods and resolvers

## Architecture

The `tap-agent` crate is designed with a modular architecture that separates concerns and allows for extensibility:

```
tap-agent
├── agent       - Core agent implementation and traits
├── config      - Agent configuration
├── cli         - Command-line interface for DID generation
├── crypto      - Cryptographic operations and message security
├── did         - DID resolution and management
├── error       - Error types and handling
├── message     - Message processing utilities
```

### Key Components

#### Agent

The `Agent` trait defines the core interface for TAP agents, with methods for:

- Retrieving the agent's DID
- Sending messages with appropriate security modes
- Finding service endpoints from DID documents
- Receiving and unpacking messages
- Validating message contents

The `DefaultAgent` implementation provides a standard implementation of this trait, using native cryptographic operations for secure message exchange. It automatically checks for service endpoints when sending messages to help with message routing.

#### DID Resolution

The DID resolution system supports multiple DID methods through a pluggable architecture:

- `SyncDIDResolver` - A trait for resolving DIDs to DID documents
- `DIDMethodResolver` - A trait for method-specific resolvers
- `KeyResolver` - A resolver for the `did:key` method
- `MultiResolver` - A resolver that manages multiple method-specific resolvers

The system supports conversion between Ed25519 verification keys and X25519 key agreement keys, enabling the same keypair to be used for both signing and encryption.

#### Cryptographic Operations

The cryptographic system provides:

- `MessagePacker` - A trait for packing and unpacking secure messages
- `DefaultMessagePacker` - An implementation supporting various security modes
- `DebugSecretsResolver` - A trait for resolving cryptographic secrets
- `BasicSecretResolver` - A simple in-memory implementation for development

#### Security Modes

The agent supports different security modes for messages:

- `Plain` - No security (for testing only)
- `Signed` - Messages are digitally signed but not encrypted
- `AuthCrypt` - Messages are both signed and encrypted (authenticated encryption)

## Features

- **Secure Identity Management**: Create and manage agent identities using DIDs
- **Message Processing**: Handle TAP message flows with proper validation
- **DID Resolution**: Resolve DIDs for message routing, key discovery, and service endpoints
- **Cryptographic Operations**: Sign, verify, encrypt, and decrypt messages
- **Key Management**: Securely manage cryptographic keys
- **DID Generation CLI**: Create DIDs and keys using a command-line interface
- **Ephemeral DIDs**: Create temporary DIDs for testing or short-lived processes
- **Asynchronous Processing**: Process messages concurrently using Tokio
- **WASM Support**: Run in browser environments with WebAssembly
- **Extensible DID Methods**: Support for did:key and did:web, with architecture for adding more methods
- **Performance Optimized**: Benchmarked for high-throughput scenarios
- **Native Cryptography**: Direct implementation of cryptographic operations without external DIDComm dependencies

## Usage Examples

### Basic Agent Setup

```rust
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{DefaultMessagePacker, BasicSecretResolver};
use tap_agent::did::{MultiResolver, KeyResolver};
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use std::sync::Arc;

// Create agent configuration with a DID
let config = AgentConfig::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string());

// Set up DID resolver with support for did:key
let mut did_resolver = MultiResolver::new();
did_resolver.register_method("key", KeyResolver::new());
let did_resolver = Arc::new(did_resolver);

// Set up secret resolver with the agent's key
let mut secret_resolver = BasicSecretResolver::new();
let secret = Secret {
    id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#keys-1".to_string(),
    type_: SecretType::JsonWebKey2020,
    secret_material: SecretMaterial::JWK {
        private_key_jwk: serde_json::json!({
            "kty": "OKP",
            "crv": "Ed25519",
            "x": "11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo",
            "d": "nWGxne_9WmC6hEr-BQh-uDpW6n7dZsN4c4C9rFfIz3Yh"
        }),
    },
};
secret_resolver.add_secret("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK", secret);
let secret_resolver = Arc::new(secret_resolver);

// Create message packer
let message_packer = Arc::new(DefaultMessagePacker::new(did_resolver, secret_resolver));

// Create the agent
let agent = DefaultAgent::new(config, message_packer);
```

### Sending a Message

```rust
use tap_msg::message::Transfer;
use tap_caip::AssetId;
use std::str::FromStr;
use std::collections::HashMap;

// Create a transfer message
let transfer = Transfer {
    asset: AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
    originator: Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    },
    beneficiary: Some(Participant {
        id: "did:key:z6MkhFvVnYxkqLNEiWQmUwhQuVpXiCfNmRUVi5yZ4Cg9w15k".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    }),
    amount: "100.0".to_string(),
    agents: vec![],
    settlement_id: None,
    memo: Some("Test transfer".to_string()),
    metadata: HashMap::new(),
};

// Look up the recipient's service endpoint (for DIDs with service endpoints like did:web or did:key)
let recipient_did = "did:key:z6MkhFvVnYxkqLNEiWQmUwhQuVpXiCfNmRUVi5yZ4Cg9w15k";

// Method 1: Use the convenience method to get the service endpoint directly
if let Ok(Some(endpoint)) = agent.get_did_service_endpoint(recipient_did).await {
    println!("Found service endpoint: {}", endpoint);
    // In a complete implementation, you would send the packed message to this endpoint
}

// Method 2: Use send_message with delivery parameter for automatic delivery
// This will automatically send the message to the service endpoint if found
// The third parameter (true) indicates to deliver the message automatically
let (packed_message, delivery_results) = agent.send_message(&transfer, vec![recipient_did], true).await?;

// Check delivery results
for result in delivery_results {
    if let Some(status) = result.status {
        println!("Message delivered to {} at endpoint {}, status: {}", 
                 result.did, result.endpoint, status);
    } else if let Some(error) = &result.error {
        println!("Failed to deliver message to {}: {}", result.did, error);
    }
}

// The packed_message is also returned and can be used for other purposes
```

### Receiving a Message

```rust
// Receive and process an incoming message
let packed_message = "..."; // Received from network/transport
let transfer: Transfer = agent.receive_message(packed_message).await?;

// Now you can access the transfer details
println!("Received transfer of {} {}", transfer.amount, transfer.asset);
```

### Using the Built-in Resolvers

```rust
use tap_agent::did::MultiResolver;
use std::sync::Arc;

// Create a default resolver with built-in support for did:key and did:web
let resolver = MultiResolver::default();
let resolver = Arc::new(resolver);

// Use the resolver to resolve DIDs
let rt = tokio::runtime::Runtime::new()?;
let did_doc = rt.block_on(async {
    resolver.resolve("did:web:example.com").await
})?;

if let Some(doc) = did_doc {
    println!("Resolved DID: {}", doc.id);
    
    // Check for service endpoints
    if !doc.service.is_empty() {
        println!("Service endpoints found:");
        for (i, service) in doc.service.iter().enumerate() {
            println!("  [{}] ID: {}", i+1, service.id);
            println!("      Endpoint: {:?}", service.service_endpoint);
        }
    }
    
    // Process the DID document...
}
```

### Custom DID Method Resolver

You can also implement and add your own DID method resolver:

```rust
use tap_agent::did::{DIDMethodResolver, MultiResolver};
use tap_agent::did::DIDDoc;
use async_trait::async_trait;
use tap_agent::error::Result;

#[derive(Debug)]
struct CustomResolver;

#[async_trait]
impl DIDMethodResolver for CustomResolver {
    fn method(&self) -> &str {
        "example" // For did:example:123
    }

    async fn resolve_method(&self, did: &str) -> Result<Option<DIDDoc>> {
        // Implementation for resolving custom DID method
        // ...
    }
}

// Create a resolver with the default resolvers
let mut resolver = MultiResolver::default();

// Register the custom resolver
resolver.register_method("example", CustomResolver::new());
```

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

If you have the tap-rs repository cloned:

```bash
cargo install --path tap-agent
```

### Usage

#### Generating a DID

```bash
# Generate a did:key with Ed25519
tap-agent-cli generate --method key --key-type ed25519

# Generate a did:key with P-256
tap-agent-cli generate --method key --key-type p256

# Generate a did:key with Secp256k1
tap-agent-cli generate --method key --key-type secp256k1

# Generate a did:web for a domain
tap-agent-cli generate --method web --domain example.com
```

#### Saving Output to Files

```bash
# Save DID document to did.json and key to key.json
tap-agent-cli generate --output did.json --key-output key.json

# Save did:web document (to be placed at /.well-known/did.json on the domain)
tap-agent-cli generate --method web --domain example.com --output did.json
```

#### Looking up a DID Document

```bash
# Look up a DID and display its DID Document
tap-agent-cli lookup did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

# Look up a DID and save the DID Document to a file
tap-agent-cli lookup did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK --output did-document.json
```

The `lookup` command resolves a DID to its DID document and displays detailed information about:
- Verification methods (with key material)
- Authentication methods
- Key agreement methods
- Services (with their endpoints, useful for message routing)

The resolver supports the following DID methods by default:
- `did:key` - Resolves DIDs based on Ed25519 public keys 
- `did:web` - Resolves DIDs from web domains according to the [DID Web Method Specification](https://w3c-ccg.github.io/did-method-web/)

Additional DID methods can be added by implementing custom resolvers.

### Using Generated DIDs

For `did:web`, you'll need to:
1. Generate the DID using the CLI: `tap-agent-cli generate --method web --domain yourdomain.com --output did.json`
2. Place the generated DID document at one of these locations based on your DID format:
   - For `did:web:example.com`: Place at `https://example.com/.well-known/did.json`
   - For `did:web:example.com:path:to:resource`: Place at `https://example.com/path/to/resource/did.json`

#### Looking up WebDIDs
```bash
# Look up a simple WebDID
tap-agent-cli lookup did:web:example.com

# Look up a WebDID with a path
tap-agent-cli lookup did:web:example.com:path:to:resource
```

The resolver will automatically fetch the DID document from the appropriate URL based on the DID format.

## Working with Service Endpoints

Service endpoints in DID documents provide URLs where the DID subject can receive messages. The `tap-agent` crate makes it easy to work with service endpoints:

```rust
// Get a service endpoint for a DID
async fn get_service_endpoint(agent: &DefaultAgent, did: &str) -> Result<()> {
    // Look up the service endpoint
    match agent.get_did_service_endpoint(did).await? {
        Some(endpoint) => {
            println!("Found service endpoint for {}: {}", did, endpoint);
            
            // Use the endpoint for sending messages
            // For example, with an HTTP client:
            // let client = reqwest::Client::new();
            // let packed_message = agent.send_message(&message, did).await?;
            // let response = client.post(endpoint)
            //     .header("Content-Type", "application/didcomm-encrypted+json")
            //     .body(packed_message)
            //     .send()
            //     .await?;
        },
        None => println!("No service endpoint found for {}", did),
    }
    
    Ok(())
}
```

### Service Endpoint Types and Message Delivery

The `tap-agent` crate can work with different types of service endpoints defined in DID documents and automatically deliver messages to them:

1. **DIDCommMessaging** endpoints - Specifically designed for secure message exchange:
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

2. **Other** types of endpoints - General purpose service endpoints:
   ```json
   {
     "service": [{
       "id": "did:example:123#agent",
       "type": "TapAgent",
       "serviceEndpoint": "https://agent.example.com/tap"
     }]
   }
   ```

The `get_service_endpoint` method will prioritize DIDCommMessaging endpoints but will fall back to other types if needed.

### Automatic Message Delivery

The agent can automatically deliver messages to service endpoints using HTTP POST requests:

```rust
// Send a message to a single recipient with automatic delivery
let (packed_message, delivery_results) = agent.send_message(&message, vec![recipient_did], true).await?;

// Send a message to multiple DIDs with automatic delivery
let recipients = vec!["did:web:example.com", "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"];
let (packed_message, delivery_results) = agent.send_message(&message, recipients, true).await?;

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

The delivery process:
1. Resolves each recipient's DID to find service endpoints
2. For each recipient with a service endpoint:
   - Logs "Found service endpoint for [DID]: [endpoint]"
   - Sends the packed message via HTTP POST with Content-Type: application/didcomm-encrypted+json
   - On success: Logs "Delivered message [ID] to [DID] at [endpoint]"
   - On failure: Logs error but continues without failing
3. For recipients without a service endpoint:
   - Logs "No service endpoint found for [DID], skipping delivery" and continues
4. Returns delivery results for recipients where delivery was attempted in a `DeliveryResult` structure:
   - `did`: The DID that was the target of the delivery
   - `endpoint`: The service endpoint URL that was used
   - `status`: HTTP status code if successful
   - `error`: Error message if delivery failed

The agent is designed to be resilient - it will log issues but won't fail the operation if service endpoints can't be found or messages can't be delivered.

## Creating Ephemeral DIDs

For testing or short-lived processes, the `DefaultAgent` can create ephemeral DIDs:

```rust
// Create an agent with an ephemeral did:key (Ed25519)
let (agent, did) = DefaultAgent::new_ephemeral()?;

// The agent is ready to use with the generated did:key
println!("Agent DID: {}", did);
```

## Feature Flags

The crate provides several feature flags to customize functionality:

- **native** (default): Enables native platform features using Tokio and HTTP support for did:web resolution
- **wasm**: Enables WebAssembly support for browser environments

Note that did:web resolution requires the **native** feature to be enabled, as it depends on HTTP requests to fetch DID documents.

## Cryptographic Support

The `tap-agent` crate now implements direct cryptographic operations without external DIDComm libraries, supporting:

- **Ed25519** - For digital signatures (EdDSA)
- **P-256** - For ECDSA signatures and ECDH key agreement
- **secp256k1** - For ECDSA signatures with blockchain compatibility
- **AES-GCM** - For authenticated encryption (A256GCM)
- **ECDH-ES+A256KW** - For key agreement and key wrapping

The implementation follows the JWS (JSON Web Signature) and JWE (JSON Web Encryption) formats for secure messaging, ensuring compatibility with standards-based secure messaging systems.

## License

This crate is licensed under the [MIT License](LICENSE).