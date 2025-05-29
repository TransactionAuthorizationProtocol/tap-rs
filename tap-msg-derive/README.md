# TAP Message Derive Macro

Procedural derive macro for automatically implementing TAP message traits.

## Overview

This crate provides the `#[derive(TapMessage)]` procedural macro that automatically implements both `TapMessage` and `MessageContext` traits for TAP protocol message types. It reduces boilerplate by generating implementations based on struct field attributes.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
tap-msg = "0.2.0"
tap-msg-derive = "0.2.0"
serde = { version = "1.0", features = ["derive"] }
```

## Basic Example

```rust
use tap_msg::TapMessage;
use tap_msg::message::{Participant, TapMessageBody};
use tap_msg::didcomm::PlainMessage;
use tap_msg::error::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
pub struct Transfer {
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
    pub asset_id: String,
}

// You still need to implement TapMessageBody for message-specific logic
impl TapMessageBody for Transfer {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#transfer"
    }
    
    fn validate(&self) -> Result<()> {
        if self.amount.is_empty() {
            return Err(tap_msg::error::Error::Validation("Amount required".to_string()));
        }
        Ok(())
    }
    
    fn to_didcomm(&self, from: &str) -> Result<PlainMessage> {
        // Convert to DIDComm message format
        // Implementation details...
    }
}
```

## Supported Attributes

### Field-level Attributes

- `#[tap(participant)]` - Marks a field as a single participant (type: `Participant` or `Option<Participant>`)
- `#[tap(participant_list)]` - Marks a field as a list of participants (type: `Vec<Participant>`)
- `#[tap(transaction_id)]` - Marks the transaction ID field (type: `String`)
- `#[tap(optional_transaction_id)]` - Marks an optional transaction ID field (type: `Option<String>`)
- `#[tap(thread_id)]` - Marks a thread ID field for thread-based messages (type: `Option<String>`)

## What Gets Generated

The derive macro generates implementations for two traits:

### 1. TapMessage Trait

```rust
pub trait TapMessage {
    fn validate(&self) -> Result<()>;
    fn is_tap_message(&self) -> bool;
    fn get_tap_type(&self) -> Option<String>;
    fn get_all_participants(&self) -> Vec<String>;
    fn create_reply<T: TapMessageBody>(&self, body: &T, creator_did: &str) -> Result<PlainMessage>;
    fn message_type(&self) -> &'static str;
    fn thread_id(&self) -> Option<&str>;
    fn parent_thread_id(&self) -> Option<&str>;
    fn message_id(&self) -> &str;
}
```

The generated implementation:
- Extracts participant DIDs from all marked fields
- Returns the appropriate thread/transaction ID
- Creates properly threaded reply messages
- Delegates validation to your `TapMessageBody` implementation

### 2. MessageContext Trait

```rust
pub trait MessageContext {
    fn participants(&self) -> Vec<&Participant>;
    fn participant_dids(&self) -> Vec<String>;
    fn transaction_context(&self) -> Option<TransactionContext>;
}
```

The generated implementation:
- Collects references to all `Participant` objects
- Extracts DIDs from all participants
- Creates transaction context with ID and message type

## Advanced Examples

### Message with Optional Transaction ID

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
pub struct Presentation {
    #[tap(participant)]
    pub presenter: Participant,
    
    #[tap(optional_transaction_id)]
    pub transaction_id: Option<String>,
    
    pub credentials: Vec<String>,
}
```

### Thread-based Message

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
pub struct DIDCommPresentation {
    #[tap(thread_id)]
    pub thid: Option<String>,
    
    pub formats: Vec<String>,
    pub attachments: Vec<Attachment>,
}
```

### Message with Multiple Participant Lists

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
pub struct MultiPartyTransfer {
    #[tap(participant)]
    pub initiator: Participant,
    
    #[tap(participant_list)]
    pub senders: Vec<Participant>,
    
    #[tap(participant_list)]
    pub receivers: Vec<Participant>,
    
    #[tap(participant_list)]
    pub validators: Vec<Participant>,
    
    #[tap(transaction_id)]
    pub transaction_id: String,
}
```

## Integration with tap-msg

This derive macro is designed to work seamlessly with the tap-msg crate. When both are used together:

1. Define your message struct with the derive macro
2. Implement `TapMessageBody` for message-specific behavior
3. The macro handles the boilerplate trait implementations
4. Your message type is ready to use with TAP agents and protocols

## Limitations

- Only works with structs that have named fields
- Requires serde `Serialize` and `Deserialize` to be derived
- The `TapMessageBody` trait must still be implemented manually
- Field types must match exactly (e.g., `Participant`, not a type alias)

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE-MIT) file for details.