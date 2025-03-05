# TAP Message

Core message processing for the Transaction Authorization Protocol (TAP) with integrated DIDComm support.

## Features

- **TAP Message Types**: Complete implementation of all TAP message types
- **DIDComm Integration**: Direct conversion between TAP messages and DIDComm messages
- **Validation**: Proper validation of all message fields and formats
- **CAIP Support**: Validation for chain-agnostic identifiers (CAIP-2, CAIP-10, CAIP-19)
- **Authorization Flows**: Support for authorization, rejection, and settlement flows
- **Agent Policies**: TAIP-7 compliant policy implementation for defining agent requirements
- **Extensibility**: Easy addition of new message types

## Usage

```rust
use tap_msg::message::types::{Transfer, Participant};
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_caip::AssetId;
use std::collections::HashMap;
use std::str::FromStr;

// Create a Transfer message body
let asset = AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7")?;

let originator = Participant {
    id: "did:example:sender".to_string(),
    role: Some("originator".to_string()),
};

let beneficiary = Participant {
    id: "did:example:receiver".to_string(),
    role: Some("beneficiary".to_string()),
};

let transfer = Transfer {
    asset,
    originator,
    beneficiary: Some(beneficiary),
    amount: "100.0".to_string(),
    agents: vec![],
    settlement_id: None,
    memo: Some("Test transfer".to_string()),
    metadata: HashMap::new(),
};

// Convert to a DIDComm message
let message = transfer.to_didcomm()?;

// Or with routing information
let message_with_route = transfer.to_didcomm_with_route(
    Some("did:example:sender"), 
    ["did:example:receiver"].iter().copied()
)?;

// Create a TAP message from a DIDComm message
let received_transfer = Transfer::from_didcomm(&message)?;
```

## Message Types

### Transfer

The `Transfer` struct represents a TAP transfer message, which is the core message type in the protocol:

```rust
pub struct Transfer {
    pub asset: AssetId,
    pub originator: Participant,
    pub beneficiary: Option<Participant>,
    pub amount: String,
    pub agents: Vec<Participant>,
    pub settlement_id: Option<String>,
    pub memo: Option<String>,
    pub metadata: HashMap<String, String>,
}
```

### Agent Policies (TAIP-7)

The TAP protocol supports defining agent policies according to TAIP-7, which allows agents to specify requirements for authorizing transactions:

```rust
use tap_msg::message::{
    Participant, Policy, RequireAuthorization, RequirePresentation, 
    RequireProofOfControl, UpdatePolicies
};
use std::collections::HashMap;

// Create a participant with a policy
let auth_policy = RequireAuthorization {
    type_: "RequireAuthorization".to_string(),
    from: Some(vec!["did:example:alice".to_string()]),
    from_role: None,
    from_agent: None,
    purpose: Some("Authorization required from Alice".to_string()),
};

let participant = Participant {
    id: "did:example:bob".to_string(),
    role: Some("beneficiary".to_string()),
    policies: Some(vec![Policy::RequireAuthorization(auth_policy)]),
};

// Create an UpdatePolicies message to dynamically update policies
let proof_policy = RequireProofOfControl::default(); // Uses random nonce
let update_policies = UpdatePolicies {
    transfer_id: "transfer_12345".to_string(),
    context: "https://tap.rsvp/schemas/1.0".to_string(),
    policies: vec![Policy::RequireProofOfControl(proof_policy)],
    metadata: HashMap::new(),
};

// Validate the message
update_policies.validate().unwrap();

// Convert to DIDComm message and send to all participants
let didcomm_msg = update_policies.to_didcomm_with_route(
    Some("did:example:originator_vasp"),
    ["did:example:beneficiary", "did:example:beneficiary_vasp"].iter().copied()
).unwrap();
```

### Authorization Messages

TAP supports various authorization messages for compliance workflows:

```rust
// Authorization response
pub struct AuthorizationResponse {
    pub transfer_id: Uuid,
    pub status: String,
    pub note: Option<String>,
    pub metadata: HashMap<String, String>,
}

// Authorization rejection
pub struct Rejection {
    pub transfer_id: Uuid,
    pub note: Option<String>,
    pub metadata: HashMap<String, String>,
}

// Settlement message
pub struct Settlement {
    pub transfer_id: Uuid,
    pub txid: String,
    pub note: Option<String>,
    pub metadata: HashMap<String, String>,
}
```

## DIDComm Integration

The `TapMessageBody` trait provides methods for converting between TAP messages and DIDComm messages:

```rust
pub trait TapMessageBody: DeserializeOwned + Serialize + Send + Sync {
    /// Gets the message type string for this TAP message type
    fn message_type() -> &'static str;
    
    /// Converts a DIDComm message to this TAP message type
    fn from_didcomm(msg: &Message) -> Result<Self, Error>;
    
    /// Validates the message content
    fn validate(&self) -> Result<(), Error>;
    
    /// Converts this TAP message to a DIDComm message
    fn to_didcomm(&self) -> Result<Message, Error>;
    
    /// Converts this TAP message to a DIDComm message with routing information
    fn to_didcomm_with_route<'a, I>(&self, from: Option<&str>, to: I) -> Result<Message, Error>
    where
        I: Iterator<Item = &'a str>;
}
```

## Authorizable Trait

The `Authorizable` trait provides methods for handling authorization, rejection, and settlement flows:

```rust
pub trait Authorizable {
    fn authorize(&self, note: Option<String>, metadata: HashMap<String, String>) -> AuthorizationResponse;
    fn reject(&self, note: Option<String>, metadata: HashMap<String, String>) -> Rejection;
    fn settle(&self, txid: String, note: Option<String>, metadata: HashMap<String, String>) -> Settlement;
}
```

## Examples

See the [examples directory](./examples) for more detailed examples of using TAP messages.
