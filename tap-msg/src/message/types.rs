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

/// Participant in a transfer (TAIP-3, TAIP-11).
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

    /// Legal Entity Identifier (LEI) according to TAIP-11 (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[allow(non_snake_case)]
    pub leiCode: Option<String>,
}

impl Participant {
    /// Create a new participant with the given DID.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            role: None,
            policies: None,
            leiCode: None,
        }
    }

    /// Create a new participant with the given DID and role.
    pub fn with_role(id: &str, role: &str) -> Self {
        Self {
            id: id.to_string(),
            role: Some(role.to_string()),
            policies: None,
            leiCode: None,
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
            settlement_address: None,
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

    fn confirm_relationship(
        &self,
        agent_id: String,
        for_id: String,
        role: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> ConfirmRelationship {
        ConfirmRelationship {
            transfer_id: self.message_id(),
            agent_id,
            for_id,
            role,
            metadata,
        }
    }

    fn update_party(
        &self,
        party_type: String,
        party: Participant,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> UpdateParty {
        UpdateParty {
            transfer_id: self.message_id(),
            party_type,
            party,
            note,
            metadata,
            context: Some("https://tap.rsvp/schema/1.0".to_string()),
        }
    }

    fn update_policies(
        &self,
        policies: Vec<Policy>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> UpdatePolicies {
        UpdatePolicies {
            transfer_id: self.message_id(),
            policies,
            metadata,
        }
    }

    fn add_agents(
        &self,
        agents: Vec<Participant>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> AddAgents {
        AddAgents {
            transfer_id: self.message_id(),
            agents,
            metadata,
        }
    }

    fn replace_agent(
        &self,
        original: String,
        replacement: Participant,
        metadata: HashMap<String, serde_json::Value>,
    ) -> ReplaceAgent {
        ReplaceAgent {
            transfer_id: self.message_id(),
            original,
            replacement,
            metadata,
        }
    }

    fn remove_agent(
        &self,
        agent: String,
        metadata: HashMap<String, serde_json::Value>,
    ) -> RemoveAgent {
        RemoveAgent {
            transfer_id: self.message_id(),
            agent,
            metadata,
        }
    }
}

/// Request Presentation message body (TAIP-10).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPresentation {
    /// Transfer ID that this request is related to.
    pub transfer_id: String,

    /// Presentation definition identifier or URI.
    pub presentation_definition: String,

    /// Description of the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Challenge to be included in the response.
    pub challenge: String,

    /// Whether the request is for the originator's information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub for_originator: Option<bool>,

    /// Whether the request is for the beneficiary's information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub for_beneficiary: Option<bool>,

    /// Note for the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Presentation message body (TAIP-8, TAIP-10).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    /// Challenge from the request.
    pub challenge: String,

    /// Credential data.
    pub credentials: Vec<serde_json::Value>,

    /// Transfer ID that this presentation is related to (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer_id: Option<String>,

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

    /// Optional settlement address in CAIP-10 format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_address: Option<String>,

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

/// Cancel message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cancel {
    /// ID of the transfer being cancelled.
    pub transfer_id: String,

    /// Optional reason for cancellation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Timestamp when the cancellation was created.
    pub timestamp: String,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Revert message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revert {
    /// ID of the transfer being reverted.
    pub transfer_id: String,

    /// Settlement address in CAIP-10 format to return the funds to.
    pub settlement_address: String,

    /// Reason for the reversal request.
    pub reason: String,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Timestamp when the revert request was created.
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

/// ConfirmRelationship message body (TAIP-9).
///
/// This message type allows confirming a relationship between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmRelationship {
    /// ID of the transfer related to this message.
    pub transfer_id: String,

    /// DID of the agent whose relationship is being confirmed.
    pub agent_id: String,

    /// DID of the entity that the agent acts on behalf of.
    #[serde(rename = "for")]
    pub for_id: String,

    /// Role of the agent in the transaction (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Additional metadata (optional).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ConfirmRelationship {
    /// Creates a new ConfirmRelationship message body.
    pub fn new(transfer_id: &str, agent_id: &str, for_id: &str, role: Option<String>) -> Self {
        Self {
            transfer_id: transfer_id.to_string(),
            agent_id: agent_id.to_string(),
            for_id: for_id.to_string(),
            role,
            metadata: HashMap::new(),
        }
    }

    /// Validates the ConfirmRelationship message body.
    pub fn validate(&self) -> Result<()> {
        if self.transfer_id.is_empty() {
            return Err(Error::Validation(
                "Transfer ID is required in ConfirmRelationship".to_string(),
            ));
        }

        if self.agent_id.is_empty() {
            return Err(Error::Validation(
                "Agent ID is required in ConfirmRelationship".to_string(),
            ));
        }

        if self.for_id.is_empty() {
            return Err(Error::Validation(
                "For ID is required in ConfirmRelationship".to_string(),
            ));
        }

        Ok(())
    }
}

/// UpdateParty message body (TAIP-6).
///
/// This message type allows agents to update party information in a transaction.
/// It enables a participant to modify their details or role within an existing transfer without
/// creating a new transaction. This is particularly useful for situations where participant
/// information changes during the lifecycle of a transaction.
///
/// # TAIP-6 Specification
/// The UpdateParty message follows the TAIP-6 specification for updating party information
/// in a TAP transaction. It includes JSON-LD compatibility with an optional @context field.
///
/// # Example
/// ```
/// use tap_msg::message::types::UpdateParty;
/// use tap_msg::Participant;
/// use std::collections::HashMap;
///
/// // Create a participant with updated information
/// let updated_participant = Participant {
///     id: "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx".to_string(),
///     role: Some("new_role".to_string()),
///     policies: None,
///     leiCode: None,
/// };
///
/// // Create an UpdateParty message
/// let update_party = UpdateParty::new(
///     "transfer-123",
///     "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx",
///     updated_participant
/// );
///
/// // Add an optional note
/// let update_party_with_note = UpdateParty {
///     note: Some("Updating role after compliance check".to_string()),
///     ..update_party
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateParty {
    /// ID of the transaction this update relates to.
    pub transfer_id: String,

    /// Type of party being updated (e.g., 'originator', 'beneficiary').
    #[serde(rename = "partyType")]
    pub party_type: String,

    /// Updated party information.
    pub party: Participant,

    /// Optional note regarding the update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Additional metadata for the update.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Optional JSON-LD context.
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl UpdateParty {
    /// Creates a new UpdateParty message body.
    pub fn new(transfer_id: &str, party_type: &str, party: Participant) -> Self {
        Self {
            transfer_id: transfer_id.to_string(),
            party_type: party_type.to_string(),
            party,
            note: None,
            metadata: HashMap::new(),
            context: Some("https://tap.rsvp/schema/1.0".to_string()),
        }
    }

    /// Validates the UpdateParty message body.
    pub fn validate(&self) -> Result<()> {
        if self.transfer_id.is_empty() {
            return Err(Error::Validation("transfer_id cannot be empty".to_string()));
        }

        if self.party_type.is_empty() {
            return Err(Error::Validation("partyType cannot be empty".to_string()));
        }

        if self.party.id.is_empty() {
            return Err(Error::Validation("party.id cannot be empty".to_string()));
        }

        Ok(())
    }
}

impl TapMessageBody for UpdateParty {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#updateparty"
    }

    fn validate(&self) -> Result<()> {
        self.validate()
    }

    fn to_didcomm(&self) -> Result<Message> {
        // Serialize the UpdateParty to a JSON value
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        let now = crate::utils::get_current_time()?;

        // Create a new Message with required fields
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: None,
            to: None,
            thid: None,
            pthid: None,
            extra_headers: std::collections::HashMap::new(),
            created_time: Some(now),
            expires_time: None,
            from_prior: None,
            attachments: None,
        };

        Ok(message)
    }

    fn from_didcomm(message: &Message) -> Result<Self> {
        let body = message
            .body
            .as_object()
            .ok_or_else(|| Error::Validation("Message body is not a JSON object".to_string()))?;

        let transfer_id = body
            .get("transfer_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing or invalid transfer_id".to_string()))?;

        let party_type = body
            .get("partyType")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing or invalid partyType".to_string()))?;

        let party = body
            .get("party")
            .ok_or_else(|| Error::Validation("Missing party information".to_string()))?;

        let party: Participant = serde_json::from_value(party.clone())
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        let note = body
            .get("note")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        // Get context if available
        let context = body
            .get("@context")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let mut metadata = HashMap::new();
        for (k, v) in body.iter() {
            if !["transfer_id", "partyType", "party", "note", "@context"].contains(&k.as_str()) {
                metadata.insert(k.clone(), v.clone());
            }
        }

        let update_party = Self {
            transfer_id: transfer_id.to_string(),
            party_type: party_type.to_string(),
            party,
            note,
            metadata,
            context,
        };

        update_party.validate()?;

        Ok(update_party)
    }
}

/// UpdatePolicies message body (TAIP-7).
///
/// This message type allows agents to update their policies for a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicies {
    #[serde(rename = "transferId")]
    pub transfer_id: String,
    pub policies: Vec<Policy>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl UpdatePolicies {
    pub fn new(transfer_id: &str, policies: Vec<Policy>) -> Self {
        Self {
            transfer_id: transfer_id.to_string(),
            policies,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.transfer_id.is_empty() {
            return Err(Error::Validation(
                "UpdatePolicies must have a transfer_id".to_string(),
            ));
        }

        if self.policies.is_empty() {
            return Err(Error::Validation(
                "UpdatePolicies must have at least one policy".to_string(),
            ));
        }

        for policy in &self.policies {
            policy.validate()?;
        }

        Ok(())
    }
}

impl TapMessageBody for UpdatePolicies {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#updatepolicies"
    }

    fn validate(&self) -> Result<()> {
        UpdatePolicies::validate(self)
    }

    fn to_didcomm(&self) -> Result<Message> {
        // Serialize the UpdatePolicies to a JSON value
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        let now = crate::utils::get_current_time()?;

        // Create a new Message with required fields
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: None,
            to: None,
            thid: None,
            pthid: None,
            extra_headers: std::collections::HashMap::new(),
            created_time: Some(now),
            expires_time: None,
            from_prior: None,
            attachments: None,
        };

        Ok(message)
    }

    fn from_didcomm(message: &Message) -> Result<Self> {
        // Verify this is the correct message type
        if message.type_ != Self::message_type() {
            return Err(Error::InvalidMessageType(format!(
                "Expected message type {}, but found {}",
                Self::message_type(),
                message.type_
            )));
        }

        // Create a copy of the body that we can modify
        let mut body_json = message.body.clone();

        // Remove the @type field if present as we no longer need it in our struct
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.remove("@type");
        }

        // Deserialize the body
        let update_policies = serde_json::from_value(body_json).map_err(|e| {
            Error::SerializationError(format!("Failed to deserialize UpdatePolicies: {}", e))
        })?;

        Ok(update_policies)
    }
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

    /// Confirms a relationship between agents, creating a ConfirmRelationship message as a response
    ///
    /// # Arguments
    ///
    /// * `agent_id` - DID of the agent whose relationship is being confirmed
    /// * `for_id` - DID of the entity that the agent acts on behalf of
    /// * `role` - Optional role of the agent in the transaction
    /// * `metadata` - Additional metadata for the confirmation
    ///
    /// # Returns
    ///
    /// A new ConfirmRelationship message body
    fn confirm_relationship(
        &self,
        agent_id: String,
        for_id: String,
        role: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> ConfirmRelationship;

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

    /// Updates a party in the transaction, creating an UpdateParty message as a response
    ///
    /// # Arguments
    ///
    /// * `party_type` - Type of party being updated (e.g., 'originator', 'beneficiary')
    /// * `party` - Updated party information
    /// * `note` - Optional note about the update
    /// * `metadata` - Additional metadata for the update
    ///
    /// # Returns
    ///
    /// A new UpdateParty message body
    fn update_party(
        &self,
        party_type: String,
        party: Participant,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> UpdateParty;

    /// Updates policies for this message, creating an UpdatePolicies message as a response
    ///
    /// # Arguments
    ///
    /// * `policies` - Vector of policies to be applied
    /// * `metadata` - Additional metadata for the update
    ///
    /// # Returns
    ///
    /// A new UpdatePolicies message body
    fn update_policies(
        &self,
        policies: Vec<Policy>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> UpdatePolicies;

    /// Adds agents to this message, creating an AddAgents message as a response
    ///
    /// # Arguments
    ///
    /// * `agents` - Vector of participants to be added
    /// * `metadata` - Additional metadata for the update
    ///
    /// # Returns
    ///
    /// A new AddAgents message body
    fn add_agents(
        &self,
        agents: Vec<Participant>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> AddAgents;

    /// Replaces an agent in this message, creating a ReplaceAgent message as a response
    ///
    /// # Arguments
    ///
    /// * `original` - DID of the original agent to be replaced
    /// * `replacement` - New participant replacing the original agent
    /// * `metadata` - Additional metadata for the update
    ///
    /// # Returns
    ///
    /// A new ReplaceAgent message body
    fn replace_agent(
        &self,
        original: String,
        replacement: Participant,
        metadata: HashMap<String, serde_json::Value>,
    ) -> ReplaceAgent;

    /// Removes an agent from this message, creating a RemoveAgent message as a response
    ///
    /// # Arguments
    ///
    /// * `agent` - DID of the agent to be removed
    /// * `metadata` - Additional metadata for the update
    ///
    /// # Returns
    ///
    /// A new RemoveAgent message body
    fn remove_agent(
        &self,
        agent: String,
        metadata: HashMap<String, serde_json::Value>,
    ) -> RemoveAgent;
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

impl TapMessageBody for ConfirmRelationship {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#confirmrelationship"
    }

    fn validate(&self) -> Result<()> {
        self.validate()
    }

    fn to_didcomm(&self) -> Result<Message> {
        // Serialize the ConfirmRelationship to a JSON value
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );

            // Change for_id to "for" in the serialized object
            if let Some(for_id) = body_obj.remove("for_id") {
                body_obj.insert("for".to_string(), for_id);
            }
        }

        let now = crate::utils::get_current_time()?;

        // Create a new Message with required fields
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: None,
            to: None,
            thid: None,
            pthid: None,
            extra_headers: std::collections::HashMap::new(),
            created_time: Some(now),
            expires_time: None,
            from_prior: None,
            attachments: None,
        };

        Ok(message)
    }

    fn from_didcomm(message: &Message) -> Result<Self> {
        let body = message
            .body
            .as_object()
            .ok_or_else(|| Error::Validation("Message body is not a JSON object".to_string()))?;

        let transfer_id = body
            .get("transfer_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing or invalid transfer_id".to_string()))?;

        let agent_id = body
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing or invalid agent_id".to_string()))?;

        let for_id = body
            .get("for")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing or invalid for".to_string()))?;

        let role = body
            .get("role")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let mut metadata = HashMap::new();
        for (k, v) in body.iter() {
            if !["transfer_id", "agent_id", "for", "role"].contains(&k.as_str()) {
                metadata.insert(k.clone(), v.clone());
            }
        }

        Ok(Self {
            transfer_id: transfer_id.to_string(),
            agent_id: agent_id.to_string(),
            for_id: for_id.to_string(),
            role,
            metadata,
        })
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
            settlement_address: None,
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

    fn confirm_relationship(
        &self,
        agent_id: String,
        for_id: String,
        role: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> ConfirmRelationship {
        ConfirmRelationship {
            transfer_id: self.id.clone(),
            agent_id,
            for_id,
            role,
            metadata,
        }
    }

    fn update_policies(
        &self,
        policies: Vec<Policy>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> UpdatePolicies {
        UpdatePolicies {
            transfer_id: self.id.clone(),
            policies,
            metadata,
        }
    }

    fn add_agents(
        &self,
        agents: Vec<Participant>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> AddAgents {
        AddAgents {
            transfer_id: self.id.clone(),
            agents,
            metadata,
        }
    }

    fn replace_agent(
        &self,
        original: String,
        replacement: Participant,
        metadata: HashMap<String, serde_json::Value>,
    ) -> ReplaceAgent {
        ReplaceAgent {
            transfer_id: self.id.clone(),
            original,
            replacement,
            metadata,
        }
    }

    fn update_party(
        &self,
        party_type: String,
        party: Participant,
        note: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> UpdateParty {
        UpdateParty {
            transfer_id: self.id.clone(),
            party_type,
            party,
            note,
            metadata,
            context: Some("https://tap.rsvp/schema/1.0".to_string()),
        }
    }

    fn remove_agent(
        &self,
        agent: String,
        metadata: HashMap<String, serde_json::Value>,
    ) -> RemoveAgent {
        RemoveAgent {
            transfer_id: self.id.clone(),
            agent,
            metadata,
        }
    }
}
