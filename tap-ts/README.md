# TAP-TS

A TypeScript wrapper for the Transaction Authorization Protocol (TAP) with full DIDComm messaging support.

## Features

- **TypeScript API**: Idiomatic TypeScript wrapper for TAP functionality
- **DIDComm Integration**: Complete DIDComm v2 integration
- **Message Types**: Full support for all TAP message types (Transfer, Authorization, Rejection, Settlement)
- **Agent Management**: Agent creation, configuration, and messaging
- **Key Management**: Secure management of cryptographic keys
- **Browser & Node.js Support**: Runs in both browser and Node.js environments
- **Async API**: Promise-based API for all asynchronous operations
- **Type Safety**: Full TypeScript definitions for all TAP structures

## Installation

```bash
npm install @notabene/tap-ts
# or
yarn add @notabene/tap-ts
```

## Usage

### Creating an Agent

```typescript
import { Agent, AgentConfig } from '@notabene/tap-ts';

async function init() {
  // Create an agent with a DID
  const config = new AgentConfig({
    did: 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
    label: 'My TAP Agent'
  });
  
  const agent = await Agent.create(config);
  console.log(`Agent created with DID: ${agent.did}`);
  
  return agent;
}
```

### Creating and Sending a Transfer Message

```typescript
import { Agent, AgentConfig, CaipAssetId, TapTransfer, Participant } from '@notabene/tap-ts';

async function sendTransfer(agent: Agent) {
  // Create originator participant
  const originator = new Participant({
    id: agent.did,
    role: 'originator'
  });
  
  // Create beneficiary participant
  const beneficiary = new Participant({
    id: 'did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL',
    role: 'beneficiary'
  });
  
  // Create asset ID
  const asset = CaipAssetId.parse('eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7');
  
  // Create a transfer message
  const transfer = new TapTransfer({
    asset: asset.toString(),
    originator,
    beneficiary,
    amount: '100.0',
    memo: 'Test transfer'
  });
  
  // Pack and send the message
  const message = await agent.packMessage(transfer, beneficiary.id);
  console.log('Message created:', message);
  
  // Send the message (example implementation)
  const response = await fetch('https://example.com/didcomm', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(message)
  });
  
  return response.json();
}
```

### Processing Incoming Messages

```typescript
import { Agent, AgentConfig } from '@notabene/tap-ts';

async function processMessage(agent: Agent, rawMessage: string) {
  // Unpack and process the message
  const result = await agent.unpackMessage(rawMessage);
  
  if (result.isTransfer()) {
    console.log('Received transfer message:', result.asTransfer());
    
    // Send authorization response
    const authResponse = result.asTransfer().authorize({
      note: 'Transfer approved',
      metadata: { 'approval_time': new Date().toISOString() }
    });
    
    // Pack the authorization response
    const packedResponse = await agent.packMessage(
      authResponse,
      result.message.from
    );
    
    // Send the response (implementation depends on your transport)
    return packedResponse;
  }
  
  console.log('Received message of type:', result.type());
  return null;
}
```

### Key Management

```typescript
import { KeyPair, Agent } from '@notabene/tap-ts';

async function manageKeys() {
  // Generate a new key pair
  const keyPair = await KeyPair.generate();
  console.log('Generated DID:', keyPair.did);
  console.log('Public Key:', keyPair.publicKey);
  
  // Create agent with the generated key pair
  const agent = await Agent.createWithKeys({
    did: keyPair.did,
    label: 'New Agent'
  }, keyPair);
  
  // Import an existing key
  const importedKeyPair = await KeyPair.fromSeed('your-seed-phrase');
  console.log('Imported DID:', importedKeyPair.did);
  
  return { agent, keyPair };
}
```

### Using with TAP Node

```typescript
import { Agent, TapNode } from '@notabene/tap-ts';

async function setupNode(agents: Agent[]) {
  // Create a TAP node
  const node = new TapNode();
  
  // Register multiple agents with the node
  for (const agent of agents) {
    await node.registerAgent(agent);
  }
  
  // Process an incoming message
  async function handleIncomingMessage(rawMessage: string) {
    const result = await node.processMessage(rawMessage);
    return result;
  }
  
  return node;
}
```

## API Reference

### Agent

```typescript
class Agent {
  static create(config: AgentConfig): Promise<Agent>;
  static createWithKeys(config: AgentConfig, keyPair: KeyPair): Promise<Agent>;
  
  get did(): string;
  get config(): AgentConfig;
  
  packMessage(message: TapMessage, to: string): Promise<Record<string, any>>;
  unpackMessage(message: string): Promise<MessageUnpackResult>;
  
  createTransfer(options: TransferOptions): TapTransfer;
  createAuthorization(options: AuthorizationOptions): TapAuthorization;
  createRejection(options: RejectionOptions): TapRejection;
  createSettlement(options: SettlementOptions): TapSettlement;
}
```

### Message Types

```typescript
class TapTransfer {
  constructor(options: TransferOptions);
  
  authorize(options?: AuthorizeOptions): TapAuthorization;
  reject(options?: RejectOptions): TapRejection;
  
  get asset(): string;
  get originator(): Participant;
  get beneficiary(): Participant | undefined;
  get amount(): string;
  get memo(): string | undefined;
}

class TapAuthorization {
  constructor(options: AuthorizationOptions);
  
  settle(options?: SettleOptions): TapSettlement;
  
  get transfer(): TapTransfer;
  get note(): string | undefined;
}

class TapRejection {
  constructor(options: RejectionOptions);
  
  get transfer(): TapTransfer;
  get reason(): string;
}

class TapSettlement {
  constructor(options: SettlementOptions);
  
  get authorization(): TapAuthorization;
  get txid(): string;
}
```

## Examples

See the [examples directory](./examples) for more detailed usage examples.

## Integration with TAP-WASM

This package is built on top of the `tap-wasm` WebAssembly module, which provides the core functionality from the Rust implementation. The TypeScript wrapper provides a more idiomatic API for JavaScript and TypeScript developers.

## Building from Source

```bash
# Clone the repository
git clone https://github.com/notabene/tap-rs.git
cd tap-rs

# Build the WASM bindings
cd tap-wasm
wasm-pack build --target bundler

# Build the TypeScript wrapper
cd ../tap-ts
npm install
npm run build
```

## Testing

```bash
npm test
