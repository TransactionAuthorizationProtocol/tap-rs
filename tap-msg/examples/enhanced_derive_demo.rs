//! Enhanced derive macro demo showing the new features
//!
//! This example demonstrates:
//! 1. Using the message_type parameter to auto-generate TapMessageBody
//! 2. Automatic to_didcomm implementation with participant extraction

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{Agent, MessageContext};
use tap_msg::{error::Result, TapMessage};

/// Example transfer message using the enhanced derive macro
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#example-enhanced-transfer")]
pub struct EnhancedTransfer {
    /// Originator participant - automatically extracted for routing
    #[tap(participant)]
    pub originator: Agent,

    /// Optional beneficiary participant - automatically extracted for routing
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tap(participant)]
    pub beneficiary: Option<Agent>,

    /// List of agent participants - automatically extracted for routing
    #[serde(default)]
    #[tap(participant_list)]
    pub agents: Vec<Agent>,

    /// Transaction ID for tracking - used for message threading
    #[tap(transaction_id)]
    pub transaction_id: String,

    /// Transfer amount
    pub amount: String,

    /// Asset identifier (simplified for this example)
    pub asset_id: String,

    /// Additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

// The macro automatically implements TapMessageBody including:
// - message_type() returning "https://tap.rsvp/schema/1.0#example-enhanced-transfer"
// - validate() with basic validation (always returns Ok(()))
// - to_didcomm() with automatic participant extraction and message construction

// We can add custom validation by implementing a separate validation method
impl EnhancedTransfer {
    /// Custom validation that can be called in addition to the basic generated validation
    pub fn validate_enhanced(&self) -> Result<()> {
        // Custom validation logic
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

        if self.amount.parse::<f64>().is_err() {
            return Err(tap_msg::error::Error::Validation(
                "Amount must be a valid number".to_string(),
            ));
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    // Create participants
    let originator = Agent::new("did:example:alice", "originator", "did:example:alice");

    let beneficiary = Agent::new("did:example:bob", "beneficiary", "did:example:bob");

    let agent = Agent::new("did:example:agent", "agent", "did:example:agent");

    // Create the enhanced transfer message
    let transfer = EnhancedTransfer {
        originator,
        beneficiary: Some(beneficiary),
        agents: vec![agent],
        transaction_id: "tx-enhanced-123".to_string(),
        amount: "250.00".to_string(),
        asset_id: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
        metadata: HashMap::new(),
    };

    println!("=== Enhanced TapMessage Derive Macro Demo ===\n");

    // Demonstrate automatic TapMessageBody implementation
    println!("1. Message Type (auto-generated):");
    println!(
        "   {}",
        <EnhancedTransfer as TapMessageBody>::message_type()
    );

    // Demonstrate validation (both generated and custom)
    println!("\n2. Validation:");

    // Basic validation (generated by macro)
    match TapMessageBody::validate(&transfer) {
        Ok(()) => println!("   ✓ Basic validation passed"),
        Err(e) => println!("   ✗ Basic validation failed: {}", e),
    }

    // Enhanced validation (custom implementation)
    match transfer.validate_enhanced() {
        Ok(()) => println!("   ✓ Enhanced validation passed"),
        Err(e) => println!("   ✗ Enhanced validation failed: {}", e),
    }

    // Demonstrate automatic participant extraction via MessageContext
    println!("\n3. Automatic Participant Extraction:");
    let participants = transfer.participant_dids();
    println!("   Number of participants: {}", participants.len());
    for (i, participant_did) in participants.iter().enumerate() {
        println!("   Participant {}: {}", i + 1, participant_did);
    }

    // Demonstrate participant DID extraction via MessageContext
    println!("\n4. Participant DID Extraction:");
    let participant_dids = transfer.participant_dids();
    println!("   Participant DIDs: {:?}", participant_dids);

    // Demonstrate transaction context
    println!("\n5. Transaction Context:");
    if let Some(tx_context) = transfer.transaction_context() {
        println!("   Transaction ID: {}", tx_context.transaction_id);
        println!("   Transaction Type: {}", tx_context.transaction_type);
    }

    // Demonstrate DIDComm conversion with automatic routing
    println!("\n6. DIDComm Message Creation (auto-generated to_didcomm):");
    let didcomm_msg = transfer.to_didcomm("did:example:sender")?;
    println!("   Message ID: {}", didcomm_msg.id);
    println!("   From: {}", didcomm_msg.from);
    println!("   To: {:?}", didcomm_msg.to);
    println!("   Type: {}", didcomm_msg.type_);
    println!("   Thread ID: {:?}", didcomm_msg.thid);

    println!("\n=== Demo Complete ===");
    println!("\nThe enhanced derive macro automatically generated:");
    println!("- TapMessageBody trait implementation with message_type() and to_didcomm()");
    println!("- TapMessage trait implementation with proper threading and participant extraction");
    println!("- MessageContext trait implementation with participant and transaction context");
    println!("- Automatic participant routing in to_didcomm() based on field attributes");

    Ok(())
}
