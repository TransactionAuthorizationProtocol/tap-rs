# @taprsvp/tap

The `@taprsvp/tap` library is a TypeScript SDK for the **Transaction Authorization Protocol (TAP)**—a decentralized protocol for multi-party transaction authorization. It wraps a Rust core (via WebAssembly) to combine **performance** and **security** with a **developer-friendly TypeScript API**.

## Features

- **Complete TAP Model Support:** All TAP message types and data structures as defined in the specification
- **WebAssembly Performance:** Core cryptographic operations and message handling powered by Rust/WASM
- **TypeScript Type Safety:** Fully typed API with strict interfaces for all message bodies
- **Modular Agent Architecture:** Flexible identity and cryptography primitives
- **Fluent Message API:** Intuitive builder pattern for creating and chaining messages
- **Cross-Environment Compatibility:** Works in browsers, Node.js, and other JavaScript runtimes

## Installation

```bash
# npm
npm install @taprsvp/tap

# yarn
yarn add @taprsvp/tap

# pnpm
pnpm add @taprsvp/tap
```

## Quick Start

```typescript
import { TAPAgent, Transfer } from '@taprsvp/tap';

// Initialize an agent with your DID and signer
const agent = new TAPAgent({
  did: 'did:web:originator.example',
  signer: mySigner
});

// Create a transfer message
const transfer = new Transfer({
  asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', // USDC
  amount: '100.50',
  originator: { 
    '@id': 'did:web:originator.example', 
    '@type': 'Party',
    role: 'originator' 
  },
  beneficiary: { 
    '@id': 'did:web:beneficiary.example', 
    '@type': 'Party',
    role: 'beneficiary' 
  },
  agents: [
    { '@id': 'did:web:originator.example', '@type': 'Agent' },
    { '@id': 'did:web:beneficiary.example', '@type': 'Agent' }
  ]
});

// Sign the message with your agent
await agent.sign(transfer);

// Send the message to the recipient
// This depends on your transport layer
await sendMessage(transfer, 'https://beneficiary.example/endpoint');

// On the recipient side, verify the message
const verified = await receivingAgent.verify(incomingMessage);
if (verified) {
  // Create an authorization response
  const authorize = incomingMessage.authorize(
    'eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e', // Settlement address
    'Compliance checks passed'
  );
  
  // Sign and send the response
  await receivingAgent.sign(authorize);
  await sendMessage(authorize, 'https://originator.example/endpoint');
}
```

## API Reference

### TAPAgent

The `TAPAgent` class is your entry point for cryptographic operations:

```typescript
// Create a new agent with a provided DID and signer
const agent = new TAPAgent({
  did: 'did:web:example.com',
  signer: mySigner,
  resolver: myResolver // Optional, defaults to basic resolver
});

// Generate a new agent with a random key
const newAgent = await TAPAgent.create();

// Sign a message
const signedMessage = await agent.sign(message);

// Verify a message
const isValid = await agent.verify(incomingMessage);
```

### Message Classes

The library provides classes for all TAP message types:

#### Starter Messages

```typescript
// Create a transfer
const transfer = new Transfer({
  asset: 'eip155:1/slip44:60',
  amount: '1.23',
  originator: { did: 'did:web:alice.bank', '@type': 'Party' },
  // ...other required fields
});

// Create a payment request
const payment = new Payment({
  asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
  amount: '100.00',
  merchant: { 
    '@id': 'did:web:merchant.example', 
    '@type': 'Party',
    name: 'Example Store', 
    mcc: '5812'
  },
  // ...other required fields
});

// Create a connect request
const connect = new Connect({
  for: 'did:web:party.example',
  constraints: {
    purposes: ['CASH'],
    limits: {
      per_transaction: '1000',
      daily: '5000',
      currency: 'USD'
    }
  }
});
```

#### Reply Methods

Each message provides methods to generate appropriate replies:

```typescript
// From a Transfer
const authorize = transfer.authorize('eip155:1:0x123...', 'Approved');
const reject = transfer.reject('Insufficient funds');
const settle = transfer.settle('eip155:1/tx/0x4a56...', '1.23');
const cancel = transfer.cancel('User requested');
const revert = transfer.revert({
  settlementAddress: 'eip155:1:0x456...',
  reason: 'Compliance requirement'
});

// From a Payment
const complete = payment.complete('eip155:1:0x789...', '95.50');
const settle = payment.settle('eip155:1/tx/0xabc...');
const cancel = payment.cancel('Customer declined');
```

## WebAssembly Bridge

The library automatically initializes the WASM module:

```typescript
// Auto-initialization happens when the module is imported
import { ensureInitialized } from '@taprsvp/tap';

// You can explicitly wait for initialization if needed
await ensureInitialized();
```

## Examples

### Complete Transfer Flow

```typescript
// Originator creates and signs a transfer
const transfer = new Transfer({
  asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
  amount: '100.00',
  originator: { 
    '@id': originatorDid, 
    '@type': 'Party', 
    role: 'originator' 
  },
  beneficiary: { 
    '@id': beneficiaryDid, 
    '@type': 'Party', 
    role: 'beneficiary' 
  },
  agents: [
    { '@id': originatorDid, '@type': 'Agent' },
    { '@id': beneficiaryDid, '@type': 'Agent' }
  ]
});

await originatorAgent.sign(transfer);
// Send to beneficiary...

// Beneficiary verifies and authorizes
await beneficiaryAgent.verify(transfer);
const authorize = transfer.authorize('eip155:1:0x123abc...');
await beneficiaryAgent.sign(authorize);
// Send back to originator...

// Originator settles on-chain then confirms
// Execute settlement transaction on-chain first...
const settle = transfer.settle('eip155:1/tx/0x4a563af33c4871b51a8b108aa2fe1dd5280a30df');
await originatorAgent.sign(settle);
// Send to beneficiary...
```

## License

MIT