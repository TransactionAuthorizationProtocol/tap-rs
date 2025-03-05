# Implementing TAP Flows

This tutorial guides you through implementing complete Transaction Authorization Protocol (TAP) flows in your application using the TAP-RS library. 

## Overview of TAP Flows

The Transaction Authorization Protocol defines several message flows for different scenarios in the payment/settlement process. The primary flows are:

1. **Basic Transfer Flow** - Simple transfer request and authorization
2. **Transfer with Rejection Flow** - Transfer that gets rejected by the beneficiary
3. **Settlement Flow** - Complete flow including settlement confirmation
4. **Fallback Flow** - Handling fallbacks when a direct messaging path isn't available

## Prerequisites

Before implementing TAP flows, make sure you:

- Have completed the [Getting Started](./getting_started.md) tutorial
- Understand the basic TAP message types and structures
- Have set up TAP agents for the parties involved in the transaction

## Basic Transfer Flow

The basic transfer flow involves:
1. Originator sends a Transfer message
2. Beneficiary responds with an Authorize message
3. Originator completes the settlement off-protocol
4. Optionally, the originator confirms completion with a Receipt

### Rust Implementation

```rust
use tap_agent::{Participant, ParticipantConfig};
use tap_msg::{
    did::KeyPair,
    message::{Transfer, Authorize, ReceiptBody, TapMessageBody, Participant as MessageParticipant},
};
use tap_caip::AssetId;
use didcomm::Message;
use std::{collections::HashMap, sync::Arc};

async fn implement_basic_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Create and configure agents
    let originator_key = KeyPair::generate_ed25519().await?;
    let beneficiary_key = KeyPair::generate_ed25519().await?;
    
    let originator_did = originator_key.get_did_key();
    let beneficiary_did = beneficiary_key.get_did_key();
    
    let originator_agent = Participant::new(
        ParticipantConfig::new().with_did(originator_did.clone()).with_name("Originator"),
        Arc::new(originator_key),
    )?;
    
    let beneficiary_agent = Participant::new(
        ParticipantConfig::new().with_did(beneficiary_did.clone()).with_name("Beneficiary"),
        Arc::new(beneficiary_key),
    )?;
    
    // Step 1: Originator creates and sends a Transfer message
    let originator_msg_participant = MessageParticipant {
        id: originator_did.clone(),
        role: Some("originator".to_string()),
    };
    
    let beneficiary_msg_participant = MessageParticipant {
        id: beneficiary_did.clone(),
        role: Some("beneficiary".to_string()),
    };
    
    let transfer_body = Transfer {
        asset: AssetId::parse("eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap(),
        originator: originator_msg_participant,
        beneficiary: Some(beneficiary_msg_participant),
        amount: "100.0".to_string(),
        agents: vec![],
        settlement_id: None,
        memo: Some("Payment for services".to_string()),
        metadata: HashMap::new(),
    };
    
    let transfer_message = transfer_body.to_didcomm()?
        .set_from(Some(originator_did.clone()))
        .set_to(Some(vec![beneficiary_did.clone()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    // In a real scenario, this would be sent over a transport
    // For this example, we'll manually handle the messages
    println!("1. Originator sends Transfer message");
    let transfer_id = transfer_message.id.clone();
    
    // Step 2: Beneficiary processes the Transfer and responds with Authorize
    let received_transfer_body = Transfer::from_didcomm(&transfer_message)?;
    
    let authorize_body = Authorize {
        transfer_id: transfer_id.clone(),
        note: Some("Transfer authorized, please proceed with on-chain settlement".to_string()),
        metadata: HashMap::new(),
    };
    
    let authorize_message = authorize_body.to_didcomm()?
        .set_from(Some(beneficiary_did.clone()))
        .set_to(Some(vec![originator_did.clone()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    println!("2. Beneficiary sends Authorize message");
    
    // Step 3: Originator processes the Authorize message
    let received_authorize_body = Authorize::from_didcomm(&authorize_message)?;
    println!("3. Originator receives authorization: {}", 
             received_authorize_body.note.unwrap_or_default());
    
    // Step 4: Originator performs the on-chain settlement (simulated here)
    println!("4. Originator performs on-chain settlement");
    let settlement_id = "0x1234567890abcdef1234567890abcdef12345678";
    
    // Step 5: Originator sends a Receipt message confirming settlement
    let receipt_body = ReceiptBody {
        transfer_id: transfer_id.clone(),
        settlement_id: Some(settlement_id.to_string()),
        note: Some("Settlement completed successfully".to_string()),
        metadata: HashMap::new(),
    };
    
    let receipt_message = receipt_body.to_didcomm()?
        .set_from(Some(originator_did.clone()))
        .set_to(Some(vec![beneficiary_did.clone()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    println!("5. Originator sends Receipt message");
    
    // Step 6: Beneficiary processes the Receipt
    let received_receipt_body = ReceiptBody::from_didcomm(&receipt_message)?;
    println!("6. Beneficiary confirms receipt of settlement: {}", 
             received_receipt_body.note.unwrap_or_default());
    
    Ok(())
}
```

