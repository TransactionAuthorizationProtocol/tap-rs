//! Demonstration of the new TapMessage derive macro
//!
//! This example shows how to use the new procedural derive macro
//! to automatically implement TapMessage and MessageContext traits.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::TapMessage as TapMessageTrait;
use tap_msg::message::{MessageContext, Participant, TapMessageBody};
use tap_msg::{didcomm::PlainMessage, error::Result, TapMessage};

/// Example message using the new derive macro
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
pub struct ExampleTransfer {
    /// Originator participant - automatically extracted
    #[tap(participant)]
    pub originator: Participant,

    /// Optional beneficiary participant - automatically extracted
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tap(participant)]
    pub beneficiary: Option<Participant>,

    /// List of agent participants - automatically extracted
    #[serde(default)]
    #[tap(participant_list)]
    pub agents: Vec<Participant>,

    /// Transaction ID for tracking - used for message threading
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// Transfer amount
    pub amount: String,

    /// Asset being transferred
    pub asset: AssetId,

    /// Additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TapMessageBody for ExampleTransfer {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#example-transfer"
    }

    fn validate(&self) -> Result<()> {
        if self.originator.id.is_empty() {
            return Err(tap_msg::error::Error::Validation(
                "Originator ID is required".to_string(),
            ));
        }

        if self.amount.is_empty() {
            return Err(tap_msg::error::Error::Validation(
                "Amount is required".to_string(),
            ));
        }

        Ok(())
    }

    fn to_didcomm(&self, from: &str) -> Result<PlainMessage> {
        // Use the automatically implemented get_all_participants method
        let participant_dids = self.get_all_participants();
        let recipients: Vec<String> = participant_dids
            .into_iter()
            .filter(|did| did != from)
            .collect();

        let body_json = serde_json::to_value(self)?;

        Ok(PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: <Self as TapMessageBody>::message_type().to_string(),
            body: body_json,
            from: from.to_string(),
            to: recipients,
            thid: None,
            pthid: None,
            created_time: Some(chrono::Utc::now().timestamp() as u64),
            expires_time: None,
            from_prior: None,
            attachments: None,
            extra_headers: HashMap::new(),
        })
    }
}

fn main() -> Result<()> {
    // Create participants
    let originator = Participant {
        id: "did:example:alice".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
        name: Some("Alice".to_string()),
    };

    let beneficiary = Participant {
        id: "did:example:bob".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
        name: Some("Bob".to_string()),
    };

    let agent = Participant {
        id: "did:example:agent".to_string(),
        role: Some("agent".to_string()),
        policies: None,
        leiCode: None,
        name: Some("TAP Agent".to_string()),
    };

    // Create asset ID
    let chain_id = tap_caip::ChainId::new("eip155", "1").unwrap();
    let asset = AssetId::new(
        chain_id,
        "erc20",
        "0x6b175474e89094c44da98b954eedeac495271d0f",
    )
    .unwrap();

    // Create the transfer message using the new derive macro
    let transfer = ExampleTransfer {
        originator,
        beneficiary: Some(beneficiary),
        agents: vec![agent],
        transaction_id: "tx-12345".to_string(),
        amount: "100.00".to_string(),
        asset,
        metadata: HashMap::new(),
    };

    println!("=== TapMessage Derive Macro Demo ===\n");

    // Demonstrate automatic participant extraction via MessageContext
    println!("1. Automatic Participant Extraction:");
    let participants = transfer.participants();
    println!("   Number of participants: {}", participants.len());
    for (i, participant) in participants.iter().enumerate() {
        println!(
            "   Participant {}: {} ({})",
            i + 1,
            participant.id,
            participant.role.as_deref().unwrap_or("unknown")
        );
    }

    // Demonstrate participant DID extraction
    println!("\n2. Participant DID Extraction:");
    let participant_dids = transfer.participant_dids();
    println!("   Participant DIDs: {:?}", participant_dids);

    // Demonstrate automatic TapMessage implementation
    println!("\n3. TapMessage Implementation:");
    println!(
        "   Message type: {}",
        <ExampleTransfer as TapMessageBody>::message_type()
    );
    println!("   Thread ID: {:?}", transfer.thread_id());
    println!("   Message ID: {}", transfer.message_id());
    println!("   All participants: {:?}", transfer.get_all_participants());

    // Demonstrate transaction context
    println!("\n4. Transaction Context:");
    if let Some(tx_context) = transfer.transaction_context() {
        println!("   Transaction ID: {}", tx_context.transaction_id);
        println!("   Transaction Type: {}", tx_context.transaction_type);
    }

    // Demonstrate DIDComm conversion with automatic routing
    println!("\n5. DIDComm Message Creation:");
    let didcomm_msg = transfer.to_didcomm("did:example:sender")?;
    println!("   Message ID: {}", didcomm_msg.id);
    println!("   From: {}", didcomm_msg.from);
    println!("   To: {:?}", didcomm_msg.to);
    println!("   Type: {}", didcomm_msg.type_);

    println!("\n=== Demo Complete ===");
    println!("\nThe derive macro automatically generated:");
    println!("- TapMessage trait implementation with proper threading");
    println!("- MessageContext trait implementation with participant extraction");
    println!("- All based on the #[tap(...)] field attributes!");

    Ok(())
}
