//! TAP message types and structures.
//!
//! This module defines the structure of all TAP message types according to the specification.

use crate::error::{Error, Result};
use crate::message::policy::Policy;
use crate::message::tap_message_trait::TapMessageBody;
use chrono;
use didcomm::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tap_caip::AssetId;

/// Participant in a transfer (TAIP-3).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Participant {
    /// DID of the participant.
    #[serde(default)]
    pub id: String,

    /// Role of the participant (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub role: Option<String>,

    /// Policies of the participant according to TAIP-7 (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub policies: Option<Vec<Policy>>,
}

impl Participant {
    /// Create a new participant with the given DID.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            role: None,
            policies: None,
        }
    }

    /// Create a new participant with the given DID and role.
    pub fn with_role(id: &str, role: &str) -> Self {
        Self {
            id: id.to_string(),
            role: Some(role.to_string()),
            policies: None,
        }
    }
}

/// Attachment data for a TAP message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentData {
    /// Base64-encoded data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base64: Option<String>,

    /// JSON data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<serde_json::Value>,
}

/// Attachment for a TAP message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// ID of the attachment.
    pub id: String,

    /// Media type of the attachment.
    #[serde(rename = "media_type")]
    pub media_type: String,

    /// Attachment data (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<AttachmentData>,
}

/// Attachment format enumeration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentFormat {
    /// Base64-encoded data.
    Base64,

    /// JSON data.
    Json(serde_json::Value),

    /// External link to data.
    Links { links: Vec<String> },
}

/// Transfer message body (TAIP-3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    /// Network asset identifier (CAIP-19 format).
    pub asset: AssetId,

    /// Originator information.
    #[serde(rename = "originator")]
    pub originator: Participant,

    /// Beneficiary information (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beneficiary: Option<Participant>,

    /// Transfer amount.
    pub amount: String,

    /// Agents involved in the transfer.
    #[serde(default)]
    pub agents: Vec<Participant>,

    /// Settlement identifier (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_id: Option<String>,

    /// Memo/note for the transfer (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Additional metadata for the transfer.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Transfer {
    /// Generates a unique message ID for authorization, rejection, or settlement
    pub fn message_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

