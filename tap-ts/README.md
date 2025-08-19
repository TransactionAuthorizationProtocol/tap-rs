# @taprsvp/agent

TypeScript wrapper for TAP WASM Agent - Browser-optimized message packing/unpacking with flexible key management.

[![npm version](https://badge.fury.io/js/%40taprsvp%2Fagent.svg)](https://www.npmjs.com/package/@taprsvp/agent)
[![TypeScript](https://img.shields.io/badge/%3C%2F%3E-TypeScript-%230074c1.svg)](http://www.typescriptlang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- 🚀 **Browser-First**: Optimized for browser environments with minimal bundle size
- 🔐 **Flexible Key Management**: Export/import private keys for custom storage solutions
- 📦 **Type Safety**: Full TypeScript support with comprehensive type definitions
- 🔌 **DID Resolution**: Built-in DID:key support with pluggable resolver interface  
- ⚡ **High Performance**: WASM-powered core with efficient message operations
- 🛡️ **TAP Compliant**: Supports all TAP message types and specifications

## Installation

```bash
npm install @taprsvp/agent
```

## Quick Start

```typescript
import { TapAgent } from '@taprsvp/agent';

// Create a new agent
const agent = await TapAgent.create({
  keyType: 'Ed25519',
  nickname: 'my-agent'
});

console.log('Agent DID:', agent.did);

// Create and pack a message
const message = agent.createMessage('Transfer', {
  amount: '100.0',
  asset: 'eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7',
  originator: { '@id': agent.did },
  beneficiary: { '@id': 'did:key:recipient' },
  agents: []
});

const packed = await agent.pack(message);
console.log('Packed message ready for transmission');
```

## Key Management

### Generate New Keys

```typescript
import { TapAgent, generatePrivateKey } from '@taprsvp/agent';

// Generate a new private key
const privateKey = generatePrivateKey('Ed25519');

// Create agent from the key
const agent = await TapAgent.fromPrivateKey(privateKey);
```

### Export for Storage

```typescript
// Export private key for browser storage
const privateKey = agent.exportPrivateKey();

// Store in IndexedDB, localStorage, etc.
localStorage.setItem('tapAgent.privateKey', privateKey);

// Restore from storage
const restoredAgent = await TapAgent.fromPrivateKey(
  localStorage.getItem('tapAgent.privateKey')!
);
```

### Supported Key Types

- **Ed25519**: Recommended for most use cases
- **P256**: NIST P-256 curve for enterprise environments  
- **secp256k1**: Bitcoin/Ethereum compatible keys

## Message Operations

### Creating Messages

```typescript
// Basic transfer message
const transfer = agent.createMessage('Transfer', {
  amount: '50.0',
  asset: 'USD',
  originator: { '@id': agent.did },
  beneficiary: { '@id': recipientDid },
  agents: []
});

// Message with thread reference
const authorization = agent.createMessage('Authorize', {
  transaction_id: transferId,
  settlement_address: 'ethereum:0x...'
}, {
  to: [originatorDid],
  thid: transferId // Thread reference
});
```

### Packing and Unpacking

```typescript
// Pack for transmission
const packed = await agent.pack(message, {
  to: [recipientDid],
  expires_time: Date.now() + 3600000 // 1 hour
});

// Unpack received message
const unpacked = await agent.unpack(packedMessage, {
  expectedType: 'Transfer',
  maxAge: 3600 // Max age in seconds
});
```

## DID Resolution

### Built-in DID:key Support

```typescript
// Resolves DID:key methods automatically
const didDoc = await agent.resolveDID('did:key:z6MkhaXgBZDvotDkL5...');
```

### Custom DID Resolver

```typescript
import { Resolver } from 'did-resolver';
import { getResolver as getWebResolver } from 'web-did-resolver';
import { getResolver as getEthrResolver } from 'ethr-did-resolver';

const didResolver = new Resolver({
  ...getWebResolver(),
  ...getEthrResolver({ infuraProjectId: 'your-project-id' })
});

const agent = await TapAgent.create({ didResolver });

// Now supports did:web and did:ethr
const webDidDoc = await agent.resolveDID('did:web:example.com');
const ethrDidDoc = await agent.resolveDID('did:ethr:0x...');
```

## Supported Message Types

All TAP message types are supported:

- `Transfer` - Transaction proposals (TAIP-3)
- `Payment` - Payment requests (TAIP-14)  
- `Authorize` - Authorization responses (TAIP-4)
- `Reject` - Rejection responses (TAIP-4)
- `Settle` - Settlement notifications (TAIP-6)
- `Cancel` - Cancellation messages (TAIP-5)
- `Revert` - Revert requests (TAIP-12)
- `Connect` - Connection requests (TAIP-15)
- `Escrow` - Escrow requests (TAIP-17)
- `Capture` - Escrow capture (TAIP-17)
- Plus agent and policy management messages

## Error Handling

```typescript
import { 
  TapAgentError, 
  TapAgentKeyError, 
  TapAgentMessageError 
} from '@taprsvp/agent';

try {
  const agent = await TapAgent.fromPrivateKey(invalidKey);
} catch (error) {
  if (error instanceof TapAgentKeyError) {
    console.error('Key error:', error.message);
  } else if (error instanceof TapAgentMessageError) {
    console.error('Message error:', error.message);
  }
}
```

## Advanced Usage

### Message Attachments

```typescript
const messageWithAttachment = agent.createMessage('Payment', {
  amount: '100.0',
  invoice_id: 'inv-123'
});

messageWithAttachment.attachments = [{
  id: 'invoice-pdf',
  filename: 'invoice_123.pdf',
  media_type: 'application/pdf',
  data: {
    encoding: 'base64',
    content: base64PdfData
  }
}];

const packed = await agent.pack(messageWithAttachment);
```

### Agent Metrics

```typescript
const metrics = agent.getMetrics();
console.log({
  messagesPacked: metrics.messagesPacked,
  messagesUnpacked: metrics.messagesUnpacked,
  uptime: metrics.uptime
});
```

### Resource Cleanup

```typescript
// Always dispose when done to free WASM memory
agent.dispose();
```

## Bundle Size

This package is optimized for minimal bundle size:

- **WASM Binary**: < 500KB
- **TypeScript Bundle**: < 50KB gzipped
- **Tree-shakable**: Only import what you use

## Browser Support

- Chrome 80+
- Firefox 75+  
- Safari 13+
- Edge 80+

## TypeScript Integration

Full TypeScript support with generic message types:

```typescript
interface CustomTransfer {
  amount: string;
  currency: 'USD' | 'EUR' | 'GBP';
  reference: string;
}

const message = agent.createMessage<CustomTransfer>('Transfer', {
  amount: '100.0',
  currency: 'USD',
  reference: 'TXN-001'
});

// TypeScript enforces the body structure
const packed = await agent.pack(message);
const unpacked = await agent.unpack<CustomTransfer>(packedMessage);
```

## API Reference

See [API Documentation](./docs/api.md) for complete API reference.

## License

MIT License - see [LICENSE](./LICENSE) file for details.

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines.

## Security

For security concerns, see our [Security Policy](../SECURITY.md).