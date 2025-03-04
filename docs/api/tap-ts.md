# tap-ts API Reference

The `tap-ts` library provides a TypeScript wrapper around the WebAssembly (WASM) bindings for the TAP protocol. It offers a more idiomatic TypeScript API and handles the complexities of working with the WASM module.

## Installation

```bash
npm install @notabene/tap-ts
```

## Core Classes

### `Agent`

The main TAP Agent implementation for TypeScript.

```typescript
class Agent {
  /**
   * Create a new Agent with the given name and key pair
   */
  static async create(name: string, keyPair?: KeyPair): Promise<Agent>;
  
  /**
   * Create a new Agent from a JSON Web Key (JWK)
   */
  static async fromJwk(name: string, jwk: JsonWebKey): Promise<Agent>;
  
  /**
   * Get the agent's DID
   */
  did(): string;
  
  /**
   * Get the agent's name
   */
  name(): string;
  
  /**
   * Get the agent's key pair
   */
  keyPair(): KeyPair;
  
  /**
   * Create a TAP transfer message
   */
  async createTransfer(params: {
    asset: string;
    amount: string;
    beneficiaryDid: string;
    memo?: string;
    metadata?: Record<string, string>;
  }): Promise<Message>;
  
  /**
   * Create a TAP authorize message
   */
  async createAuthorize(params: {
    transferId: string;
    note?: string;
    metadata?: Record<string, string>;
  }): Promise<Message>;
  
  /**
   * Create a TAP reject message
   */
  async createReject(params: {
    transferId: string;
    code: string;
    description?: string;
    metadata?: Record<string, string>;
  }): Promise<Message>;
  
  /**
   * Create a TAP receipt message
   */
  async createReceipt(params: {
    transferId: string;
    settlementId?: string;
    note?: string;
    metadata?: Record<string, string>;
  }): Promise<Message>;
  
  /**
   * Create a TAP settlement message
   */
  async createSettlement(params: {
    transferId: string;
    settlementId: string;
    status: string;
    note?: string;
    metadata?: Record<string, string>;
  }): Promise<Message>;
  
  /**
   * Process an incoming TAP message
   */
  async processMessage(message: Message): Promise<Message | null>;
  
  /**
   * Set a message handler for a specific message type
   */
  setMessageHandler(
    messageType: string,
    handler: (message: Message) => Promise<Message | null>
  ): void;
  
  /**
   * Encrypt a message for the recipient(s)
   */
  async encryptMessage(message: Message, to: string[]): Promise<Message>;
  
  /**
   * Decrypt a message
   */
  async decryptMessage(message: Message): Promise<Message>;
  
  /**
   * Export this agent's key pair as a JWK
   */
  async exportJwk(): Promise<JsonWebKey>;
}
```

### `KeyPair`

Represents a cryptographic key pair for DID operations.

```typescript
class KeyPair {
  /**
   * Generate a new Ed25519 key pair
   */
  static async generateEd25519(): Promise<KeyPair>;
  
  /**
   * Generate a new X25519 key pair
   */
  static async generateX25519(): Promise<KeyPair>;
  
  /**
   * Create a KeyPair from a JSON Web Key (JWK)
   */
  static async fromJwk(jwk: JsonWebKey): Promise<KeyPair>;
  
  /**
   * Get the DID:key representation of this key pair
   */
  getDidKey(): string;
  
  /**
   * Get the public key as a Uint8Array
   */
  getPublicKey(): Uint8Array;
  
  /**
   * Export this key pair as a JWK
   */
  async exportJwk(): Promise<JsonWebKey>;
  
  /**
   * Sign data with this key pair
   */
  async sign(data: Uint8Array): Promise<Uint8Array>;
  
  /**
   * Verify a signature with this key pair
   */
  async verify(data: Uint8Array, signature: Uint8Array): Promise<boolean>;
}
```

### `Message`

Represents a TAP message.

```typescript
interface Message {
  /**
   * Unique identifier for this message
   */
  id: string;
  
  /**
   * The type of this message (e.g., "TAP_TRANSFER")
   */
  type: string;
  
  /**
   * The DID of the sender
   */
  from?: string;
  
  /**
   * The DIDs of the recipients
   */
  to?: string[];
  
  /**
   * When this message was created (ISO 8601 string)
   */
  created_time?: string;
  
  /**
   * When this message expires (ISO 8601 string)
   */
  expires_time?: string;
  
  /**
   * The message body
   */
  body: any;
  
  /**
   * Additional attachments
   */
  attachments?: any[];
  
  /**
   * Set the sender of this message
   */
  setFrom(from: string): Message;
  
  /**
   * Set the recipients of this message
   */
  setTo(to: string[]): Message;
  
  /**
   * Set the created time of this message
   */
  setCreatedTime(time: string): Message;
  
  /**
   * Set the expiration time of this message
   */
  setExpiresTime(time: string): Message;
}
```

