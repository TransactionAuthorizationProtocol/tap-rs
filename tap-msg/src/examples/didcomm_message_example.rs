//! Example usage of the new DIDComm Message approach for TAP messages.

use crate::didcomm::pack_tap_body;
use crate::error::Result;
use crate::message::{
    Transfer, TapMessageBody, Authorize, Reject, Settle, Participant,
};
use didcomm::Message;
use std::collections::HashMap;
use tap_caip::AssetId;

/// Example function to create a Transfer message using the new approach.
pub async fn create_transfer_message_example() -> Result<Message> {
    // Create originator and beneficiary participants
    let originator = Participant {
        id: "did:example:alice".to_string(),
        role: Some("originator".to_string()),
    };
    
    let beneficiary = Participant {
        id: "did:example:bob".to_string(),
        role: Some("beneficiary".to_string()),
    };
    
    // Create a transfer body
    let transfer_body = Transfer {
        asset: AssetId::parse("eip155:1/erc20:0x123456789abcdef").unwrap(),
        originator,
        beneficiary: Some(beneficiary),
        amount: "10.00".to_string(),
        participants: vec![],
        settlement_id: None,
        memo: Some("Payment for services".to_string()),
        metadata: HashMap::new(),
    };
    
    // Create a DIDComm message directly from the transfer body
    let message = transfer_body.to_didcomm()?;
    
    // Set the sender and recipients
    let message = message
        .set_from(Some("did:example:alice".to_string()))
        .set_to(Some(vec!["did:example:bob".to_string()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    // The message is now ready to be encrypted and sent
    Ok(message)
}

/// Example function to process a received Transfer message.
pub fn process_transfer_message_example(message: &Message) -> Result<()> {
    // First, check if this is a TAP message
    if message.type_.as_ref().map_or(false, |t| !t.starts_with("TAP_")) {
        return Err(crate::error::Error::Validation(
            "Not a TAP message".to_string(),
        ));
    }
    
    // For a Transfer message, extract the transfer body
    if message.type_.as_ref().map_or(false, |t| t == "TAP_TRANSFER") {
        let transfer_body = Transfer::from_didcomm(message)?;
        
        // Now we can work with the transfer body
        println!("Received transfer from originator: {:?}", transfer_body.originator);
        if let Some(ref beneficiary) = transfer_body.beneficiary {
            println!("To beneficiary: {:?}", beneficiary);
        }
        println!("Amount: {}", transfer_body.amount);
        
        // Create a response message (either Authorize or Reject)
        let authorize_body = Authorize {
            transfer_id: message.id.clone(), // Reference the original transfer ID
            note: Some("Transfer authorized".to_string()),
            metadata: HashMap::new(),
        };
        
        // The authorize_body can now be converted to a DIDComm message
        // and sent back to the originator
    }
    
    Ok(())
}

/// Example function to create a Reject message.
pub async fn create_reject_message_example(transfer_id: &str) -> Result<Message> {
    let reject_body = Reject {
        transfer_id: transfer_id.to_string(),
        code: "COMPLIANCE_FAILURE".to_string(),
        description: "Unable to comply with transfer requirements".to_string(),
        note: Some("Further documentation needed".to_string()),
        metadata: HashMap::new(),
    };
    
    // Create a DIDComm message directly from the reject body
    let message = reject_body.to_didcomm()?;
    
    // Set the sender and recipients
    let message = message
        .set_from(Some("did:example:bob".to_string()))
        .set_to(Some(vec!["did:example:alice".to_string()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    Ok(message)
}

/// Example function to create a Settle message.
pub async fn create_settle_message_example(transfer_id: &str) -> Result<Message> {
    let settle_body = Settle {
        transfer_id: transfer_id.to_string(),
        transaction_id: "0x123456789abcdef".to_string(),
        transaction_hash: Some("0xabcdef123456789".to_string()),
        block_height: Some(12345678),
        note: Some("Transfer settled successfully".to_string()),
        metadata: HashMap::new(),
    };
    
    // Create a DIDComm message directly from the settle body
    let message = settle_body.to_didcomm()?;
    
    // Set the sender and recipients
    let message = message
        .set_from(Some("did:example:alice".to_string()))
        .set_to(Some(vec!["did:example:bob".to_string()]))
        .set_created_time(Some(chrono::Utc::now().to_rfc3339()));
    
    Ok(message)
}

/// This example shows how to use the common interface to work with various TAP message types.
pub async fn process_any_tap_message_example(message: &Message) -> Result<()> {
    // Get the message type
    let message_type = message.type_.as_ref().map(|t| t.as_str());
    
    // Check the message type and process accordingly
    match message_type {
        Some("TAP_TRANSFER") => {
            let body = Transfer::from_didcomm(message)?;
            println!("Processing transfer: {:?} -> {:?}", body.originator, body.beneficiary);
        }
        Some("TAP_AUTHORIZE") => {
            let body = Authorize::from_didcomm(message)?;
            println!("Authorization received for transfer: {}", body.transfer_id);
        }
        Some("TAP_REJECT") => {
            let body = Reject::from_didcomm(message)?;
            println!("Rejection received for transfer: {}", body.transfer_id);
            println!("Reason: {} - {}", body.code, body.description);
        }
        Some("TAP_SETTLE") => {
            let body = Settle::from_didcomm(message)?;
            println!("Settlement received for transfer: {}", body.transfer_id);
            println!("Transaction ID: {}", body.transaction_id);
        }
        _ => {
            println!("Unhandled message type: {:?}", message_type);
        }
    }
    
    Ok(())
}
