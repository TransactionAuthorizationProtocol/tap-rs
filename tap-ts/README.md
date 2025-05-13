# @taprsvp/tap-agent

TypeScript wrapper for TAP-RS library implementing the Transaction Authorization Protocol (TAP).

## Installation

```bash
npm install @taprsvp/tap-agent
```

## Features

- **Type-Safe API**: Fully typed using TypeScript interfaces from `@taprsvp/types`
- **Agent Management**: Create and manage TAP agents with key material
- **Message Creation**: Simplified API for creating different message types
- **Message Signing**: Cryptographic signing of messages using agent keys
- **Message Verification**: Verify message signatures
- **TAP Flows**: Helper methods for common message flows (transfer, payment, connection)
- **Fluent Response API**: Chain message responses for natural conversation flow
- **DID Resolution**: Integrated DID resolver support for various DID methods

## Usage

```typescript
import { TAPAgent } from '@taprsvp/tap-agent';

// Create an agent
const agent = new TAPAgent({
  nickname: "My Agent",
  debug: true
});

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

## License

MIT
