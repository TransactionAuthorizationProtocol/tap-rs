# Getting Started with @taprsvp/agent

This guide will help you get started with the TAP TypeScript Agent SDK.

## Prerequisites

- Node.js 16+ or a modern browser
- npm or yarn package manager
- Basic understanding of TypeScript/JavaScript

## Installation

Install the package using npm:

```bash
npm install @taprsvp/agent
```

Or using yarn:

```bash
yarn add @taprsvp/agent
```

## Basic Concepts

### DIDs (Decentralized Identifiers)

DIDs are unique identifiers for agents in the TAP network. Each agent has a DID that looks like:
```
did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
```

### TAP Messages

TAP messages follow the DIDComm v2 specification and contain:
- **Header**: Message metadata (id, type, from, to, timestamps)
- **Body**: Message-specific data (amounts, assets, parties)
- **Security**: Cryptographic signatures for authenticity

### Message Flow

1. Create a message with specific TAP type
2. Pack the message (sign it cryptographically)
3. Send the packed message over any transport
4. Recipient unpacks the message (verify signature)
5. Process the message body

## Your First TAP Agent

### Step 1: Create an Agent

```typescript
import { TapAgent } from '@taprsvp/agent';

async function createMyAgent() {
  // Create a new agent with Ed25519 keys
  const agent = await TapAgent.create({ 
    keyType: 'Ed25519' 
  });
  
  console.log('My agent DID:', agent.did);
  
  // Export the private key for storage
  const privateKey = agent.exportPrivateKey();
  console.log('Store this securely:', privateKey);
  
  return agent;
}
```

### Step 2: Send a Transfer Message

```typescript
async function sendTransfer(agent: TapAgent, recipientDid: string) {
  // Create a TAP Transfer message
  const transfer = await agent.createMessage('Transfer', {
    amount: '100.00',
    asset: 'eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48', // USDC
    originator: {
      '@id': agent.did,
      metadata: {
        name: 'Alice Smith',
        accountNumber: '1234567890'
      }
    },
    beneficiary: {
      '@id': recipientDid,
      metadata: {
        name: 'Bob Jones',
        accountNumber: '0987654321'
      }
    },
    memo: 'Payment for services',
    agents: []  // Add any agents involved in the transaction
  });
  
  // Set the recipient
  transfer.to = [recipientDid];
  
  // Pack the message for transmission
  const packed = await agent.pack(transfer);
  
  console.log('Packed message size:', packed.message.length, 'bytes');
  
  // In a real application, you would send packed.message
  // over your transport layer (HTTP, WebSocket, etc.)
  return packed.message;
}
```

### Step 3: Receive and Process Messages

```typescript
async function receiveMessage(agent: TapAgent, packedMessage: string) {
  // Unpack the received message
  const message = await agent.unpack(packedMessage);
  
  console.log('Received message type:', message.type);
  console.log('From:', message.from);
  console.log('Message ID:', message.id);
  
  // Process based on message type
  if (message.type === 'https://tap.rsvp/schema/1.0#Transfer') {
    console.log('Transfer details:');
    console.log('- Amount:', message.body.amount);
    console.log('- Asset:', message.body.asset);
    console.log('- Memo:', message.body.memo);
    
    // Respond with authorization
    return await authorizeTransfer(agent, message);
  }
}
```

### Step 4: Respond to Messages

```typescript
async function authorizeTransfer(agent: TapAgent, transfer: any) {
  // Create an authorization response
  const authorize = await agent.createMessage('Authorize', {
    transaction_id: transfer.id,
    settlement_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7',
    expiry: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString() // 24 hours
  }, {
    thid: transfer.id, // Thread ID references the original transfer
    to: [transfer.from]
  });
  
  // Pack and return
  const packed = await agent.pack(authorize);
  return packed.message;
}
```

## Complete Example

Here's a complete example that demonstrates a full TAP transaction flow:

