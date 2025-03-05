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

## Development Notes

### Dependency Requirements

The TAP-TS module has specific dependency requirements that must be followed:

- **UUID Version**: The underlying Rust code requires UUID v0.8.2 due to compatibility with the didcomm crate.
- **WASM Dependencies**: The build process requires specific feature flags for WASM compatibility.

When building or modifying the WASM module, please consult the project's [DEPENDENCIES.md](../DEPENDENCIES.md) for important version constraints.

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
  Participant,
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

// Create a TAP transfer message
const transferMessage = new Message({
  type: MessageType.TRANSFER,
});

// Set the transfer data following TAIP-3
transferMessage.setTransferData({
  asset: "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  originator: {
    "@id": aliceParticipant.getDid(),
    role: "originator"
  },
  amount: "100.00",
  beneficiary: {
    "@id": bobParticipant.getDid(),
    role: "beneficiary"
  },
  participants: [
    {
      "@id": aliceParticipant.getDid(),
      role: "originator"
    },
    {
      "@id": bobParticipant.getDid(),
      role: "beneficiary"
    }
  ],
  memo: "Example transfer"
});

// Send the message
await aliceParticipant.sendMessage(bobParticipant.getDid(), transferMessage);

// On Bob's side, set up a handler for transfer messages
bobParticipant.registerHandler(MessageType.TRANSFER, async (message, metadata) => {
  console.log("Received transfer message:", message.getId());
  
  const transferData = message.getTransferData();
  if (transferData) {
    console.log("Transfer details:", transferData);
    
    // Create an authorize response
    const authorizeMessage = new Message({
      type: MessageType.AUTHORIZE,
      correlation: message.getId(),
    });
    
    // Set authorize data
    authorizeMessage.setAuthorizeData({
      transfer_id: message.getId(),
      note: "Transfer authorized"
    });
    
    // Send the authorization response
    await bobParticipant.sendMessage(metadata?.senderDid || "", authorizeMessage);
    console.log("Authorization sent");
  }
});
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
  Participant,
  Message,
  MessageType,
  wasmLoader,
} from "https://deno.land/x/tap_ts/mod.ts";

// Load the WASM module
await wasmLoader.load();

// Create a participant with a DID
const participant = new Participant({
  did: "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  nickname: "Alice",
});

// Add a custom key to the participant
const privateKeyBytes = new Uint8Array([/* private key bytes */]);
const publicKeyBytes = new Uint8Array([/* public key bytes */]);
participant.addKey("did:key:z6MkCustomKey", "Ed25519", privateKeyBytes, publicKeyBytes);

// Get information about available keys
const keysInfo = participant.getKeysInfo();
console.log("Available keys:", keysInfo);

// Create and sign a message
const message = new Message({
  type: MessageType.TRANSFER,
});

// Set transfer data
message.setTransferData({
  asset: "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  originator: {
    "@id": participant.getDid(),
    role: "originator"
  },
  amount: "100.00",
  participants: [
    {
      "@id": participant.getDid(),
      role: "originator"
    }
  ]
});

// Sign the message
participant.signMessage(message);

// Verify a message signature
const isValid = participant.verifyMessage(message);
console.log("Signature is valid:", isValid);
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
