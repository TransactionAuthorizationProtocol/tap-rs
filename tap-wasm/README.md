# TAP-WASM

Lightweight WebAssembly bindings for the Transaction Authorization Protocol (TAP), focusing on cryptographic operations that cannot be performed natively in JavaScript.

## Features

- **Cryptographic Operations Only**: Focused on signing/verification that requires WASM
- **DIDComm v2 Support**: Pack and unpack messages with JWS signatures  
- **Multiple Key Types**: Ed25519, P-256, and secp256k1 support
- **TypeScript Ready**: Full type definitions included
- **Optimized Size**: Minimal API surface for smaller bundles (~272KB gzipped)

## Purpose

The TAP WASM module provides a minimal set of cryptographic operations for TAP messages:
- **Key Management**: Generate, import, and export cryptographic keys
- **Message Signing**: Pack messages with DIDComm v2 JWS signatures
- **Signature Verification**: Unpack and verify signed messages

Message creation, structuring, and business logic are handled by the TypeScript SDK (`@taprsvp/agent`), keeping the WASM bundle focused and small.

## Installation

For most users, install the TypeScript SDK which includes the WASM module:

```bash
npm install @taprsvp/agent
```

For direct WASM usage (advanced):

```bash
npm install tap-wasm
```

## Basic Usage

### TypeScript/JavaScript (Recommended)

Use the TypeScript SDK for a complete TAP implementation:

```javascript
import { TapAgent } from '@taprsvp/agent';

async function main() {
  // Create agent with auto-generated keys
  const agent = await TapAgent.create({ keyType: 'Ed25519' });
  console.log('Agent DID:', agent.did);
  
  // Create message in TypeScript
  const message = await agent.createMessage('Transfer', {
    amount: '100.00',
    asset: 'USD',
    originator: { '@id': agent.did },
    beneficiary: { '@id': 'did:key:recipient' }
  });
  
  // Use WASM for cryptographic operations
  const packed = await agent.pack(message);  // Signs with WASM
  const unpacked = await agent.unpack(packed.message);  // Verifies with WASM
}

main();
```

### Direct WASM Usage (Advanced)

For direct WASM usage without the TypeScript SDK:

```javascript
import init, { WasmTapAgent } from 'tap-wasm';

async function main() {
  // Initialize WASM module
  await init();
  
  // Create agent
  const agent = new WasmTapAgent({});
  console.log('DID:', agent.get_did());
  
  // Export keys
  const privateKey = agent.exportPrivateKey();
  const publicKey = agent.exportPublicKey();
  
  // Pack a message (must be properly formatted)
  const message = {
    id: 'msg_123',
    type: 'https://tap.rsvp/schema/1.0#Transfer',
    from: agent.get_did(),
    to: ['did:key:recipient'],
    body: { /* TAP message body */ }
  };
  
  const packed = await agent.packMessage(message);
  const unpacked = await agent.unpackMessage(packed.message);
}

main();
```

## API Reference

### WasmTapAgent

The core WASM agent providing cryptographic operations.

#### Creation

```javascript
// Create with auto-generated keys
const agent = new WasmTapAgent({
  nickname: 'optional-nickname',
  debug: false
});

// Create from existing private key
const agent = await WasmTapAgent.fromPrivateKey(
  privateKeyHex,  // Hex-encoded private key
  'Ed25519'       // Key type: 'Ed25519', 'P256', or 'Secp256k1'
);
```

#### Key Management

```javascript
// Get agent's DID
const did = agent.get_did();

// Export keys
const privateKey = agent.exportPrivateKey();  // Hex string
const publicKey = agent.exportPublicKey();    // Hex string

// Get nickname
const nickname = agent.nickname();  // Optional string
```

#### Message Operations

```javascript
// Pack (sign) a message
const packedResult = await agent.packMessage(message);
// Returns: { message: string, metadata: {...} }

// Unpack (verify) a message
const unpacked = await agent.unpackMessage(
  packedMessage,      // JWS string
  expectedType        // Optional: expected message type for validation
);
// Returns: { id, type, from, to, body, ... }
```

### Utility Functions

```javascript
import { generate_uuid_v4, generatePrivateKey } from 'tap-wasm';

// Generate UUID
const uuid = generate_uuid_v4();

// Generate private key
const privateKey = generatePrivateKey('Ed25519');  // Returns hex string
```

## Key Types

Supported cryptographic key types:

- **Ed25519**: Fast, secure, recommended for most use cases
- **P256**: NIST standard, good compatibility  
- **Secp256k1**: Bitcoin/Ethereum compatible

## Integration with TypeScript SDK

The TypeScript SDK (`@taprsvp/agent`) provides:
- Message creation and structuring
- Type safety and validation
- DID resolution
- Business logic

While WASM provides:
- Cryptographic key operations
- Message signing (pack)
- Signature verification (unpack)

This separation keeps the WASM bundle small while providing a complete TAP implementation.

## Building from Source

```bash
# Clone repository
git clone https://github.com/notabene-id/tap-rs.git
cd tap-rs/tap-wasm

# Build WASM
wasm-pack build --target web

# Output in pkg/ directory
```

## Performance

- WASM module: ~272KB gzipped
- Pack operation: < 5ms typical
- Unpack operation: < 5ms typical
- Key generation: < 2ms typical

## License

MIT License