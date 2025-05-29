//! Example of unpacking messages to both PlainMessage and TAP message types

use std::sync::Arc;
use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
use tap_agent::did::{DIDGenerationOptions, KeyType};
use tap_agent::key_manager::KeyManager;
use tap_agent::message_packing::{
    PackOptions, Packable, UnpackOptions, Unpackable, UnpackedMessage,
};
use tap_msg::didcomm::PlainMessage;
use tap_msg::message::TapMessage;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a key manager
    let key_manager = Arc::new(AgentKeyManagerBuilder::new().build()?);

    // Generate a key for the sender
    let sender_key = key_manager.generate_key(DIDGenerationOptions {
        key_type: KeyType::Ed25519,
    })?;

    println!("Sender DID: {}", sender_key.did);

    // Create a TAP Transfer message
    let plain_message = PlainMessage {
        id: "example-transfer-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0#transfer".to_string(),
        body: serde_json::json!({
            "@type": "https://tap.rsvp/schema/1.0#transfer",
            "asset": {
                "chain_id": {
                    "namespace": "eip155",
                    "reference": "1"
                },
                "namespace": "erc20",
                "reference": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
            },
            "originator": {
                "id": sender_key.did.clone(),
                "name": "Alice"
            },
            "beneficiary": {
                "id": "did:example:bob",
                "name": "Bob"
            },
            "amount": "1000",
            "agents": [],
            "memo": "Payment for services",
            "metadata": {
                "invoice_id": "INV-2024-001"
            }
        }),
        from: sender_key.did.clone(),
        to: vec!["did:example:bob".to_string()],
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };

    // Pack the message (plain mode for simplicity)
    let pack_options = PackOptions::new().with_plain();
    let packed = plain_message.pack(&*key_manager, pack_options).await?;

    println!("\nPacked message:");
    println!("{}", packed);

    // Unpack the message to get both PlainMessage and TAP message
    let unpack_options = UnpackOptions::new();
    let unpacked: UnpackedMessage = String::unpack(&packed, &*key_manager, unpack_options).await?;

    println!("\n--- Unpacked PlainMessage ---");
    println!("ID: {}", unpacked.plain_message.id);
    println!("Type: {}", unpacked.plain_message.type_);
    println!("From: {}", unpacked.plain_message.from);
    println!("To: {:?}", unpacked.plain_message.to);

    // Check if we successfully parsed the TAP message
    if let Some(tap_message) = unpacked.tap_message {
        println!("\n--- Parsed TAP Message ---");
        match tap_message {
            TapMessage::Transfer(transfer) => {
                println!("Message Type: Transfer");
                println!("Asset: {}", transfer.asset);
                println!("Amount: {}", transfer.amount);
                println!(
                    "Originator: {} ({})",
                    transfer.originator.id,
                    transfer.originator.name.as_deref().unwrap_or("Unknown")
                );
                if let Some(beneficiary) = &transfer.beneficiary {
                    println!(
                        "Beneficiary: {} ({})",
                        beneficiary.id,
                        beneficiary.name.as_deref().unwrap_or("Unknown")
                    );
                }
                if let Some(memo) = &transfer.memo {
                    println!("Memo: {}", memo);
                }
                if !transfer.metadata.is_empty() {
                    println!("Metadata: {:?}", transfer.metadata);
                }
            }
            _ => {
                println!("Unexpected message type");
            }
        }
    } else {
        println!("\nCould not parse as TAP message");
    }

    Ok(())
}
