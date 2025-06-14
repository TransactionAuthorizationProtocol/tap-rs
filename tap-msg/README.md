# TAP Message

Core message processing for the Transaction Authorization Protocol (TAP) providing secure message types and validation.

## Features

- **TAP Message Types**: Complete implementation of all TAP message types
- **Generic Typed Messages**: Compile-time type safety with `PlainMessage<Transfer>` while maintaining backward compatibility
- **Derive Macro**: Automatic implementation of `TapMessage` and `MessageContext` traits with `#[derive(TapMessage)]`
- **Message Security**: Support for secure message formats with JWS (signed) and JWE (encrypted) capabilities
- **Attachments Support**: Full support for message attachments in Base64, JSON, and Links formats with optional JWS
- **Validation**: Proper validation of all message fields and formats
- **CAIP Support**: Validation for chain-agnostic identifiers (CAIP-2, CAIP-10, CAIP-19)
- **Authorization Flows**: Support for authorization, rejection, and settlement flows
- **Agent Policies**: TAIP-7 compliant policy implementation for defining agent requirements
- **Invoice Support**: TAIP-16 compliant structured invoice implementation with tax and line item support
- **Payment Requests**: TAIP-14 compliant payment requests with currency and asset options
- **Name Hashing**: TAIP-12 compliant name hashing for privacy-preserving Travel Rule compliance
- **Extensibility**: Easy addition of new message types

## Usage

### Basic Transfer Message

```rust
use tap_msg::message::Transfer;
use tap_msg::message::Participant;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_caip::AssetId;
use std::collections::HashMap;
use std::str::FromStr;

// Create a Transfer message body
let asset = AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7")?;

let originator = Participant {
    id: "did:example:sender".to_string(),
    role: Some("originator".to_string()),
    policies: None,
    leiCode: None,
    name: None,
};

let beneficiary = Participant {
    id: "did:example:receiver".to_string(),
    role: Some("beneficiary".to_string()),
    policies: None,
    leiCode: None,
    name: None,
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

// Use the TapMessageBody trait for validation
transfer.validate()?;

// In a full implementation, you would use an agent to send this transfer message:
// let (packed_message, _) = agent.send_message(&transfer, vec![&beneficiary.id], true).await?;
```

### Payment Request with Invoice

```rust
use tap_msg::message::{Payment, Invoice, InvoiceReference, LineItem, Participant};
use tap_msg::message::tap_message_trait::TapMessageBody;
use std::collections::HashMap;

// Create a merchant and a basic invoice
let merchant = Participant {
    id: "did:example:merchant".to_string(),
    role: Some("merchant".to_string()),
    policies: None,
    leiCode: None,
    name: None,
};

// Create line items for the invoice
let line_items = vec![
    LineItem {
        id: "1".to_string(),
        description: "Premium Service".to_string(),
        quantity: 1.0,
        unit_code: None,
        unit_price: 100.0,
        line_total: 100.0,
        tax_category: None,
    }
];

// Create a basic invoice
let invoice = Invoice::new(
    "INV-2023-001".to_string(),
    "2023-06-01".to_string(),
    "USD".to_string(),
    line_items,
    100.0,
);

// Create a payment request with the invoice
let mut payment_request = Payment::with_currency(
    "USD".to_string(),
    "100.0".to_string(),
    merchant,
    vec![],  // No additional agents in this simple example
);
payment_request.invoice = Some(InvoiceReference::Invoice(invoice));

// Validate the message
payment_request.validate()?;
```

## Message Types

### Plain Message

The `PlainMessage` struct is the core representation of a message in TAP:

```rust
pub struct PlainMessage {
    pub id: String,
    pub typ: String,
    pub type_: String,
    pub body: Value,
    pub from: String,
    pub to: Vec<String>,
    pub thid: Option<String>,
    pub pthid: Option<String>,
    pub extra_headers: HashMap<String, Value>,
    pub created_time: Option<u64>,
    pub expires_time: Option<u64>,
    pub from_prior: Option<String>,
    pub attachments: Option<Vec<Attachment>>,
}
```

