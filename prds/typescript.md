# TAP-TS: TypeScript Wrapper for TAP-WASM

## Overview

`tap-ts` is an idiomatic TypeScript wrapper around the Rust-based `@tap-wasm` library for implementing the Transaction Authorization Protocol (TAP). This library provides a more TypeScript-friendly API that leverages the underlying WebAssembly implementation while providing type safety, better error handling, and a more idiomatic interface for JavaScript/TypeScript developers.

## Objectives

1. Provide a seamless TypeScript experience for working with TAP
2. Provide full TypeScript type definitions for all TAP message types
3. Wrap the WASM implementation with better error handling and idiomatic TypeScript patterns
4. Enable easy creation, signing, verification, and processing of TAP messages
5. Support all message types defined in the TAP specification

## Features

### Core Features

- **Type-Safe API**: Fully typed using TypeScript interfaces from `@taprsvp/types`
- **Agent Management**: Create and manage TAP agents with key material
- **Message Creation**: Simplified API for creating different message types
- **Message Signing**: Cryptographic signing of messages using agent keys
- **Message Verification**: Verify message signatures
- **TAP Flows**: Helper methods for common message flows (transfer, payment, connection)
- **Fluent Response API**: Chain message responses for natural conversation flow

### Key Components

1. **TAPAgent**: The main entry point to the library
2. **Message Types**: TypeScript interfaces for all TAP message types
3. **Message Factories**: Helper functions for creating typed messages
4. **Response Objects**: Wrappers that enable chained responses

## API Design

### TAPAgent

The `TAPAgent` class is the primary interface for the library:

```typescript
export class TAPAgent {
  /**
   * Create a new TAP agent
   */
  constructor(options: {
    did?: string;
    nickname?: string;
    keyManager?: KeyManager;
    didResolver?: DIDResolver;
    debug?: boolean;
  }) {
    // Initialize with WASM agent
  }

  /**
   * Get the agent's DID
   */
  getDID(): string;

  /**
   * Create a transfer message
   */
  transfer(params: Omit<Transfer, '@type' | '@context'>): TransferObject;

  /**
   * Create a payment message
   */
  payment(params: Omit<Payment, '@type' | '@context'>): PaymentObject;

  /**
   * Create a connect message
   */
  connect(params: Omit<Connect, '@type' | '@context'>): ConnectionObject;

  /**
   * Process a received message
   */
  processMessage(message: TAPMessage): Promise<TAPMessage | null>;

  /**
   * Sign a message
   */
  signMessage(message: TAPMessage): Promise<TAPMessage>;

  /**
   * Verify a message
   */
  verifyMessage(message: TAPMessage): Promise<boolean>;
}
```

### Message Flows and Response Objects

The library should provide fluent interfaces for common message flows:

```typescript
// Create transfer message
const transfer = agent.transfer({
  asset: "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
  amount: "100.0",
  originator: {
    id: agent.getDID(),
    role: "originator"
  },
  beneficiary: {
    id: "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL",
    role: "beneficiary"
  }
});

// Send the message
await transfer.send();

// Create response to the transfer
const authorization = transfer.authorize({
  settlement_address: "eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e"
});

// Send the authorization
await authorization.send();
```

## Type Definitions

The library should use the type definitions from `@taprsvp/types` found in the TAP-TS project. These types will be directly imported and used for the public API.

Key interfaces include:

- `TAPMessage` - Base interface for all TAP messages
- `Transfer` - Transfer message properties
- `Payment` - Payment request properties
- `Connect` - Connection request properties
- `Authorize` - Authorization properties
- And all other message types defined in the specification

## Response Objects

Response objects should wrap the underlying message types and provide convenience methods:

```typescript
class TransferObject implements Transfer, TAPMessageMeta {
  // All Transfer properties
  // All TAPMessage metadata properties (id, to, from, etc.)

  // Response methods
  authorize(params: Omit<Authorize, '@type' | '@context'>): AuthorizeObject;
  reject(params: Omit<Reject, '@type' | '@context'>): RejectObject;
  cancel(params: Omit<Cancel, '@type' | '@context'>): CancelObject;
  
  // Send method
  send(): Promise<void>;
}
```

Similar classes should be implemented for all primary message types.

## Error Handling

The library should provide custom error types for common failure scenarios:

```typescript
class TAPError extends Error {
  constructor(message: string, public code: string) {
    super(message);
  }
}

class SigningError extends TAPError {}
class ValidationError extends TAPError {}
class NetworkError extends TAPError {}
```

## Implementation Approach

1. Create a thin wrapper around the WASM functions
2. Use TypeScript interfaces from `@taprsvp/types`
3. Implement the TAPAgent class
4. Implement fluent response objects
5. Add comprehensive error handling
6. Document with JSDoc comments

## Dependencies

- `@tap-wasm`: The WebAssembly implementation of TAP
- `@taprsvp/types`: TypeScript type definitions for TAP

## Example Usage

```typescript
import { TAPAgent } from '@taprsvp/tap-ts';

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
    id: agent.getDID(),
    role: "originator"
  },
  beneficiary: {
    id: "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL",
    role: "beneficiary"
  }
});

// Send the transfer
await transfer.send();

// Process a received message
agent.onMessage((message) => {
  if (message.message_type === "Authorize") {
    console.log("Transfer authorized!");
  }
});
```

## Development Roadmap

1. Setup TypeScript project with proper configuration
2. Create type definitions and wrappers for WASM functions
3. Implement TAPAgent class with core methods
4. Implement message object wrappers with fluent interfaces
5. Add comprehensive error handling
6. Write tests and examples
7. Document API with JSDoc and README

## Success Criteria

- Full TypeScript type safety
- Support for all TAP message types
- Intuitive, idiomatic TypeScript API
- Comprehensive test coverage
- Well-documented API with examples