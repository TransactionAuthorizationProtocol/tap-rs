# @taprsvp/tap

The `@taprsvp/tap` library is a TypeScript SDK for the **Transaction Authorization Protocol (TAP)**—a decentralized protocol for multi-party transaction authorization. It wraps a Rust core (via WebAssembly) to combine **performance** and **security** with a **developer-friendly TypeScript API**.

## Features

- **Direct Use of TAP Types:** Uses standard message types from `@taprsvp/types` without reimplementation
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
import { TAPAgent, ensureInitialized } from '@taprsvp/tap';

// Ensure WASM is initialized
await ensureInitialized();

// Create an agent
const agent = await TAPAgent.create();
// Or with your own DID and signer:
// const agent = new TAPAgent({
//   did: 'did:web:originator.example',
//   signer: mySigner
// });

// Create a transfer message
const transfer = agent.transfer({
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
const signedTransfer = await agent.sign(transfer);

// Send the message to the recipient
// This depends on your transport layer
await sendMessage(signedTransfer, 'https://beneficiary.example/endpoint');

// On the recipient side, verify the message
const verified = await receivingAgent.verify(incomingMessage);
if (verified) {
  // Create an authorization response
  const authorize = incomingMessage.authorize(
    'eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e', // Settlement address
    'Compliance checks passed'
  );

  // Sign and send the response
  const signedAuthorize = await receivingAgent.sign(authorize);
  await sendMessage(signedAuthorize, 'https://originator.example/endpoint');
}
```

## Architecture

The library is designed around three core concepts:

1. **Standard TAP Types** - Direct use of message types from `@taprsvp/types`
2. **Message Wrappers** - Handles DIDComm envelope functionality
3. **TAP Agent** - Creates, signs, and verifies messages

This design ensures clear separation between TAP message content and DIDComm transport while maintaining a convenient API.

## API Reference

### TAPAgent

The `TAPAgent` class is your entry point for creating and signing messages:

```typescript
// Create a new agent with a provided DID and signer
const agent = new TAPAgent({
  did: 'did:web:example.com',
  signer: mySigner,
  resolver: myResolver // Optional, defaults to basic resolver
});

// Generate a new agent with a random key
const newAgent = await TAPAgent.create();

// Create messages
const transfer = agent.transfer({ /* transfer options */ });
const payment = agent.paymentRequest({ /* payment options */ });

// Create replies to existing messages
const authorize = agent.authorize(transfer, { reason: 'Approved' });
const reject = agent.reject(transfer, { reason: 'Insufficient funds' });

// Sign a message
const signedMessage = await agent.sign(message);

// Verify a message
const isValid = await agent.verify(incomingMessage);
```

### Creating Messages

The agent provides methods to create all TAP message types:

```typescript
// Create a transfer
const transfer = agent.transfer({
  asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
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
  ],
  memo: 'Test transfer',
  purpose: 'CASH'
});

// Create a payment request
const payment = agent.paymentRequest({
  amount: '50.75',
  merchant: {
    '@id': 'did:example:merchant123',
    '@type': 'Party',
    role: 'merchant'
  },
  agents: [{
    '@id': 'did:example:agent456',
    '@type': 'Agent',
    role: 'agent'
  }],
  asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
  customer: {
    '@id': 'did:example:customer789',
    '@type': 'Party',
    role: 'customer'
  }
});
```

### Reply Methods

Wrapped messages provide methods to generate appropriate replies:

```typescript
// From a Transfer
const authorize = transfer.authorize(
  'eip155:1:0x123...', // Settlement address
  'Approved'           // Reason
);

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

### MessageWrapper

The `MessageWrapper` class wraps TAP message objects in a DIDComm envelope:

```typescript
// For receiving messages
import { MessageWrapper, TapTypes } from '@taprsvp/tap';

// Create a wrapper for a received message
const transfer = new MessageWrapper<TapTypes.Transfer>(
  receivedMessage.type,
  receivedMessage.body,
  {
    id: receivedMessage.id,
    thid: receivedMessage.thid
  }
);

// Set additional properties from the received message
transfer.from = receivedMessage.from;
transfer.to = receivedMessage.to;
transfer.created_time = receivedMessage.created_time;

// Set the agent to enable reply methods
transfer.setAgent(agent);

// Now you can use reply methods
const authorize = transfer.authorize('eip155:1:0x123...', 'Approved');
```

## WebAssembly Bridge

The library automatically initializes the WASM module:

```typescript
// Auto-initialization happens when the module is imported
import { ensureInitialized } from '@taprsvp/tap';

// You can explicitly wait for initialization if needed
await ensureInitialized();
```

## Complete Transfer Flow Example

```typescript
import { TAPAgent, ensureInitialized } from '@taprsvp/tap';

// Ensure WASM is initialized
await ensureInitialized();

// Create agents for both parties
const originatorAgent = await TAPAgent.create();
const beneficiaryAgent = await TAPAgent.create();

// Originator creates and signs a transfer
const transfer = originatorAgent.transfer({
  asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
  amount: '100.00',
  originator: {
    '@id': originatorAgent.getDID(),
    '@type': 'Party',
    role: 'originator'
  },
  beneficiary: {
    '@id': beneficiaryAgent.getDID(),
    '@type': 'Party',
    role: 'beneficiary'
  },
  agents: [
    { '@id': originatorAgent.getDID(), '@type': 'Agent' },
    { '@id': beneficiaryAgent.getDID(), '@type': 'Agent' }
  ]
});

const signedTransfer = await originatorAgent.sign(transfer);
// Send to beneficiary...

// Beneficiary verifies and authorizes
const isValid = await beneficiaryAgent.verify(transfer);
if (isValid) {
  const authorize = transfer.authorize('eip155:1:0x123abc...', 'Compliance checks passed');
  const signedAuthorize = await beneficiaryAgent.sign(authorize);
  // Send back to originator...
}

// Originator settles on-chain then confirms
// Execute settlement transaction on-chain first...
const settle = transfer.settle('eip155:1/tx/0x4a563af33c4871b51a8b108aa2fe1dd5280a30df');
const signedSettle = await originatorAgent.sign(settle);
// Send to beneficiary...
```

## License

MIT