The `attachments` field supports all attachment formats and can be used to include additional data with messages.

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
    pub metadata: HashMap<String, serde_json::Value>,
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
    leiCode: None,
    name: None,
};

// Create an UpdatePolicies message to dynamically update policies
let proof_policy = RequireProofOfControl::default(); // Uses default values
let update_policies = UpdatePolicies {
    transaction_id: "transfer_12345".to_string(),
    policies: vec![Policy::RequireProofOfControl(proof_policy)],
};

// Validate the message
update_policies.validate()?;
```

### Authorization Messages

TAP supports various authorization messages for compliance workflows:

```rust
// Authorization message
pub struct Authorize {
    pub transfer_id: String,
    pub note: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

// Rejection message
pub struct Reject {
    pub transaction_id: String,
    pub reason: Option<String>,
    pub note: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

// Settlement message
pub struct Settle {
    pub transaction_id: String,
    pub settlement_id: Option<String>,
    pub amount: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

### Error Messages

TAP provides standardized error messages:

```rust
pub struct ErrorBody {
    /// Error code.
    pub code: String,

    /// Error description.
    pub description: String,

    /// Original message ID (if applicable).
    pub original_message_id: Option<String>,

    /// Additional metadata.
    pub metadata: HashMap<String, serde_json::Value>,
}
```

### Presentation

The `Presentation` struct represents a verifiable presentation message:

```rust
pub struct Presentation {
    pub formats: Vec<String>,
    pub attachments: Vec<Attachment>,
    pub thid: Option<String>,
}
```

This structure enables compatibility with the present-proof protocol and enforces the requirement for format field and attachments validation. The format field is required for each attachment and must match one of the formats specified in the presentation.

### Invoice and Payment Requests (TAIP-14, TAIP-16)

TAP supports structured invoices according to TAIP-16, which can be embedded in payment requests (TAIP-14):

```rust
use tap_msg::message::{Payment, Invoice, InvoiceReference, LineItem, TaxCategory, TaxTotal, TaxSubtotal};
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::Participant;
use std::collections::HashMap;

// Create a merchant participant
let merchant = Participant {
    id: "did:example:merchant".to_string(),
    role: Some("merchant".to_string()),
    policies: None,
    leiCode: None,
    name: None,
};

// Create a simple invoice with line items
let invoice = Invoice {
    id: "INV001".to_string(),
    issue_date: "2023-05-15".to_string(),
    currency_code: "USD".to_string(),
    line_items: vec![
        LineItem {
            id: "1".to_string(),
            description: "Product A".to_string(),
            quantity: 2.0,
            unit_code: Some("EA".to_string()),
            unit_price: 10.0,
            line_total: 20.0,
            tax_category: None,
        },
        LineItem {
            id: "2".to_string(),
            description: "Product B".to_string(),
            quantity: 1.0,
            unit_code: Some("EA".to_string()),
            unit_price: 5.0,
            line_total: 5.0,
            tax_category: None,
        },
    ],
    tax_total: None,
    total: 25.0,
    sub_total: Some(25.0),
    due_date: None,
    note: None,
    payment_terms: None,
    accounting_cost: None,
    order_reference: None,
    additional_document_reference: None,
    metadata: HashMap::new(),
};

// Create a payment request with the invoice
let mut payment_request = Payment::with_currency(
    "USD".to_string(),
    "25.0".to_string(),
    merchant.clone(),
    vec![],  // Agents involved in the payment
);

// Add the invoice to the payment request
payment_request.invoice = Some(InvoiceReference::Invoice(invoice));

// Alternatively, you can reference an invoice by URL
// payment_request.invoice = Some(InvoiceReference::Url("https://example.com/invoice/123".to_string()));
```

## Generic Typed Messages

TAP-MSG now supports compile-time type safety through generic `PlainMessage<T>` while maintaining 100% backward compatibility:

### Creating Typed Messages

```rust
use tap_msg::{PlainMessage, Transfer, Participant};
use std::collections::HashMap;

// Create a Transfer message body
let transfer = Transfer {
    asset: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".parse()?,
    originator: Participant::new("did:example:alice"),
    beneficiary: Some(Participant::new("did:example:bob")),
    amount: "100".to_string(),
    agents: vec![],
    memo: Some("Payment for services".to_string()),
    settlement_id: None,
    transaction_id: "tx-123".to_string(),
    metadata: HashMap::new(),
};

// Create a strongly-typed message with builder pattern
let typed_msg = PlainMessage::new_typed(transfer, "did:example:alice")
    .with_recipient("did:example:bob")
    .with_thread_id(Some("payment-123".to_string()))
    .with_expires_at(chrono::Utc::now().timestamp() as u64 + 3600);

// Access the typed body with compile-time safety
println!("Transfer amount: {}", typed_msg.body.amount);
println!("Originator: {}", typed_msg.body.originator.id);
```

### Converting Between Typed and Untyped

```rust
// Convert typed message to untyped for serialization/transport
let untyped_msg: PlainMessage<Value> = typed_msg.to_plain_message()?;

// Parse untyped message to specific type
let untyped_msg: PlainMessage<Value> = serde_json::from_str(json_data)?;
let typed_msg: PlainMessage<Transfer> = untyped_msg.parse_body()?;

// Alternative using parse_as method
let typed_msg: PlainMessage<Transfer> = untyped_msg.parse_as()?;
```

### Runtime Dispatch with TapMessage Enum

```rust
use tap_msg::message::TapMessage;

// Parse any TAP message for runtime dispatch
let plain_msg: PlainMessage<Value> = serde_json::from_str(json_data)?;

match plain_msg.parse_tap_message()? {
    TapMessage::Transfer(transfer) => {
        println!("Transfer amount: {}", transfer.amount);
    },
    TapMessage::Authorize(auth) => {
        println!("Authorization for: {}", auth.transaction_id);
    },
    TapMessage::Reject(reject) => {
        println!("Rejection reason: {}", reject.reason);
    },
    // ... handle other message types
    _ => println!("Other message type"),
}
```

### Backward Compatibility

```rust
// Existing code continues to work unchanged
let plain_msg: PlainMessage = serde_json::from_str(json_data)?;
// This is now PlainMessage<Value> due to default type parameter

// All existing methods work unchanged
println!("From: {}", plain_msg.from);
println!("Type: {}", plain_msg.type_);
```

See [GENERIC_PLAINMESSAGE.md](../GENERIC_PLAINMESSAGE.md) for complete documentation.

## Message Security

The TAP protocol provides several security modes:

- **Plain**: No security, for testing only
- **Signed**: Messages are signed to ensure integrity
- **AuthCrypt**: Messages are both signed and encrypted for confidentiality

When using the `tap-agent` crate with this message library, you can specify the security mode when sending messages:

```rust
use tap_agent::agent::Agent;
use tap_agent::message::SecurityMode;
use tap_msg::message::Transfer;

async fn send_secure_message(
    agent: &impl Agent,
    transfer: &Transfer,
    recipient: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Send with authenticated encryption
    let (packed_message, _) = agent.send_message(
        transfer,
        vec![recipient],
        SecurityMode::AuthCrypt,
        false, // Don't automatically deliver
    ).await?;

    // The packed_message is now a secure JWE format message
    Ok(packed_message)
}
```

## Message Attachments

TAP supports message attachments through the `Attachment` struct and related types:

```rust
use tap_msg::didcomm::{Attachment, AttachmentData, JsonAttachmentData};
use serde_json::json;

// Create a JSON attachment
let attachment = Attachment {
    id: Some("attachment-1".to_string()),
    media_type: Some("application/json".to_string()),
    data: AttachmentData::Json {
        value: JsonAttachmentData {
            json: json!({
                "key": "value",
                "nested": {
                    "data": "example"
                }
            }),
            jws: None,
        },
    },
    description: Some("Example attachment".to_string()),
    filename: None,
    format: Some("json/schema@v1".to_string()),
    lastmod_time: None,
    byte_count: None,
};

// Create a presentation with attachment
let presentation = Presentation {
    formats: vec!["dif/presentation-exchange/submission@v1.0".to_string()],
    attachments: vec![attachment],
    thid: Some("thread-123".to_string()),
};

// Validate the presentation
presentation.validate()?;
```

## Message Validation

TAP messages implement the `TapMessageBody` trait, which provides a `validate()` method for checking message correctness:

```rust
pub trait TapMessageBody: DeserializeOwned + Serialize + Send + Sync {
    /// Gets the message type string for this TAP message type
    fn message_type() -> &'static str;

    /// Converts the message to its wire format representation
    fn to_wire(&self) -> Result<Value>;

    /// Validates the message content
    fn validate(&self) -> Result<()>;
}
```

## Authorizable Trait

The `Authorizable` trait provides methods for handling authorization, rejection, and settlement flows:

```rust
pub trait Authorizable {
    /// Get the message ID for this message
    fn message_id(&self) -> &str;

    /// Create an authorization message for this message
    fn authorize(&self, note: Option<String>) -> Authorize;

    /// Create a rejection message for this message
    fn reject(&self, reason: String, note: Option<String>) -> Reject;

    /// Create a settlement message for this message
    fn settle(
        &self,
        settlement_id: String,
        amount: Option<String>,
    ) -> Settle;

    /// Create a cancellation message for this message
    fn cancel(&self, reason: Option<String>, note: Option<String>) -> Cancel;

    /// Updates policies for this message, creating an UpdatePolicies message as a response
    fn update_policies(&self, transaction_id: String, policies: Vec<Policy>) -> UpdatePolicies;

    /// Add agents to this message, creating an AddAgents message as a response
    fn add_agents(&self, transaction_id: String, agents: Vec<Participant>) -> AddAgents;

    /// Replace an agent in this message, creating a ReplaceAgent message as a response
    fn replace_agent(
        &self,
        transaction_id: String,
        original: String,
        replacement: Participant,
    ) -> ReplaceAgent;

    /// Remove an agent from this message, creating a RemoveAgent message as a response
    fn remove_agent(&self, transaction_id: String, agent: String) -> RemoveAgent;
}
```

## Derive Macro for TAP Messages

The `#[derive(TapMessage)]` macro automatically implements both `TapMessage` and `MessageContext` traits based on field attributes:

### Basic Usage

```rust
use tap_msg::TapMessage;
use tap_msg::message::{Participant, TapMessageBody};
use tap_msg::didcomm::PlainMessage;
use tap_msg::error::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
pub struct CustomTransfer {
    /// Originator participant - automatically extracted
    #[tap(participant)]
    pub originator: Participant,
    
    /// Optional beneficiary - automatically handled
    #[tap(participant)]
    pub beneficiary: Option<Participant>,
    
    /// List of agents - automatically extracted
    #[tap(participant_list)]
    pub agents: Vec<Participant>,
    
    /// Transaction ID for message threading
    #[tap(transaction_id)]
    pub transaction_id: String,
    
    // Regular fields don't need attributes
    pub amount: String,
    pub memo: Option<String>,
}

// You still need to implement TapMessageBody for message-specific logic
impl TapMessageBody for CustomTransfer {
    fn message_type() -> &'static str {
        "https://example.com/custom-transfer"
    }
    
    fn validate(&self) -> Result<()> {
        if self.amount.is_empty() {
            return Err(tap_msg::error::Error::Validation("Amount required".to_string()));
        }
        Ok(())
    }
    
    fn to_didcomm(&self, from: &str) -> Result<PlainMessage> {
        // Implementation details...
    }
}
```

### Supported Attributes

- `#[tap(participant)]` - Single participant field (required or optional)
- `#[tap(participant_list)]` - Vec<Participant> field
- `#[tap(transaction_id)]` - Transaction ID field for threading
- `#[tap(optional_transaction_id)]` - Optional transaction ID
- `#[tap(thread_id)]` - Thread ID field for thread-based messages

### What the Macro Provides

The derive macro automatically implements:

1. **TapMessage trait**:
   - `thread_id()` - Returns transaction/thread ID
   - `message_id()` - Returns appropriate message identifier
   - `get_all_participants()` - Extracts all participant DIDs
   - `create_reply()` - Creates properly threaded reply messages

2. **MessageContext trait**:
   - `participants()` - Returns references to all Participant objects
   - `participant_dids()` - Returns all participant DIDs
   - `transaction_context()` - Returns transaction context with ID and type

### Example with Generated Methods

```rust
let transfer = CustomTransfer {
    originator: Participant::new("did:example:alice"),
    beneficiary: Some(Participant::new("did:example:bob")),
    agents: vec![Participant::new("did:example:agent")],
    transaction_id: "tx-123".to_string(),
    amount: "100".to_string(),
    memo: None,
};

// Automatically implemented methods:
println!("Thread ID: {:?}", transfer.thread_id());  // Some("tx-123")
println!("All participants: {:?}", transfer.get_all_participants());
// ["did:example:alice", "did:example:bob", "did:example:agent"]

// MessageContext methods:
let participants = transfer.participants();  // Vec of &Participant
let tx_context = transfer.transaction_context();  // TransactionContext
```

## Name Hashing (TAIP-12)

TAP supports privacy-preserving name sharing through TAIP-12 compliant hashing:

```rust
use tap_msg::utils::{hash_name, NameHashable};
use tap_msg::message::Party;

// Hash a name directly
let hash = hash_name("Alice Lee");
assert_eq!(hash, "b117f44426c9670da91b563db728cd0bc8bafa7d1a6bb5e764d1aad2ca25032e");

// Use with Party objects
let party = Party::new("did:example:alice")
    .with_name_hash("Alice Lee");

// The party now includes the nameHash metadata
assert_eq!(
    party.name_hash().unwrap(),
    "b117f44426c9670da91b563db728cd0bc8bafa7d1a6bb5e764d1aad2ca25032e"
);

// Implement NameHashable for your own types
struct Customer {
    name: String,
}

impl NameHashable for Customer {}

// Now you can use the trait method
let hash = Customer::hash_name("Bob Smith");
assert_eq!(hash, "5432e86b4d4a3a2b4be57b713b12c5c576c88459fe1cfdd760fd6c99a0e06686");
```

The name hashing follows TAIP-12 normalization:
1. Remove all whitespace characters
2. Convert to uppercase
3. Apply SHA-256 hash
4. Return as lowercase hex string

This enables compliance with Travel Rule requirements while preserving privacy by sharing only hashed names that can be matched against sanctions lists without revealing personal information.

## Adding New Message Types

To add a new TAP message type, you have two options:

### Option 1: Using the Derive Macro (Recommended)

1. Define your message struct with appropriate field attributes
2. Add `#[derive(TapMessage)]` to automatically implement traits
3. Implement the `TapMessageBody` trait for message-specific logic
4. Optional: Implement the `Authorizable` trait for messages that can be authorized

### Option 2: Manual Implementation

To add a new TAP message type manually, follow these steps:

1. Define your message struct with required fields
2. Implement the `TapMessageBody` trait for your struct
3. Implement the `TapMessage` trait manually
4. Optional: Implement the `MessageContext` trait for participant extraction
5. Optional: Implement the `Authorizable` trait for messages that can be authorized

Here's an example of manual implementation:

```rust
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::error::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Define your new message struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyNewMessage {
    /// Unique identifier for the message
    pub id: String,

    /// Other fields specific to your message
    pub field1: String,
    pub field2: Option<String>,

    /// Optional metadata field
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Implement the TapMessageBody trait
impl TapMessageBody for MyNewMessage {
    /// Define the message type string (typically follows the TAP schema format)
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#mynewmessage"
    }

    /// Convert to wire format (serialized JSON structure)
    fn to_wire(&self) -> Result<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| tap_msg::error::Error::Serialization(e.to_string()))
    }

    /// Implement validation logic for your message fields
    fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(tap_msg::error::Error::Validation("ID is required".to_string()));
        }

        if self.field1.is_empty() {
            return Err(tap_msg::error::Error::Validation("Field1 is required".to_string()));
        }

        Ok(())
    }
}
```

## Examples

See the [examples directory](./examples) for more detailed examples of using TAP messages.
