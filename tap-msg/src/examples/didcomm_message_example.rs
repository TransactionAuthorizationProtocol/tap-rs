//! Example usage of the new DIDComm Message approach for TAP messages.

use crate::error::Result;
use crate::message::{Authorize, Participant, Reject, Settle, TapMessageBody, Transfer};
use crate::utils::get_current_time;
use didcomm::Message;
use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;

/// Example function to create a Transfer message using the new approach.
pub fn create_transfer_message_example() -> Result<Message> {
    // Create originator and beneficiary participants
    let originator = Participant {
        id: "did:example:alice".to_string(),
        role: Some("originator".to_string()),
        policies: None,
    };

    let beneficiary = Participant {
        id: "did:example:bob".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
    };

    // Create a transfer body
    let transfer_body = Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0x123456789abcdef").unwrap(),
        originator,
        beneficiary: Some(beneficiary),
        amount: "10.00".to_string(),
        agents: vec![],
        settlement_id: None,
        memo: Some("Payment for services".to_string()),
        metadata: HashMap::new(),
    };

    // Create a DIDComm message directly from the transfer body
    let message = transfer_body.to_didcomm()?;

    // The message is ready to be encrypted and sent
    Ok(message)
}

/// Example function to process a received Transfer message.
pub fn process_transfer_message_example(message: &Message) -> Result<()> {
    // First, check if this is a TAP message
    if message.type_.contains("transfer") {
        println!(
            "Received message is a TAP message of type: {}",
            message.type_
        );

        // Extract the transfer body
        let transfer = Transfer::from_didcomm(message)?;

        // Now we can work with the transfer data
        println!("Transfer amount: {}", transfer.amount);
        println!("Asset: {:?}", transfer.asset);

        if let Some(ref beneficiary) = transfer.beneficiary {
            println!("Beneficiary: {}", beneficiary.id);
        }

        println!("Originator: {}", transfer.originator.id);
    } else {
        println!("Not a transfer message");
    }

    Ok(())
}

/// Example function to create a Reject message.
pub fn create_reject_message_example(transfer_id: &str) -> Result<Message> {
    let reject_body = Reject {
        transfer_id: transfer_id.to_string(),
        code: "COMPLIANCE_FAILURE".to_string(),
        description: "Unable to comply with transfer requirements".to_string(),
        note: Some("Further documentation needed".to_string()),
        timestamp: get_current_time()?.to_string(),
        metadata: HashMap::new(),
    };

    // Create a DIDComm message directly from the reject body
    let message = reject_body.to_didcomm()?;

    // The message is ready to be encrypted and sent
    Ok(message)
}

/// Example function to create a Settle message.
pub fn create_settle_message_example(transfer_id: &str) -> Result<Message> {
    let settle_body = Settle {
        transfer_id: transfer_id.to_string(),
        transaction_id: "0x123456789abcdef".to_string(),
        transaction_hash: Some("0xabcdef123456789".to_string()),
        block_height: Some(12345678),
        note: Some("Transfer settled successfully".to_string()),
        timestamp: get_current_time()?.to_string(),
        metadata: HashMap::new(),
    };

    // Create a DIDComm message directly from the settle body
    let message = settle_body.to_didcomm()?;

    // The message is ready to be encrypted and sent
    Ok(message)
}

/// This example shows how to use the common interface to work with various TAP message types.
pub fn process_any_tap_message_example(message: &Message) -> Result<()> {
    // Get the message type
    let type_str = &message.type_;

    match () {
        _ if type_str.contains("transfer") => {
            // Handle Transfer message
            let transfer = Transfer::from_didcomm(message)?;
            println!("Processing Transfer: {}", transfer.amount);
        }
        _ if type_str.contains("authorize") => {
            // Handle Authorize message
            let authorize = Authorize::from_didcomm(message)?;
            println!(
                "Processing Authorization for transfer: {}",
                authorize.transfer_id
            );
        }
        _ if type_str.contains("reject") => {
            // Handle Reject message
            let reject = Reject::from_didcomm(message)?;
            println!("Processing Rejection for transfer: {}", reject.transfer_id);
            println!("Reason: {}", reject.description);
        }
        _ if type_str.contains("settle") => {
            // Handle Settle message
            let settle = Settle::from_didcomm(message)?;
            println!("Processing Settlement for transfer: {}", settle.transfer_id);
            println!("Transaction ID: {}", settle.transaction_id);
        }
        _ => {
            println!("Unknown message type");
        }
    }

    Ok(())
}
