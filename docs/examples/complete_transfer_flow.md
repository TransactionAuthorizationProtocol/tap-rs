# Complete TAP Transfer Flow Example

This example demonstrates a complete TAP transfer flow involving multiple components:

1. Creating TAP participants for the originator and beneficiary
2. Implementing a TAP node for message routing
3. Setting up a simple HTTP server for communication
4. Processing a complete transfer flow (Transfer → Authorize → Receipt → Settlement)

## Prerequisites

This example uses the following TAP-RS crates:
- `tap-msg`
- `tap-agent`
- `tap-caip`
- `tap-node`
- `tap-http`

## Setup

First, let's add the necessary dependencies to your Cargo.toml:

```toml
[dependencies]
tap-msg = { path = "../tap-msg" }
tap-agent = { path = "../tap-agent" }
tap-caip = { path = "../tap-caip" }
tap-node = { path = "../tap-node" }
tap-http = { path = "../tap-http" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
std-futures = "0.2"
log = "0.4"
env_logger = "0.10"
chrono = "0.4"
```

## Complete Example

```rust
use std::sync::Arc;
use std::str::FromStr;
use std::collections::HashMap;
use std::time::Duration;

use tap_agent::{Participant, ParticipantConfig};
use tap_msg::message::{
    Transfer, Authorize, ReceiptBody, SettlementBody, Reject,
    TapMessageBody, Participant as TapParticipant
};
use tap_msg::did::KeyPair;
use tap_caip::AssetId;
use tap_node::{Node, NodeConfig};
use tap_node::message::{
    MessageProcessorType, LoggingMessageProcessor, ValidationMessageProcessor,
    CompositeMessageProcessor
};
use tap_http::{TapServer, ServerConfig};

use didcomm::Message;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("Starting TAP transfer flow example...");

    // 1. Create key pairs for the participants
    let originator_key = KeyPair::generate_ed25519().await?;
    let beneficiary_key = KeyPair::generate_ed25519().await?;

    println!("Generated key pairs:");
    println!("  Originator: {}", originator_key.get_did_key());
    println!("  Beneficiary: {}", beneficiary_key.get_did_key());

    // 2. Create the TAP participants
    let originator_participant = Participant::new(
        ParticipantConfig::new().with_name("Originator".to_string()),
        Arc::new(originator_key),
    )?;

    let beneficiary_participant = Participant::new(
        ParticipantConfig::new().with_name("Beneficiary".to_string()),
        Arc::new(beneficiary_key),
    )?;

    println!("Created TAP participants:");
    println!("  Originator: {} ({})", originator_participant.name(), originator_participant.did());
    println!("  Beneficiary: {} ({})", beneficiary_participant.name(), beneficiary_participant.did());

    // 3. Set up the TAP node
    let node_config = NodeConfig::new()
        .with_max_participants(10)
        .with_logging(true);

    let node = Node::new(node_config);

    // Add processors to the node for message handling
    let mut composite_processor = CompositeMessageProcessor::new();
    composite_processor.add_processor(
        MessageProcessorType::Logging(LoggingMessageProcessor::new())
    )?;
    composite_processor.add_processor(
        MessageProcessorType::Validation(ValidationMessageProcessor::new())
    )?;

    node.add_processor(MessageProcessorType::Composite(composite_processor))?;

    // Register the participants with the node
    node.register_participant(Arc::new(originator_participant.clone())).await?;
    node.register_participant(Arc::new(beneficiary_participant.clone())).await?;

    println!("Set up TAP node and registered participants");

    // 4. Set up message handlers

    // Set up a handler for the beneficiary to process transfer requests
    let beneficiary_clone = beneficiary_participant.clone();
    let beneficiary_did = beneficiary_participant.did().to_string();

    // Handler for processing transfers
    node.register_message_handler(beneficiary_did, move |message| {
        let msg_clone = message.clone();
        let beneficiary = beneficiary_clone.clone();

        tokio::spawn(async move {
            if let Some(msg_type) = &msg_clone.type_ {
                if msg_type == "TAP_TRANSFER" {
                    println!("Beneficiary received transfer request: {}", msg_clone.id);

                    // Create an authorize response
                    let authorize = Authorize {
                        transfer_id: msg_clone.id.clone(),
                        note: Some("Transfer authorized by beneficiary".to_string()),
                        metadata: HashMap::new(),
                    };

                    let from_did = msg_clone.from.clone().unwrap_or_default();

                    // Convert to DIDComm message and send
                    let response = authorize.to_didcomm()?;
                    let response = response
                        .set_from(Some(beneficiary.did().to_string()))
                        .set_to(Some(vec![from_did]))
                        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));

                    println!("Beneficiary sending authorization: {}", response.id);
                    beneficiary.send_message(&from_did, response).await?;
                }
            }

            Ok::<(), tap_agent::Error>(())
        });

        Ok(())
    }).await?;

    // Set up a handler for the originator to process authorizations
    let originator_clone = originator_participant.clone();
    let originator_did = originator_participant.did().to_string();

    // Handler for processing authorizations
    node.register_message_handler(originator_did, move |message| {
        let msg_clone = message.clone();
        let originator = originator_clone.clone();

        tokio::spawn(async move {
            if let Some(msg_type) = &msg_clone.type_ {
                if msg_type == "TAP_AUTHORIZE" {
                    println!("Originator received authorization: {}", msg_clone.id);

                    // Extract the transfer ID from the authorization
                    let auth_body = Authorize::from_didcomm(&msg_clone)?;

                    // Create a receipt
                    let receipt = ReceiptBody {
                        transfer_id: auth_body.transfer_id.clone(),
                        settlement_id: None,
                        note: Some("Receipt confirmed by originator".to_string()),
                        metadata: HashMap::new(),
                    };

                    let from_did = msg_clone.from.clone().unwrap_or_default();

                    // Convert to DIDComm message and send
                    let response = receipt.to_didcomm()?;
                    let response = response
                        .set_from(Some(originator.did().to_string()))
                        .set_to(Some(vec![from_did.clone()]))
                        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));

                    println!("Originator sending receipt: {}", response.id);
                    originator.send_message(&from_did, response).await?;

                    // Simulate a blockchain transaction and then send settlement
                    println!("Simulating blockchain transaction...");
                    sleep(Duration::from_secs(2)).await;

                    // Create a settlement message
                    let settlement = SettlementBody {
                        transfer_id: auth_body.transfer_id.clone(),
                        settlement_id: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
                        status: "completed".to_string(),
                        note: Some("Transaction confirmed on blockchain".to_string()),
                        metadata: HashMap::new(),
                    };

                    // Convert to DIDComm message and send
                    let settlement_msg = settlement.to_didcomm()?;
                    let settlement_msg = settlement_msg
                        .set_from(Some(originator.did().to_string()))
                        .set_to(Some(vec![from_did]))
                        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));

                    println!("Originator sending settlement: {}", settlement_msg.id);
                    originator.send_message(&from_did, settlement_msg).await?;
                }
            }

            Ok::<(), tap_agent::Error>(())
        });

        Ok(())
    }).await?;

    // 5. Set up an HTTP server for the node
    let server_config = ServerConfig::new()
        .with_address("127.0.0.1".to_string())
        .with_port(8080);

    let server = TapServer::new(Arc::new(node));

    // Start the server in a separate task
    let server_handle = tokio::spawn(async move {
        server.start("127.0.0.1", 8080).await.unwrap();
    });

    println!("Started TAP HTTP server on http://127.0.0.1:8080");

    // 6. Initiate the transfer flow
    // Prepare the transfer message
    println!("Initiating transfer from originator to beneficiary...");

    // Create participant representations for the message
    let originator = TapParticipant {
        id: originator_participant.did().to_string(),
        role: Some("originator".to_string()),
    };

    let beneficiary = TapParticipant {
        id: beneficiary_participant.did().to_string(),
        role: Some("beneficiary".to_string()),
    };

    // Parse the asset ID (DAI stablecoin on Ethereum)
    let asset = AssetId::from_str("eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F")?;

    // Create the transfer body
    let transfer_body = Transfer {
        asset,
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents: vec![],
        settlement_id: None,
        memo: Some("Payment for services".to_string()),
        metadata: HashMap::new(),
    };

    // Create and sign the message
    let transfer_msg = transfer_body.to_didcomm()?;
    let transfer_msg = transfer_msg
        .set_from(Some(originator_participant.did().to_string()))
        .set_to(Some(vec![beneficiary_participant.did().to_string()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));

    println!("Originator created transfer message: {}", transfer_msg.id);

    // Send the message
    originator_participant.send_message(beneficiary_participant.did(), transfer_msg).await?;

    // Wait to see the whole flow complete
    println!("Waiting for the transfer flow to complete...");
    sleep(Duration::from_secs(5)).await;

    println!("Transfer flow completed!");

    // Shutdown the server
    server_handle.abort();

    Ok(())
}
```