```typescript
import { TapAgent, generatePrivateKey } from '@taprsvp/agent';

async function fullExample() {
  // Create two agents (Alice and Bob)
  const alice = await TapAgent.create({ keyType: 'Ed25519' });
  const bob = await TapAgent.create({ keyType: 'Ed25519' });
  
  console.log('Alice DID:', alice.did);
  console.log('Bob DID:', bob.did);
  
  // Step 1: Alice sends a transfer to Bob
  const transfer = await alice.createMessage('Transfer', {
    amount: '1000.00',
    asset: 'eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
    originator: {
      '@id': alice.did,
      metadata: { name: 'Alice' }
    },
    beneficiary: {
      '@id': bob.did,
      metadata: { name: 'Bob' }
    },
    memo: 'Monthly payment',
    agents: []  // Could include settlement agents, compliance officers, etc.
  });
  transfer.to = [bob.did];
  
  const packedTransfer = await alice.pack(transfer);
  console.log('\n1. Alice sent transfer');
  
  // Step 2: Bob receives and unpacks the transfer
  const receivedTransfer = await bob.unpack(packedTransfer.message);
  console.log('2. Bob received transfer for:', receivedTransfer.body.amount);
  
  // Step 3: Bob authorizes the transfer
  const authorize = await bob.createMessage('Authorize', {
    transaction_id: receivedTransfer.id,
    settlement_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7'
  }, {
    thid: receivedTransfer.id,
    to: [alice.did]
  });
  
  const packedAuth = await bob.pack(authorize);
  console.log('3. Bob sent authorization');
  
  // Step 4: Alice receives the authorization
  const receivedAuth = await alice.unpack(packedAuth.message);
  console.log('4. Alice received authorization for transaction:', receivedAuth.body.transaction_id);
  
  // Step 5: Alice sends settlement confirmation
  const settle = await alice.createMessage('Settle', {
    transaction_id: receivedTransfer.id,
    settlement_id: 'eip155:1:0x1234567890abcdef...',
    amount: '1000.00'
  }, {
    thid: receivedTransfer.id,
    to: [bob.did]
  });
  
  const packedSettle = await alice.pack(settle);
  console.log('5. Alice sent settlement confirmation');
  
  // Step 6: Bob receives settlement
  const receivedSettle = await bob.unpack(packedSettle.message);
  console.log('6. Bob received settlement:', receivedSettle.body.settlement_id);
  
  console.log('\nTransaction complete!');
  
  // Clean up
  alice.dispose();
  bob.dispose();
}

// Run the example
fullExample().catch(console.error);
```

## Browser Usage

For browser environments, you can use the SDK with ES modules:

```html
<!DOCTYPE html>
<html>
<head>
  <title>TAP Agent Example</title>
</head>
<body>
  <h1>TAP Agent Browser Example</h1>
  <div id="output"></div>
  
  <script type="module">
    import { TapAgent, generateUUID } from 'https://unpkg.com/@taprsvp/agent@latest/dist/index.js';
    
    async function runExample() {
      const output = document.getElementById('output');
      
      // Create an agent
      const agent = await TapAgent.create();
      
      output.innerHTML = `
        <p>Agent Created!</p>
        <p>DID: ${agent.did}</p>
        <p>Private Key: ${agent.exportPrivateKey()}</p>
      `;
      
      // Create a message
      const message = {
        id: generateUUID(),
        type: 'https://didcomm.org/basicmessage/2.0/message',
        from: agent.did,
        to: [],
        created_time: Date.now(),
        body: {
          content: 'Hello from the browser!'
        }
      };
      
      // Pack it
      const packed = await agent.pack(message);
      output.innerHTML += `<p>Message packed: ${packed.message.length} bytes</p>`;
    }
    
    runExample().catch(console.error);
  </script>
</body>
</html>
```

## Key Storage Best Practices

### Browser Storage

