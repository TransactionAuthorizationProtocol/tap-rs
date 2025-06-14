//! Customer event handler for automatic customer data extraction
//!
//! This handler listens to TAP message events and automatically:
//! - Extracts party information from Transfer messages
//! - Updates customer records from UpdateParty messages
//! - Manages relationships from ConfirmRelationship messages
//! - Generates IVMS101 data when needed

use crate::customer::CustomerManager;
use crate::error::Result;
use crate::event::{EventSubscriber, NodeEvent};
use crate::storage::Storage;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tap_msg::message::{transfer::Transfer, update_party::UpdateParty};

/// Event handler that automatically extracts and manages customer data
pub struct CustomerEventHandler {
    storage: Arc<Storage>,
    agent_did: String,
}

impl CustomerEventHandler {
    /// Create a new customer event handler
    pub fn new(storage: Arc<Storage>, agent_did: String) -> Self {
        Self { storage, agent_did }
    }
}

#[async_trait]
impl EventSubscriber for CustomerEventHandler {
    async fn handle_event(&self, event: NodeEvent) {
        let result: Result<()> = match &event {
            NodeEvent::MessageReceived { message, .. } | NodeEvent::MessageSent { message, .. } => {
                // Handle different message types
                match message.type_.as_str() {
                    "https://tap.rsvp/schema/1.0#Transfer" => {
                        self.handle_transfer_message(message).await
                    }
                    "https://tap.rsvp/schema/1.0#UpdateParty" => {
                        self.handle_update_party_message(message).await
                    }
                    "https://tap.rsvp/schema/1.0#ConfirmRelationship" => {
                        self.handle_confirm_relationship_message(message).await
                    }
                    _ => Ok(()),
                }
            }
            NodeEvent::TransactionCreated {
                transaction,
                agent_did,
            } => {
                log::debug!("Handling TransactionCreated event for agent: {}", agent_did);

                // Extract customer data from new transactions
                // Parse the message JSON to get the message body
                if let Ok(plain_message) = serde_json::from_value::<tap_msg::didcomm::PlainMessage>(
                    transaction.message_json.clone(),
                ) {
                    if let Ok(transfer) = serde_json::from_value::<Transfer>(plain_message.body) {
                        let manager = CustomerManager::new(self.storage.clone());

                        // Extract originator if present
                        if let Some(originator) = &transfer.originator {
                            match manager
                                .extract_customer_from_party(
                                    originator,
                                    &self.agent_did,
                                    "originator",
                                )
                                .await
                            {
                                Ok(customer_id) => {
                                    log::debug!(
                                        "Created/updated originator customer: {}",
                                        customer_id
                                    )
                                }
                                Err(e) => log::error!("Failed to extract originator: {}", e),
                            }
                        }

                        // Extract beneficiary
                        if let Some(beneficiary) = &transfer.beneficiary {
                            match manager
                                .extract_customer_from_party(
                                    beneficiary,
                                    &self.agent_did,
                                    "beneficiary",
                                )
                                .await
                            {
                                Ok(customer_id) => log::debug!(
                                    "Created/updated beneficiary customer: {}",
                                    customer_id
                                ),
                                Err(e) => log::error!("Failed to extract beneficiary: {}", e),
                            }
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        };

        if let Err(e) = result {
            log::error!("Customer event handler error: {}", e);
        }
    }
}

impl CustomerEventHandler {
    async fn handle_transfer_message(
        &self,
        message: &tap_msg::didcomm::PlainMessage,
    ) -> Result<()> {
        // Parse the transfer message
        if let Ok(transfer) = serde_json::from_value::<Transfer>(message.body.clone()) {
            let manager = CustomerManager::new(self.storage.clone());

            // Extract originator information if present
            if let Some(originator) = &transfer.originator {
                let customer_id = manager
                    .extract_customer_from_party(originator, &self.agent_did, "originator")
                    .await?;

                log::debug!("Extracted originator customer: {}", customer_id);
            }

            // Extract beneficiary information
            if let Some(beneficiary) = &transfer.beneficiary {
                let customer_id = manager
                    .extract_customer_from_party(beneficiary, &self.agent_did, "beneficiary")
                    .await?;

                log::debug!("Extracted beneficiary customer: {}", customer_id);
            }

            // Extract agent relationships
            for agent in &transfer.agents {
                for agent_for in &agent.for_parties.0 {
                    // Create relationship between agent and party
                    if let Ok(Some(customer)) = self
                        .storage
                        .get_customer_by_identifier(&agent.id)
                        .await
                        .map_err(|e| crate::error::Error::Storage(e.to_string()))
                    {
                        let _ = manager
                            .add_relationship(&customer.id, "acts_for", agent_for, None)
                            .await;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_update_party_message(
        &self,
        message: &tap_msg::didcomm::PlainMessage,
    ) -> Result<()> {
        // Parse the UpdateParty message
        if let Ok(update_party) = serde_json::from_value::<UpdateParty>(message.body.clone()) {
            let manager = CustomerManager::new(self.storage.clone());

            // Extract the party information
            let customer_id = manager
                .extract_customer_from_party(
                    &update_party.party,
                    &self.agent_did,
                    &update_party.party_type,
                )
                .await?;

            // If the party has additional schema.org data, update the profile
            if let Some(profile_data) = extract_schema_org_data(&update_party.party) {
                manager
                    .update_customer_profile(&customer_id, profile_data)
                    .await?;
            }

            log::debug!(
                "Updated {} customer: {}",
                update_party.party_type,
                customer_id
            );
        }

        Ok(())
    }

    async fn handle_confirm_relationship_message(
        &self,
        message: &tap_msg::didcomm::PlainMessage,
    ) -> Result<()> {
        // Extract relationship confirmation from the message body
        if let Some(body) = message.body.as_object() {
            if let (Some(agent_id), Some(for_id)) = (
                body.get("@id").and_then(|v| v.as_str()),
                body.get("for").and_then(|v| v.as_str()),
            ) {
                let manager = CustomerManager::new(self.storage.clone());

                // Find the customer for the agent
                if let Ok(Some(customer)) = self
                    .storage
                    .get_customer_by_identifier(agent_id)
                    .await
                    .map_err(|e| crate::error::Error::Storage(e.to_string()))
                {
                    // Add the confirmed relationship
                    let proof = json!({
                        "type": "ConfirmRelationship",
                        "message_id": message.id,
                        "from": message.from,
                        "timestamp": message.created_time
                    });

                    manager
                        .add_relationship(&customer.id, "confirmed_acts_for", for_id, Some(proof))
                        .await?;

                    log::debug!("Confirmed relationship: {} acts for {}", agent_id, for_id);
                }
            }
        }

        Ok(())
    }
}

/// Extract schema.org data from a Party object
fn extract_schema_org_data(party: &tap_msg::message::Party) -> Option<Value> {
    // In a real implementation, this would parse the party's metadata
    // to extract schema.org compatible data

    // For now, return a basic schema.org Person object if we have a name in metadata
    party
        .metadata
        .get("name")
        .or_else(|| party.metadata.get("https://schema.org/name"))
        .map(|name| {
            json!({
                "@context": "https://schema.org",
                "@type": "Person",
                "name": name
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tap_msg::message::Party;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_customer_extraction_from_transfer() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = Arc::new(Storage::new(Some(db_path)).await.unwrap());

        let handler = CustomerEventHandler::new(storage.clone(), "did:key:agent".to_string());

        use std::collections::HashMap;

        let mut alice_metadata = HashMap::new();
        alice_metadata.insert("name".to_string(), json!("Alice"));
        let originator = Party::with_metadata("did:key:alice", alice_metadata);

        let mut bob_metadata = HashMap::new();
        bob_metadata.insert("name".to_string(), json!("Bob"));
        let beneficiary = Party::with_metadata("bob@example.com", bob_metadata);

        let transfer = Transfer {
            asset: "eip155:1/slip44:60".parse().unwrap(),
            originator: Some(originator),
            beneficiary: Some(beneficiary),
            amount: "100".to_string(),
            agents: vec![],
            memo: None,
            settlement_id: None,
            connection_id: None,
            transaction_id: "tx-123".to_string(),
            metadata: Default::default(),
        };

        let message = tap_msg::didcomm::PlainMessage {
            id: "msg-123".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
            body: serde_json::to_value(&transfer).unwrap(),
            from: "did:key:sender".to_string(),
            to: vec!["did:key:receiver".to_string()],
            thid: None,
            pthid: None,
            extra_headers: Default::default(),
            attachments: None,
            created_time: None,
            expires_time: None,
            from_prior: None,
        };

        let event = NodeEvent::MessageReceived {
            message,
            source: "test".to_string(),
        };

        handler.handle_event(event).await;

        // Verify customers were created
        let alice = storage
            .get_customer_by_identifier("did:key:alice")
            .await
            .unwrap();
        assert!(alice.is_some());

        let bob = storage
            .get_customer_by_identifier("mailto:bob@example.com")
            .await
            .unwrap();
        assert!(bob.is_some());
    }
}
