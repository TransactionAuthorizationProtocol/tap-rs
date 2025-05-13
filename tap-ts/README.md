# @taprsvp/tap-agent

TypeScript wrapper for TAP-RS library implementing the Transaction Authorization Protocol (TAP).

## Installation

```bash
npm install @taprsvp/tap-agent
```

## Features

- **Type-Safe API**: Fully typed using TypeScript interfaces from `@taprsvp/types`
- **Agent Management**: Create and manage TAP agents with automatic key generation
- **Message Creation**: Simplified API for creating different message types
- **Message Signing**: Cryptographic signing of messages using agent keys
- **Message Verification**: Verify message signatures
- **TAP Flows**: Helper methods for common message flows (transfer, payment, connection)
- **Fluent Response API**: Chain message responses for natural conversation flow
- **DID Resolution**: Integrated DID resolver support for various DID methods
- **DID Generation**: Create DIDs using different key types (Ed25519, P-256, Secp256k1)
- **Key Management**: Manage cryptographic keys for DID operations
- **CLI Tools**: Command-line utilities for DID generation and management
- **Zero Configuration**: Automatic Ed25519 DID generation for quick setup

## Usage

```typescript
import { TAPAgent } from '@taprsvp/tap-agent';

// Create an agent (a new Ed25519 did:key will be generated automatically)
const agent = new TAPAgent({
  nickname: "My Agent",
  debug: true
});

// The agent now has a valid DID automatically
console.log(`Agent DID: ${agent.did}`); // e.g., did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

// Create a transfer message
const transfer = agent.transfer({
  asset: "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
  amount: "100.0",
  originator: {
    '@id': agent.did,
    '@type': 'Party',
    role: "originator"
  },
  beneficiary: {
    '@id': "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL",
    '@type': 'Party',
    role: "beneficiary"
  },
  agents: []
});

// Send the transfer
await transfer.send();

// Create response to the transfer
const authorization = transfer.authorize({
  settlementAddress: "eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
});

// Send the authorization
await authorization.send();
```

## DID Generation and Management

### Automatic DID Generation

The TAP-TS package now automatically generates a new Ed25519 did:key when creating a TAPAgent without specifying a DID:

```typescript
import { TAPAgent } from '@taprsvp/tap-agent';

// Create a new agent with automatic DID generation
const agent = new TAPAgent({
  nickname: "My Agent",
  debug: true
});

// The agent already has a valid DID - no extra steps required!
console.log(`Agent DID: ${agent.did}`); // e.g., did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

// You can start using the agent immediately with this DID
```

This makes it easier to get started with TAP-TS, as you no longer need to manually generate a DID before creating an agent.

### Manual DID Generation

For more control over the DID generation process, you can also manually generate DIDs with different key types:

```typescript
import { TAPAgent, DIDKeyType, createDIDKey, createDIDWeb } from '@taprsvp/tap-agent';

// Create a new agent
const agent = new TAPAgent({
  nickname: "My Agent",
  debug: true
});

// Generate a new DID with Ed25519 key type directly from the agent
const edDID = await agent.generateDID(DIDKeyType.Ed25519);
console.log(`Generated DID: ${edDID.did}`);
console.log(`Public Key (hex): ${edDID.getPublicKeyHex()}`);
console.log(`DID Document:\n${edDID.didDocument}`);

// Generate a new DID with P-256 key type
const p256DID = await agent.generateDID(DIDKeyType.P256);

// Generate a new DID with Secp256k1 key type
const secp256k1DID = await agent.generateDID(DIDKeyType.Secp256k1);

// Generate a new web DID for a domain
const webDID = await agent.generateWebDID('example.com', DIDKeyType.Ed25519);

// Alternatively, use the standalone functions
const keyDID = await createDIDKey(DIDKeyType.Ed25519);
const domainDID = await createDIDWeb('example.com', DIDKeyType.P256);
```

For detailed information on DID generation and key management, see [DID-GENERATION.md](DID-GENERATION.md).

### Using the CLI Tool

The package includes a command-line tool for DID generation:

```bash
# Install globally
npm install -g @taprsvp/tap-agent

# Or use npx directly
npx @taprsvp/tap-agent

# Use the interactive mode
tap-did interactive

# Create a did:key with Ed25519 key type
tap-did key --type Ed25519 --output my-did.json

# Create a did:web for a domain
tap-did web --domain example.com --type P256 --output web-did.json
```

## DID Resolver Support

The TAP-TS package includes built-in support for resolving DIDs using various DID methods:

```typescript
import { TAPAgent, ResolverOptions } from '@taprsvp/tap-agent';

// Configure DID resolver options
const resolverOptions: ResolverOptions = {
  resolvers: {
    key: true,    // did:key method
    ethr: true,   // did:ethr method
    pkh: true,    // did:pkh method
    web: true     // did:web method
  },
  ethrOptions: {
    networks: [
      {
        name: 'mainnet',
        rpcUrl: 'https://mainnet.infura.io/v3/YOUR_INFURA_KEY'
      }
    ]
  }
};

// Create agent with custom resolver configuration
const agent = new TAPAgent({
  nickname: "My Agent",
  resolverOptions
});
```

For detailed information on DID resolver support, see [DID-RESOLVER.md](DID-RESOLVER.md).

## Key Management

The TAP-TS package provides key management functionality for DID operations:

```typescript
import { TAPAgent, DIDKeyType } from '@taprsvp/tap-agent';

// Create an agent
const agent = new TAPAgent({
  nickname: "My Agent",
  debug: true
});

// Generate a new DID
const did = await agent.generateDID(DIDKeyType.Ed25519);

// List all DIDs managed by the agent
const dids = await agent.listDIDs();
console.log(`Managed DIDs: ${dids.join(', ')}`);

// Get information about the agent's keys
const keysInfo = agent.getKeysInfo();
console.log('Keys info:', keysInfo);

// Get information about the key manager
const keyManagerInfo = agent.getKeyManagerInfo();
console.log('Key manager info:', keyManagerInfo);
```

## Development

### Prerequisites

- Node.js (v16+)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) - Required for building the WASM module

### Building

```bash
# Install dependencies
npm install

# Build the project
npm run build
```

This will:
1. Build the WASM module from the Rust source
2. Copy the WASM files to the package
3. Compile the TypeScript code

### Testing

```bash
# Run tests
npm test

# Run tests in watch mode
npm run test:watch
```

## CLI Documentation

The package includes a CLI tool for working with DIDs:

```
Usage: did-generator [options] [command]

CLI tool for generating DIDs (Decentralized Identifiers)

Options:
  -V, --version                 output the version number
  -h, --help                    display help for command

Commands:
  interactive                   Start an interactive session to create a DID
  key [options]                 Create a did:key identifier
  web [options]                 Create a did:web identifier
  help [command]                display help for command
```

### Interactive Mode

```
tap-did interactive
```

This starts an interactive session that guides you through creating a DID by prompting for:
- DID method (key or web)
- Key type (Ed25519, P-256, or Secp256k1)
- Domain name (for web DIDs)
- Output file path to save the DID document

### Creating a did:key

```
tap-did key [options]

Options:
  -t, --type <type>      Key type (Ed25519, P256, or Secp256k1) (default: "Ed25519")
  -o, --output <file>    Output file for the DID document
  -h, --help             display help for command
```

### Creating a did:web

```
tap-did web [options]

Options:
  -d, --domain <domain>  Domain for the did:web (e.g., example.com)
  -t, --type <type>      Key type (Ed25519, P256, or Secp256k1) (default: "Ed25519")
  -o, --output <file>    Output file for the DID document
  -h, --help             display help for command
```

## License

MIT