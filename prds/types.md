# TAP Message Types PRD

## Overview

This document outlines the type system required for implementing the Transaction Authorization Protocol (TAP) in Rust. It follows the specifications defined in the TAIPs (TAP Improvement Proposals) and aims to provide a comprehensive type hierarchy that mirrors the TypeScript implementation while leveraging Rust's strengths.

## Goals

- Create a type system that accurately reflects the TAP protocol specifications
- Ensure compatibility with JSON-LD formats used in the protocol
- Provide a clear, maintainable, and well-documented API for developers
- Support serialization/deserialization with custom handling for JSON-LD contexts
- Allow for extensibility with arbitrary data fields
- Enable validation of message structures according to the protocol rules
- Ensure WASM compatibility for browser-based implementations

## Core Message Architecture

TAP distinguishes between two types of messages:

1. **Primary Messages** - Can be created independently without a thread context:
   - `Transfer` - For asset transfers (TAIP-3)
   - `Payment` - For payment instructions (TAIP-14)
   - `Connect` - For establishing connections (TAIP-15)

2. **Response Messages** - Always created in response to a primary message and require a thread ID:
   - `Authorize`, `Reject`, `Settle`, etc.

The architecture uses these core traits:

1. **TapMessageBody** - For message conversion and validation
2. **Authorizable** - For creating response messages from primary messages
3. **Connectable** - For connecting Transfer and Payment to a previous Connect message

## Core Types Hierarchy

### Base Types and Common Structures

#### Identifier Types

```rust
// Decentralized Identifier (DID)
pub type DID = String; // Format: "did:method:method-specific-id"

// Internationalized Resource Identifier (IRI)
pub type IRI = String; // Format: "scheme:path"

// Chain Agnostic Identifiers
pub type CAIP2 = String; // Chain ID format: "namespace:reference"
pub type CAIP10 = String; // Account format: "namespace:reference:address"
pub type CAIP19 = String; // Asset format: "namespace:reference/asset_namespace:asset_reference"
pub type CAIP220 = String; // Transaction format: "namespace:reference/tx/txid"

// Digital Trust Identifier
pub type DTI = String; // Traditional finance identifier

// Asset identifier (union of blockchain and traditional assets)
pub type Asset = String; // Either CAIP19 or DTI

// ISO8601 DateTime
pub type ISO8601DateTime = String;

// Amount as a decimal string
pub type Amount = String;
```

#### Context and Type Handling

```rust
// Base structure for JSON-LD objects with context and type
pub struct JsonLdObject {
    #[serde(rename = "@context")]
    pub context: Option<serde_json::Value>, // Can be a string or object

    #[serde(rename = "@type")]
    pub type_: String,

    // Additional fields handled via dynamic map
    #[serde(flatten)]
    pub additional_fields: std::collections::HashMap<String, serde_json::Value>,
}

// TAP Context constants
pub const TAP_CONTEXT_V1: &str = "https://tap.rsvp/schema/1.0";
```

### Participant Structures

```rust
// Enum for participant types
pub enum ParticipantType {
    Party,
    Agent,
}

// Participant structure (for both parties and agents)
pub struct Participant {
    // JSON-LD identifier
    #[serde(rename = "@id")]
    pub id: String, // DID or IRI

    // JSON-LD type
    #[serde(rename = "@type")]
    pub type_: ParticipantType,

    // Optional LEI code
    #[serde(rename = "lei:leiCode", skip_serializing_if = "Option::is_none")]
    pub lei_code: Option<String>,

    // Optional human-readable name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    // Optional name hash for privacy-preserving matching
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_hash: Option<String>,

    // Optional role in the transaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    // Optional party this participant acts for (when participant is an agent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub for_: Option<String>,

    // Optional policies that apply to this participant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policies: Option<Vec<Policy>>,

    // Optional merchant category code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcc: Option<String>,

    // Additional fields
    #[serde(flatten)]
    pub additional_fields: std::collections::HashMap<String, serde_json::Value>,
}
```

### Policy Structures

