# tap-wasm: WebAssembly Bindings for the TAP Protocol

The `tap-wasm` crate provides WebAssembly bindings for the Transaction Authorization Protocol (TAP), enabling TAP functionality in browser and JavaScript/TypeScript environments.

## Features

- Complete WebAssembly bindings for TAP functionality
- Support for all TAP message types and operations
- Integration with the DIDComm v2 library for secure messaging
- Key management through the SecretsResolver implementation
- JavaScript-friendly API with Promise support for asynchronous operations
- Lightweight and efficient implementation for browser environments

## Usage

This crate is primarily used as a dependency by the `tap-ts` TypeScript package, which provides a more ergonomic API for JavaScript and TypeScript developers. However, you can also use it directly in your JavaScript/TypeScript projects.

### Building from Source

To build the WebAssembly module from source:

```bash
# Navigate to the tap-wasm directory
cd tap-rs/tap-wasm

# Build the WebAssembly module using wasm-pack
wasm-pack build --target web

# Or for Node.js environments
wasm-pack build --target nodejs
```

### Importing in JavaScript/TypeScript

```javascript
// Using ES modules (browser)
import * as tapWasm from './pkg/tap_wasm.js';

// Initialize the module
async function init() {
  await tapWasm.default();
  
  // Create a new agent
  const agent = new tapWasm.Agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
  
  // Create a new message
  const message = new tapWasm.Message();
  message.setMessageType(tapWasm.MessageType.AuthorizationRequest);
  
  // Work with the message
  message.setFromDid("did:example:sender");
  message.setToDid("did:example:receiver");
  
  // Sign the message
  agent.signMessage(message);
  
  console.log("Created signed message:", message.serialize());
}

init().catch(console.error);
```

## Key Management with SecretsResolver

The `tap-wasm` crate implements the DIDComm SecretsResolver trait to manage cryptographic keys securely:

```javascript
// Create an agent with a specific DID
const agent = new tapWasm.Agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

// Add a new key to the agent
const privateKey = new Uint8Array([/* your private key bytes */]);
const publicKey = new Uint8Array([/* your public key bytes */]);
agent.addKey("did:key:z6MkNewKey", "Ed25519", privateKey, publicKey);

// Get information about the agent's keys
const keysInfo = agent.getKeysInfo();
console.log("Available keys:", keysInfo);

// Verify a message signature
const isValid = agent.verifyMessage(signedMessage);
console.log("Signature valid:", isValid);
```

## Message Processing

```javascript
// Create a new agent
const agent = new tapWasm.Agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

// Create a TAP node with the agent
const node = new tapWasm.TapNode("my-node");
node.addAgent(agent);

// Process an incoming message
const incomingMessage = "..."; // Serialized message JSON
node.processMessage(incomingMessage)
  .then(response => {
    console.log("Processed message, response:", response);
  })
  .catch(error => {
    console.error("Error processing message:", error);
  });
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Related Projects

- [tap-core](../tap-core/README.md): Core message processing for TAP
- [tap-agent](../tap-agent/README.md): TAP agent functionality and identity management
- [tap-ts](../tap-ts/README.md): TypeScript wrapper for browser and Node.js environments
