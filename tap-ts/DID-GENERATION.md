# DID Generation and Key Management in TAP-TS

This document explains how to use the DID generation and key management functionality in the TAP-TS package.

> **New Feature**: When creating a new TAPAgent without specifying a DID, the agent will automatically generate a new Ed25519 did:key by default. This makes it easy to get started without having to manually generate a DID first.

## Table of Contents

- [Overview](#overview)
- [Supported Key Types](#supported-key-types)
- [Supported DID Methods](#supported-did-methods)
- [API Reference](#api-reference)
  - [Agent Methods](#agent-methods)
  - [Standalone Functions](#standalone-functions)
  - [CLI Tool](#cli-tool)
- [Examples](#examples)
  - [Basic DID Generation](#basic-did-generation)
  - [Web DID Generation](#web-did-generation)
  - [Key Management](#key-management)
  - [CLI Usage](#cli-usage)
- [Technical Details](#technical-details)
  - [DID Key Format](#did-key-format)
  - [DID Document Structure](#did-document-structure)
  - [Key Agreement](#key-agreement)

## Overview

The TAP-TS package provides functionality for generating and managing Decentralized Identifiers (DIDs) with various key types. This functionality is built on top of the Rust cryptographic libraries in the `tap-agent` crate, exposed through WebAssembly.

The DID generation features enable:

1. Creating DIDs with different cryptographic key types
2. Generating DIDs for different DID methods (`did:key` and `did:web`)
3. Managing keys for DID operations
4. Using keys for DIDComm message signing and verification

## Supported Key Types

The following key types are supported for DID generation:

- **Ed25519**: Edwards-curve Digital Signature Algorithm (EdDSA) using the Edwards25519 curve
- **P-256**: NIST P-256 Elliptic Curve (ECDSA secp256r1)
- **Secp256k1**: ECDSA using the secp256k1 curve (used in Bitcoin and Ethereum)

Each key type has different characteristics in terms of security, performance, and compatibility with different systems.

## Supported DID Methods

The package currently supports the following DID methods:

- **did:key**: A self-contained DID method that embeds the public key directly in the DID
- **did:web**: A DID method that uses a web domain for resolution

Future versions may add support for additional DID methods.

## API Reference

### Agent Methods

The `TAPAgent` class provides methods for working with DIDs:

#### `generateDID(keyType?: DIDKeyType): Promise<DIDKey>`

Generates a new DID with the specified key type.

```typescript
const did = await agent.generateDID(DIDKeyType.Ed25519);
```

#### `generateWebDID(domain: string, keyType?: DIDKeyType): Promise<DIDKey>`

Generates a new web DID for the specified domain with the specified key type.

```typescript
const webDID = await agent.generateWebDID('example.com', DIDKeyType.P256);
```

#### `listDIDs(): Promise<string[]>`

Lists all DIDs managed by this agent.

```typescript
const dids = await agent.listDIDs();
```

#### `getKeysInfo(): any`

Gets information about the agent's keys.

```typescript
const keysInfo = agent.getKeysInfo();
```

#### `getKeyManagerInfo(): any`

Gets information about the agent's key manager.

```typescript
const keyManagerInfo = agent.getKeyManagerInfo();
```

#### `useKeyManagerResolver(): Promise<void>`

Configures the agent to use the key manager's resolver for DIDComm operations.

```typescript
await agent.useKeyManagerResolver();
```

### Standalone Functions

The package also provides standalone functions for working with DIDs:

#### `createDIDKey(keyType?: DIDKeyType): Promise<DIDKey>`

Creates a new DID key with the specified key type.

```typescript
import { createDIDKey, DIDKeyType } from '@taprsvp/tap-agent';

const did = await createDIDKey(DIDKeyType.Ed25519);
```

#### `createDIDWeb(domain: string, keyType?: DIDKeyType): Promise<DIDKey>`

Creates a new web DID for the specified domain with the specified key type.

```typescript
import { createDIDWeb, DIDKeyType } from '@taprsvp/tap-agent';

const webDID = await createDIDWeb('example.com', DIDKeyType.P256);
```

### DIDKey Interface

The `DIDKey` interface represents a generated DID:

```typescript
interface DIDKey {
  // The DID string
  did: string;
  
  // The DID document as a JSON string
  didDocument: string;
  
  // Get the public key as a hex string
  getPublicKeyHex(): string;
  
  // Get the private key as a hex string
  getPrivateKeyHex(): string;
  
  // Get the public key as a base64 string
  getPublicKeyBase64(): string;
  
  // Get the private key as a base64 string
  getPrivateKeyBase64(): string;
  
  // Get the key type as a string
  getKeyType(): string;
}
```

### CLI Tool

The package includes a command-line tool for working with DIDs:

#### Installation

```bash
# Install globally
npm install -g @taprsvp/tap-agent

# Or use npx directly
npx @taprsvp/tap-agent
```

#### Usage

```
tap-did [command] [options]
```

Commands:
- `interactive`: Start an interactive session to create a DID
- `key`: Create a did:key identifier
- `web`: Create a did:web identifier

For detailed usage, see the [CLI Documentation](./README.md#cli-documentation) section in the main README.

## Examples

For a complete example demonstrating all DID generation features, see [did-generation-example.ts](src/examples/did-generation-example.ts).

### Basic DID Generation

#### Automatic DID Generation

```typescript
import { TAPAgent } from '@taprsvp/tap-agent';

// Create a new agent - an Ed25519 did:key will be automatically generated
const agent = new TAPAgent({
  nickname: "My Agent",
  debug: true
});

// The agent already has a valid DID
console.log(`Agent DID: ${agent.did}`); // e.g., did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

// You can start using the agent immediately with this DID
```

#### Explicit DID Generation

```typescript
import { TAPAgent, DIDKeyType } from '@taprsvp/tap-agent';

// Create a new agent
const agent = new TAPAgent({
  nickname: "My Agent",
  debug: true
});

// Generate a new DID with Ed25519 key type
const did = await agent.generateDID(DIDKeyType.Ed25519);

console.log(`Generated DID: ${did.did}`);
console.log(`Public Key (hex): ${did.getPublicKeyHex()}`);
console.log(`DID Document:\n${did.didDocument}`);

// Get the DID document as a parsed object
const didDoc = JSON.parse(did.didDocument);
```

### Web DID Generation

```typescript
import { TAPAgent, DIDKeyType } from '@taprsvp/tap-agent';

// Create a new agent
const agent = new TAPAgent({
  nickname: "My Agent",
  debug: true
});

// Generate a new web DID for example.com with P-256 key type
const webDID = await agent.generateWebDID('example.com', DIDKeyType.P256);

console.log(`Generated Web DID: ${webDID.did}`);
console.log(`DID Document:\n${webDID.didDocument}`);

// Save the DID document to a file
const fs = require('fs');
fs.writeFileSync('web-did.json', webDID.didDocument);
```

### Key Management

```typescript
import { TAPAgent, DIDKeyType } from '@taprsvp/tap-agent';

// Create a new agent
const agent = new TAPAgent({
  nickname: "My Agent",
  debug: true
});

// Generate multiple DIDs with different key types
const edDID = await agent.generateDID(DIDKeyType.Ed25519);
const p256DID = await agent.generateDID(DIDKeyType.P256);
const secp256k1DID = await agent.generateDID(DIDKeyType.Secp256k1);

// List all DIDs managed by the agent
const dids = await agent.listDIDs();
console.log(`Managed DIDs: ${dids.join(', ')}`);

// Get information about the agent's keys
const keysInfo = agent.getKeysInfo();
console.log('Keys info:', keysInfo);

// Use the DIDs for message signing
agent.id = edDID.did; // Set the agent's DID for signing

const message = agent.createMessage(MessageType.Transfer);
await agent.signMessage(message);
```

### CLI Usage

#### Interactive Mode

```bash
tap-did interactive
```

This starts an interactive session that guides you through creating a DID.

#### Creating a did:key Directly

```bash
tap-did key --type Ed25519 --output my-did.json
```

#### Creating a did:web Directly

```bash
tap-did web --domain example.com --type P256 --output web-did.json
```

## Technical Details

### DID Key Format

The format of a `did:key` identifier follows the [did:key Method Specification](https://w3c-ccg.github.io/did-method-key/). The identifier includes a multibase-encoded public key with a multicodec prefix that indicates the key type.

For example, a `did:key` using Ed25519:

```
did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
```

- `z` indicates the base58btc encoding
- The prefix bytes indicate the key type (0xed01 for Ed25519)
- The remaining bytes are the public key

### DID Document Structure

A DID document for a `did:key` includes:

- The DID as the `id` field
- Verification methods derived from the key
- Authentication capabilities using the key
- Key agreement capabilities (if applicable)

Example DID document:

```json
{
  "@context": [
    "https://www.w3.org/ns/did/v1",
    "https://w3id.org/security/suites/ed25519-2020/v1"
  ],
  "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "verificationMethod": [
    {
      "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "type": "Ed25519VerificationKey2020",
      "controller": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
    }
  ],
  "authentication": [
    "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
  ],
  "assertionMethod": [
    "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
  ],
  "keyAgreement": [
    "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc"
  ]
}
```

### Key Agreement

For Ed25519 keys, the implementation also derives an X25519 key for key agreement (encryption and decryption). This is necessary because Ed25519 is a signature scheme, not an encryption scheme.

The X25519 key is included in the DID document under the `keyAgreement` capability.