## Message Type Interfaces

### `TransferBody`

```typescript
interface TransferBody {
  /**
   * The asset being transferred (CAIP-19 Asset ID)
   */
  asset: string;
  
  /**
   * The originator of the transfer
   */
  originator: Agent;
  
  /**
   * The beneficiary of the transfer
   */
  beneficiary?: Agent;
  
  /**
   * The amount being transferred as a string
   */
  amount: string;
  
  /**
   * Additional agents involved in the transfer
   */
  agents?: Agent[];
  
  /**
   * Optional settlement ID
   */
  settlement_id?: string;
  
  /**
   * Optional memo describing the purpose of the transfer
   */
  memo?: string;
  
  /**
   * Additional metadata as key-value pairs
   */
  metadata?: Record<string, string>;
}
```

### `AuthorizeBody`

```typescript
interface AuthorizeBody {
  /**
   * The ID of the transfer being authorized
   */
  transfer_id: string;
  
  /**
   * Optional note providing context for the authorization
   */
  note?: string;
  
  /**
   * Additional metadata as key-value pairs
   */
  metadata?: Record<string, string>;
}
```

### `RejectBody`

```typescript
interface RejectBody {
  /**
   * The ID of the transfer being rejected
   */
  transfer_id: string;
  
  /**
   * Reason code for the rejection
   */
  code: string;
  
  /**
   * Optional detailed description of the rejection reason
   */
  description?: string;
  
  /**
   * Additional metadata as key-value pairs
   */
  metadata?: Record<string, string>;
}
```

### `ReceiptBody`

```typescript
interface ReceiptBody {
  /**
   * The ID of the transfer this receipt is for
   */
  transfer_id: string;
  
  /**
   * Optional settlement ID reference
   */
  settlement_id?: string;
  
  /**
   * Optional note providing context for the receipt
   */
  note?: string;
  
  /**
   * Additional metadata as key-value pairs
   */
  metadata?: Record<string, string>;
}
```

### `SettlementBody`

```typescript
interface SettlementBody {
  /**
   * The ID of the transfer this settlement is for
   */
  transfer_id: string;
  
  /**
   * The settlement identifier (often a transaction hash)
   */
  settlement_id: string;
  
  /**
   * Status of the settlement (e.g., "pending", "completed", "failed")
   */
  status: string;
  
  /**
   * Optional note providing context for the settlement
   */
  note?: string;
  
  /**
   * Additional metadata as key-value pairs
   */
  metadata?: Record<string, string>;
}
```

## Constants

```typescript
/**
 * TAP Transfer message type
 */
export const TAP_TRANSFER_TYPE = "TAP_TRANSFER";

/**
 * TAP Authorize message type
 */
export const TAP_AUTHORIZE_TYPE = "TAP_AUTHORIZE";

/**
 * TAP Reject message type
 */
export const TAP_REJECT_TYPE = "TAP_REJECT";

/**
 * TAP Receipt message type
 */
export const TAP_RECEIPT_TYPE = "TAP_RECEIPT";

/**
 * TAP Settlement message type
 */
export const TAP_SETTLEMENT_TYPE = "TAP_SETTLEMENT";
```

## Utility Functions

```typescript
/**
 * Initialize the TAP library
 * This should be called before using any other functions
 */
export async function init(): Promise<void>;

/**
 * Parse an AssetId from a string
 */
export function parseAssetId(assetId: string): {
  chainNamespace: string;
  chainReference: string;
  assetNamespace: string;
  assetReference: string;
};

/**
 * Create an Agent from a mnemonic phrase
 */
export async function agentFromMnemonic(
  name: string,
  mnemonic: string,
  path: string = "m/44'/0'/0'/0/0"
): Promise<Agent>;

/**
 * Generate a random mnemonic phrase
 */
export function generateMnemonic(wordCount: 12 | 24 = 12): string;
```

## Examples

### Basic Usage

```typescript
import { Agent, KeyPair, init, TAP_TRANSFER_TYPE } from '@notabene/tap-ts';

async function example() {
  // Initialize the TAP library
  await init();
  
  // Create an agent
  const keyPair = await KeyPair.generateEd25519();
  const alice = await Agent.create("Alice", keyPair);
  
  console.log(`Created agent with DID: ${alice.did()}`);
  
  // Export the key for later use
  const jwk = await alice.exportJwk();
  localStorage.setItem('alice_key', JSON.stringify(jwk));
  
  // Later, restore the agent
  const savedJwk = JSON.parse(localStorage.getItem('alice_key'));
  const restoredAlice = await Agent.fromJwk("Alice", savedJwk);
}
```

