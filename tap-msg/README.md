# tap-msg: TAP Core Message Processing

The `tap-msg` crate provides core message processing functionality for the Transaction Authorization Protocol (TAP). This library handles message types, validation, and serialization according to the TAP specification.

## Features

- Comprehensive implementation of all TAP message types
- Message validation according to the TAP protocol specification
- Serialization and deserialization of TAP messages
- DIDComm v2 integration directly built into TAP message types
- WASM compatibility for browser environments
- Authorization flow support for Transfer messages

## Usage

Add `tap-msg` to your `Cargo.toml`:

```toml
[dependencies]
tap-msg = "0.1.0"
```

### Creating a TAP Message

```rust
use tap_msg::message::{TapMessageEnvelope, TapMessageType};
use serde_json::json;

async fn create_message_example() {
    // Create a new message with the builder pattern
    let message = TapMessageEnvelope::new()
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
use tap_msg::message::{TapMessageEnvelope, Validate};
use serde_json::Value;

fn parse_message_example(json_string: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse JSON string into a TapMessageEnvelope
    let message: TapMessageEnvelope = serde_json::from_str(json_string)?;
    
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

### Converting TAP Messages to DIDComm Messages

The TAP message types implement `TapMessageBody` trait which provides methods to convert to and from DIDComm messages:

```rust
use tap_msg::message::{Participant, Transfer, TapMessageBody};
use tap_caip::AssetId;
use std::str::FromStr;
use std::collections::HashMap;

fn didcomm_conversion_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a TAP transfer message
    let asset = AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7")?;
    
    let originator = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
    };
    
    let beneficiary = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("beneficiary".to_string()),
    };
    
    let agents = vec![
        Participant {
            id: "did:key:z6MkhaXgCDEv1tDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
            role: Some("agent".to_string()),
        }
    ];
    
    let transfer = Transfer {
        asset,
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents,
        settlement_id: None,
        memo: Some("Test transfer".to_string()),
        metadata: HashMap::new(),
    };
    
    // Convert to DIDComm message
    let didcomm_message = transfer.to_didcomm()?;
    
    // Now you can send this DIDComm message
    println!("DIDComm message ID: {}", didcomm_message.id);
    
    Ok(())
}
```

### Using the Authorizable Trait for Transfer Authorization Flow

The `Authorizable` trait provides a streamlined way to handle the authorization, rejection, and settlement flows for Transfer messages:

```rust
use tap_msg::message::types::Authorizable;
use tap_msg::{Transfer, Participant};
use tap_caip::AssetId;
use std::str::FromStr;
use std::collections::HashMap;
use tap_msg::message::tap_message_trait::TapMessageBody;

fn authorization_flow_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Transfer message
    let asset = AssetId::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?;
    
    let originator = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
    };
    
    let beneficiary = Participant {
        id: "did:key:z6MkhaDgCZDv1tDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("beneficiary".to_string()),
    };

    let agents = vec![
        Participant {
            id: "did:key:z6MkhaXgCDEv1tDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
            role: Some("agent".to_string()),
        }
    ];
    
    let transfer = Transfer {
        asset,
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents,
        settlement_id: None,
        memo: Some("Test transfer".to_string()),
        metadata: HashMap::new(),
    };
    
    // Convert to DIDComm message for transmission
    let didcomm_message = transfer.to_didcomm()?;
    
    // Recipient receives the message and can now authorize, reject, or settle it
    
    // To authorize the transfer:
    let auth = didcomm_message.authorize(
        Some("Authorization approved".to_string()),
        HashMap::new(),
    );
    
    // To reject the transfer:
    let reject = didcomm_message.reject(
        "REJECT-001".to_string(),
        "Rejected due to compliance issues".to_string(),
        Some("Additional rejection note".to_string()),
        HashMap::new(),
    );
    
    // To settle the transfer:
    let settle = didcomm_message.settle(
        "tx-12345".to_string(),
        Some("0x1234567890abcdef".to_string()),
        Some(1234567),
        Some("Settlement note".to_string()),
        HashMap::new(),
    );
    
    // Convert the response messages to DIDComm format for transmission
    let auth_message = auth.to_didcomm()?;
    let reject_message = reject.to_didcomm()?;
    let settle_message = settle.to_didcomm()?;
    
    Ok(())
}
```

## Message Types

TAP supports various message types, including:

- Transfer: Initiates a transfer proposal
- Authorize: Approves a transfer
- Reject: Rejects a transfer with reason
- Settle: Confirms settlement of a transfer
- RequestPresentation: Requests identity or credential verification
- Presentation: Provides requested identity or credential information

## Error Handling

The library uses a custom error type for consistent error handling:

```rust
fn handle_message(message: &TapMessageEnvelope) {
    match message.message_type {
        TapMessageType::TransactionProposal => {
            // Handle transaction proposal
            println!("Received transaction proposal with ID: {}", message.id);
        }
        // Handle other message types
        _ => println!("Received message with type: {:?}", message.message_type),
    }
}
```

## License

This project is licensed under the [Apache License 2.0](LICENSE).
