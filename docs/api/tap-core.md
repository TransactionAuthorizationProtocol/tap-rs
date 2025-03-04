# tap-core API Reference

The `tap-core` crate provides the fundamental types and utilities for working with TAP messages. This is the foundation upon which the rest of the TAP-RS library is built.

## Message Types

### `TapMessageBody` Trait

The foundation trait implemented by all TAP message types.

```rust
pub trait TapMessageBody: Sized + Send + Sync + Clone + Debug {
    /// Convert to a DIDComm message
    fn to_didcomm(&self) -> Result<Message, Error>;
    
    /// Convert to a DIDComm message with a specific route
    fn to_didcomm_with_route(&self, route: Option<Vec<String>>) -> Result<Message, Error>;
    
    /// Convert from a DIDComm message
    fn from_didcomm(message: &Message) -> Result<Self, Error>;
}
```

### `TransferBody`

Represents a transfer request in the TAP protocol.

```rust
pub struct TransferBody {
    /// The asset being transferred, represented as a CAIP-19 Asset ID
    pub asset: AssetId,
    
    /// The originator of the transfer
    pub originator: Agent,
    
    /// The beneficiary of the transfer
    pub beneficiary: Option<Agent>,
    
    /// The amount being transferred as a string
    pub amount: String,
    
    /// Additional agents involved in the transfer
    pub agents: Vec<Agent>,
    
    /// Optional settlement ID for when this is tied to a known settlement
    pub settlement_id: Option<String>,
    
    /// Optional memo describing the purpose of the transfer
    pub memo: Option<String>,
    
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
}
```

#### Methods

- `fn new(asset: AssetId, originator: Agent, amount: String) -> Self` - Create a new TransferBody with minimum required fields
- `fn with_beneficiary(mut self, beneficiary: Agent) -> Self` - Add a beneficiary to the transfer
- `fn with_memo(mut self, memo: String) -> Self` - Add a memo to the transfer
- `fn with_metadata(mut self, key: String, value: String) -> Self` - Add metadata to the transfer

#### Example

```rust
use tap_core::message::{TransferBody, Agent};
use tap_caip::AssetId;
use std::collections::HashMap;
use std::str::FromStr;

// Create originator and beneficiary
let originator = Agent {
    id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
    role: Some("originator".to_string()),
};

let beneficiary = Agent {
    id: "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH".to_string(),
    role: Some("beneficiary".to_string()),
};

// Parse asset ID
let asset = AssetId::from_str("eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap();

// Create transfer body
let transfer = TransferBody {
    asset,
    originator,
    beneficiary: Some(beneficiary),
    amount: "100.0".to_string(),
    agents: vec![],
    settlement_id: None,
    memo: Some("Payment for services".to_string()),
    metadata: HashMap::new(),
};

// Convert to DIDComm message
let message = transfer.to_didcomm().unwrap();
```

### `AuthorizeBody`

Represents an authorization response to a transfer request.

```rust
pub struct AuthorizeBody {
    /// The ID of the transfer being authorized
    pub transfer_id: String,
    
    /// Optional note providing context for the authorization
    pub note: Option<String>,
    
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
}
```

#### Methods

- `fn new(transfer_id: String) -> Self` - Create a new AuthorizeBody with the transfer ID
- `fn with_note(mut self, note: String) -> Self` - Add a note to the authorization
- `fn with_metadata(mut self, key: String, value: String) -> Self` - Add metadata to the authorization

#### Example

```rust
use tap_core::message::AuthorizeBody;
use std::collections::HashMap;

// Create authorize body
let authorize = AuthorizeBody {
    transfer_id: "12345-67890-abcdef".to_string(),
    note: Some("Transfer authorized".to_string()),
    metadata: HashMap::new(),
};

// Convert to DIDComm message
let message = authorize.to_didcomm().unwrap();
```

### `RejectBody`

Represents a rejection response to a transfer request.

```rust
pub struct RejectBody {
    /// The ID of the transfer being rejected
    pub transfer_id: String,
    
    /// Reason code for the rejection
    pub code: String,
    
    /// Optional detailed description of the rejection reason
    pub description: Option<String>,
    
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
}
```

#### Methods

- `fn new(transfer_id: String, code: String) -> Self` - Create a new RejectBody with the transfer ID and rejection code
- `fn with_description(mut self, description: String) -> Self` - Add a description to the rejection
- `fn with_metadata(mut self, key: String, value: String) -> Self` - Add metadata to the rejection