### Implementing a TAP Flow

```typescript
import { Agent, KeyPair, init, TAP_TRANSFER_TYPE, TAP_AUTHORIZE_TYPE } from '@notabene/tap-ts';

async function implementTapFlow() {
  // Initialize the TAP library
  await init();
  
  // Create two agents
  const alice = await Agent.create("Alice");
  const bob = await Agent.create("Bob");
  
  // Bob sets up a handler for transfer messages
  bob.setMessageHandler(TAP_TRANSFER_TYPE, async (message) => {
    console.log("Bob received a transfer request:", message);
    
    // Create an authorize response
    const authorizeMsg = await bob.createAuthorize({
      transferId: message.id,
      note: "Transfer authorized by Bob"
    });
    
    return authorizeMsg;
  });
  
  // Alice sets up a handler for authorize messages
  alice.setMessageHandler(TAP_AUTHORIZE_TYPE, async (message) => {
    console.log("Alice received an authorization:", message);
    
    // Create a receipt
    const receiptMsg = await alice.createReceipt({
      transferId: message.body.transfer_id,
      note: "Receipt confirmed by Alice"
    });
    
    return receiptMsg;
  });
  
  // Alice creates and sends a transfer
  const transferMsg = await alice.createTransfer({
    asset: "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F",
    amount: "100.0",
    beneficiaryDid: bob.did(),
    memo: "Payment for services"
  });
  
  console.log("Alice created transfer:", transferMsg);
  
  // Process the message flow
  const bobResponse = await bob.processMessage(transferMsg);
  
  if (bobResponse) {
    console.log("Bob authorized the transfer");
    
    const aliceReceipt = await alice.processMessage(bobResponse);
    
    if (aliceReceipt) {
      console.log("Alice confirmed receipt");
      await bob.processMessage(aliceReceipt);
    }
  }
}
```

### Working with Multiple Agents

```typescript
import { Agent, KeyPair, init } from '@notabene/tap-ts';

async function multiAgentExample() {
  // Initialize the TAP library
  await init();
  
  // Create multiple agents
  const agents = await Promise.all([
    Agent.create("Alice"),
    Agent.create("Bob"),
    Agent.create("Charlie"),
    Agent.create("Dave")
  ]);
  
  // Save the agents' keys
  const jwks = await Promise.all(agents.map(agent => agent.exportJwk()));
  
  // Create a registry of agents by DID
  const agentsByDid = new Map();
  agents.forEach(agent => {
    agentsByDid.set(agent.did(), agent);
  });
  
  // Function to route messages
  async function routeMessage(message, fromDid) {
    const to = message.to || [];
    
    for (const toDid of to) {
      const agent = agentsByDid.get(toDid);
      if (agent) {
        console.log(`Routing message from ${fromDid} to ${toDid}`);
        const response = await agent.processMessage(message);
        
        if (response) {
          await routeMessage(response, toDid);
        }
      }
    }
  }
  
  // Example: Alice initiates a transfer to Bob
  const alice = agents[0];
  const bob = agents[1];
  
  const transferMsg = await alice.createTransfer({
    asset: "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F",
    amount: "100.0",
    beneficiaryDid: bob.did(),
    memo: "Payment for services"
  });
  
  await routeMessage(transferMsg, alice.did());
}
```

## Browser Integration

To use tap-ts in a browser environment, you'll need to include the WASM file in your build process. Most bundlers (like webpack, Rollup, or Parcel) have plugins for handling WASM files.

### Using with webpack

```javascript
// webpack.config.js
module.exports = {
  experiments: {
    asyncWebAssembly: true,
  },
  module: {
    rules: [
      {
        test: /\.wasm$/,
        type: 'webassembly/async',
      },
    ],
  },
};
```

### Using with Vite

```javascript
// vite.config.js
export default {
  plugins: [
    // No additional configuration needed, Vite handles WASM files by default
  ],
};
```

## Node.js Integration

In Node.js environments, the tap-ts package will automatically load the WASM file.

```javascript
// ESM
import { init, Agent } from '@notabene/tap-ts';

// CommonJS
const tap = require('@notabene/tap-ts');
```

## Error Handling

The tap-ts library uses standard JavaScript Error objects for error handling. Every asynchronous method that can fail will reject with an Error object that includes a descriptive message.

```typescript
try {
  const agent = await Agent.create("Alice");
  // ...
} catch (error) {
  console.error("Failed to create agent:", error.message);
}
```
