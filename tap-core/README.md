# tap-core: TAP Core Message Processing

The `tap-core` crate provides core message processing functionality for the Transaction Authorization Protocol (TAP). This library handles message types, validation, and serialization according to the TAP specification.

## Features

- Comprehensive implementation of all TAP message types
- Message validation according to the TAP protocol specification
- Serialization and deserialization of TAP messages
- DIDComm v2 integration for secure, encrypted messaging
- WASM compatibility for browser environments

## Usage

Add `tap-core` to your `Cargo.toml`:

```toml
[dependencies]
tap-core = "0.1.0"
```

### Creating a TAP Message

```rust
use tap_core::message::{TapMessage, TapMessageType};
use serde_json::json;

async fn create_message_example() {
    // Create a new message with the builder pattern
    let message = TapMessage::new()
        .with_message_type(TapMessageType::TransactionProposal)
        .with_body(json!({
            "transaction": {
                "amount": "100.00",
                "currency": "USD",
                "sender": "did:example:sender",
                "receiver": "did:example:receiver"
            }
        }))
        .with_from("did:example:sender")
        .with_to("did:example:receiver")
        .build();
    
    // Validate the message
    message.validate().expect("Message is valid");
    
    // Convert to JSON
    let json_message = serde_json::to_string(&message).expect("Serialization succeeds");
    println!("Message: {}", json_message);
}
```

### Parsing and Validating a TAP Message

```rust
use tap_core::message::{TapMessage, Validate};
use serde_json::Value;

fn parse_message_example(json_string: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse JSON string into a TapMessage
    let message: TapMessage = serde_json::from_str(json_string)?;
    
    // Validate the message
    message.validate()?;
    
    // Access message fields
    println!("Message ID: {}", message.id);
    println!("Message Type: {:?}", message.message_type);
    
    // Parse the body into a specific type
    if let Some(body) = &message.body {
        let transaction: Value = serde_json::from_value(body.clone())?;
        println!("Transaction details: {:#?}", transaction);
    }
    
    Ok(())
}
```

### Working with Different Message Types

```rust
use tap_core::message::{TapMessage, TapMessageType};

fn handle_message(message: &TapMessage) {
    match message.message_type {
        TapMessageType::TransactionProposal => {
            // Handle transaction proposal
            println!("Received transaction proposal with ID: {}", message.id);
        },
        TapMessageType::TransactionAuthorization => {
            // Handle transaction authorization
            println!("Received transaction authorization with ID: {}", message.id);
        },
        TapMessageType::IdentityExchange => {
            // Handle identity exchange
            println!("Received identity exchange with ID: {}", message.id);
        },
        TapMessageType::Error => {
            // Handle error message
            println!("Received error message with ID: {}", message.id);
        },
        TapMessageType::Custom(ref custom_type) => {
            // Handle custom message type
            println!("Received custom message type {} with ID: {}", custom_type, message.id);
        },
    }
}
```

## Advanced Usage

### Deserializing Typed Message Bodies

```rust
use tap_core::message::TapMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Transaction {
    amount: String,
    currency: String,
    sender: String,
    receiver: String,
}

fn process_transaction_proposal(message: &TapMessage) -> Result<(), Box<dyn std::error::Error>> {
    // Deserialize the body to a specific struct
    let transaction: Transaction = message.body_as()?;
    
    println!("Processing transaction: {} {}", transaction.amount, transaction.currency);
    println!("From: {} To: {}", transaction.sender, transaction.receiver);
    
    Ok(())
}
```

### Handling Message Attachments

```rust
use tap_core::message::{TapMessage, Attachment};
use base64::{Engine as _, engine::general_purpose::STANDARD as Base64};

fn process_attachments(message: &TapMessage) {
    if let Some(attachments) = &message.attachments {
        for (i, attachment) in attachments.iter().enumerate() {
            match &attachment.data {
                Some(data) => {
                    if let Some(base64) = &data.base64 {
                        // Decode base64 attachment
                        match Base64.decode(base64) {
                            Ok(bytes) => {
                                println!("Attachment {}: {} bytes", i, bytes.len());
                                // Process the bytes as needed
                            },
                            Err(e) => {
                                println!("Error decoding attachment {}: {}", i, e);
                            }
                        }
                    } else if let Some(json) = &data.json {
                        println!("JSON attachment {}: {}", i, json);
                    }
                },
                None => {
                    println!("Attachment {} has no data", i);
                }
            }
        }
    }
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
