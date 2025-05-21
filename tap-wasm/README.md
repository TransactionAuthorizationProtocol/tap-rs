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
- **Shared Core**: Uses the same core implementation as the native TAP agent

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
  WasmTapAgent, 
  TapNode, 
  MessageType 
} from 'tap-wasm';

// Recommended pattern using a static create method
class TAPAgent {
  static async create(options = {}) {
    // Initialize WASM first
    await init();
    
    // Then create the agent
    return new WasmTapAgent(options);
  }
}

async function main() {
  try {
    // Create an agent using the static factory method
    // This ensures WASM is initialized before agent creation
    const agent = await TAPAgent.create({
      nickname: "Test Agent",
      debug: true
    });
    console.log(`Agent created with DID: ${agent.get_did()}`);

    // Create a transfer message
    const message = agent.createMessage('https://tap.rsvp/schema/1.0#Transfer');
    
    // Set the transfer message body
    message.body = {
      asset: "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7",
      originator: {
        '@id': agent.get_did(),
        role: "originator"
      },
      beneficiary: {
        '@id': "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL",
        role: "beneficiary"
      },
      amount: "100.0",
      agents: [],
      memo: "Test transfer"
    };

    // Pack the message
    const packedResult = await agent.packMessage(message);
    console.log("Packed message:", packedResult.message);
    
    // Unpack the message
    const unpackedMessage = await agent.unpackMessage(packedResult.message);
    console.log("Unpacked message:", unpackedMessage);
  } catch (error) {
    console.error("Error:", error);
  }
}

main();
```

### Node.js

```javascript
const tap_wasm = require('tap-wasm');

// Recommended pattern using a static create method
class TAPAgent {
  static async create(options = {}) {
    // Initialize WASM first
    await tap_wasm.default();
    
    // Then create the agent
    return new tap_wasm.WasmTapAgent(options);
  }
}

async function main() {
  try {
    // Create an agent using the static factory method
    // This ensures WASM is initialized before agent creation
    const agent = await TAPAgent.create({
      nickname: "Test Agent",
      debug: true
    });
    console.log(`Agent created with DID: ${agent.get_did()}`);
    
    // Create a transfer message
    const message = agent.createMessage('https://tap.rsvp/schema/1.0#Transfer');
    
    // Set the message body (similar to browser example)
    message.body = {
      asset: "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7",
      originator: {
        '@id': agent.get_did(),
        role: "originator"
      },
      beneficiary: {
        '@id': "did:key:z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL",
        role: "beneficiary"
      },
      amount: "100.0",
      agents: []
    };
    
    // Pack and send the message
    const packed = await agent.packMessage(message);
    console.log("Message packed successfully:", packed.message);
  } catch (error) {
    console.error("Error:", error);
  }
}

main();
```

## API Reference

### Message Creation and Handling

#### Creating a Message

```javascript
// Create a new message
const message = agent.createMessage('https://tap.rsvp/schema/1.0#Transfer');

// The message will have the following structure:
// {
//   id: "msg_...", // Auto-generated UUID
//   type: "https://tap.rsvp/schema/1.0#Transfer",
//   from: "agent's DID",
//   to: [],
//   body: {},
//   created: <timestamp>
// }
```

#### Message Properties

```javascript
// Access and modify message properties
message.id = "msg_123"; // Message ID
message.type = "https://tap.rsvp/schema/1.0#Transfer"; // Message type
message.from = "did:example:123"; // Sender DID
message.to = ["did:example:456"]; // Recipient DIDs
message.body = {...}; // Message body
message.created = Date.now(); // Created timestamp
message.expires = Date.now() + 3600000; // Expiry timestamp
message.thid = "thread_123"; // Thread ID
message.pthid = "parent_thread_123"; // Parent thread ID
```

### Agent Management

#### Creating an Agent

```javascript
// RECOMMENDED: Create a new agent using static factory pattern
class TAPAgent {
  static async create(options = {}) {
    // Initialize WASM first
    await init();
    
    // Then create the agent
    return new WasmTapAgent(options);
  }
}

// Use the factory method (RECOMMENDED)
const agent = await TAPAgent.create({
  did: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK", // Optional - generated if not provided
  nickname: "Example Agent",
  debug: true // Optional - logs to console if true
});

// ALTERNATIVELY: Create directly (NOT RECOMMENDED - may cause WASM initialization errors)
// Only use this approach if you're certain WASM is already initialized
const agent2 = new WasmTapAgent({
  nickname: "Example Agent",
  debug: true
});
```

#### Agent Operations

```javascript
// Get agent properties
const did = agent.get_did();
const nickname = agent.nickname();

// Message packing and unpacking
const packedResult = await agent.packMessage(message);
const unpackedMessage = await agent.unpackMessage(packedResult.message);

// Create a message
const newMessage = agent.createMessage('https://tap.rsvp/schema/1.0#Transfer');

// Register a message handler
agent.registerMessageHandler('https://tap.rsvp/schema/1.0#Transfer', (message, metadata) => {
  console.log("Received transfer message:", message);
  // Process the message
  return Promise.resolve(responseMessage); // Optional response message
});

// Process an incoming message
const result = await agent.processMessage(message, { source: "browser" });

// Subscribe to all messages
const unsubscribe = agent.subscribeToMessages((message, metadata) => {
  console.log("Processing message:", message);
});
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
const allAgents = node.list_agents();

// Remove an agent
node.remove_agent("did:example:123");
```

### Utility Functions

```javascript
// Generate a UUID
const uuid = generate_uuid_v4();
```

## Integration with tap-agent

This implementation wraps the core `tap-agent` Rust crate, using its WASM-compatible features. This ensures compatibility and consistency between the WASM bindings and the native Rust implementation.

The integration:

1. Uses the `WasmAgent` trait from the `tap-agent` crate
2. Wraps the `TapAgent` implementation with WASM bindings
3. Provides JavaScript-friendly methods for all operations
4. Leverages the same cryptographic operations as the native agent

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

### Browser Example

A complete browser example is available at [examples/browser-agent-example.html](./examples/browser-agent-example.html). It demonstrates:

- Creating a TAP agent
- Creating and modifying TAP messages
- Packing and unpacking messages
- Handling events

## License

MIT License