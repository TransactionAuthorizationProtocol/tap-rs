# TAP WASM

WebAssembly bindings for the Transaction Authorization Protocol (TAP) with DIDComm integration.

## Features

- **WebAssembly Support**: Run TAP in browser and Node.js environments
- **DIDComm Integration**: Full support for DIDComm v2 messaging
- **TAP Message Types**: Support for all TAP message types
- **Secure Key Management**: Integration with DIDComm SecretsResolver for key management
- **Serialization**: Efficient serialization between Rust and JavaScript
- **Performance**: Optimized for browser performance

## Usage

### In a Web Application

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>TAP WASM Example</title>
</head>
<body>
  <script type="module">
    import init, { 
      create_transfer_message, 
      parse_didcomm_message,
      sign_message 
    } from './pkg/tap_wasm.js';

    async function run() {
      // Initialize the WASM module
      await init();

      // Create a transfer message
      const transferMessage = create_transfer_message({
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
        memo: "Test transfer"
      });

      console.log("Transfer Message:", transferMessage);

      // Sign the message
      const signedMessage = sign_message(
        transferMessage,
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "z6MkrJVSYwmQgxBBCnZWuYpKSJ4qWRhWGsc9hhsVf43yirpL",
        "5HAPpXmY2FHkHsNitVh9Uy5wKCXZ2DJgNCKyiuPqYgQwKcXfiRtS5Y7D9kSuTwLfgJQDZ52VhVrcNtrHy9TLSN6J"
      );

      console.log("Signed Message:", signedMessage);

      // Parse a received DIDComm message
      const parsedMessage = parse_didcomm_message(signedMessage);
      console.log("Parsed Message:", parsedMessage);
    }

    run();
  </script>
</body>
</html>
```

### With TypeScript

```typescript
import init, { 
  create_transfer_message, 
  parse_didcomm_message, 
  sign_message 
} from 'tap-wasm';

async function run() {
  // Initialize the WASM module
  await init();

  // Create a transfer message
  const transferMessage = create_transfer_message({
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
    memo: "Test transfer"
  });

  console.log("Transfer Message:", transferMessage);
}

run();
```

## Exported Functions

### Message Creation

```typescript
// Create a transfer message
function create_transfer_message(transferData: TransferData): string;

// Create other message types
function create_authorization_message(authData: AuthorizationData): string;
function create_rejection_message(rejectionData: RejectionData): string;
function create_settlement_message(settlementData: SettlementData): string;
```

### Message Processing

```typescript
// Parse a DIDComm message to extract its content
function parse_didcomm_message(message: string): any;

// Sign a message
function sign_message(message: string, senderDid: string, recipientDid: string, privateKey: string): string;

// Encrypt a message
function encrypt_message(message: string, senderDid: string, recipientDid: string, privateKey: string): string;
```

### Key Management

```typescript
// Generate a new key pair
function generate_keypair(): KeyPair;

// Convert between key formats
function ed25519_to_x25519(ed25519Key: string): string;
```

## Data Types

```typescript
// Transfer message data
interface TransferData {
  asset: string;
  originator: Participant;
  beneficiary?: Participant;
  amount: string;
  agents?: Participant[];
  settlement_id?: string;
  memo?: string;
  metadata?: Record<string, string>;
}

// Participant information
interface Participant {
  id: string;
  role?: string;
}

// Key pair result
interface KeyPair {
  publicKey: string;
  privateKey: string;
  did: string;
}
```

## Building from Source

```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Build the package
wasm-pack build --target web

# For bundlers (webpack, rollup, etc)
wasm-pack build --target bundler

# For Node.js
wasm-pack build --target nodejs
```

## Examples

See the [examples directory](./examples) for more detailed usage examples.

## Integration with tap-ts

For a higher-level TypeScript wrapper, see the [tap-ts](../tap-ts) package, which provides a more idiomatic TypeScript API on top of these WASM bindings.