### TypeScript Implementation

```typescript
import { Participant, Message, MessageType } from "@tap-rs/tap-ts";

async function implementBasicFlow() {
    // Create agents
    const originatorAgent = new Participant({
        nickname: "Originator",
        // In a real implementation, you would provide or generate keys
    });
    
    const beneficiaryAgent = new Participant({
        nickname: "Beneficiary",
    });
    
    const originatorDid = originatorAgent.did;
    const beneficiaryDid = beneficiaryAgent.did;
    
    // Step 1: Originator creates and sends Transfer message
    const transfer = new Message({
        type: MessageType.TRANSFER,
    });
    
    transfer.setTransferData({
        asset: "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F",
        amount: "100.0",
        originatorDid: originatorDid,
        beneficiaryDid: beneficiaryDid,
        memo: "Payment for services"
    });
    
    console.log("1. Originator sends Transfer message");
    const transferId = transfer.id;
    
    // In a real scenario, this would be sent over a transport
    // For this example, we'll manually handle the messages
    
    // Step 2: Beneficiary processes the Transfer and responds with Authorize
    const authorize = new Message({
        type: MessageType.AUTHORIZE,
        correlation: transferId,
    });
    
    authorize.setAuthorizeData({
        note: "Transfer authorized, please proceed with on-chain settlement"
    });
    
    console.log("2. Beneficiary sends Authorize message");
    
    // Step 3: Originator processes the Authorize message
    console.log("3. Originator receives authorization");
    
    // Step 4: Originator performs the on-chain settlement (simulated here)
    console.log("4. Originator performs on-chain settlement");
    const settlementId = "0x1234567890abcdef1234567890abcdef12345678";
    
    // Step 5: Originator sends a Receipt message confirming settlement
    const receipt = new Message({
        type: MessageType.RECEIPT,
        correlation: transferId,
    });
    
    receipt.setReceiptData({
        settlementId: settlementId,
        note: "Settlement completed successfully"
    });
    
    console.log("5. Originator sends Receipt message");
    
    // Step 6: Beneficiary processes the Receipt
    console.log("6. Beneficiary confirms receipt of settlement");
}
```

## Transfer with Rejection Flow

In some cases, a beneficiary may need to reject a transfer request for various reasons (e.g., policy violations, incorrect amounts, etc.).

### Rust Implementation

```rust
async fn implement_rejection_flow() -> Result<(), Box<dyn std::error::Error>> {
    // ... Setup code similar to basic flow ...
    
    // Step:1-2 Same as basic flow, originator sends Transfer
    
    // Step 3: Beneficiary decides to reject the transfer
    let reject_body = Reject {
        transfer_id: transfer_id.clone(),
        code: "policy_violation".to_string(),
        description: Some("Amount exceeds daily transfer limit".to_string()),
        metadata: HashMap::new(),
    };
    
    let reject_message = reject_body.to_didcomm()?
        .set_from(Some(beneficiary_did.clone()))
        .set_to(Some(vec![originator_did.clone()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    println!("3. Beneficiary rejects transfer: {}", 
             reject_body.description.unwrap_or_default());
    
    // Step 4: Originator processes the rejection
    let received_reject_body = Reject::from_didcomm(&reject_message)?;
    println!("4. Originator receives rejection: {} - {}", 
             received_reject_body.code,
             received_reject_body.description.unwrap_or_default());
    
    // Step 5: Originator could attempt a new transfer with modified parameters
    
    Ok(())
}
```

## Settlement Flow

The complete settlement flow includes additional messages to track the settlement status.

### Rust Implementation

