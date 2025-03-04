# TAP-TS: TypeScript WASM Wrapper for the Transaction Authorization Protocol

A Deno-based TypeScript wrapper for the Transaction Authorization Protocol (TAP) Rust implementation.

## Features

- **WASM Integration:** Provides a bridge to the Rust implementation using WebAssembly.
- **DID Resolution:** Supports multiple DID methods (did:key, did:web, did:pkh) using standard libraries.
- **Message Handling:** Create, process, and manage TAP messages.
- **Agent Implementation:** Create and manage TAP agents that can send and receive messages.
- **Node Implementation:** Create TAP nodes that can host multiple agents and route messages.
- **Cryptographic Operations:** Secure key management with DIDComm SecretsResolver integration.
- **Message Signing and Verification:** Support for Ed25519 and other key types.
- **Minimal Dependencies:** Built with minimal external dependencies.
- **Browser Compatibility:** Works in both Deno and browser environments.

## Prerequisites

- [Deno](https://deno.land/) 2.2 or higher
- [Rust](https://www.rust-lang.org/) 1.70 or higher
- [wasm-pack](https://rustwasm.github.io/wasm-pack/) for building WASM modules

## Installation

Clone the repository and build the project:

```bash
# Clone the repository
git clone https://github.com/notabene/tap-rs.git
cd tap-rs/tap-ts

# Build the project
deno task build
```

## Development

### Building the WASM module

To build the WASM module from the Rust implementation:

```bash
deno task build:wasm
```

### Running tests

```bash
# Run all tests
deno task test

# Run tests in a browser environment (requires Playwright)
deno task test:browser
```

### Cleaning build artifacts

```bash
deno task clean
```

## Usage

### Basic Example

```typescript
import {
  Agent,
  TapNode,
  Message,
  MessageType,
  wasmLoader,
} from "https://deno.land/x/tap_ts/mod.ts";

// Load the WASM module
await wasmLoader.load();

// Create a TAP node
const node = new TapNode({
  debug: true,
  network: {
    peers: ["https://example.com/tap"],
  },
});

// Create and register agents
const aliceAgent = new Agent({
  did: "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  nickname: "Alice",
});

const bobAgent = new Agent({
  did: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  nickname: "Bob",
});

// Register the agents with the node
node.registerAgent(aliceAgent);
node.registerAgent(bobAgent);

// Create a TAP authorization request message
const authRequest = new Message({
  type: MessageType.AUTHORIZATION_REQUEST,
});

// Set the authorization request data
authRequest.setAuthorizationRequestData({
  transactionHash: "0x1234567890abcdef",
  sender: "0xAliceSender",
  receiver: "0xBobReceiver",
  amount: "100.0",
  asset: "BTC",
});

// Send the message
await aliceAgent.sendMessage(bobAgent.did, authRequest);
```

### DID Resolution

```typescript
import { resolveDID, canResolveDID } from "https://deno.land/x/tap_ts/mod.ts";

// Check if a DID is resolvable
if (canResolveDID("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH")) {
  // Resolve the DID
  const resolution = await resolveDID("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH");
  console.log(resolution.didDocument);
}
```

### Key Management and Message Signing

The TAP-TS library includes built-in support for cryptographic key management and message signing through the DIDComm SecretsResolver integration:

```typescript
import {
  Agent,
  Message,
  MessageType,
  wasmLoader,
} from "https://deno.land/x/tap_ts/mod.ts";

// Load the WASM module
await wasmLoader.load();

// Create an agent with a DID
const agent = new Agent({
  did: "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  nickname: "Alice",
});

// Add a custom key to the agent
const privateKeyBytes = new Uint8Array([/* private key bytes */]);
const publicKeyBytes = new Uint8Array([/* public key bytes */]);
agent.addKey("did:key:z6MkCustomKey", "Ed25519", privateKeyBytes, publicKeyBytes);

// Get information about available keys
const keysInfo = agent.getKeysInfo();
console.log("Available keys:", keysInfo);

// Create and sign a message
const message = new Message({
  type: MessageType.AUTHORIZATION_REQUEST,
});

// Set message data
message.setAuthorizationRequestData({
  transactionHash: "0x1234567890abcdef",
  sender: "0xAliceSender",
  receiver: "0xBobReceiver",
  amount: "100.0",
  asset: "BTC",
});

// Sign the message
agent.signMessage(message);

// Verify a message signature
const isValid = agent.verifyMessage(message);
console.log("Signature is valid:", isValid);
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