```rust
// Base policy structure
pub struct Policy {
    // JSON-LD type
    #[serde(rename = "@type")]
    pub type_: String,

    // Optional DID of party required to fulfill policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,

    // Optional role required to fulfill policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_role: Option<String>,

    // Optional agent representing party required to fulfill policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_agent: Option<String>,

    // Optional purpose description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,

    // Type-specific fields based on policy type
    #[serde(flatten)]
    pub details: PolicyDetails,

    // Additional fields
    #[serde(flatten)]
    pub additional_fields: std::collections::HashMap<String, serde_json::Value>,
}

// Enum for different policy types
pub enum PolicyDetails {
    RequireAuthorization {},
    RequirePresentation {
        about_party: Option<String>,
        about_agent: Option<String>,
        presentation_definition: String,
        credential_type: Option<String>,
    },
    RequireRelationshipConfirmation {
        nonce: String,
    },
    RequirePurpose {
        fields: Vec<String>, // Can be "purpose" or "categoryPurpose"
    },
}
```

### DIDComm Message Structure

```rust
// Base DIDComm message structure
pub struct DIDCommMessage<T> {
    pub id: String,
    pub typ: String, // Media type
    pub type_: String, // Message type
    pub from: Option<String>,  // DID of sender
    pub to: Option<Vec<String>>,  // Array of recipient DIDs
    pub thid: Option<String>,  // Thread ID
    pub pthid: Option<String>,  // Parent thread ID
    pub created_time: Option<u64>,  // Unix timestamp
    pub expires_time: Option<u64>,  // Optional Unix timestamp
    pub body: T,  // Message body (generic)
    pub attachments: Option<Vec<Attachment>>, // Optional attachments
    pub extra_headers: std::collections::HashMap<String, serde_json::Value>, // Additional headers
}
```

## Core Traits

### TapMessageBody Trait

```rust
/// A trait for TAP message body types that can be serialized to and deserialized from DIDComm messages.
pub trait TapMessageBody: Serialize + DeserializeOwned {
    /// Get the message type string for this body type.
    fn message_type() -> &'static str;

    /// Validate the message body.
    fn validate(&self) -> Result<()>;

    /// Convert this body to a DIDComm message, automatically including all agents in the 'to' field.
    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message>;

    /// Convert this body to a DIDComm message with specific routing information.
    fn to_didcomm_with_route<'a, I>(&self, from: Option<&str>, to: I) -> Result<Message>
    where
        I: IntoIterator<Item = &'a str>;

    /// Create a reply message to an existing message.
    fn create_reply(
        &self,
        original: &Message,
        creator_did: &str,
        participant_dids: &[&str],
    ) -> Result<Message>;

    /// Extract this body type from a DIDComm message.
    fn from_didcomm(message: &Message) -> Result<Self>;
}
```

### Authorizable Trait

```rust
/// A trait for message types that can be responded to with authorizations, rejections, settlements, etc.
pub trait Authorizable {
    /// Authorizes this message, creating an Authorize message as a response
    fn authorize(
        &self,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Authorize;

    /// Rejects this message, creating a Reject message as a response
    fn reject(
        &self,
        code: String,
        description: String,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Reject;

    /// Settles this message, creating a Settle message as a response
    fn settle(
        &self,
        transaction_id: String,
        transaction_hash: Option<String>,
        block_height: Option<u64>,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Settle;

    /// Confirms a relationship between agents, creating a ConfirmRelationship message
    fn confirm_relationship(
        &self,
        agent_id: String,
        for_id: String,
        role: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> ConfirmRelationship;

    /// Updates policies for this message, creating an UpdatePolicies message
    fn update_policies(
        &self,
        policies: Vec<Policy>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> UpdatePolicies;

    /// Adds agents to this message, creating an AddAgents message
    fn add_agents(
        &self,
        agents: Vec<Participant>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> AddAgents;

    /// Replaces an agent in this message, creating a ReplaceAgent message
    fn replace_agent(
        &self,
        original: String,
        replacement: Participant,
        metadata: HashMap<String, serde_json::Value>,
    ) -> ReplaceAgent;

    /// Removes an agent from this message, creating a RemoveAgent message
    fn remove_agent(
        &self,
        agent: String,
        metadata: HashMap<String, serde_json::Value>,
    ) -> RemoveAgent;
}
```

