# tap-wasm API Reference

The `tap-wasm` crate provides WebAssembly (WASM) bindings for the TAP protocol, allowing the TAP-RS library to be used in browser and Node.js environments. This crate uses `wasm-bindgen` to expose Rust functionality to JavaScript/TypeScript.

## Core Exports

### `Agent`

The main TAP Agent implementation exposed to JavaScript/TypeScript.

```rust
#[wasm_bindgen]
pub struct Agent {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new Agent with the given name and key pair
#[wasm_bindgen]
pub async fn new(name: String, key_pair: JsValue) -> Result<Agent, JsError>;

/// Get the agent's DID
#[wasm_bindgen]
pub fn did(&self) -> String;

/// Get the agent's name
#[wasm_bindgen]
pub fn name(&self) -> String;

/// Create a TAP transfer message
#[wasm_bindgen]
pub async fn create_transfer(
    &self,
    asset: String,
    amount: String,
    beneficiary_did: String,
    memo: Option<String>,
) -> Result<JsValue, JsError>;

/// Create a TAP authorize message
#[wasm_bindgen]
pub async fn create_authorize(
    &self,
    transfer_id: String,
    note: Option<String>,
) -> Result<JsValue, JsError>;

/// Create a TAP reject message
#[wasm_bindgen]
pub async fn create_reject(
    &self,
    transfer_id: String,
    code: String,
    description: Option<String>,
) -> Result<JsValue, JsError>;

/// Create a TAP receipt message
#[wasm_bindgen]
pub async fn create_receipt(
    &self,
    transfer_id: String,
    settlement_id: Option<String>,
    note: Option<String>,
) -> Result<JsValue, JsError>;

/// Create a TAP settlement message
#[wasm_bindgen]
pub async fn create_settlement(
    &self,
    transfer_id: String,
    settlement_id: String,
    status: String,
    note: Option<String>,
) -> Result<JsValue, JsError>;

/// Process an incoming TAP message
#[wasm_bindgen]
pub async fn process_message(&self, message: JsValue) -> Result<Option<JsValue>, JsError>;

/// Set a message handler callback for a specific message type
#[wasm_bindgen]
pub fn set_message_handler(
    &self,
    message_type: String,
    callback: js_sys::Function,
) -> Result<(), JsError>;
```

### `KeyPair`

WebAssembly binding for cryptographic key pairs.

```rust
#[wasm_bindgen]
pub struct KeyPair {
    // Internal implementation details
}
```

#### Methods

```rust
/// Generate a new Ed25519 key pair
#[wasm_bindgen]
pub async fn generate_ed25519() -> Result<KeyPair, JsError>;

/// Generate a new X25519 key pair
#[wasm_bindgen]
pub async fn generate_x25519() -> Result<KeyPair, JsError>;

/// Get the DID:key representation of this key pair
#[wasm_bindgen]
pub fn get_did_key(&self) -> String;

/// Get the public key bytes
#[wasm_bindgen]
pub fn get_public_key(&self) -> js_sys::Uint8Array;

/// Export the key pair as a JWK
#[wasm_bindgen]
pub fn export_jwk(&self) -> Result<JsValue, JsError>;

/// Import a key pair from a JWK
#[wasm_bindgen]
pub async fn import_jwk(jwk: JsValue) -> Result<KeyPair, JsError>;

/// Sign data with this key pair
#[wasm_bindgen]
pub async fn sign(&self, data: js_sys::Uint8Array) -> Result<js_sys::Uint8Array, JsError>;

/// Verify a signature with this key pair
#[wasm_bindgen]
pub async fn verify(
    &self,
    data: js_sys::Uint8Array,
    signature: js_sys::Uint8Array,
) -> Result<bool, JsError>;
```

### `AssetId`

WebAssembly binding for CAIP-19 Asset IDs.

```rust
#[wasm_bindgen]
pub struct AssetId {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new AssetId from a string
#[wasm_bindgen]
pub fn from_string(s: String) -> Result<AssetId, JsError>;

/// Get the namespace of the chain
#[wasm_bindgen]
pub fn chain_namespace(&self) -> String;

/// Get the reference of the chain
#[wasm_bindgen]
pub fn chain_reference(&self) -> String;

/// Get the asset namespace
#[wasm_bindgen]
pub fn asset_namespace(&self) -> String;

/// Get the asset reference
#[wasm_bindgen]
pub fn asset_reference(&self) -> String;

/// Convert the AssetId to a string
#[wasm_bindgen]
pub fn to_string(&self) -> String;
```

