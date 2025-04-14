//! TAP message types and structures.
//!
//! This module defines the structure of all TAP message types according to the specification.

extern crate serde;
extern crate serde_json;

use didcomm::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use tap_caip::AssetId;

use crate::error::{Error, Result};
use crate::message::policy::Policy;
use crate::message::tap_message_trait::TapMessageBody;

/// Participant in a transfer (TAIP-3, TAIP-11).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(non_snake_case)]
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

    /// Legal Entity Identifier (LEI) code of the participant (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
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

impl Transfer {
    fn from_didcomm(message: &Message) -> Result<Self> {
        let body = message
            .body
            .as_object()
            .ok_or_else(|| Error::Validation("Message body is not a JSON object".to_string()))?;

        // Parse the asset - handle both string and object representations
        let asset = if let Some(asset_str) = body.get("asset").and_then(|v| v.as_str()) {
            // Handle string representation (backward compatibility)
            AssetId::from_str(asset_str)
                .map_err(|e| Error::Validation(format!("Invalid asset string: {}", e)))?
        } else if let Some(asset_obj) = body.get("asset") {
            // Handle object representation
            serde_json::from_value(asset_obj.clone())
                .map_err(|e| Error::Validation(format!("Invalid asset object: {}", e)))?
        } else {
            return Err(Error::Validation("Missing asset field".to_string()));
        };

        // Parse required fields
        let originator = body
            .get("originator")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(|| Error::Validation("Missing or invalid originator".to_string()))?;

        let amount = body
            .get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing or invalid amount".to_string()))?;

        // Parse optional fields
        let beneficiary = body
            .get("beneficiary")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let settlement_id = body
            .get("settlementId")
            .or_else(|| body.get("settlement_id"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let memo = body
            .get("memo")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        // Parse agents array (default to empty if missing)
        let agents = body
            .get("agents")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        // Parse metadata (fields not explicitly handled)
        let mut metadata = HashMap::new();
        for (k, v) in body.iter() {
            if ![
                "asset",
                "originator",
                "beneficiary",
                "amount",
                "agents",
                "settlementId",
                "settlement_id",
                "memo",
                "@type",
            ]
            .contains(&k.as_str())
            {
                metadata.insert(k.clone(), v.clone());
            }
        }

        let transfer = Self {
            asset,
            originator,
            beneficiary,
            amount: amount.to_string(),
            agents,
            settlement_id,
            memo,
            metadata,
        };

        transfer.validate()?;

        Ok(transfer)
    }
}

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

    fn to_didcomm(&self) -> Result<Message> {
        // Serialize the Transfer to a JSON value
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

/// Payment Request message body (TAIP-14)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    /// Asset identifier in CAIP-19 format (optional if currency is provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<AssetId>,

    /// ISO 4217 currency code for fiat amount (optional if asset is provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// Amount requested in the specified asset or currency
    pub amount: String,

    /// Array of CAIP-19 asset identifiers that can be used to settle a fiat currency amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_assets: Option<Vec<String>>,

    /// URI to an invoice
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice: Option<String>,

    /// ISO 8601 timestamp when the request expires
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,

    /// Party information for the merchant (beneficiary)
    pub merchant: Participant,

    /// Party information for the customer (originator) (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<Participant>,

    /// Array of agents involved in the payment request
    pub agents: Vec<Participant>,

    /// Additional metadata for the payment request
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PaymentRequest {
    /// Creates a new PaymentRequest message body
    pub fn new(amount: String, merchant: Participant, agents: Vec<Participant>) -> Self {
        Self {
            asset: None,
            currency: None,
            amount,
            supported_assets: None,
            invoice: None,
            expiry: None,
            merchant,
            customer: None,
            agents,
            metadata: HashMap::new(),
        }
    }

    /// Creates a new PaymentRequest message with asset specification
    pub fn with_asset(
        asset: AssetId,
        amount: String,
        merchant: Participant,
        agents: Vec<Participant>,
    ) -> Self {
        Self {
            asset: Some(asset),
            currency: None,
            amount,
            supported_assets: None,
            invoice: None,
            expiry: None,
            merchant,
            customer: None,
            agents,
            metadata: HashMap::new(),
        }
    }

    /// Creates a new PaymentRequest message with currency specification
    pub fn with_currency(
        currency: String,
        amount: String,
        merchant: Participant,
        agents: Vec<Participant>,
    ) -> Self {
        Self {
            asset: None,
            currency: Some(currency),
            amount,
            supported_assets: None,
            invoice: None,
            expiry: None,
            merchant,
            customer: None,
            agents,
            metadata: HashMap::new(),
        }
    }

    /// Validates the PaymentRequest message body
    pub fn validate(&self) -> Result<()> {
        // Check if at least one of asset or currency is specified
        if self.asset.is_none() && self.currency.is_none() {
            return Err(Error::Validation(
                "Either asset or currency must be specified".to_string(),
            ));
        }

        // Validate amount (must be a valid numeric string)
        if self.amount.trim().is_empty() {
            return Err(Error::Validation("Amount cannot be empty".to_string()));
        }

        // Validate that merchant is specified
        if self.merchant.id.trim().is_empty() {
            return Err(Error::Validation(
                "Merchant DID must be specified".to_string(),
            ));
        }

        // Validate expiry date format if provided
        if let Some(expiry) = &self.expiry {
            if chrono::DateTime::parse_from_rfc3339(expiry).is_err() {
                return Err(Error::Validation(
                    "Expiry must be a valid ISO 8601 timestamp".to_string(),
                ));
            }
        }

        // Validate agents field is not empty
        if self.agents.is_empty() {
            return Err(Error::Validation("Agents cannot be empty".to_string()));
        }

        Ok(())
    }
}

impl TapMessageBody for PaymentRequest {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#paymentrequest"
    }

    fn validate(&self) -> Result<()> {
        // Check if at least one of asset or currency is specified
        if self.asset.is_none() && self.currency.is_none() {
            return Err(Error::Validation(
                "Either asset or currency must be specified".to_string(),
            ));
        }

        // Validate amount (must be a valid numeric string)
        if self.amount.trim().is_empty() {
            return Err(Error::Validation("Amount cannot be empty".to_string()));
        }

        // Validate that merchant is specified
        if self.merchant.id.trim().is_empty() {
            return Err(Error::Validation(
                "Merchant DID must be specified".to_string(),
            ));
        }

        // Validate expiry date format if provided
        if let Some(expiry) = &self.expiry {
            if chrono::DateTime::parse_from_rfc3339(expiry).is_err() {
                return Err(Error::Validation(
                    "Expiry must be a valid ISO 8601 timestamp".to_string(),
                ));
            }
        }

        // Validate agents field is not empty
        if self.agents.is_empty() {
            return Err(Error::Validation("Agents cannot be empty".to_string()));
        }

        Ok(())
    }
}

/// Constraints for the connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConstraints {
    /// Array of ISO 20022 purpose codes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purposes: Option<Vec<String>>,

    /// Array of ISO 20022 category purpose codes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_purposes: Option<Vec<String>>,

    /// Transaction limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<TransactionLimits>,
}

/// Transaction limits for connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLimits {
    /// Maximum amount per transaction
    #[serde(rename = "per_transaction", skip_serializing_if = "Option::is_none")]
    pub per_transaction: Option<String>,