### Connectable Trait

```rust
/// A trait for messages that can be connected to a prior Connect message
pub trait Connectable {
    /// Connect this message to a prior Connect message
    fn with_connection(&mut self, connect_id: &str) -> &mut Self;
    
    /// Check if this message is connected to a prior Connect message
    fn has_connection(&self) -> bool;
    
    /// Get the connection ID if present
    fn connection_id(&self) -> Option<&str>;
}
```

## Message Type Definitions

### Core Transaction Messages

```rust
// Transfer message (TAIP-3)
pub struct Transfer {
    pub asset: AssetId,  // CAIP19 or DTI
    pub amount: String,  // Decimal amount

    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,  // ISO 20022 purpose code

    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_purpose: Option<String>,  // ISO 20022 category purpose code

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,  // ISO 8601 DateTime

    pub originator: Participant,  // Sending party

    #[serde(skip_serializing_if = "Option::is_none")]
    pub beneficiary: Option<Participant>,  // Receiving party

    #[serde(default)]
    pub agents: Vec<Participant>,  // Involved agents

    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_id: Option<String>,  // CAIP-220 settlement transaction

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,  // Additional information

    // Additional fields
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

// Payment message (TAIP-14)
pub struct Payment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<AssetId>,  // CAIP-19 asset identifier

    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,  // ISO 4217 currency code

    pub amount: String,  // Decimal amount
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_assets: Option<Vec<String>>,  // List of acceptable CAIP-19 assets

    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice: Option<serde_json::Value>,  // Invoice object or URI

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,  // ISO 8601 DateTime

    pub merchant: Participant,  // Party requesting payment

    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<Participant>,  // Party from whom payment is requested

    #[serde(default)]
    pub agents: Vec<Participant>,  // Involved agents

    // Additional fields
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

// Connect message (TAIP-15)
pub struct Connect {
    #[serde(default)]
    pub policies: Vec<Policy>,  // Policies for this connection
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_constraints: Option<TransactionConstraints>,  // Optional constraints

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,  // ISO 8601 DateTime
    
    // Additional fields
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

// Transaction constraints for connections (TAIP-15)
pub struct TransactionConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purposes: Option<Vec<String>>,  // ISO 20022 purpose codes
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_purposes: Option<Vec<String>>,  // ISO 20022 category purpose codes
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<TransactionLimits>,  // Monetary limits
}

// Transaction limits for connections (TAIP-15)
pub struct TransactionLimits {
    pub per_transaction: String,  // Maximum amount per transaction
    pub daily: String,  // Maximum daily total
    pub currency: String,  // ISO 4217 currency code
}
```

### Response Messages

```rust
// Authorization message (TAIP-4)
pub struct Authorize {
    // ID of the transfer/payment being authorized (required)
    pub transfer_id: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,  // Optional note about the authorization

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,  // ISO 8601 DateTime

    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_address: Option<String>,  // Optional settlement address

    // Additional fields
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

// Reject message (TAIP-4)
pub struct Reject {
    // ID of the transfer/payment being rejected (required)
    pub transfer_id: String,
    
    // Rejection code (required)
    pub code: String,
    
    // Description of rejection reason (required)
    pub description: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,  // Optional note about the rejection
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,  // ISO 8601 DateTime

    // Additional fields
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

// Settle message (TAIP-4)
pub struct Settle {
    // ID of the transfer/payment being settled (required)
    pub transfer_id: String,
    
    // Transaction ID (required)
    pub transaction_id: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,  // Optional transaction hash
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_height: Option<u64>,  // Optional block height
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,  // Optional note about the settlement
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,  // ISO 8601 DateTime

    // Additional fields
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}
```

### Attachment Structure

```rust
// Attachment data for TAP messages
pub struct AttachmentData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base64: Option<String>,  // Base64-encoded data

    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<serde_json::Value>,  // JSON data
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,  // Links to external data
}

// Attachment for TAP messages
pub struct Attachment {
    pub id: String,  // Attachment ID

    #[serde(rename = "media_type")]
    pub media_type: String,  // Media type

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<AttachmentData>,  // Attachment data
}
```

## Implementation Plan

### Builder Pattern Implementation

