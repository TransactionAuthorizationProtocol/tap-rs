//! Comprehensive demo of the MessageContext Pattern
//!
//! This example demonstrates the complete usage of the new MessageContext pattern
//! for declarative participant extraction and enhanced message routing.

use std::collections::HashMap;
use tap_msg::{
    didcomm::PlainMessage,
    error::Result,
    impl_message_context, impl_tap_message,
    message::{MessageContext, TapMessageBody, TransactionContext},
};
use serde::{Deserialize, Serialize};

/// Demo participant struct for examples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub role: Option<String>,
    pub policies: Option<Vec<String>>,
    #[serde(rename = "leiCode")]
    pub leiCode: Option<String>,
    pub name: Option<String>,
}

/// Example Transfer message using the new MessageContext pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoTransfer {
    /// Originator participant - automatically extracted
    pub originator: Participant,
    
    /// Optional beneficiary participant - automatically extracted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beneficiary: Option<Participant>,
    
    /// List of agent participants - automatically extracted
    #[serde(default)]
    pub agents: Vec<Participant>,
    
    /// Transaction ID for tracking
    pub transaction_id: String,
    
    /// Transfer amount
    pub amount: String,
    
    /// Asset identifier
    pub asset: String,
}

impl TapMessageBody for DemoTransfer {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#demo-transfer"
    }

    fn validate(&self) -> Result<()> {
        if self.originator.id.is_empty() {
            return Err(tap_msg::error::Error::Validation("Originator ID is required".to_string()));
        }
        
        if self.amount.is_empty() {
            return Err(tap_msg::error::Error::Validation("Amount is required".to_string()));
        }
        
        if self.asset.is_empty() {
            return Err(tap_msg::error::Error::Validation("Asset is required".to_string()));
        }
        
        Ok(())
    }

    fn to_didcomm(&self, from: &str) -> Result<PlainMessage> {
        // Use MessageContext to extract participants automatically
        let participant_dids = self.participant_dids();
        let recipients: Vec<String> = participant_dids
            .into_iter()
            .filter(|did| did != from)
            .collect();

        let body_json = serde_json::to_value(self)?;

        Ok(PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
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

// Implement both TapMessage and MessageContext using the new macros
impl_tap_message!(DemoTransfer);
impl_message_context!(DemoTransfer, 
    participants: [originator, (beneficiary optional), (agents list)],
    transaction_id: transaction_id
);

fn main() -> Result<()> {
    println!("🚀 TAP MessageContext Pattern Demo\n");
    
    // 1. Create participants
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

    let agent1 = Participant {
        id: "did:example:agent1".to_string(),
        role: Some("agent".to_string()),
        policies: None,
        leiCode: None,
        name: Some("TAP Agent 1".to_string()),
    };

    let agent2 = Participant {
        id: "did:example:agent2".to_string(),
        role: Some("agent".to_string()),
        policies: None,
        leiCode: None,
        name: Some("TAP Agent 2".to_string()),
    };

    // 2. Create the transfer message
    let transfer = DemoTransfer {
        originator,
        beneficiary: Some(beneficiary),
        agents: vec![agent1, agent2],
        transaction_id: "tx-demo-12345".to_string(),
        amount: "1000.00".to_string(),
        asset: "USD".to_string(),
    };

    println!("📝 Created transfer message:");
    println!("   Transaction ID: {}", transfer.transaction_id);
    println!("   Amount: {} {}", transfer.amount, transfer.asset);
    
    // 3. Demonstrate automatic participant extraction
    println!("\n👥 Automatic Participant Extraction:");
    let participants = transfer.participants();
    println!("   Found {} participants:", participants.len());
    for (i, participant) in participants.iter().enumerate() {
        println!("   {}. {} ({})", 
            i + 1, 
            participant.id, 
            participant.role.as_ref().unwrap_or(&"unknown".to_string())
        );
    }

    let participant_dids = transfer.participant_dids();
    println!("\n📋 Participant DIDs for routing:");
    for did in &participant_dids {
        println!("   • {}", did);
    }

    // 4. Demonstrate transaction context
    println!("\n🔗 Transaction Context:");
    if let Some(tx_context) = transfer.transaction_context() {
        println!("   Transaction ID: {}", tx_context.transaction_id);
        println!("   Transaction Type: {}", tx_context.transaction_type);
        println!("   Parent Transaction: {:?}", tx_context.parent_transaction_id);
    }

    // 5. Demonstrate routing hints
    println!("\n🛤️  Routing Hints:");
    let routing_hints = transfer.routing_hints();
    println!("   Priority: {:?}", routing_hints.priority);
    println!("   Require Encryption: {}", routing_hints.require_encryption);
    println!("   Preferred Endpoints: {:?}", routing_hints.preferred_endpoints);

    // 6. Create DIDComm message with automatic recipient detection
    println!("\n📨 DIDComm Message Creation:");
    let sender_did = "did:example:sender";
    let didcomm_msg = transfer.to_didcomm(sender_did)?;
    
    println!("   Message ID: {}", didcomm_msg.id);
    println!("   Message Type: {}", didcomm_msg.type_);
    println!("   From: {}", didcomm_msg.from);
    println!("   To Recipients ({}):", didcomm_msg.to.len());
    for recipient in &didcomm_msg.to {
        println!("     • {}", recipient);
    }

    // 7. Demonstrate typed PlainMessage usage
    println!("\n🔧 Typed PlainMessage Usage:");
    let typed_message = PlainMessage::new_typed(transfer.clone(), sender_did);
    let extracted_participants = typed_message.extract_participants();
    println!("   Extracted {} participants via PlainMessage", extracted_participants.len());

    // 8. Show enhanced PlainMessage with MessageContext
    println!("\n⚡ Enhanced PlainMessage with MessageContext:");
    // Note: This would work if Transfer implemented MessageContext, which it does!
    let enhanced_message = PlainMessage::new_typed_with_context(transfer.clone(), sender_did);
    println!("   Auto-detected recipients: {:?}", enhanced_message.to);
    
    let context_participants = enhanced_message.extract_participants_with_context();
    println!("   Context-extracted participants: {:?}", context_participants);

    // 9. Validation
    println!("\n✅ Message Validation:");
    match transfer.validate() {
        Ok(()) => println!("   ✓ Transfer message is valid"),
        Err(e) => println!("   ✗ Transfer message is invalid: {}", e),
    }

    println!("\n🎉 Demo completed successfully!");
    println!("\n💡 Key Benefits of MessageContext Pattern:");
    println!("   • Declarative participant extraction");
    println!("   • Automatic routing configuration");
    println!("   • Type-safe message context");
    println!("   • Reduced boilerplate code");
    println!("   • Enhanced PlainMessage integration");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_transfer_context() {
        let transfer = DemoTransfer {
            originator: Participant {
                id: "did:example:alice".to_string(),
                role: Some("originator".to_string()),
                policies: None,
                leiCode: None,
                name: None,
            },
            beneficiary: Some(Participant {
                id: "did:example:bob".to_string(),
                role: Some("beneficiary".to_string()),
                policies: None,
                leiCode: None,
                name: None,
            }),
            agents: vec![Participant {
                id: "did:example:agent".to_string(),
                role: Some("agent".to_string()),
                policies: None,
                leiCode: None,
                name: None,
            }],
            transaction_id: "tx-test".to_string(),
            amount: "100.00".to_string(),
            asset: "USD".to_string(),
        };

        // Test participant extraction
        assert_eq!(transfer.participants().len(), 3);
        assert_eq!(transfer.participant_dids().len(), 3);

        // Test transaction context
        let tx_context = transfer.transaction_context().unwrap();
        assert_eq!(tx_context.transaction_id, "tx-test");
        assert_eq!(tx_context.transaction_type, "https://tap.rsvp/schema/1.0#demo-transfer");

        // Test DIDComm generation
        let didcomm = transfer.to_didcomm("did:example:sender").unwrap();
        assert_eq!(didcomm.to.len(), 3); // All participants except sender
        assert!(!didcomm.to.contains(&"did:example:sender".to_string()));
    }

    #[test]
    fn test_typed_plain_message_with_context() {
        let transfer = DemoTransfer {
            originator: Participant {
                id: "did:example:alice".to_string(),
                role: Some("originator".to_string()),
                policies: None,
                leiCode: None,
                name: None,
            },
            beneficiary: None, // Test without beneficiary
            agents: vec![],    // Test without agents
            transaction_id: "tx-simple".to_string(),
            amount: "50.00".to_string(),
            asset: "EUR".to_string(),
        };

        let typed_msg = PlainMessage::new_typed_with_context(transfer, "did:example:sender");
        
        // Should have only originator, and sender should be excluded from recipients
        assert_eq!(typed_msg.to.len(), 1);
        assert_eq!(typed_msg.to[0], "did:example:alice");
        
        let participants = typed_msg.extract_participants_with_context();
        assert_eq!(participants.len(), 1);
        assert_eq!(participants[0], "did:example:alice");
    }
}