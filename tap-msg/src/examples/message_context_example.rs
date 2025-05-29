//! Example showing how to use the new MessageContext pattern with attribute-based participant extraction.
//!
//! This example demonstrates the declarative approach to defining TAP messages
//! with automatic participant extraction and routing.

use crate::didcomm::PlainMessage;
use crate::error::Result;
use crate::message::{MessageContext, Participant, TapMessageBody, TransactionContext};
use crate::{impl_message_context, impl_tap_message};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Example transfer message using the new MessageContext pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleTransfer {
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
            return Err(crate::error::Error::Validation(
                "Originator ID is required".to_string(),
            ));
        }

        if self.amount.is_empty() {
            return Err(crate::error::Error::Validation(
                "Amount is required".to_string(),
            ));
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

// Implement both TapMessage and MessageContext
impl_tap_message!(ExampleTransfer);
impl_message_context!(ExampleTransfer,
    participants: [originator, (beneficiary optional), (agents list)],
    transaction_id: transaction_id
);

/// Example showing how to create and use messages with the new pattern
pub fn example_usage() -> Result<()> {
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

    // Create the transfer message
    let transfer = ExampleTransfer {
        originator,
        beneficiary: Some(beneficiary),
        agents: vec![agent],
        transaction_id: "tx-12345".to_string(),
        amount: "100.00".to_string(),
        metadata: HashMap::new(),
    };

    // Automatic participant extraction
    let participants = transfer.participants();
    println!("Participants: {:?}", participants.len()); // Should be 3

    let participant_dids = transfer.participant_dids();
    println!("Participant DIDs: {:?}", participant_dids);

    // Transaction context
    if let Some(tx_context) = transfer.transaction_context() {
        println!("Transaction ID: {}", tx_context.transaction_id);
        println!("Transaction Type: {}", tx_context.transaction_type);
    }

    // Create a DIDComm message with automatic routing
    let didcomm_msg = transfer.to_didcomm("did:example:sender")?;
    println!("Recipients: {:?}", didcomm_msg.to);

    // Use with PlainMessage for enhanced functionality
    let typed_message = PlainMessage::new_typed(transfer, "did:example:sender");
    let extracted_participants = typed_message.extract_participants();
    println!("Extracted participants: {:?}", extracted_participants);

    Ok(())
}

/// Example of a message with optional transaction ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamplePresentation {
    /// Message ID
    pub id: String,

    /// Presenter participant
    pub presenter: Participant,

    /// Optional verifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier: Option<Participant>,

    /// Optional transaction ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,

    /// Presentation data
    pub presentation: serde_json::Value,
}

impl TapMessageBody for ExamplePresentation {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#example-presentation"
    }

    fn validate(&self) -> Result<()> {
        if self.presenter.id.is_empty() {
            return Err(crate::error::Error::Validation(
                "Presenter ID is required".to_string(),
            ));
        }
        Ok(())
    }

    fn to_didcomm(&self, from: &str) -> Result<PlainMessage> {
        let participant_dids = self.participant_dids();
        let recipients: Vec<String> = participant_dids
            .into_iter()
            .filter(|did| did != from)
            .collect();

        let body_json = serde_json::to_value(self)?;

        Ok(PlainMessage {
            id: self
                .transaction_id
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
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

// For this one, we'll manually implement using the old macros since it uses optional transaction ID
// TODO: Update the derive macro to support optional_transaction_id attribute
impl crate::message::tap_message_trait::TapMessage for ExamplePresentation {
    fn validate(&self) -> crate::error::Result<()> {
        <Self as crate::message::tap_message_trait::TapMessageBody>::validate(self)
    }
    fn is_tap_message(&self) -> bool {
        false
    }
    fn get_tap_type(&self) -> Option<String> {
        Some(
            <Self as crate::message::tap_message_trait::TapMessageBody>::message_type().to_string(),
        )
    }
    fn body_as<T: crate::message::tap_message_trait::TapMessageBody>(
        &self,
    ) -> crate::error::Result<T> {
        unimplemented!()
    }
    fn get_all_participants(&self) -> Vec<String> {
        self.participant_dids()
    }
    fn create_reply<T: crate::message::tap_message_trait::TapMessageBody>(
        &self,
        body: &T,
        creator_did: &str,
    ) -> crate::error::Result<crate::didcomm::PlainMessage> {
        let mut message = body.to_didcomm(creator_did)?;
        if let Some(thread_id) = self.thread_id() {
            message.thid = Some(thread_id.to_string());
        } else {
            message.thid = Some(self.message_id().to_string());
        }
        if let Some(parent_thread_id) = self.parent_thread_id() {
            message.pthid = Some(parent_thread_id.to_string());
        }
        Ok(message)
    }
    fn thread_id(&self) -> Option<&str> {
        self.transaction_id.as_deref()
    }
    fn parent_thread_id(&self) -> Option<&str> {
        None
    }
    fn message_id(&self) -> &str {
        if let Some(ref id) = self.transaction_id {
            id
        } else {
            &self.id
        }
    }
}

// MessageContext implementation for optional transaction ID
impl MessageContext for ExamplePresentation {
    fn participants(&self) -> Vec<&Participant> {
        let mut participants = vec![&self.presenter];
        if let Some(ref verifier) = self.verifier {
            participants.push(verifier);
        }
        participants
    }

    fn transaction_context(&self) -> Option<TransactionContext> {
        self.transaction_id
            .as_ref()
            .map(|tx_id| TransactionContext::new(tx_id.clone(), Self::message_type().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_transfer_context() {
        let originator = Participant {
            id: "did:example:alice".to_string(),
            role: Some("originator".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        };

        let beneficiary = Participant {
            id: "did:example:bob".to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        };

        let transfer = ExampleTransfer {
            originator,
            beneficiary: Some(beneficiary),
            agents: vec![],
            transaction_id: "tx-123".to_string(),
            amount: "100.00".to_string(),
            metadata: HashMap::new(),
        };

        // Test participant extraction
        let participants = transfer.participants();
        assert_eq!(participants.len(), 2);

        let participant_dids = transfer.participant_dids();
        assert_eq!(participant_dids.len(), 2);
        assert!(participant_dids.contains(&"did:example:alice".to_string()));
        assert!(participant_dids.contains(&"did:example:bob".to_string()));

        // Test transaction context
        let tx_context = transfer.transaction_context().unwrap();
        assert_eq!(tx_context.transaction_id, "tx-123");
        assert_eq!(
            tx_context.transaction_type,
            "https://tap.rsvp/schema/1.0#example-transfer"
        );
    }

    #[test]
    fn test_example_presentation_optional_context() {
        let presenter = Participant {
            id: "did:example:presenter".to_string(),
            role: Some("presenter".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        };

        // Test without transaction ID
        let presentation = ExamplePresentation {
            id: "pres-123".to_string(),
            presenter: presenter.clone(),
            verifier: None,
            transaction_id: None,
            presentation: serde_json::json!({"test": "data"}),
        };

        assert_eq!(presentation.participants().len(), 1);
        assert!(presentation.transaction_context().is_none());

        // Test with transaction ID
        let presentation_with_tx = ExamplePresentation {
            id: "pres-456".to_string(),
            presenter,
            verifier: None,
            transaction_id: Some("tx-456".to_string()),
            presentation: serde_json::json!({"test": "data"}),
        };

        assert!(presentation_with_tx.transaction_context().is_some());
        assert_eq!(
            presentation_with_tx
                .transaction_context()
                .unwrap()
                .transaction_id,
            "tx-456"
        );
    }
}