All primary message types (Transfer, Payment, Connect) should have an associated builder:

```rust
pub struct TransferBuilder {
    // Fields for building a Transfer
    asset: Option<AssetId>,
    amount: Option<String>,
    purpose: Option<String>,
    category_purpose: Option<String>,
    expiry: Option<String>,
    originator: Option<Participant>,
    beneficiary: Option<Participant>,
    agents: Vec<Participant>,
    settlement_id: Option<String>,
    memo: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl TransferBuilder {
    // Constructor
    pub fn new() -> Self {
        Self {
            asset: None,
            amount: None,
            purpose: None,
            category_purpose: None,
            expiry: None,
            originator: None,
            beneficiary: None,
            agents: Vec::new(),
            settlement_id: None,
            memo: None,
            metadata: HashMap::new(),
        }
    }
    
    // Builder methods
    pub fn asset(mut self, asset: AssetId) -> Self {
        self.asset = Some(asset);
        self
    }
    
    pub fn amount(mut self, amount: impl Into<String>) -> Self {
        self.amount = Some(amount.into());
        self
    }
    
    // Other setter methods...
    
    // Build method
    pub fn build(self) -> Result<Transfer, Error> {
        // Validate required fields
        let asset = self.asset.ok_or_else(|| Error::MissingField("asset"))?;
        let amount = self.amount.ok_or_else(|| Error::MissingField("amount"))?;
        let originator = self.originator.ok_or_else(|| Error::MissingField("originator"))?;
        
        let transfer = Transfer {
            asset,
            amount,
            purpose: self.purpose,
            category_purpose: self.category_purpose,
            expiry: self.expiry,
            originator,
            beneficiary: self.beneficiary,
            agents: self.agents,
            settlement_id: self.settlement_id,
            memo: self.memo,
            metadata: self.metadata,
        };
        
        // Validate the built Transfer
        transfer.validate()?;
        
        Ok(transfer)
    }
}
```

## Implementation Task List

### Common Types

- [x] Implement all base type aliases (DID, IRI, CAIP types, etc.)
- [x] Create JsonLdObject structure with context handling
- [x] Define TAP context constants

### Participants and Policies

- [x] Implement ParticipantType enum
- [x] Create Participant struct with proper JSON-LD serialization
- [x] Implement Policy structure with PolicyDetails enum
- [x] Ensure proper serialization/deserialization for all structures

### Core Traits

- [x] Implement TapMessageBody trait
  - [x] Create message_type static method
  - [x] Implement validation method
  - [x] Implement to_didcomm and from_didcomm methods
  - [x] Implement to_didcomm_with_route method
  - [x] Create create_reply method

- [x] Implement Authorizable trait
  - [x] Create authorize method for creating Authorize responses
  - [x] Create reject method for creating Reject responses
  - [x] Create settle method for creating Settle responses
  - [x] Implement confirm_relationship method
  - [x] Implement update_policies method
  - [x] Implement agent management methods (add_agents, replace_agent, remove_agent)

- [x] Implement Connectable trait
  - [x] Create with_connection method for setting parent thread ID
  - [x] Implement has_connection and connection_id methods

### Primary Message Types

- [x] Implement Transfer struct
  - [x] Create builder pattern
  - [x] Implement TapMessageBody trait
  - [x] Implement Authorizable trait
  - [x] Implement Connectable trait
  - [x] Add validation logic

- [x] Implement Payment struct
  - [x] Create builder pattern
  - [x] Implement TapMessageBody trait
  - [x] Implement Authorizable trait
  - [x] Implement Connectable trait
  - [x] Add validation logic

- [x] Implement Connect struct
  - [x] Create builder pattern
  - [x] Implement TapMessageBody trait
  - [x] Add validation logic

### Response Message Types

- [x] Implement Authorize struct with TapMessageBody trait
- [x] Implement Reject struct with TapMessageBody trait
- [x] Implement Settle struct with TapMessageBody trait
- [x] Implement other response types with TapMessageBody trait

### Serialization/Deserialization

- [x] Implement custom JSON-LD serialization/deserialization
- [x] Create proper handling for context fields
- [x] Handle type field conversion

### Validation 

