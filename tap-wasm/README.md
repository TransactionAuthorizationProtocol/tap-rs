# TAP-WASM

WebAssembly bindings for the Transaction Authorization Protocol (TAP).

## Features

- **WebAssembly Support**: Run TAP in browser and Node.js environments
- **DIDComm Integration**: Full support for DIDComm v2 messaging
- **TAP Message Types**: Support for all TAP message types
- **Agent Management**: Create and manage TAP agents
- **Message Handling**: Create, sign, and verify TAP messages
- **Serialization**: Efficient serialization between Rust and JavaScript
- **Performance**: Optimized for browser performance

## Installation

```bash
# Using npm
npm install tap-wasm

# Using yarn
yarn add tap-wasm
```

## Basic Usage

### Browser with ES modules

```javascript
import init, { 
  init_tap_wasm, 
  Message, 
  TapAgent, 
  MessageType 
} from 'tap-wasm';

async function main() {
  // Initialize the WASM module
  await init();
  init_tap_wasm();

  // Create a new agent
  const agent = new TapAgent({
    nickname: "Test Agent",
    debug: true
  });
  console.log(`Agent created with DID: ${agent.get_did()}`);

  // Create a transfer message
  const message = new Message('msg_123', 'Transfer', '1.0');
  
  // Set the transfer message body
  message.set_transfer_body({
    asset: "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7",
    originator: {
      id: agent.get_did(),
      role: "originator"
    },
    beneficiary: {
      id: "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL",
      role: "beneficiary"
    },
    amount: "100.0",
    agents: [],
    memo: "Test transfer"
  });

  // Sign the message
  agent.sign_message(message);
  
  console.log("Signed message:", {
    id: message.id(),
    type: message.message_type(),
    from: message.from_did(),
    transfer: message.get_transfer_body()
  });
}

main().catch(console.error);
```

### Node.js

```javascript
const tap_wasm = require('tap-wasm');

async function main() {
  // Initialize the WASM module
  await tap_wasm.default();
  tap_wasm.init_tap_wasm();

  // Create a new agent
  const agent = new tap_wasm.TapAgent({
    nickname: "Test Agent",
    debug: true
  });
  
  // Create a transfer message
  const message = new tap_wasm.Message('msg_123', 'Transfer', '1.0');
  
  // Set the transfer message body and sign it
  // ...similar to the browser example
}

main().catch(console.error);
```

## API Reference

### Message Creation and Handling

#### Creating a Message

```javascript
// Create a new message
const message = new Message(id, messageType, version);

// Example
const message = new Message('msg_123', 'Transfer', '1.0');
```

#### Setting Message Bodies

```javascript
// Set a transfer message body
message.set_transfer_body({
  asset: "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7",
  originator: {
    id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    role: "originator"
  },
  beneficiary: {
    id: "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL",
    role: "beneficiary"
  },
  amount: "100.0",
  agents: [],
  memo: "Test transfer"
});

// Set other message types
message.set_payment_request_body({...});
message.set_authorize_body({...});
message.set_reject_body({...});
message.set_settle_body({...});
// ... and other message types
```

#### Getting Message Bodies

```javascript
// Get a transfer message body
const transferBody = message.get_transfer_body();

// Get other message types
const paymentRequestBody = message.get_payment_request_body();
const authorizeBody = message.get_authorize_body();
const rejectBody = message.get_reject_body();
// ... and other message types
```

#### Message Properties

```javascript
// Get message properties
const id = message.id();
const type = message.message_type();
const version = message.version();
const fromDid = message.from_did();
const toDid = message.to_did();

// Set message properties
message.set_id('new-id');
message.set_message_type('Authorize');
message.set_version('1.1');
message.set_from_did('did:example:123');
message.set_to_did('did:example:456');
```

### Agent Management

#### Creating an Agent

```javascript
// Create a new agent
const agent = new TapAgent({
  did: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK", // Optional - generated if not provided
  nickname: "Example Agent",
  debug: true // Optional - logs to console if true
});
```

#### Agent Operations

```javascript
// Get agent properties
const did = agent.get_did();
const nickname = agent.nickname();

// Message handling
agent.set_from(message); // Set the message's from field to this agent's DID
agent.set_to(message, "did:example:recipient"); // Set the message's to field
agent.sign_message(message); // Sign a message
agent.verify_message(message); // Verify a message's signature

// Create a message from this agent
const newMessage = agent.create_message(MessageType.Transfer);

// Register a message handler
agent.register_message_handler(MessageType.Transfer, (message, metadata) => {
  console.log("Received transfer message:", message);
  // Process the message
  return responseMessage; // Optional response message
});

// Process an incoming message
const result = await agent.process_message(message, { source: "browser" });
```

### Node Management

```javascript
// Create a TAP node
const node = new TapNode({ debug: true });

// Add agents to the node
node.add_agent(agent1);
node.add_agent(agent2);

// Get agents
const agent = node.get_agent("did:example:123");
const allAgents = node.get_agents();

// Process a message through the node
const result = await node.process_message(message, { source: "browser" });
```

### Utility Functions

```javascript
// Generate a UUID
const uuid = generate_uuid_v4();

// Create a DID key
const didKey = create_did_key();
```

## Building from Source

### Prerequisites

- Rust and Cargo (https://rustup.rs/)
- wasm-pack (https://rustwasm.github.io/wasm-pack/installer/)

### Build Steps

```bash
# Clone the repository
git clone https://github.com/TransactionAuthorizationProtocol/tap-rs.git
cd tap-rs

# Build the WebAssembly package
cd tap-wasm
wasm-pack build --target web

# The output will be in the pkg/ directory
```

## Examples

For more examples, see the [examples directory](./examples).

## License

MIT License