    /// Maximum daily amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily: Option<String>,

    /// Currency code for limits
    pub currency: String,
}

/// Connect message body (TAIP-15)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connect {
    /// Agent details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<Agent>,

    /// DID of the party the agent represents
    pub for_id: String,

    /// Transaction constraints for the connection
    pub constraints: ConnectionConstraints,

    /// Additional metadata for the connection
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Connect {
    /// Creates a new Connect message body
    pub fn new(for_id: String, constraints: ConnectionConstraints) -> Self {
        Self {
            agent: None,
            for_id,
            constraints,
            metadata: HashMap::new(),
        }
    }

    /// Creates a new Connect message body with agent details
    pub fn with_agent(agent: Agent, for_id: String, constraints: ConnectionConstraints) -> Self {
        Self {
            agent: Some(agent),
            for_id,
            constraints,
            metadata: HashMap::new(),
        }
    }

    /// Validates the Connect message body
    pub fn validate(&self) -> Result<()> {
        // Validate for_id field
        if self.for_id.trim().is_empty() {
            return Err(Error::Validation("For ID cannot be empty".to_string()));
        }

        // Validate constraints
        if let Some(limits) = &self.constraints.limits {
            if limits.currency.trim().is_empty() {
                return Err(Error::Validation(
                    "Currency code must be specified for limits".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl TapMessageBody for Connect {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#connect"
    }

    fn validate(&self) -> Result<()> {
        // Validate for_id field
        if self.for_id.trim().is_empty() {
            return Err(Error::Validation("For ID cannot be empty".to_string()));
        }

        // Validate constraints
        if let Some(limits) = &self.constraints.limits {
            if limits.currency.trim().is_empty() {
                return Err(Error::Validation(
                    "Currency code must be specified for limits".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Agent details for Connect message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// DID of the agent
    pub id: String,

    /// Name of the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Type of the agent
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,

    /// Service URL for the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_url: Option<String>,
}

/// AuthorizationRequired message body (TAIP-15)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequired {
    /// URL where the customer can review and approve the connection
    pub authorization_url: String,

    /// ISO 8601 timestamp when the authorization URL expires
    pub expires: String,

    /// Additional metadata for the authorization
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AuthorizationRequired {
    /// Creates a new AuthorizationRequired message body
    pub fn new(authorization_url: String, expires: String) -> Self {
        Self {
            authorization_url,
            expires,
            metadata: HashMap::new(),
        }
    }

    /// Validates the AuthorizationRequired message body
    pub fn validate(&self) -> Result<()> {
        // Validate authorization_url field
        if self.authorization_url.trim().is_empty() {
            return Err(Error::Validation(
                "Authorization URL cannot be empty".to_string(),
            ));
        }

        // Validate expires field format
        if chrono::DateTime::parse_from_rfc3339(&self.expires).is_err() {
            return Err(Error::Validation(
                "Expires must be a valid ISO 8601 timestamp".to_string(),
            ));
        }

        Ok(())
    }
}

impl TapMessageBody for AuthorizationRequired {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#authorizationrequired"
    }

    fn validate(&self) -> Result<()> {
        // Validate authorization_url field
        if self.authorization_url.trim().is_empty() {
            return Err(Error::Validation(
                "Authorization URL cannot be empty".to_string(),
            ));
        }

        // Validate expires field format
        if chrono::DateTime::parse_from_rfc3339(&self.expires).is_err() {
            return Err(Error::Validation(
                "Expires must be a valid ISO 8601 timestamp".to_string(),
            ));
        }

        Ok(())
    }
}

/// OutOfBand message body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutOfBand {
    /// Goal code for the out-of-band message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,

    /// Human-readable goal for the out-of-band message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,

    /// Array of attachments
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,

    /// Accept property for the out-of-band message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept: Option<Vec<String>>,

    /// Handshake protocols
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handshake_protocols: Option<Vec<String>>,

    /// Additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl OutOfBand {
    /// Creates a new OutOfBand message body
    pub fn new(
        goal_code: Option<String>,
        goal: Option<String>,
        attachments: Vec<Attachment>,
    ) -> Self {
        Self {
            goal_code,
            goal,
            attachments,
            accept: None,
            handshake_protocols: None,
            metadata: HashMap::new(),
        }
    }

    /// Validates the OutOfBand message body
    pub fn validate(&self) -> Result<()> {
        // Validate attachments if any are provided
        for attachment in &self.attachments {
            if attachment.id.trim().is_empty() {
                return Err(Error::Validation(
                    "Attachment ID cannot be empty".to_string(),
                ));
            }
            if attachment.media_type.trim().is_empty() {
                return Err(Error::Validation(
                    "Attachment media type cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl TapMessageBody for OutOfBand {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#outofband"
    }

    fn validate(&self) -> Result<()> {
        // Validate attachments if any are provided
        for attachment in &self.attachments {
            if attachment.id.trim().is_empty() {
                return Err(Error::Validation(
                    "Attachment ID cannot be empty".to_string(),
                ));
            }
            if attachment.media_type.trim().is_empty() {
                return Err(Error::Validation(
                    "Attachment media type cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// DIDComm Presentation format (using present-proof protocol)
///
/// This struct implements the standard DIDComm present-proof protocol format as defined in
/// [DIDComm Messaging Present Proof Protocol 3.0](https://github.com/decentralized-identity/waci-didcomm/tree/main/present_proof).
///
/// It is used for exchanging verifiable presentations between parties in a DIDComm conversation.
/// The presentation may contain identity credentials, proof of control, or other verifiable claims.
///
/// # Compatibility Notes
///
/// - This implementation is fully compatible with the standard DIDComm present-proof protocol.
/// - The message type used is `https://didcomm.org/present-proof/3.0/presentation`.
/// - Thread ID (`thid`) is required for proper message correlation.
/// - At least one attachment containing a verifiable presentation is required.
/// - Attachment data must include either base64 or JSON format data.
/// - Verifiable presentations in JSON format must include `@context` and `type` fields.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use tap_msg::message::{Attachment, AttachmentData, DIDCommPresentation};
/// use serde_json::json;
///
/// // Create a presentation with a verifiable credential
/// let presentation = DIDCommPresentation {
///     thid: Some("123e4567-e89b-12d3-a456-426614174000".to_string()),
///     comment: Some("Proof of identity".to_string()),
///     goal_code: Some("kyc.individual".to_string()),
///     attachments: vec![
///         Attachment {
///             id: "credential-1".to_string(),
///             media_type: "application/json".to_string(),
///             data: Some(AttachmentData {
///                 base64: None,
///                 json: Some(json!({
///                     "@context": ["https://www.w3.org/2018/credentials/v1"],
///                     "type": ["VerifiablePresentation"],
///                     "verifiableCredential": [{
///                         "@context": ["https://www.w3.org/2018/credentials/v1"],
///                         "type": ["VerifiableCredential"],
///                         "issuer": "did:web:issuer.example",
///                         "issuanceDate": "2022-01-01T19:23:24Z",
///                         "credentialSubject": {
///                             "id": "did:example:ebfeb1f712ebc6f1c276e12ec21"
///                         }
///                     }]
///                 })),
///             }),
///         }
///     ],
///     metadata: HashMap::new(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDCommPresentation {
    /// Reference to a previous message in the thread
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thid: Option<String>,

    /// Optional comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Goal code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<String>,

    /// Attachments containing the verifiable presentations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,

    /// Additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl DIDCommPresentation {
    /// Creates a new DIDComm Presentation
    pub fn new(thid: Option<String>, attachments: Vec<Attachment>) -> Self {
        Self {
            thid,
            comment: None,
            goal_code: None,
            attachments,
            metadata: HashMap::new(),
        }
    }

    /// Validate the DIDComm Presentation
    ///
    /// This method validates a DIDComm presentation according to the standard protocol requirements.
    /// For a presentation to be valid, it must satisfy the following criteria:
    ///
    /// - Must have a thread ID (`thid`) for message correlation
    /// - Must include at least one attachment
    /// - Each attachment must have a non-empty ID and media type
    /// - Each attachment must include data in either base64 or JSON format
    /// - JSON data must include `@context` and `type` fields
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the presentation is valid
    /// - `Err(Error::Validation)` with a descriptive message if validation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use tap_msg::message::DIDCommPresentation;
    /// use tap_msg::Result;
    ///
    /// fn check_presentation(presentation: &DIDCommPresentation) -> Result<()> {
    ///     // Validate the presentation
    ///     presentation.validate()?;
    ///     
    ///     // If we get here, the presentation is valid
    ///     Ok(())
    /// }
    /// ```
    pub fn validate(&self) -> Result<()> {
        // Validate thread ID (thid) - required according to test vectors
        if self.thid.is_none() {
            return Err(Error::Validation(
                "Thread ID (thid) is required for presentation message".to_string(),
            ));
        }

        // Validate attachments if any are provided
        if self.attachments.is_empty() {
            return Err(Error::Validation(
                "Presentation must include at least one attachment".to_string(),
            ));
        }

        for attachment in &self.attachments {
            // Validate attachment ID
            if attachment.id.trim().is_empty() {
                return Err(Error::Validation(
                    "Attachment ID cannot be empty".to_string(),
                ));
            }

            // Validate media type
            if attachment.media_type.trim().is_empty() {
                return Err(Error::Validation(
                    "Attachment media type cannot be empty".to_string(),
                ));
            }

            // Check for attachment data
            if attachment.data.is_none() {
                return Err(Error::Validation(
                    "Attachment must include data".to_string(),
                ));
            }

            // Check attachment data content
            if let Some(data) = &attachment.data {
                if data.base64.is_none() && data.json.is_none() {
                    return Err(Error::Validation(
                        "Attachment data must include either base64 or json".to_string(),
                    ));
                }

                // If JSON data is present, validate required fields in presentation data
                if let Some(json_data) = &data.json {
                    // Check for @context field in JSON data - required by test vectors
                    if json_data.get("@context").is_none() {
                        return Err(Error::Validation(
                            "Attachment JSON data must include @context field".to_string(),
                        ));
                    }

                    // Check for type field in JSON data - required by test vectors
                    if json_data.get("type").is_none() {
                        return Err(Error::Validation(
                            "Attachment JSON data must include type field".to_string(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

/// Implementation of TapMessageBody for DIDCommPresentation
///
/// This implementation ensures that DIDCommPresentation can be converted to and from
/// didcomm::Message objects, allowing seamless integration with the DIDComm
/// messaging protocol.
///
/// # Details
///
/// - **Message Type**: Uses `https://didcomm.org/present-proof/3.0/presentation` as specified by the standard protocol
/// - **Conversion to DIDComm**: Converts attachments to didcomm::Attachment format with appropriate data representation
/// - **Conversion from DIDComm**: Extracts presentation data from DIDComm message, handling both Base64 and JSON formats
///
/// This implementation follows the [DIDComm Messaging Specification](https://identity.foundation/didcomm-messaging/spec/)
/// and the [Present Proof Protocol 3.0](https://github.com/decentralized-identity/waci-didcomm/tree/main/present_proof).
impl TapMessageBody for DIDCommPresentation {
    fn message_type() -> &'static str {
        "https://didcomm.org/present-proof/3.0/presentation"
    }

    fn validate(&self) -> Result<()> {
        self.validate()
    }

    fn from_didcomm(message: &Message) -> Result<Self> {
        // Check if this is the correct message type
        if message.type_ != Self::message_type() {
            return Err(Error::InvalidMessageType(format!(
                "Expected message type {}, but found {}",
                Self::message_type(),
                message.type_
            )));
        }

        // Extract body and attachments
        let body = message
            .body
            .as_object()
            .ok_or_else(|| Error::Validation("Message body is not a JSON object".to_string()))?;

        // Extract the thread id
        let thid = message.thid.clone();

        // Extract comment if present
        let comment = body
            .get("comment")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        // Extract goal_code if present
        let goal_code = body
            .get("goal_code")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        // Extract attachments
        let attachments = if let Some(didcomm_attachments) = &message.attachments {
            didcomm_attachments
                .iter()
                .filter_map(|a| {
                    // Both id and media_type must be present
                    if a.id.is_none() || a.media_type.is_none() {
                        return None;
                    }

                    // Convert the didcomm::AttachmentData to our AttachmentData if present
                    let data = match &a.data {
                        didcomm::AttachmentData::Base64 { value } => Some(AttachmentData {
                            base64: Some(value.base64.clone()),
                            json: None,
                        }),
                        didcomm::AttachmentData::Json { value } => Some(AttachmentData {
                            base64: None,
                            json: Some(value.json.clone()),
                        }),
                        didcomm::AttachmentData::Links { .. } => {
                            // We don't currently support links in our AttachmentData
                            None
                        }
                    };

                    // Create our Attachment
                    Some(Attachment {
                        id: a.id.as_ref().unwrap().clone(),
                        media_type: a.media_type.as_ref().unwrap().clone(),
                        data,
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        // Parse metadata (excluding known fields)
        let mut metadata = HashMap::new();
        for (k, v) in body.iter() {
            if !["comment", "goal_code"].contains(&k.as_str()) {
                metadata.insert(k.clone(), v.clone());
            }
        }

        let presentation = Self {
            thid,
            comment,
            goal_code,
            attachments,
            metadata,
        };

        presentation.validate()?;

        Ok(presentation)
    }

    fn to_didcomm(&self) -> Result<Message> {
        // Create message body
        let mut body = serde_json::Map::new();

        // Add optional fields if present
        if let Some(comment) = &self.comment {
            body.insert(
                "comment".to_string(),
                serde_json::Value::String(comment.clone()),
            );
        }

        if let Some(goal_code) = &self.goal_code {
            body.insert(
                "goal_code".to_string(),
                serde_json::Value::String(goal_code.clone()),
            );
        }

        // Add metadata fields
        for (key, value) in &self.metadata {
            body.insert(key.clone(), value.clone());
        }

        // Convert our attachments to didcomm::Attachment
        let didcomm_attachments = if !self.attachments.is_empty() {
            let attachments: Vec<didcomm::Attachment> = self
                .attachments
                .iter()
                .filter_map(|a| {
                    // Convert our AttachmentData to didcomm's if present
                    let didcomm_data = match &a.data {
                        Some(data) => {
                            if let Some(base64_data) = &data.base64 {
                                // Create Base64 attachment data
                                didcomm::AttachmentData::Base64 {
                                    value: didcomm::Base64AttachmentData {
                                        base64: base64_data.clone(),
                                        jws: None,
                                    },
                                }
                            } else if let Some(json_data) = &data.json {
                                // Create JSON attachment data
                                didcomm::AttachmentData::Json {
                                    value: didcomm::JsonAttachmentData {
                                        json: json_data.clone(),
                                        jws: None,
                                    },
                                }
                            } else {
                                // If neither base64 nor json is present, skip this attachment
                                return None;
                            }
                        }
                        None => {
                            // If no data is present, skip this attachment
                            return None;
                        }
                    };

                    // Create the didcomm Attachment
                    Some(didcomm::Attachment {
                        id: Some(a.id.clone()),
                        media_type: Some(a.media_type.clone()),
                        data: didcomm_data,
                        filename: None,
                        format: None,
                        byte_count: None,
                        lastmod_time: None,
                        description: None,
                    })
                })
                .collect();

            if attachments.is_empty() {
                None
            } else {
                Some(attachments)
            }
        } else {
            None
        };

        // Create the didcomm message
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: serde_json::Value::Object(body),
            from: None,
            to: None,
            thid: self.thid.clone(),
            pthid: None,
            extra_headers: HashMap::new(),
            created_time: Some(crate::utils::get_current_time()?),
            expires_time: None,
            from_prior: None,
            attachments: didcomm_attachments,
        };

        Ok(message)
    }
}