- [x] Create validation logic for all message types
- [x] Implement cross-field validation
- [x] Create informative validation error messages

### Builder Patterns

- [x] Create builders for all primary message types
- [x] Implement proper error handling in builders
- [x] Create ergonomic builder interfaces

### WASM Compatibility

- [ ] Ensure all types are WASM-compatible
- [ ] Create proper bindings for JavaScript interoperability
- [ ] Test serialization/deserialization in WASM context

### Testing

- [x] Create unit tests for all types and traits
- [x] Test serialization/deserialization
- [x] Test validation logic
- [x] Test builder patterns
- [ ] Create integration tests with sample messages

## Implementation Status

The TAP message types implementation is now largely complete. The core architecture has been successfully implemented with:

1. **Core Traits**:
   - The `TapMessageBody` trait provides conversion between TAP message bodies and DIDComm messages
   - The `Authorizable` trait enables creating response messages from primary messages
   - The `Connectable` trait allows connecting Transfer and Payment messages to a previous Connect message

2. **Primary Message Types**:
   - Transfer, Payment, and Connect message types are fully implemented
   - Builder patterns are in place for ergonomic message creation
   - Validation logic ensures message integrity

3. **Response Message Types**:
   - All response types (Authorize, Reject, Settle, etc.) are implemented
   - Proper thread handling for message correlation

4. **Remaining Work**:
   - WASM compatibility needs to be addressed for browser implementations
   - Additional integration tests with sample messages would improve test coverage

The implementation follows Rust best practices with proper error handling, type safety, and comprehensive documentation. The architecture maintains a clean separation of concerns between TAP protocol logic and DIDComm transport while providing a straightforward API.

## API Usage Examples

### Creating a Transfer Message

```rust
use tap_msg::{Transfer, TransferBuilder, Participant, AssetId};
use std::str::FromStr;

// Create a new Transfer using the builder pattern
let asset = AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap(); // USDC on Ethereum
let originator = Participant::new("did:web:originator.example");
let beneficiary = Participant::new("did:web:beneficiary.example");

let transfer = TransferBuilder::new()
    .asset(asset)
    .amount("100.00")
    .originator(originator)
    .beneficiary(beneficiary)
    .memo("Payment for services")
    .build()?;

// Convert to DIDComm message
let message = transfer.to_didcomm(Some("did:web:sender.example"))?;

// Send the message (implementation-dependent)
// ...
```

### Responding to a Transfer Message

```rust
use tap_msg::{Authorizable, Message};
use std::collections::HashMap;

// Assume we've received a transfer DIDComm message
let received_message: Message = receive_message()?;

// Authorize the transfer
let authorize_body = received_message.authorize(
    Some("Transfer approved".to_string()),
    HashMap::new(),
);

// Create a DIDComm reply message
let reply = authorize_body.create_reply(
    &received_message,
    "did:web:authorizer.example",
    &["did:web:recipient1.example", "did:web:recipient2.example"],
)?;

// Send the authorization reply
// ...
```

### Connecting a Payment to a Previous Connect Message

```rust
use tap_msg::{Payment, PaymentBuilder, Participant, Connectable};
use std::collections::HashMap;

// Create a payment
let merchant = Participant::new("did:web:merchant.example");
let payment = PaymentBuilder::new()
    .amount("50.00")
    .currency("USD")
    .merchant(merchant)
    .build()?;

// Convert to DIDComm message
let mut message = payment.to_didcomm(Some("did:web:sender.example"))?;

// Connect to previous connect message
message.with_connection("abc123-connect-message-id");

// Send the message
// ...
```

## Conclusion

This PRD outlines a comprehensive approach to implementing the TAP protocol type system in Rust. By following this plan, we can create a robust, maintainable, and protocol-compliant implementation that leverages Rust's strengths while maintaining compatibility with the JSON-LD format required by TAP.

The design focuses on a trait-based approach with:
1. Clear distinction between primary and response messages
2. Ergonomic builder patterns for message creation
3. Comprehensive validation
4. Clean DIDComm message conversion
5. Strong WASM compatibility for browser implementations

The implementation will follow Rust best practices, including proper error handling, type safety, and documentation.
