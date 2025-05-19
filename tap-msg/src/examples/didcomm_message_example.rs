//! Example usage of the new DIDComm Message approach for TAP messages.

use crate::didcomm::{Attachment, AttachmentData, JsonAttachmentData, PlainMessage};
use crate::error::Result;
use crate::message::{
    Authorize, DIDCommPresentation, Participant, Reject, Settle, TapMessageBody, Transfer,
};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;

/// Example function to create a Transfer message using the new approach.
pub fn create_transfer_message_example() -> Result<PlainMessage> {
    // Create originator and beneficiary participants
    let originator = Participant {
        id: "did:example:alice".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let beneficiary = Participant {
        id: "did:example:bob".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    // Create a transfer body
    let transfer_body = Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0x123456789abcdef").unwrap(),
        originator,
        beneficiary: Some(beneficiary),
        amount: "10.00".to_string(),
        memo: None,
        agents: vec![],
        settlement_id: None,
        transaction_id: uuid::Uuid::new_v4().to_string(),
        metadata: HashMap::new(),
    };

    // Convert the Transfer body to a DIDComm message
    let message =
        transfer_body.to_didcomm("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")?;

    // The message is ready to be encrypted and sent
    Ok(message)
}

/// Example function to process a received Transfer message.
pub fn process_transfer_message_example(message: &PlainMessage) -> Result<()> {
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
pub fn create_reject_message_example(transaction_id: &str) -> Result<PlainMessage> {
    let reject_body = Reject {
        transaction_id: transaction_id.to_string(),
        reason: "COMPLIANCE_FAILURE: Unable to comply with transfer requirements. Further documentation needed.".to_string(),
    };

    // Convert the Reject body to a DIDComm message
    let message =
        reject_body.to_didcomm("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6")?;

    // The message is ready to be encrypted and sent
    Ok(message)
}

/// Example function to create a Settle message.
pub fn create_settle_message_example(transaction_id: &str) -> Result<PlainMessage> {
    let settle_body = Settle {
        transaction_id: transaction_id.to_string(),
        settlement_id: "0x123456789abcdef".to_string(),
        amount: Some("100.0".to_string()),
    };

    // Convert the Settle body to a DIDComm message
    let message =
        settle_body.to_didcomm("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6")?;

    // The message is ready to be encrypted and sent
    Ok(message)
}

/// Example function for creating a DIDCommPresentation with attachments.
pub fn create_presentation_with_attachments_example() -> Result<PlainMessage> {
    // Create a presentation attachment with required format field
    let attachment = Attachment {
        id: Some("test-attachment-id".to_string()),
        media_type: Some("application/json".to_string()),
        data: AttachmentData::Json {
            value: JsonAttachmentData {
                json: json!({
                    "@context": ["https://www.w3.org/2018/credentials/v1"],
                    "type": ["VerifiablePresentation"],
                    "verifiableCredential": [{
                        "@context": ["https://www.w3.org/2018/credentials/v1"],
                        "id": "https://example.com/credentials/1234",
                        "type": ["VerifiableCredential"],
                        "issuer": "did:example:issuer",
                        "issuanceDate": "2023-01-01T12:00:00Z",
                        "credentialSubject": {
                            "id": "did:example:subject",
                            "name": "Test User"
                        }
                    }]
                }),
                jws: None,
            },
        },
        description: Some("Verifiable Presentation".to_string()),
        filename: None,
        format: Some("dif/presentation-exchange/submission@v1.0".to_string()),
        lastmod_time: None,
        byte_count: None,
    };

    // Create a DIDCommPresentation with format and attachment
    let presentation = DIDCommPresentation {
        formats: vec!["dif/presentation-exchange/submission@v1.0".to_string()],
        attachments: vec![attachment],
        thid: Some("test-thread-id".to_string()),
    };

    // Convert to a DIDComm message
    let message = presentation.to_didcomm("did:example:sender")?;

    Ok(message)
}

/// This example shows how to use the common interface to work with various TAP message types.
pub fn process_any_tap_message_example(message: &PlainMessage) -> Result<()> {
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
                authorize.transaction_id
            );
        }
        _ if type_str.contains("reject") => {
            // Handle Reject message
            let reject = Reject::from_didcomm(message)?;
            println!(
                "Processing Rejection for transfer: {}",
                reject.transaction_id
            );
            println!("Reason: {}", reject.reason);
        }
        _ if type_str.contains("settle") => {
            // Handle Settle message
            let settle = Settle::from_didcomm(message)?;
            println!(
                "Processing Settlement for transfer: {}",
                settle.transaction_id
            );
            println!("Settlement ID: {}", settle.settlement_id);
            println!("Amount: {}", settle.amount.unwrap_or_default());
        }
        _ if type_str.contains("presentation") => {
            // Handle DIDCommPresentation message
            let presentation = DIDCommPresentation::from_didcomm(message)?;
            println!(
                "Processing Presentation with {} attachments",
                presentation.attachments.len()
            );

            // Process attachments
            for (i, attachment) in presentation.attachments.iter().enumerate() {
                println!(
                    "Attachment {}: Format: {}",
                    i,
                    attachment.format.as_ref().unwrap_or(&"unknown".to_string())
                );

                // Check attachment data type
                match &attachment.data {
                    AttachmentData::Json { value } => {
                        println!("  JSON data: {}", value.json);
                    }
                    AttachmentData::Base64 { value } => {
                        println!("  Base64 data: {} (truncated)", &value.base64[..20]);
                    }
                    AttachmentData::Links { value } => {
                        println!("  Links: {:?}", value.links);
                    }
                }
            }
        }
        _ => {
            println!("Unknown message type");
        }
    }

    Ok(())
}