```typescript
// Use IndexedDB for secure storage
async function storeAgentKey(privateKey: string) {
  const db = await openDB('tap-agent-store', 1, {
    upgrade(db) {
      db.createObjectStore('keys');
    }
  });
  
  await db.put('keys', privateKey, 'agent-key');
}

async function loadAgentKey(): Promise<string | undefined> {
  const db = await openDB('tap-agent-store', 1);
  return await db.get('keys', 'agent-key');
}

// Usage
const agent = await TapAgent.create();
await storeAgentKey(agent.exportPrivateKey());

// Later...
const storedKey = await loadAgentKey();
if (storedKey) {
  const restoredAgent = await TapAgent.fromPrivateKey(storedKey, {
    keyType: 'Ed25519'
  });
}
```

### Node.js Storage

```typescript
import { writeFileSync, readFileSync, existsSync } from 'fs';
import { join } from 'path';
import { homedir } from 'os';

const KEY_FILE = join(homedir(), '.tap', 'agent.key');

function savePrivateKey(privateKey: string) {
  const dir = join(homedir(), '.tap');
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
  
  writeFileSync(KEY_FILE, privateKey, { mode: 0o600 });
}

function loadPrivateKey(): string | null {
  if (existsSync(KEY_FILE)) {
    return readFileSync(KEY_FILE, 'utf-8');
  }
  return null;
}
```

## Error Handling

Always handle errors when working with cryptographic operations:

```typescript
import { TapAgent } from '@taprsvp/agent';

async function safeMessageHandling() {
  try {
    const agent = await TapAgent.create();
    
    // Invalid private key
    try {
      const badAgent = await TapAgent.fromPrivateKey('invalid-key', {
        keyType: 'Ed25519'
      });
    } catch (error) {
      console.error('Invalid private key:', error.message);
    }
    
    // Invalid packed message
    try {
      const message = await agent.unpack('not-a-valid-message');
    } catch (error) {
      console.error('Invalid message:', error.message);
    }
    
    // Invalid DID
    try {
      const transfer = await agent.createMessage('Transfer', {
        amount: '100',
        asset: 'USD',
        originator: { '@id': 'invalid-did' },
        beneficiary: { '@id': agent.did }
      });
    } catch (error) {
      console.error('Invalid DID:', error.message);
    }
    
  } catch (error) {
    console.error('Agent creation failed:', error);
  }
}
```

## Next Steps

Now that you understand the basics:

1. **Explore Message Types**: Learn about all [TAP message types](./message-types.md)
2. **Integration Guide**: See how to [integrate with your application](./integration.md)
3. **API Reference**: Check the complete [API documentation](./api-reference.md)
4. **Examples**: Browse more [example code](./examples/)

## Getting Help

- **Documentation**: [Full documentation](https://github.com/notabene-id/tap-rs)
- **Issues**: [Report issues](https://github.com/notabene-id/tap-rs/issues)
- **Discord**: Join our community (coming soon)

## Tips and Tricks

### Message IDs

Every message has a unique ID. Use this for tracking:

```typescript
const message = await agent.createMessage('Transfer', data);
console.log('Tracking ID:', message.id);
// Store message.id for later reference
```

### Threading

Link related messages using thread IDs:

```typescript
const response = await agent.createMessage('Authorize', authData, {
  thid: originalMessage.id  // Links to original message
});
```

### Message Expiry

Set expiration times for time-sensitive messages:

```typescript
const message = await agent.createMessage('Payment', paymentData, {
  expires_time: Date.now() + (60 * 60 * 1000)  // 1 hour
});
```

### Custom Metadata

Add custom metadata to parties:

```typescript
const transfer = await agent.createMessage('Transfer', {
  amount: '100.00',
  asset: 'USD',
  originator: {
    '@id': agent.did,
    metadata: {
      name: 'Alice Smith',
      email: 'alice@example.com',
      customField: 'custom-value'
    }
  },
  beneficiary: { '@id': recipientDid },
  agents: [
    {
      '@id': settlementAgentDid,
      role: 'SettlementAddress',
      for: agent.did
    }
  ]
});
```

Happy coding with TAP! 🚀