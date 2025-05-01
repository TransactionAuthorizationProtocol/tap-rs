# TAP Message

Core message processing for the Transaction Authorization Protocol (TAP) with integrated DIDComm support.

## Features

- **TAP Message Types**: Complete implementation of all TAP message types
- **DIDComm Integration**: Direct conversion between TAP messages and DIDComm messages
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

### Payment Request with Invoice

```rust
use tap_msg::{PaymentRequest, Invoice, LineItem, Participant};
use tap_msg::message::tap_message_trait::TapMessageBody;
use std::collections::HashMap;

// Create a merchant and a basic invoice
let merchant = Participant {
    id: "did:example:merchant".to_string(),
    role: Some("merchant".to_string()),
    policies: None,
    leiCode: None,
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
let mut payment_request = PaymentRequest::with_currency(
    "USD".to_string(),
    "100.0".to_string(),
    merchant,
    vec![],  // No additional agents in this simple example
);
payment_request.invoice = Some(invoice);

// Send the payment request to a customer
let message = payment_request.to_didcomm_with_route(
    Some("did:example:merchant"),
    ["did:example:customer"].iter().copied()
)?;
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

### Invoice and Payment Requests (TAIP-14, TAIP-16)

TAP supports structured invoices according to TAIP-16, which can be embedded in payment requests (TAIP-14):

```rust
use tap_msg::{PaymentRequest, Invoice, LineItem, TaxCategory, TaxTotal, TaxSubtotal};
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::Participant;
use std::collections::HashMap;

// Create a merchant participant
let merchant = Participant {
    id: "did:example:merchant".to_string(),
    role: Some("merchant".to_string()),
    policies: None,
    leiCode: None,
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
let mut payment_request = PaymentRequest::with_currency(
    "USD".to_string(),
    "25.0".to_string(),
    merchant.clone(),
    vec![],  // Agents involved in the payment
);

// Add the invoice to the payment request
payment_request.invoice = Some(invoice);

// Convert to DIDComm message to send to the customer
let message = payment_request.to_didcomm_with_route(
    Some("did:example:merchant"),
    ["did:example:customer"].iter().copied()
).unwrap();

// When receiving a payment request, extract and validate the invoice
let received_request = PaymentRequest::from_didcomm(&message).unwrap();
received_request.validate().unwrap();

if let Some(received_invoice) = received_request.invoice {
    println!("Invoice ID: {}", received_invoice.id);
    println!("Total amount: {}", received_invoice.total);
    
    // Process line items
    for item in received_invoice.line_items {
        println!("{} x {} @ ${} = ${}", 
            item.quantity, 
            item.description, 
            item.unit_price, 
            item.line_total
        );
    }
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