## Message Types

### `TransferMessage`

WebAssembly binding for TAP transfer messages.

```rust
#[wasm_bindgen]
pub struct TransferMessage {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new TransferMessage
#[wasm_bindgen]
pub fn new(
    asset: String,
    originator_did: String,
    beneficiary_did: Option<String>,
    amount: String,
) -> Result<TransferMessage, JsError>;

/// Get the ID of this message
#[wasm_bindgen]
pub fn id(&self) -> String;

/// Get the asset ID as a string
#[wasm_bindgen]
pub fn asset(&self) -> String;

/// Get the originator's DID
#[wasm_bindgen]
pub fn originator(&self) -> String;

/// Get the beneficiary's DID, if present
#[wasm_bindgen]
pub fn beneficiary(&self) -> Option<String>;

/// Get the amount as a string
#[wasm_bindgen]
pub fn amount(&self) -> String;

/// Get the memo, if present
#[wasm_bindgen]
pub fn memo(&self) -> Option<String>;

/// Set the memo
#[wasm_bindgen]
pub fn set_memo(&mut self, memo: Option<String>);

/// Convert to a JsValue representation
#[wasm_bindgen]
pub fn to_js(&self) -> Result<JsValue, JsError>;

/// Convert from a JsValue representation
#[wasm_bindgen]
pub fn from_js(value: JsValue) -> Result<TransferMessage, JsError>;
```

### `AuthorizeMessage`

WebAssembly binding for TAP authorize messages.

```rust
#[wasm_bindgen]
pub struct AuthorizeMessage {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new AuthorizeMessage
#[wasm_bindgen]
pub fn new(transfer_id: String) -> Result<AuthorizeMessage, JsError>;

/// Get the ID of this message
#[wasm_bindgen]
pub fn id(&self) -> String;

/// Get the transfer ID being authorized
#[wasm_bindgen]
pub fn transfer_id(&self) -> String;

/// Get the note, if present
#[wasm_bindgen]
pub fn note(&self) -> Option<String>;

/// Set the note
#[wasm_bindgen]
pub fn set_note(&mut self, note: Option<String>);

/// Convert to a JsValue representation
#[wasm_bindgen]
pub fn to_js(&self) -> Result<JsValue, JsError>;

/// Convert from a JsValue representation
#[wasm_bindgen]
pub fn from_js(value: JsValue) -> Result<AuthorizeMessage, JsError>;
```

### `RejectMessage`

WebAssembly binding for TAP reject messages.

```rust
#[wasm_bindgen]
pub struct RejectMessage {
    // Internal implementation details
}
```

#### Methods

```rust
/// Create a new RejectMessage
#[wasm_bindgen]
pub fn new(transfer_id: String, code: String) -> Result<RejectMessage, JsError>;

/// Get the ID of this message
#[wasm_bindgen]
pub fn id(&self) -> String;

/// Get the transfer ID being rejected
#[wasm_bindgen]
pub fn transfer_id(&self) -> String;

/// Get the rejection code
#[wasm_bindgen]
pub fn code(&self) -> String;

/// Get the description, if present
#[wasm_bindgen]
pub fn description(&self) -> Option<String>;

/// Set the description
#[wasm_bindgen]
pub fn set_description(&mut self, description: Option<String>);

/// Convert to a JsValue representation
#[wasm_bindgen]
pub fn to_js(&self) -> Result<JsValue, JsError>;

/// Convert from a JsValue representation
#[wasm_bindgen]
pub fn from_js(value: JsValue) -> Result<RejectMessage, JsError>;
```

## Utility Functions

```rust
/// Initialize the WASM module (sets up logging, panic hooks, etc.)
#[wasm_bindgen]
pub fn init() -> Result<(), JsError>;

/// Convert a TAP message from JsValue to a DIDComm Message
#[wasm_bindgen]
pub fn js_to_didcomm(value: JsValue) -> Result<JsValue, JsError>;

/// Convert a DIDComm Message to JsValue
#[wasm_bindgen]
pub fn didcomm_to_js(message: JsValue) -> Result<JsValue, JsError>;

/// Parse an AssetId from a string
#[wasm_bindgen]
pub fn parse_asset_id(asset_id: String) -> Result<JsValue, JsError>;
```

