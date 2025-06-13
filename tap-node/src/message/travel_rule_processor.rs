//! Travel Rule message processor for TAIP-10 compliance
//!
//! This processor handles the Travel Rule flow as specified in TAIP-10, including:
//! - Handling UpdatePolicies messages that require IVMS101 presentations
//! - Processing Presentation messages containing IVMS101 data
//! - Generating and attaching IVMS101 presentations to outgoing Transfer messages

use async_trait::async_trait;
use log::{info, warn};
use serde_json::{json, Value};
use std::sync::Arc;
use tap_msg::didcomm::{Attachment, AttachmentData, PlainMessage};

use crate::customer::CustomerManager;
use crate::error::Result;
use crate::message::processor::PlainMessageProcessor;

/// Travel Rule processor that handles TAIP-10 compliant message flows
#[derive(Clone)]
pub struct TravelRuleProcessor {
    customer_manager: Arc<CustomerManager>,
}

impl TravelRuleProcessor {
    /// Create a new Travel Rule processor
    pub fn new(customer_manager: Arc<CustomerManager>) -> Self {
        Self { customer_manager }
    }

    /// Process UpdatePolicies message to check for IVMS101 requirements
    async fn handle_update_policies(&self, message: &PlainMessage) -> Result<()> {
        if let Some(policies) = message.body.get("policies").and_then(|p| p.as_array()) {
            for policy in policies {
                if let Some(policy_type) = policy.get("@type").and_then(|t| t.as_str()) {
                    if policy_type == "RequirePresentation" {
                        // Check if IVMS101 data is being requested
                        if let Some(context) = policy.get("@context").and_then(|c| c.as_array()) {
                            let requires_ivms = context.iter().any(|ctx| {
                                ctx.as_str()
                                    .map(|s| s.contains("ivms") || s.contains("intervasp"))
                                    .unwrap_or(false)
                            });

                            if requires_ivms {
                                info!(
                                    "Received IVMS101 presentation request in message {}",
                                    message.id
                                );
                                // Store this requirement for later response
                                // In a production system, you'd store this in a proper state manager
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Process Presentation message containing IVMS101 data
    async fn handle_presentation(&self, message: &PlainMessage) -> Result<()> {
        // Look for IVMS101 data in attachments
        if let Some(attachments) = message.attachments.as_ref() {
            for attachment in attachments {
                if let Some(media_type) = &attachment.media_type {
                    if media_type == "application/json" {
                        match &attachment.data {
                            AttachmentData::Json { value } => {
                                let json_data = &value.json;
                                // Check if this is IVMS101 data
                                if self.is_ivms101_credential(json_data) {
                                    info!(
                                        "Received IVMS101 presentation in message {}",
                                        message.id
                                    );

                                    // Extract IVMS101 data from the credential
                                    if let Some(ivms_data) = self.extract_ivms101_data(json_data) {
                                        // Update customer records with received IVMS101 data
                                        let from_did = &message.from;
                                        if let Ok(customer_id) =
                                            self.find_customer_by_did(from_did).await
                                        {
                                            if let Err(e) = self
                                                .customer_manager
                                                .update_customer_from_ivms101(
                                                    &customer_id,
                                                    &ivms_data,
                                                )
                                                .await
                                            {
                                                warn!(
                                                    "Failed to update customer {} with IVMS101 data: {}",
                                                    customer_id, e
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Skip non-JSON attachments
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Check if a JSON value contains IVMS101 credential data
    fn is_ivms101_credential(&self, data: &Value) -> bool {
        // Check for IVMS101 context
        if let Some(context) = data.get("@context").and_then(|c| c.as_array()) {
            if context.iter().any(|ctx| {
                ctx.as_str()
                    .map(|s| s.contains("ivms") || s.contains("intervasp"))
                    .unwrap_or(false)
            }) {
                return true;
            }
        }

        // Check for verifiable credential with IVMS101 data
        if let Some(credentials) = data
            .get("verifiableCredential")
            .and_then(|vc| vc.as_array())
        {
            return credentials.iter().any(|cred| {
                cred.get("credentialSubject")
                    .map(|cs| {
                        cs.get("originator").is_some()
                            || cs.get("beneficiary").is_some()
                            || cs.get("naturalPerson").is_some()
                            || cs.get("legalPerson").is_some()
                    })
                    .unwrap_or(false)
            });
        }

        false
    }

    /// Extract IVMS101 data from a verifiable credential
    fn extract_ivms101_data(&self, credential_data: &Value) -> Option<Value> {
        // Look for credential subject containing IVMS101 data
        if let Some(credentials) = credential_data
            .get("verifiableCredential")
            .and_then(|vc| vc.as_array())
        {
            for cred in credentials {
                if let Some(subject) = cred.get("credentialSubject") {
                    // Return the originator or beneficiary data
                    if let Some(originator) = subject.get("originator") {
                        return Some(originator.clone());
                    }
                    if let Some(beneficiary) = subject.get("beneficiary") {
                        return Some(beneficiary.clone());
                    }
                    // Direct natural/legal person data
                    if subject.get("naturalPerson").is_some()
                        || subject.get("legalPerson").is_some()
                    {
                        return Some(subject.clone());
                    }
                }
            }
        }
        None
    }

    /// Find customer by DID
    async fn find_customer_by_did(&self, did: &str) -> Result<String> {
        // In a production system, this would query the database
        // For now, we'll use the DID as the customer ID
        Ok(did.to_string())
    }

    /// Check if a Transfer message should include proactive IVMS101 data
    async fn should_attach_ivms101(&self, _message: &PlainMessage) -> bool {
        // In a production system, this would check:
        // 1. Regulatory requirements for the jurisdiction
        // 2. Transaction amount thresholds
        // 3. Counterparty policies
        // 4. Internal compliance policies

        // For now, we'll attach IVMS101 data to all transfers
        true
    }

    /// Generate IVMS101 presentation for a party
    async fn generate_ivms101_presentation(&self, party_id: &str, role: &str) -> Result<Value> {
        // Generate IVMS101 data for the customer
        let ivms_data = self
            .customer_manager
            .generate_ivms101_data(party_id)
            .await?;

        // Create a verifiable credential with the IVMS101 data
        let credential = json!({
            "@context": [
                "https://www.w3.org/2018/credentials/v1",
                "https://intervasp.org/ivms101"
            ],
            "type": ["VerifiableCredential", "TravelRuleCredential"],
            "issuer": party_id, // In production, this would be the VASP's DID
            "credentialSubject": {
                role: ivms_data
            }
        });

        // Create a verifiable presentation
        let presentation = json!({
            "@context": [
                "https://www.w3.org/2018/credentials/v1",
                "https://intervasp.org/ivms101"
            ],
            "type": ["VerifiablePresentation", "PresentationSubmission"],
            "verifiableCredential": [credential]
        });

        Ok(presentation)
    }
}

#[async_trait]
impl PlainMessageProcessor for TravelRuleProcessor {
    async fn process_incoming(&self, message: PlainMessage) -> Result<Option<PlainMessage>> {
        // Check message type
        let message_type = &message.type_;

        // Handle UpdatePolicies messages
        if message_type.contains("UpdatePolicies") {
            if let Err(e) = self.handle_update_policies(&message).await {
                warn!("Failed to handle UpdatePolicies message: {}", e);
            }
        }

        // Handle Presentation messages
        if message_type.contains("Presentation") || message_type.contains("present-proof") {
            if let Err(e) = self.handle_presentation(&message).await {
                warn!("Failed to handle Presentation message: {}", e);
            }
        }

        // Pass the message through
        Ok(Some(message))
    }

    async fn process_outgoing(&self, mut message: PlainMessage) -> Result<Option<PlainMessage>> {
        // Check if this is a Transfer message that needs IVMS101 data
        if message.type_.contains("Transfer") && self.should_attach_ivms101(&message).await {
            // Extract originator information from the message body
            if let Some(originator) = message.body.get("originator") {
                if let Some(originator_id) = originator.get("@id").and_then(|id| id.as_str()) {
                    match self
                        .generate_ivms101_presentation(originator_id, "originator")
                        .await
                    {
                        Ok(presentation) => {
                            // Add IVMS101 presentation as an attachment
                            let attachment = Attachment::json(presentation)
                                .id("ivms101-vp".to_string())
                                .media_type("application/json".to_string())
                                .format("dif/presentation-exchange/submission@v1.0".to_string())
                                .finalize();

                            // Add attachment to message
                            if message.attachments.is_none() {
                                message.attachments = Some(vec![]);
                            }
                            if let Some(attachments) = &mut message.attachments {
                                attachments.push(attachment);
                            }

                            info!(
                                "Attached IVMS101 presentation to Transfer message {}",
                                message.id
                            );
                        }
                        Err(e) => {
                            warn!(
                                "Failed to generate IVMS101 presentation for Transfer: {}",
                                e
                            );
                        }
                    }
                }
            }
        }

        // Pass the message through
        Ok(Some(message))
    }
}

impl std::fmt::Debug for TravelRuleProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TravelRuleProcessor").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_ivms101_detection() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = Arc::new(Storage::new(Some(db_path)).await.unwrap());
        let customer_manager = Arc::new(CustomerManager::new(storage));
        let processor = TravelRuleProcessor::new(customer_manager);

        // Test IVMS101 credential detection
        let ivms_credential = json!({
            "@context": [
                "https://www.w3.org/2018/credentials/v1",
                "https://intervasp.org/ivms101"
            ],
            "verifiableCredential": [{
                "credentialSubject": {
                    "originator": {
                        "naturalPerson": {
                            "name": {
                                "nameIdentifiers": [{
                                    "primaryIdentifier": "Smith",
                                    "secondaryIdentifier": "Alice"
                                }]
                            }
                        }
                    }
                }
            }]
        });

        assert!(processor.is_ivms101_credential(&ivms_credential));

        // Test non-IVMS101 credential
        let other_credential = json!({
            "@context": ["https://www.w3.org/2018/credentials/v1"],
            "type": ["VerifiableCredential"]
        });

        assert!(!processor.is_ivms101_credential(&other_credential));
    }

    #[tokio::test]
    async fn test_ivms101_extraction() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = Arc::new(Storage::new(Some(db_path)).await.unwrap());
        let customer_manager = Arc::new(CustomerManager::new(storage));
        let processor = TravelRuleProcessor::new(customer_manager);

        let credential = json!({
            "verifiableCredential": [{
                "credentialSubject": {
                    "originator": {
                        "naturalPerson": {
                            "name": {
                                "nameIdentifiers": [{
                                    "primaryIdentifier": "Smith",
                                    "secondaryIdentifier": "Alice"
                                }]
                            }
                        }
                    }
                }
            }]
        });

        let extracted = processor.extract_ivms101_data(&credential);
        assert!(extracted.is_some());

        let data = extracted.unwrap();
        assert!(data.get("naturalPerson").is_some());
    }
}