```rust
async fn implement_settlement_flow() -> Result<(), Box<dyn std::error::Error>> {
    // ... Setup code and steps 1-4 similar to basic flow ...
    
    // Step 5: Originator creates a Settlement message
    let settlement_body = SettlementBody {
        transfer_id: transfer_id.clone(),
        settlement_id: settlement_id.to_string(),
        status: "pending".to_string(),
        note: Some("Settlement transaction initiated".to_string()),
        metadata: HashMap::new(),
    };
    
    let settlement_message = settlement_body.to_didcomm()?
        .set_from(Some(originator_did.clone()))
        .set_to(Some(vec![beneficiary_did.clone()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    println!("5. Originator sends Settlement message (pending)");
    
    // Step 6: Once settlement completes, Originator sends updated Settlement
    let completed_settlement_body = SettlementBody {
        transfer_id: transfer_id.clone(),
        settlement_id: settlement_id.to_string(),
        status: "completed".to_string(),
        note: Some("Settlement transaction confirmed".to_string()),
        metadata: HashMap::new(),
    };
    
    let completed_settlement_message = completed_settlement_body.to_didcomm()?
        .set_from(Some(originator_did.clone()))
        .set_to(Some(vec![beneficiary_did.clone()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    println!("6. Originator sends Settlement message (completed)");
    
    // Step 7: Beneficiary sends Receipt acknowledging settlement
    let receipt_body = ReceiptBody {
        transfer_id: transfer_id.clone(),
        settlement_id: Some(settlement_id.to_string()),
        note: Some("Settlement verified and received".to_string()),
        metadata: HashMap::new(),
    };
    
    let receipt_message = receipt_body.to_didcomm()?
        .set_from(Some(beneficiary_did.clone()))
        .set_to(Some(vec![originator_did.clone()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    println!("7. Beneficiary confirms with Receipt message");
    
    Ok(())
}
```

## Multi-Agent Flow

TAP also supports flows involving multiple agents in addition to the originator and beneficiary.

### Rust Implementation

```rust
async fn implement_multi_agent_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Create agents for originator, intermediary, and beneficiary
    // ... Setup code for multiple agents ...
    
    // Step 1: Originator creates Transfer with multiple agents
    let agents = vec![
        MessageParticipant {
            id: intermediary_did.clone(),
            role: Some("transmitter".to_string()),
        }
    ];
    
    let transfer_body = Transfer {
        asset: AssetId::parse("eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap(),
        originator: originator_msg_participant,
        beneficiary: Some(beneficiary_msg_participant),
        amount: "100.0".to_string(),
        agents: agents,  // Include additional agents
        settlement_id: None,
        memo: Some("Payment for services".to_string()),
        metadata: HashMap::new(),
    };
    
    let transfer_message = transfer_body.to_didcomm()?
        .set_from(Some(originator_did.clone()))
        .set_to(Some(vec![intermediary_did.clone()]))  // Send to intermediary first
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    println!("1. Originator sends Transfer message to intermediary");
    
    // Step 2: Intermediary forwards to beneficiary (potentially adding compliance data)
    let received_transfer_body = Transfer::from_didcomm(&transfer_message)?;
    
    // Intermediary can add metadata or modify the message if needed
    let forwarded_transfer_body = Transfer {
        metadata: {
            let mut metadata = received_transfer_body.metadata.clone();
            metadata.insert("compliance_checked".to_string(), "true".to_string());
            metadata
        },
        ..received_transfer_body
    };
    
    let forwarded_message = forwarded_transfer_body.to_didcomm()?
        .set_from(Some(intermediary_did.clone()))
        .set_to(Some(vec![beneficiary_did.clone()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    println!("2. Intermediary forwards Transfer to beneficiary");
    
    // Step 3-7: Continue with authorization and settlement similar to previous flows
    
    Ok(())
}
```

## Best Practices

When implementing TAP flows:

1. **Error Handling**: Implement proper error handling for all message processing to handle unexpected message formats or failed validations.

2. **Message Correlation**: Always maintain proper correlation between messages using the transfer_id field to track a complete flow.

3. **Timeouts**: Implement timeouts for waiting on responses to ensure your application doesn't hang indefinitely.

4. **Idempotency**: Handle duplicate messages gracefully by checking message IDs and maintain an idempotent processing approach.

5. **Security**: Verify that messages are properly signed and that DIDs are authorized to send/receive messages related to the specific transfer.

6. **Logging**: Log all significant events in the flow for auditing and debugging purposes.

7. **Retry Logic**: Implement retry logic for message sending in case of temporary network issues.

## Next Steps

- Explore [Security Best Practices](./security_best_practices.md) for securing your TAP implementation
- Learn about [WASM Integration](./wasm_integration.md) for browser-based TAP applications
- Review the [API Reference](../api/index.md) for detailed information on all TAP-RS APIs
