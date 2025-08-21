# @taprsvp/agent

A lightweight TypeScript/JavaScript SDK for the Transaction Authorization Protocol (TAP), providing full DIDComm v2 compatibility with WASM-powered cryptography.

[![npm version](https://badge.fury.io/js/%40taprsvp%2Fagent.svg)](https://www.npmjs.com/package/@taprsvp/agent)
[![TypeScript](https://img.shields.io/badge/%3C%2F%3E-TypeScript-%230074c1.svg)](http://www.typescriptlang.org/)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

## Features

- 🔐 **Full DIDComm v2 Support** - Compatible with Veramo and other DIDComm implementations
- 🚀 **Lightweight** - Only 3.72KB gzipped TypeScript + 272KB WASM
- 🔑 **Multiple Key Types** - Ed25519, P-256, and secp256k1
- 📦 **Zero Dependencies** - Only requires `@taprsvp/types` for TypeScript types
- 🌐 **Browser & Node.js** - Works in both environments
- ⚡ **High Performance** - WASM-powered cryptography
- 🛡️ **TAP Compliant** - Supports all TAP message types and specifications

## Installation

```bash
npm install @taprsvp/agent
```

## Quick Start

```typescript
import { TapAgent } from '@taprsvp/agent';

// Create a new agent with auto-generated keys
const agent = await TapAgent.create({ keyType: 'Ed25519' });

console.log('Agent DID:', agent.did);

// Create a TAP Transfer message
const transfer = await agent.createMessage('Transfer', {
  amount: '100.00',
  asset: 'eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
  originator: {
    '@id': agent.did,
    '@type': 'https://schema.org/Person',
    name: 'Alice Smith',
    email: 'alice@example.com'
  },
  beneficiary: {
    '@id': 'did:key:z6Mkk7yqnGF3YwTrLpqrW6PGsKci7dNqh1CjnvMbzrMerSeL',
    '@type': 'https://schema.org/Person',
    name: 'Bob Jones'
  },
  agents: []  // Agents involved in the transaction
});
transfer.to = ['did:key:z6Mkk7yqnGF3YwTrLpqrW6PGsKci7dNqh1CjnvMbzrMerSeL'];

// Pack the message for secure transmission
const packed = await agent.pack(transfer);
console.log('Packed message ready for transmission');

// Unpack received messages
const unpacked = await agent.unpack(receivedMessage);
console.log('Received:', unpacked);
```

## API Reference

### TapAgent

#### Static Methods

##### `TapAgent.create(options?: TapAgentOptions): Promise<TapAgent>`

Creates a new TAP agent with auto-generated keys.

```typescript
const agent = await TapAgent.create({
  keyType: 'Ed25519', // or 'P256' or 'secp256k1'
  resolver: customResolver // optional DID resolver
});
```

##### `TapAgent.fromPrivateKey(privateKey: string, options?: TapAgentOptions): Promise<TapAgent>`

Creates a TAP agent from an existing private key.

```typescript
import { generatePrivateKey } from '@taprsvp/agent';

const privateKey = await generatePrivateKey('Ed25519');
const agent = await TapAgent.fromPrivateKey(privateKey, {
  keyType: 'Ed25519'
});
```

#### Instance Properties

- `did: string` - The agent's DID (Decentralized Identifier)

#### Instance Methods

##### `pack(message: DIDCommMessage): Promise<PackedMessage>`

Packs a DIDComm message for secure transmission.

```typescript
const packed = await agent.pack(message);
// packed.message contains the JWS signed message
```

##### `unpack(packedMessage: string): Promise<DIDCommMessage>`

Unpacks a received DIDComm message.

```typescript
const message = await agent.unpack(packedMessage);
```

##### `createMessage<T>(type: string, body: T, options?: MessageOptions): Promise<DIDCommMessage<T>>`

Creates a new TAP message with proper structure.

```typescript
const message = await agent.createMessage('Payment', {
  amount: '25.00',
  currency: 'USD',
  merchant: { '@id': merchantDid }
}, {
  thid: 'thread-123',     // Thread ID
  pthid: 'parent-456',     // Parent thread ID  
  to: [recipientDid],      // Recipients
  expires_time: Date.now() + 3600000 // 1 hour expiry
});
```

##### `exportPrivateKey(): string`

Exports the agent's private key as a hex string.

```typescript
const privateKey = agent.exportPrivateKey();
// Store securely for later use
```

##### `resolve(did: string): Promise<DIDDocument | null>`

Resolves a DID to its DID Document.

```typescript
const didDoc = await agent.resolve('did:key:z6Mkk...');
```

##### `dispose(): void`

Cleans up WASM resources.

```typescript
agent.dispose();
```

### Utility Functions

#### `generatePrivateKey(keyType?: KeyType): Promise<string>`

Generates a new private key.

```typescript
import { generatePrivateKey } from '@taprsvp/agent';

const privateKey = await generatePrivateKey('Ed25519');
```

#### `generateUUID(): Promise<string>`

Generates a UUID v4.

```typescript
import { generateUUID } from '@taprsvp/agent';

const uuid = await generateUUID();
```

#### `isValidDID(did: string): boolean`

Validates a DID format.

```typescript
import { isValidDID } from '@taprsvp/agent';

if (isValidDID('did:key:z6Mkk...')) {
  // Valid DID
}
```

#### `isValidPrivateKey(key: string): boolean`

Validates a private key format.

```typescript
import { isValidPrivateKey } from '@taprsvp/agent';

if (isValidPrivateKey(privateKeyHex)) {
  // Valid private key
}
```

## TAP Message Types

The SDK supports all TAP message types:

### Transfer
```typescript
const transfer = await agent.createMessage('Transfer', {
  amount: '100.00',
  asset: 'eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
  originator: {
    '@id': originatorDid,
    '@type': 'https://schema.org/Person',
    name: 'Alice Smith'
  },
  beneficiary: {
    '@id': beneficiaryDid,
    '@type': 'https://schema.org/Organization',
    name: 'Example Corp',
    leiCode: '969500KN90DZLPGW6898'
  },
  memo: 'Payment for services',
  agents: [  // Optional agents involved in the transaction
    {
      '@id': agentDid,
      role: 'SettlementAddress',
      for: originatorDid
    }
  ]
});
```

### Payment
```typescript
const payment = await agent.createMessage('Payment', {
  amount: '50.00',
  currency: 'USD',
  merchant: {
    '@id': merchantDid,
    '@type': 'https://schema.org/Organization',
    name: 'Example Merchant',
    mcc: '5812',  // Restaurant
    url: 'https://merchant.example.com'
  },
  invoice: {
    invoiceNumber: 'INV-001',
    items: [{ description: 'Product', quantity: 1, unitPrice: '50.00' }],
    total: '50.00'
  }
});
```

### Connect
```typescript
const connect = await agent.createMessage('Connect', {
  constraints: {
    asset_types: ['eip155:1/erc20:*'],
    currency_types: ['USD', 'EUR'],
    transaction_limits: {
      min_amount: '10.00',
      max_amount: '10000.00'
    }
  }
});
```

### Authorize / Reject / Settle
```typescript
// Authorize a transaction
const authorize = await agent.createMessage('Authorize', {
  transaction_id: 'transfer-123',
  settlement_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7'
});

// Reject a transaction
const reject = await agent.createMessage('Reject', {
  transaction_id: 'transfer-123',
  reason: 'Insufficient funds'
});

// Settle a transaction
const settle = await agent.createMessage('Settle', {
  transaction_id: 'transfer-123',
  settlement_id: 'eip155:1:0x123...abc'
});
```

## DIDComm Standard Messages

The SDK also supports standard DIDComm messages:

### BasicMessage
```typescript
const message = await agent.createMessage('BasicMessage', {
  content: 'Hello, World!'
});
```

### TrustPing
```typescript
const ping = await agent.createMessage('TrustPing', {
  response_requested: true
});
```

### TrustPingResponse
```typescript
const pingResponse = await agent.createMessage('TrustPingResponse', {}, {
  thid: originalPingId // Reference to original ping
});
```

## Threading

Support for message threading to maintain conversation context:

```typescript
const initialMessage = await agent.createMessage('Transfer', transferData, {
  thid: 'conversation-123',  // Thread ID
  pthid: 'parent-thread-456'  // Parent thread ID
});

// Continue the conversation
const response = await agent.createMessage('Authorize', authData, {
  thid: 'conversation-123'  // Same thread ID
});
```

## Custom DID Resolver

Provide a custom DID resolver for advanced use cases:

```typescript
const customResolver = async (did: string): Promise<DIDDocument | null> => {
  // Your resolution logic here
  return didDocument;
};

const agent = await TapAgent.create({
  keyType: 'Ed25519',
  resolver: customResolver
});
```

## Browser Usage

The SDK works seamlessly in browsers:

```html
<script type="module">
  import { TapAgent } from '@taprsvp/agent';
  
  const agent = await TapAgent.create();
  console.log('Agent DID:', agent.did);
</script>
```

## Node.js Usage

Full support for Node.js environments:

```javascript
import { TapAgent } from '@taprsvp/agent';

async function main() {
  const agent = await TapAgent.create();
  console.log('Agent DID:', agent.did);
}

main();
```

## TypeScript Support

Full TypeScript support with comprehensive type definitions:

```typescript
import { TapAgent, DIDCommMessage, KeyType } from '@taprsvp/agent';
import type { Transfer, Payment } from '@taprsvp/types';

const agent = await TapAgent.create({ keyType: 'Ed25519' as KeyType });

const transfer: DIDCommMessage<Transfer> = await agent.createMessage('Transfer', {
  // TypeScript will provide full type checking here
  amount: '100.00',
  asset: 'eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
  originator: { '@id': agent.did },
  beneficiary: { '@id': recipientDid }
});
```

## Interoperability

The SDK is fully compatible with:

- ✅ Veramo DIDComm implementation
- ✅ DIDComm v2 specification
- ✅ did:key method
- ✅ JWS message format
- ✅ Standard DIDComm message types

## Performance

- TypeScript bundle: **3.72KB gzipped**
- WASM module: **272KB gzipped**
- Message operations: **< 10ms** typical
- Key generation: **< 5ms** typical

## Security

- 🔐 Private keys never leave the WASM module
- 🔑 Secure key generation using cryptographically secure random
- ✅ Standard cryptographic algorithms (Ed25519, P-256, secp256k1)
- 📦 Minimal attack surface with zero runtime dependencies

## Examples

### Key Management

```typescript
import { TapAgent, generatePrivateKey } from '@taprsvp/agent';

// Generate and store a private key
const privateKey = await generatePrivateKey('Ed25519');
localStorage.setItem('tapAgent.privateKey', privateKey);

// Later, restore the agent
const storedKey = localStorage.getItem('tapAgent.privateKey');
if (storedKey) {
  const agent = await TapAgent.fromPrivateKey(storedKey, { keyType: 'Ed25519' });
}
```

### Message Exchange

```typescript
// Alice creates and sends a transfer
const alice = await TapAgent.create();
const transfer = await alice.createMessage('Transfer', {
  amount: '100.00',
  asset: 'eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
  originator: {
    '@id': alice.did,
    '@type': 'https://schema.org/Person',
    name: 'Alice Smith'
  },
  beneficiary: {
    '@id': bobDid,
    '@type': 'https://schema.org/Person',
    name: 'Bob Jones'
  },
  agents: []  // Add any agents here if needed
});
transfer.to = [bobDid];
const packed = await alice.pack(transfer);

// Send packed.message to Bob...

// Bob receives and processes the transfer
const bob = await TapAgent.create();
const received = await bob.unpack(packed.message);
console.log('Received transfer for:', received.body.amount);

// Bob authorizes the transfer
const authorize = await bob.createMessage('Authorize', {
  transaction_id: received.id,
  settlement_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7'
}, {
  thid: received.id,
  to: [alice.did]
});
const authPacked = await bob.pack(authorize);
```

## License

Apache-2.0

## Contributing

Contributions are welcome! Please see our [contributing guidelines](https://github.com/notabene-id/tap-rs/blob/main/CONTRIBUTING.md).

## Support

For issues and questions, please use the [GitHub issue tracker](https://github.com/notabene-id/tap-rs/issues).