impl Authorizable for Transfer {
    fn authorize(
        &self,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Authorize {
        let timestamp = chrono::Utc::now().to_rfc3339();

        Authorize {
            transfer_id: self.message_id(),
            note,
            timestamp,
            metadata,
        }
    }

    fn reject(
        &self,
        code: String,
        description: String,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Reject {
        let timestamp = chrono::Utc::now().to_rfc3339();

        Reject {
            transfer_id: self.message_id(),
            code,
            description,
            note,
            timestamp,
            metadata,
        }
    }

    fn settle(
        &self,
        transaction_id: String,
        transaction_hash: Option<String>,
        block_height: Option<u64>,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Settle {
        let timestamp = chrono::Utc::now().to_rfc3339();

        Settle {
            transfer_id: self.message_id(),
            transaction_id,
            transaction_hash,
            block_height,
            note,
            timestamp,
            metadata,
        }
    }
}

/// Request for presentation message body (TAIP-8).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPresentation {
    /// Challenge to include in the presentation.
    pub challenge: String,

    /// Types of credentials to include.
    #[serde(default)]
    pub credential_types: Vec<String>,

    /// Format requirements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<serde_json::Value>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Presentation message body (TAIP-8).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    /// Challenge from the request.
    pub challenge: String,

    /// Credential data.
    pub credentials: Vec<serde_json::Value>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Authorize message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorize {
    /// ID of the transfer being authorized.
    pub transfer_id: String,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Timestamp when the authorization was created.
    pub timestamp: String,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Reject message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reject {
    /// ID of the transfer being rejected.
    pub transfer_id: String,

    /// Rejection code.
    pub code: String,

    /// Rejection description.
    pub description: String,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Timestamp when the rejection was created.
    pub timestamp: String,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Settle message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settle {
    /// ID of the transfer being settled.
    pub transfer_id: String,

    /// Transaction ID/hash.
    pub transaction_id: String,

    /// Optional transaction hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,

    /// Optional block height.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_height: Option<u64>,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Timestamp when the settlement was created.
    pub timestamp: String,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Add agents message body (TAIP-5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAgents {
    /// ID of the transfer to add agents to.
    pub transfer_id: String,

    /// Agents to add.
    pub agents: Vec<Participant>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Replace agent message body (TAIP-5).
///
/// This message type allows replacing an agent with another agent in a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceAgent {
    /// ID of the transfer to replace agent in.
    pub transfer_id: String,

    /// DID of the original agent to replace.
    pub original: String,

    /// Replacement agent.
    pub replacement: Participant,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Remove agent message body (TAIP-5).
///
/// This message type allows removing an agent from a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAgent {
    /// ID of the transfer to remove agent from.
    pub transfer_id: String,

    /// DID of the agent to remove.
    pub agent: String,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// UpdatePolicies message body (TAIP-7).
///
/// This message type allows agents to update their policies for a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicies {
    /// ID of the transfer to update policies for.
    pub transfer_id: String,

    /// JSON-LD context.
    #[serde(rename = "@context")]
    pub context: String,

    /// List of policies that replace the current set of policies.
    pub policies: Vec<Policy>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Error message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Error code.
    pub code: String,

    /// Error description.
    pub description: String,

    /// Original message ID (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_message_id: Option<String>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Trait for validating TAP message structures.
pub trait Validate {
    /// Validates the structure and content of a TAP message.
    fn validate(&self) -> crate::error::Result<()>;
}

/// Trait for TAP messages that can be authorized, rejected, or settled
pub trait Authorizable {
    /// Authorizes this message, creating an Authorize message as a response
    ///
    /// # Arguments
    ///
    /// * `note` - Optional note about the authorization
    /// * `metadata` - Additional metadata for the authorization
    ///
    /// # Returns
    ///
    /// A new Authorize message body
    fn authorize(
        &self,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Authorize;

    /// Rejects this message, creating a Reject message as a response
    ///
    /// # Arguments
    ///
    /// * `code` - Rejection code
    /// * `description` - Description of rejection reason
    /// * `note` - Optional note about the rejection
    /// * `metadata` - Additional metadata for the rejection
    ///
    /// # Returns
    ///
    /// A new Reject message body
    fn reject(
        &self,
        code: String,
        description: String,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Reject;

    /// Settles this message, creating a Settle message as a response
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - Transaction ID
    /// * `transaction_hash` - Optional transaction hash
    /// * `block_height` - Optional block height
    /// * `note` - Optional note about the settlement
    /// * `metadata` - Additional metadata for the settlement
    ///
    /// # Returns
    ///
    /// A new Settle message body
    fn settle(
        &self,
        transaction_id: String,
        transaction_hash: Option<String>,
        block_height: Option<u64>,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Settle;
}

// Implementation of message type conversion for message body types

impl TapMessageBody for Transfer {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#transfer"
    }

    fn validate(&self) -> Result<()> {
        // CAIP-19 asset ID is validated by the AssetId type
        // Transfer amount validation
        if self.amount.is_empty() {
            return Err(Error::Validation("Transfer amount is required".to_string()));
        }

        // Verify originator
        if self.originator.id.is_empty() {
            return Err(Error::Validation(
                "Originator ID is required in Transfer".to_string(),
            ));
        }

        Ok(())
    }
}

impl TapMessageBody for Authorize {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#authorize"
    }

    fn validate(&self) -> Result<()> {
        if self.transfer_id.is_empty() {
            return Err(Error::Validation(
                "Transfer ID is required in Authorize".to_string(),
            ));
        }

        Ok(())
    }
}

impl TapMessageBody for Reject {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#reject"
    }

    fn validate(&self) -> Result<()> {
        if self.transfer_id.is_empty() {
            return Err(Error::Validation(
                "Transfer ID is required in Reject".to_string(),
            ));
        }

        if self.code.is_empty() {
            return Err(Error::Validation(
                "Reject code is required in Reject".to_string(),
            ));
        }

        if self.description.is_empty() {
            return Err(Error::Validation(
                "Description is required in Reject".to_string(),
            ));
        }

        Ok(())
    }
}

impl TapMessageBody for Settle {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#settle"
    }

    fn validate(&self) -> Result<()> {
        if self.transfer_id.is_empty() {
            return Err(Error::Validation(
                "Transfer ID is required in Settle".to_string(),
            ));
        }

        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transaction ID is required in Settle".to_string(),
            ));
        }

        Ok(())
    }
}

impl TapMessageBody for RequestPresentation {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#requestpresentation"
    }

    fn validate(&self) -> Result<()> {
        if self.challenge.is_empty() {
            return Err(Error::Validation(
                "Challenge is required in RequestPresentation".to_string(),
            ));
        }

        Ok(())
    }
}

impl TapMessageBody for Presentation {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#presentation"
    }

    fn validate(&self) -> Result<()> {
        if self.challenge.is_empty() {
            return Err(Error::Validation(
                "Challenge is required in Presentation".to_string(),
            ));
        }

        if self.credentials.is_empty() {
            return Err(Error::Validation(
                "Credentials are required in Presentation".to_string(),
            ));
        }

        Ok(())
    }
}

impl TapMessageBody for AddAgents {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#addagents"
    }

    fn validate(&self) -> Result<()> {
        if self.transfer_id.is_empty() {
            return Err(Error::Validation(
                "Transfer ID is required in AddAgents".to_string(),
            ));
        }

        if self.agents.is_empty() {
            return Err(Error::Validation(
                "At least one agent is required in AddAgents".to_string(),
            ));
        }

        for agent in &self.agents {
            if agent.id.is_empty() {
                return Err(Error::Validation(
                    "Agent ID is required in AddAgents".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl TapMessageBody for ReplaceAgent {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#replaceagent"
    }

    fn validate(&self) -> Result<()> {
        if self.transfer_id.is_empty() {
            return Err(Error::Validation(
                "Transfer ID is required in ReplaceAgent".to_string(),
            ));
        }

        if self.original.is_empty() {
            return Err(Error::Validation(
                "Original agent ID is required in ReplaceAgent".to_string(),
            ));
        }

        if self.replacement.id.is_empty() {
            return Err(Error::Validation(
                "Replacement agent ID is required in ReplaceAgent".to_string(),
            ));
        }

        Ok(())
    }
}

impl TapMessageBody for RemoveAgent {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#removeagent"
    }

    fn validate(&self) -> Result<()> {
        if self.transfer_id.is_empty() {
            return Err(Error::Validation(
                "Transfer ID is required in RemoveAgent".to_string(),
            ));
        }

        if self.agent.is_empty() {
            return Err(Error::Validation("Agent DID cannot be empty".to_string()));
        }

        Ok(())
    }
}

impl TapMessageBody for UpdatePolicies {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#updatepolicies"
    }

    fn validate(&self) -> Result<()> {
        if self.transfer_id.is_empty() {
            return Err(Error::Validation("Transfer ID is required".to_string()));
        }

        if self.context.is_empty() {
            return Err(Error::Validation("Context is required".to_string()));
        }

        if self.policies.is_empty() {
            return Err(Error::Validation("Policies are required".to_string()));
        }

        Ok(())
    }
}

impl TapMessageBody for ErrorBody {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#error"
    }

    fn validate(&self) -> Result<()> {
        if self.code.is_empty() {
            return Err(Error::Validation(
                "Error code is required in Error message".to_string(),
            ));
        }

        if self.description.is_empty() {
            return Err(Error::Validation(
                "Error description is required in Error message".to_string(),
            ));
        }

        Ok(())
    }
}

/// Implementation of the Authorizable trait for DIDComm Message
impl Authorizable for Message {
    fn authorize(
        &self,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Authorize {
        let timestamp = chrono::Utc::now().to_rfc3339();

        Authorize {
            transfer_id: self.id.clone(),
            note,
            timestamp,
            metadata,
        }
    }

    fn reject(
        &self,
        code: String,
        description: String,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Reject {
        let timestamp = chrono::Utc::now().to_rfc3339();

        Reject {
            transfer_id: self.id.clone(),
            code,
            description,
            note,
            timestamp,
            metadata,
        }
    }

    fn settle(
        &self,
        transaction_id: String,
        transaction_hash: Option<String>,
        block_height: Option<u64>,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Settle {
        let timestamp = chrono::Utc::now().to_rfc3339();

        Settle {
            transfer_id: self.id.clone(),
            transaction_id,
            transaction_hash,
            block_height,
            note,
            timestamp,
            metadata,
        }
    }
}
