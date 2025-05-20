# TAP Message

Core message processing for the Transaction Authorization Protocol (TAP) providing secure message types and validation.

## Features

- **TAP Message Types**: Complete implementation of all TAP message types
- **Message Security**: Support for secure message formats with JWS (signed) and JWE (encrypted) capabilities
- **Attachments Support**: Full support for message attachments in Base64, JSON, and Links formats with optional JWS
- **Validation**: Proper validation of all message fields and formats
- **CAIP Support**: Validation for chain-agnostic identifiers (CAIP-2, CAIP-10, CAIP-19)
- **Authorization Flows**: Support for authorization, rejection, and settlement flows
- **Agent Policies**: TAIP-7 compliant policy implementation for defining agent requirements
- **Invoice Support**: TAIP-16 compliant structured invoice implementation with tax and line item support
- **Payment Requests**: TAIP-14 compliant payment requests with currency and asset options
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
use tap_msg::message::{Payment, Invoice, LineItem, Participant};
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
payment_request.invoice = Some(invoice);

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
    pub reason: String,
    pub note: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

// Settlement message
pub struct Settle {
    pub transaction_id: String,
    pub settlement_id: String,
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
use tap_msg::message::{Payment, Invoice, LineItem, TaxCategory, TaxTotal, TaxSubtotal};
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
payment_request.invoice = Some(invoice);
```

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

## Adding New Message Types

To add a new TAP message type, follow these steps:

1. Define your message struct with required fields
2. Implement the `TapMessageBody` trait for your struct
3. Optional: Implement the `Authorizable` trait for messages that can be authorized

Here's an example:

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