## Running the Example

Save this code in a file named `main.rs` in your project's `examples` directory. Then run it with:

```bash
RUST_LOG=info cargo run --example complete_transfer_flow
```

## Expected Output

When running this example, you should see output similar to the following:

```
Starting TAP transfer flow example...
Generated key pairs:
  Originator: did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
  Beneficiary: did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH
Created TAP participants:
  Originator: Originator (did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK)
  Beneficiary: Beneficiary (did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH)
Set up TAP node and registered participants
Started TAP HTTP server on http://127.0.0.1:8080
Initiating transfer from originator to beneficiary...
Originator created transfer message: 1234-5678-9abc-def0
Waiting for the transfer flow to complete...
Beneficiary received transfer request: 1234-5678-9abc-def0
Beneficiary sending authorization: 2345-6789-abcd-ef01
Originator received authorization: 2345-6789-abcd-ef01
Originator sending receipt: 3456-789a-bcde-f012
Simulating blockchain transaction...
Originator sending settlement: 4567-89ab-cdef-0123
Transfer flow completed!
```

## Explanation

This example demonstrates a complete TAP transfer flow:

1. **Setup**: We create two TAP participants (originator and beneficiary) with their own key pairs, and a TAP node to route messages between them.

2. **Message Handlers**: We set up message handlers for both participants to process incoming messages:
   - The beneficiary processes transfer requests and responds with authorizations
   - The originator processes authorizations, sends receipts, and settlement messages

3. **Transfer Flow**:
   - Originator creates and sends a transfer message
   - Beneficiary receives the transfer and sends an authorization
   - Originator receives the authorization, sends a receipt, and then a settlement message

4. **HTTP Transport**: The example sets up an HTTP server to handle communication between the participants, which would be necessary in a real-world deployment.

## Next Steps

This example can be extended in various ways:

1. **Add Error Handling**: Implement more robust error handling and recovery mechanisms
2. **Use Real Key Management**: Integrate with a secure key management system
3. **Add Persistence**: Store messages and state in a database
4. **Implement Webhooks**: Notify external systems about TAP events
5. **Add Authentication**: Implement proper authentication for the HTTP server

For more details on TAP-RS, see the API Reference documentation and other [tutorials](../tutorials/getting_started.md).
