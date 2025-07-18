# @taprsvp/tap-agent

TypeScript wrapper for TAP-WASM library implementing the Transaction Authorization Protocol (TAP).

## Installation

```bash
npm install @taprsvp/tap-agent
```

## Features

- **Type-Safe API**: Fully typed using TypeScript interfaces with modern Agent/Party separation
- **Agent Management**: Create and manage TAP agents with automatic key generation
- **Message Creation**: Simplified API for creating different message types using TAIP-compliant structures
- **Message Packing**: Pack messages for secure transmission
- **Message Unpacking**: Unpack and verify received messages
- **TAP Flows**: Helper methods for common message flows (transfer, payment, connection)
- **Fluent Message API**: Chain method calls for easy message creation
- **DID Resolution**: Integrated DID resolver support for various DID methods
- **Key Management**: Based on the underlying Rust tap-agent implementation
- **WASM Integration**: Uses the core tap-agent code via WebAssembly for efficiency and code sharing
- **CLI Tools**: Command-line utilities for DID generation and management
- **Zero Configuration**: Automatic DID generation for quick setup
- **TAIP Compliance**: Implements TAIP-5 (Agent) and TAIP-6 (Party) specifications

## Core Concepts

### Agent vs Party (TAIP-5/TAIP-6)

This library implements the modern TAP specification with proper separation between Agents and Parties:

- **Party**: A real-world entity (legal or natural person) involved in a transaction
  - Examples: Individual users, companies, organizations
  - Identified by any DID or IRI
  - Contains optional metadata like country codes, LEI codes, etc.

- **Agent**: A service that executes transactions on behalf of one or more parties
  - Examples: Exchanges, custodial wallets, DeFi protocols, bridges
  - Identified by any DID or IRI
  - Must have a `role` field (e.g., "SettlementAddress", "Exchange")
  - Must specify which party/parties it acts `for`
  - Can have policies that define operational constraints

```typescript
// Party example (real-world entity)
const party: Party = {
  '@id': 'did:example:alice',
  'https://schema.org/addressCountry': 'US'
};

// Agent example (service acting for a party)
const agent: Agent = {
  '@id': 'did:web:exchange.example.com',
  role: 'SettlementAddress',
  for: 'did:example:alice', // Acts for Alice
  policies: [/* optional policies */]
};
```

## Usage

### Creating an Agent

```typescript
import { TAPAgent, Agent, Party } from '@taprsvp/tap-agent';

// Create an agent (a new DID will be generated automatically)
// IMPORTANT: Always use the static create() method which properly initializes WASM
const agent = await TAPAgent.create({
  nickname: "My Agent",
  debug: true
});

// The agent now has a valid DID automatically
console.log(`Agent DID: ${agent.did}`); // e.g., did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

// INCORRECT USAGE - DO NOT USE THE CONSTRUCTOR DIRECTLY
// const agent = new TAPAgent({ nickname: "My Agent" }); // Will cause errors when WASM isn't initialized!
```

### Creating and Sending Messages

```typescript
// Create a transfer message using Agent and Party types (TAIP-5/TAIP-6)
const transfer = agent.transfer({
  asset: "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
  amount: "100.0",
  originator: {
    '@id': agent.did // Party - real-world entity
  },
  beneficiary: {
    '@id': "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL" // Party - real-world entity
  },
  agents: [{
    '@id': "did:web:exchange.example.com",
    role: "SettlementAddress", // Agent role (TAIP-5)
    for: agent.did // Which party this agent acts for
  }]
});

// Pack the message for transmission
const packedResult = await transfer.pack();
console.log("Packed message:", packedResult.message);

// In a real application, you would send this packed message to the recipient
// ...

// The recipient would create their agent with the async factory method
const recipientAgent = await TAPAgent.create({
  nickname: "Recipient Agent",
  debug: true
});

// The recipient would then unpack the message
const unpackedMessage = await recipientAgent.unpackMessage(packedResult.message);
console.log("Unpacked message:", unpackedMessage);

// Create a response to the transfer
const authorization = recipientAgent.authorize({
  reason: "Transfer authorized",
  settlementAddress: "eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
});

// Set the thread ID to link it to the original message
authorization.setThreadId(unpackedMessage.id);

// Pack the authorization message for sending back
const packedAuthorization = await authorization.pack();
```