## Constants

```rust
#[wasm_bindgen]
pub const TAP_TRANSFER_TYPE: &str = "TAP_TRANSFER";

#[wasm_bindgen]
pub const TAP_AUTHORIZE_TYPE: &str = "TAP_AUTHORIZE";

#[wasm_bindgen]
pub const TAP_REJECT_TYPE: &str = "TAP_REJECT";

#[wasm_bindgen]
pub const TAP_RECEIPT_TYPE: &str = "TAP_RECEIPT";

#[wasm_bindgen]
pub const TAP_SETTLEMENT_TYPE: &str = "TAP_SETTLEMENT";
```

## Error Handling

Errors in the WASM bindings are converted to JavaScript errors using the `JsError` type from `wasm-bindgen`. These errors will include a message describing the issue, and in debug builds, they will also include a stack trace.

## Examples

### Creating an Agent in JavaScript

```javascript
import { Agent, KeyPair } from 'tap-wasm';

async function createAgent() {
  // Initialize the WASM module
  await init();
  
  // Generate a new key pair
  const keyPair = await KeyPair.generate_ed25519();
  
  // Create a new agent
  const agent = await Agent.new("Alice", keyPair);
  
  console.log(`Created agent with DID: ${agent.did()}`);
  
  return agent;
}
```

### Creating and Processing TAP Messages

```javascript
import { Agent, KeyPair, TAP_TRANSFER_TYPE, TAP_AUTHORIZE_TYPE } from 'tap-wasm';

async function processTapMessages() {
  // Initialize the WASM module
  await init();
  
  // Create two agents
  const aliceKeyPair = await KeyPair.generate_ed25519();
  const bobKeyPair = await KeyPair.generate_ed25519();
  
  const alice = await Agent.new("Alice", aliceKeyPair);
  const bob = await Agent.new("Bob", bobKeyPair);
  
  // Set up a message handler for Bob to automatically authorize transfers
  bob.set_message_handler(TAP_TRANSFER_TYPE, (message) => {
    console.log("Bob received a transfer request:", message);
    
    // Create an authorize message
    return bob.create_authorize(message.id, "Transfer authorized");
  });
  
  // Alice creates a transfer message
  const assetId = "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F";
  const transferMsg = await alice.create_transfer(
    assetId,
    "100.0",
    bob.did(),
    "Payment for services"
  );
  
  console.log("Alice created transfer:", transferMsg);
  
  // Bob processes the message
  const response = await bob.process_message(transferMsg);
  
  if (response) {
    console.log("Bob sent an authorization:", response);
    
    // Alice processes Bob's response
    await alice.process_message(response);
    console.log("Alice received Bob's authorization");
  }
}
```

### Exporting and Importing Keys

```javascript
import { KeyPair } from 'tap-wasm';

async function saveAndRestoreKeys() {
  // Generate a new key pair
  const keyPair = await KeyPair.generate_ed25519();
  console.log(`Generated key with DID: ${keyPair.get_did_key()}`);
  
  // Export the key as a JWK
  const jwk = await keyPair.export_jwk();
  
  // Save the JWK (e.g., to localStorage)
  localStorage.setItem('tap_key', JSON.stringify(jwk));
  
  // Later, restore the key
  const savedJwk = JSON.parse(localStorage.getItem('tap_key'));
  const restoredKeyPair = await KeyPair.import_jwk(savedJwk);
  
  console.log(`Restored key with DID: ${restoredKeyPair.get_did_key()}`);
}
```

## Performance Considerations

The WASM bindings are designed to be as performant as possible, but there are some considerations to be aware of:

1. **Memory Management**: Large objects passed between JavaScript and WASM can impact performance.
2. **Async Operations**: Cryptographic operations are performed asynchronously and may involve multiple context switches.
3. **Serialization**: Converting between JavaScript objects and WASM/Rust structures involves serialization overhead.

For high-throughput applications, consider batching operations where possible and minimizing conversions between JavaScript and WASM.