#### Example

```rust
use tap_core::message::RejectBody;
use std::collections::HashMap;

// Create reject body
let reject = RejectBody {
    transfer_id: "12345-67890-abcdef".to_string(),
    code: "policy_violation".to_string(),
    description: Some("Amount exceeds daily transfer limit".to_string()),
    metadata: HashMap::new(),
};

// Convert to DIDComm message
let message = reject.to_didcomm().unwrap();
```

### `ReceiptBody`

Represents a receipt confirmation for a transfer.

```rust
pub struct ReceiptBody {
    /// The ID of the transfer this receipt is for
    pub transfer_id: String,
    
    /// Optional settlement ID reference
    pub settlement_id: Option<String>,
    
    /// Optional note providing context for the receipt
    pub note: Option<String>,
    
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
}
```

#### Methods

- `fn new(transfer_id: String) -> Self` - Create a new ReceiptBody with the transfer ID
- `fn with_settlement_id(mut self, settlement_id: String) -> Self` - Add a settlement ID to the receipt
- `fn with_note(mut self, note: String) -> Self` - Add a note to the receipt
- `fn with_metadata(mut self, key: String, value: String) -> Self` - Add metadata to the receipt

### `SettlementBody`

Represents a settlement status message.

```rust
pub struct SettlementBody {
    /// The ID of the transfer this settlement is for
    pub transfer_id: String,
    
    /// The settlement identifier (often a transaction hash)
    pub settlement_id: String,
    
    /// Status of the settlement (e.g., "pending", "completed", "failed")
    pub status: String,
    
    /// Optional note providing context for the settlement
    pub note: Option<String>,
    
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
}
```

#### Methods

- `fn new(transfer_id: String, settlement_id: String, status: String) -> Self` - Create a new SettlementBody
- `fn with_note(mut self, note: String) -> Self` - Add a note to the settlement
- `fn with_metadata(mut self, key: String, value: String) -> Self` - Add metadata to the settlement

### `Agent`

Represents an agent in the TAP protocol.

```rust
pub struct Agent {
    /// The DID of the agent
    pub id: String,
    
    /// Optional role of the agent in the transfer
    pub role: Option<String>,
}
```

#### Methods

- `fn new(id: String) -> Self` - Create a new Agent with the given ID
- `fn with_role(mut self, role: String) -> Self` - Assign a role to the agent

## DID Functionality

### `KeyPair`

Represents a cryptographic key pair for DID operations.

```rust
pub struct KeyPair {
    // Internal implementation details
}
```

#### Methods

- `async fn generate_ed25519() -> Result<Self, Error>` - Generate a new Ed25519 key pair
- `async fn generate_x25519() -> Result<Self, Error>` - Generate a new X25519 key pair
- `fn get_did_key(&self) -> String` - Get the DID:key representation of this key pair
- `async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, Error>` - Sign data with this key pair
- `async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, Error>` - Verify a signature

### `KeyResolver`

Trait for resolving DIDs to public keys.

```rust
pub trait KeyResolver: Send + Sync {
    async fn resolve_key(&self, did: &str) -> Result<Option<Vec<u8>>, Error>;
}
```

## Utilities

### Time Utilities

```rust
/// Get the current time in RFC3339 format
pub fn get_current_time() -> String {
    chrono::Utc::now().to_rfc3339()
}
```

### Error Handling

```rust
pub enum Error {
    /// Error during DIDComm operations
    DIDComm(String),
    
    /// Error when validating message types
    InvalidMessageType(String),
    
    /// Error when validating message fields
    ValidationError(String),
    
    /// Error with cryptographic operations
    Crypto(String),
    
    /// General error
    General(String),
}
```

## Constants

```rust
/// TAP Transfer message type
pub const TAP_TRANSFER_TYPE: &str = "TAP_TRANSFER";

/// TAP Authorize message type
pub const TAP_AUTHORIZE_TYPE: &str = "TAP_AUTHORIZE";

/// TAP Reject message type
pub const TAP_REJECT_TYPE: &str = "TAP_REJECT";

/// TAP Receipt message type
pub const TAP_RECEIPT_TYPE: &str = "TAP_RECEIPT";

/// TAP Settlement message type
pub const TAP_SETTLEMENT_TYPE: &str = "TAP_SETTLEMENT";
```