## Message Types

The library supports all standard TAP message types with proper Agent/Party separation (TAIP-5/TAIP-6):

### Transfer Messages

```typescript
const transfer = agent.transfer({
  asset: "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
  amount: "100.0",
  originator: {
    '@id': agent.did // Party (real-world entity)
  },
  beneficiary: {
    '@id': recipientDid // Party (real-world entity)
  },
  memo: "Payment for services",
  agents: [{
    '@id': "did:web:exchange.example.com",
    role: "SettlementAddress", // Agent role (TAIP-5)
    for: agent.did // Which party this agent acts for
  }]
});
```

### Payment Messages

```typescript
const payment = agent.payment({
  asset: "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
  amount: "50.0",
  merchant: {
    '@id': agent.did // Party (merchant entity)
  },
  customer: {
    '@id': customerDid // Party (customer entity)
  },
  invoice: "INV-12345",
  expiry: new Date(Date.now() + 3600000).toISOString(), // 1 hour from now
  agents: [{
    '@id': "did:web:payment-processor.example.com",
    role: "Exchange", // Agent role (TAIP-5)
    for: agent.did // Acts for the merchant
  }]
});
```

### Connect Messages

```typescript
const connect = agent.connect({
  agent: {
    '@id': "did:web:connector.example.com",
    role: "connector", // Agent role (TAIP-5)
    for: agent.did // Acts for this party
  },
  for: "https://tap.company/services/compliance",
  constraints: {
    supportedAssets: ["eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f"],
    minimumAmount: "10.0",
    maximumAmount: "10000.0"
  }
});
```

### Other Message Types

The library also supports these message types:

- `authorize` - Authorize a transfer or payment
- `reject` - Reject a transfer or payment
- `settle` - Confirm settlement of a transfer
- `cancel` - Cancel a transfer or payment
- `revert` - Request reversion of a settlement

## Message Handling

Register handlers for different message types:

```typescript
// Register a handler for transfer messages
// Note: Make sure to register handlers AFTER agent creation is complete
agent.onMessage("Transfer", async (message) => {
  console.log("Received transfer message:", message);

  // Process the message and return a response
  const response = agent.authorize({
    reason: "Transfer authorized",
    settlementAddress: "0x123..."
  });

  // Link the response to the original message
  response.setThreadId(message.id);

  return response.toJSON();
});

// Process a received message
const result = await agent.processMessage(receivedMessage);
```

## Type Definitions

The library exports comprehensive TypeScript types for all TAP concepts:

```typescript
import {
  TAPAgent,
  Agent,
  Party,
  TapParticipant,
  TAPMessage,
  Transfer,
  Payment,
  Connect,
  DID,
  Asset
} from '@taprsvp/tap-agent';
```

### Key Types

- `Agent`: Service executing transactions (TAIP-5)
- `Party`: Real-world entity (TAIP-6)
- `TapParticipant`: Base interface for Agent and Party
- `TAPMessage`: Generic message structure
- `Transfer`, `Payment`, `Connect`: Specific message body types
- `DID`: DID string type
- `Asset`: Asset identifier string type

## Advanced Usage

### Accessing the WASM Agent

For advanced use cases, you can access the underlying WASM agent:

```typescript
const wasmAgent = agent.getWasmAgent();
// Now you can use the WASM agent directly for operations
// not covered by the TypeScript wrapper
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

#### Testing Considerations

When writing tests, remember that WASM initialization requires special handling:

```typescript
import { TAPAgent } from '../agent';
import { beforeEach, describe, expect, it } from 'vitest';

describe('TAPAgent', () => {
  let agent: TAPAgent;

  beforeEach(async () => {
    // IMPORTANT: Always use the async create() method in tests
    agent = await TAPAgent.create({ nickname: "Test Agent" });
  });

  it('should have a valid DID', () => {
    expect(agent.did).toBeDefined();
    expect(agent.did).toMatch(/^did:key:/);
  });

  // More tests...
});
```

## License

MIT
