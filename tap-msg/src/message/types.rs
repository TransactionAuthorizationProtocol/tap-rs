//! TAP message types and structures.
//!
//! This module defines the structure of all TAP message types according to the specification.

extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use tap_caip::AssetId;

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::impl_tap_message;
use crate::message::policy::Policy;
use crate::message::tap_message_trait::{Connectable, TapMessageBody};
use crate::message::RequireProofOfControl;
use chrono::Utc;

/// Participant in a transfer (TAIP-3, TAIP-11).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttachmentData {
    /// Base64-encoded data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base64: Option<String>,

    /// JSON data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<serde_json::Value>,
}

/// Attachment for a TAP message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    /// Memo for the transfer (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Settlement identifier (optional).
    #[serde(rename = "settlementId", skip_serializing_if = "Option::is_none")]
    pub settlement_id: Option<String>,

    /// Transaction identifier (not stored in the struct but accessible via the TapMessage trait).
    #[serde(skip)]
    pub transaction_id: String,

    /// Additional metadata for the transfer.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Transfer {
    /// Create a new Transfer
    ///
    /// # Example
    /// ```
    /// use tap_msg::message::Transfer;
    /// use tap_caip::{AssetId, ChainId};
    /// use tap_msg::message::Participant;
    /// use std::collections::HashMap;
    ///
    /// // Create chain ID and asset ID
    /// let chain_id = ChainId::new("eip155", "1").unwrap();
    /// let asset = AssetId::new(chain_id, "erc20", "0x6b175474e89094c44da98b954eedeac495271d0f").unwrap();
    ///
    /// // Create participant
    /// let originator = Participant {
    ///     id: "did:example:alice".to_string(),
    ///     role: Some("originator".to_string()),
    ///     policies: None,
    ///     leiCode: None,
    /// };
    ///
    /// // Create a transfer with required fields
    /// let transfer = Transfer::builder()
    ///     .asset(asset)
    ///     .originator(originator)
    ///     .amount("100".to_string())
    ///     .build();
    /// ```
    pub fn builder() -> TransferBuilder {
        TransferBuilder::default()
    }

    /// Generates a unique message ID for authorization, rejection, or settlement
    pub fn message_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Validate the Transfer
    pub fn validate(&self) -> Result<()> {
        // CAIP-19 asset ID is validated by the AssetId type
        // Validate asset
        if self.asset.namespace().is_empty() || self.asset.reference().is_empty() {
            return Err(Error::Validation("Asset ID is invalid".to_string()));
        }

        // Validate originator
        if self.originator.id.is_empty() {
            return Err(Error::Validation("Originator ID is required".to_string()));
        }

        // Validate amount
        if self.amount.is_empty() {
            return Err(Error::Validation("Amount is required".to_string()));
        }

        // Validate amount is a positive number
        match self.amount.parse::<f64>() {
            Ok(amount) if amount <= 0.0 => {
                return Err(Error::Validation("Amount must be positive".to_string()));
            }
            Err(_) => {
                return Err(Error::Validation(
                    "Amount must be a valid number".to_string(),
                ));
            }
            _ => {}
        }

        // Validate agents (if any are defined)
        for agent in &self.agents {
            if agent.id.is_empty() {
                return Err(Error::Validation("Agent ID cannot be empty".to_string()));
            }
        }

        Ok(())
    }
}

/// Builder for creating Transfer objects in a more idiomatic way
#[derive(Default)]
pub struct TransferBuilder {
    asset: Option<AssetId>,
    originator: Option<Participant>,
    amount: Option<String>,
    beneficiary: Option<Participant>,
    settlement_id: Option<String>,
    memo: Option<String>,
    transaction_id: Option<String>,
    agents: Vec<Participant>,
    metadata: HashMap<String, serde_json::Value>,
}

impl TransferBuilder {
    /// Set the asset for this transfer
    pub fn asset(mut self, asset: AssetId) -> Self {
        self.asset = Some(asset);
        self
    }

    /// Set the originator for this transfer
    pub fn originator(mut self, originator: Participant) -> Self {
        self.originator = Some(originator);
        self
    }

    /// Set the amount for this transfer
    pub fn amount(mut self, amount: String) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the beneficiary for this transfer
    pub fn beneficiary(mut self, beneficiary: Participant) -> Self {
        self.beneficiary = Some(beneficiary);
        self
    }

    /// Set the settlement ID for this transfer
    pub fn settlement_id(mut self, settlement_id: String) -> Self {
        self.settlement_id = Some(settlement_id);
        self
    }

    /// Set the memo for this transfer
    pub fn memo(mut self, memo: String) -> Self {
        self.memo = Some(memo);
        self
    }

    /// Set the transaction ID for this transfer
    pub fn transaction_id(mut self, transaction_id: String) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }

    /// Add an agent to this transfer
    pub fn add_agent(mut self, agent: Participant) -> Self {
        self.agents.push(agent);
        self
    }

    /// Set all agents for this transfer
    pub fn agents(mut self, agents: Vec<Participant>) -> Self {
        self.agents = agents;
        self
    }

    /// Add a metadata field
    pub fn add_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set all metadata for this transfer
    pub fn metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Build the Transfer object
    ///
    /// # Panics
    ///
    /// Panics if required fields (asset, originator, amount) are not set
    pub fn build(self) -> Transfer {
        Transfer {
            asset: self.asset.expect("Asset is required"),
            originator: self.originator.expect("Originator is required"),
            amount: self.amount.expect("Amount is required"),
            beneficiary: self.beneficiary,
            settlement_id: self.settlement_id,
            memo: self.memo,
            transaction_id: self
                .transaction_id
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            agents: self.agents,
            metadata: self.metadata,
        }
    }

    /// Try to build the Transfer object, returning an error if required fields are missing
    pub fn try_build(self) -> Result<Transfer> {
        let asset = self
            .asset
            .ok_or_else(|| Error::Validation("Asset is required".to_string()))?;
        let originator = self
            .originator
            .ok_or_else(|| Error::Validation("Originator is required".to_string()))?;
        let amount = self
            .amount
            .ok_or_else(|| Error::Validation("Amount is required".to_string()))?;

        let transfer = Transfer {
            transaction_id: self
                .transaction_id
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            asset,
            originator,
            amount,
            beneficiary: self.beneficiary,
            settlement_id: self.settlement_id,
            memo: self.memo,
            agents: self.agents,
            metadata: self.metadata,
        };

        // Validate the created transfer
        transfer.validate()?;

        Ok(transfer)
    }
}

impl Connectable for Transfer {
    fn with_connection(&mut self, connect_id: &str) -> &mut Self {
        // Store the connect_id in metadata
        self.metadata.insert(
            "connect_id".to_string(),
            serde_json::Value::String(connect_id.to_string()),
        );
        self
    }

    fn has_connection(&self) -> bool {
        self.metadata.contains_key("connect_id")
    }

    fn connection_id(&self) -> Option<&str> {
        self.metadata.get("connect_id").and_then(|v| v.as_str())
    }
}

impl TapMessageBody for Transfer {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#transfer"
    }

    fn validate(&self) -> Result<()> {
        // Validate asset
        if self.asset.namespace().is_empty() || self.asset.reference().is_empty() {
            return Err(Error::Validation("Asset ID is invalid".to_string()));
        }

        // Validate originator
        if self.originator.id.is_empty() {
            return Err(Error::Validation("Originator ID is required".to_string()));
        }

        // Validate amount
        if self.amount.is_empty() {
            return Err(Error::Validation("Amount is required".to_string()));
        }

        // Validate amount is a positive number
        match self.amount.parse::<f64>() {
            Ok(amount) if amount <= 0.0 => {
                return Err(Error::Validation("Amount must be positive".to_string()));
            }
            Err(_) => {
                return Err(Error::Validation(
                    "Amount must be a valid number".to_string(),
                ));
            }
            _ => {}
        }

        Ok(())
    }

    fn to_didcomm(&self, from_did: &str) -> Result<PlainMessage> {
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

        // Extract agent DIDs directly from the message
        let mut agent_dids = Vec::new();

        // Add originator DID
        agent_dids.push(self.originator.id.clone());

        // Add beneficiary DID if present
        if let Some(beneficiary) = &self.beneficiary {
            agent_dids.push(beneficiary.id.clone());
        }

        // Add DIDs from agents array
        for agent in &self.agents {
            agent_dids.push(agent.id.clone());
        }

        // Remove duplicates
        agent_dids.sort();
        agent_dids.dedup();

        // If from_did is provided, remove it from the recipients list to avoid sending to self
        if let Some(from) = from_did {
            agent_dids.retain(|did| did != from);
        }

        let now = Utc::now().timestamp() as u64;

        // Get the connection ID if this message is connected to a previous message
        let pthid = self
            .connection_id()
            .map(|connect_id| connect_id.to_string());

        // The from field is required in our PlainMessage, so ensure we have a valid value
        let from = from_did.map_or_else(String::new, |s| s.to_string());

        // Create a new Message with required fields
        let message = PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from,
            to: agent_dids,
            thid: None,
            pthid,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
        };

        Ok(message)
    }

    #[allow(dead_code)] // Used in tests
    fn to_didcomm_with_route<'a, I>(&self, from: Option<&str>, to: I) -> Result<Message>
    where
        I: IntoIterator<Item = &'a str>,
    {
        // First create a message with the sender, automatically extracting agent DIDs
        let mut message = self.to_didcomm(from)?;

        // Override with explicitly provided recipients if any
        let to_vec: Vec<String> = to.into_iter().map(String::from).collect();
        if !to_vec.is_empty() {
            message.to = Some(to_vec);
        }

        // Set the parent thread ID if this message is connected to a previous message
        if let Some(connect_id) = self.connection_id() {
            message.pthid = Some(connect_id.to_string());
        }

        Ok(message)
    }
}

impl_tap_message!(Transfer);

/// Request Presentation message body (TAIP-10).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPresentation {
    /// Transfer ID that this request is related to.
    pub transaction_id: String,

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
    pub transaction_id: Option<String>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Authorize message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorize {
    /// ID of the transaction being authorized.
    pub transaction_id: String,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Reject message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reject {
    /// ID of the transaction being rejected.
    pub transaction_id: String,

    /// Reason for rejection.
    pub reason: String,
}

/// Settle message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settle {
    /// ID of the transaction being settled.
    pub transaction_id: String,

    /// Settlement ID (CAIP-220 identifier of the underlying settlement transaction).
    pub settlement_id: String,

    /// Optional amount settled. If specified, must be less than or equal to the original amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
}

/// Cancel message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cancel {
    /// ID of the transfer being cancelled.
    pub transaction_id: String,

    /// Optional reason for cancellation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl TapMessageBody for Cancel {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#cancel"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Cancel message must have a transaction_id".into(),
            ));
        }
        Ok(())
    }

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        let msg_id = uuid::Uuid::new_v4().to_string();

        let body_json = serde_json::to_value(self)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize Cancel: {}", e)))?;

        Ok(Message {
            id: msg_id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: Some(self.transaction_id.clone()),
            pthid: None,
            extra_headers: HashMap::new(),
            attachments: None,
            created_time: None,
            expires_time: None,
            from_prior: None,
        })
    }

    fn from_didcomm(msg: &Message) -> Result<Self> {
        if msg.type_ != Self::message_type() {
            return Err(Error::InvalidMessageType(format!(
                "Expected message type {}, got {}",
                Self::message_type(),
                msg.type_
            )));
        }

        serde_json::from_value(msg.body.clone())
            .map_err(|e| Error::SerializationError(format!("Failed to deserialize Cancel: {}", e)))
    }
}

impl_tap_message!(Cancel);

/// Revert message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revert {
    /// ID of the transfer being reverted.
    pub transaction_id: String,

    /// Settlement address in CAIP-10 format to return the funds to.
    pub settlement_address: String,

    /// Reason for the reversal request.
    pub reason: String,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Add agents message body (TAIP-5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAgents {
    /// ID of the transaction to add agents to.
    #[serde(rename = "transfer_id")]
    pub transaction_id: String,

    /// Agents to add.
    pub agents: Vec<Participant>,
}

/// Replace agent message body (TAIP-5).
///
/// This message type allows replacing an agent with another agent in a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceAgent {
    /// ID of the transaction to replace agent in.
    #[serde(rename = "transfer_id")]
    pub transaction_id: String,

    /// DID of the original agent to replace.
    pub original: String,

    /// Replacement agent.
    pub replacement: Participant,
}

/// Remove agent message body (TAIP-5).
///
/// This message type allows removing an agent from a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAgent {
    /// ID of the transaction to remove agent from.
    #[serde(rename = "transfer_id")]
    pub transaction_id: String,

    /// DID of the agent to remove.
    pub agent: String,
}

/// ConfirmRelationship message body (TAIP-9).
///
/// This message type allows confirming a relationship between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmRelationship {
    /// ID of the transaction related to this message.
    #[serde(rename = "transfer_id")]
    pub transaction_id: String,

    /// DID of the agent whose relationship is being confirmed.
    pub agent_id: String,

    /// DID of the entity that the agent acts on behalf of.
    #[serde(rename = "for")]
    pub for_id: String,

    /// Role of the agent in the transaction (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

impl ConfirmRelationship {
    /// Creates a new ConfirmRelationship message body.
    pub fn new(transaction_id: &str, agent_id: &str, for_id: &str, role: Option<String>) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            agent_id: agent_id.to_string(),
            for_id: for_id.to_string(),
            role,
        }
    }

    /// Validates the ConfirmRelationship message body.
    pub fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
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

impl TapMessageBody for ConfirmRelationship {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#confirmrelationship"
    }

    fn validate(&self) -> Result<()> {
        self.validate()
    }

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // 1. Serialize self to JSON value
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // 2. Add/ensure '@type' field
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
            // Note: serde handles #[serde(rename = "for")] automatically during serialization
        }

        // 3. Generate ID and timestamp
        let id = uuid::Uuid::new_v4().to_string(); // Use new_v4 as per workspace UUID settings
        let created_time = Utc::now().timestamp_millis() as u64;

        // 4. Explicitly set the recipient using agent_id
        let to = Some(vec![self.agent_id.clone()]);

        // 5. Create the Message struct
        let message = Message {
            id,
            typ: "application/didcomm-plain+json".to_string(), // Standard type
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to, // Use the explicitly set 'to' field
            thid: Some(self.transaction_id.clone()),
            pthid: None, // Parent Thread ID usually set later
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
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

        let confirm_relationship = Self {
            transaction_id: transfer_id.to_string(),
            agent_id: agent_id.to_string(),
            for_id: for_id.to_string(),
            role,
        };

        confirm_relationship.validate()?;

        Ok(confirm_relationship)
    }
}

impl_tap_message!(ConfirmRelationship);

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
    pub transaction_id: String,

    /// Type of party being updated (e.g., 'originator', 'beneficiary').
    #[serde(rename = "partyType")]
    pub party_type: String,

    /// Updated party information.
    pub party: Participant,

    /// Optional note regarding the update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Optional context for the update.
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl UpdateParty {
    /// Creates a new UpdateParty message body.
    pub fn new(transaction_id: &str, party_type: &str, party: Participant) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            party_type: party_type.to_string(),
            party,
            note: None,
            context: None,
        }
    }

    /// Validates the UpdateParty message body.
    pub fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "transaction_id cannot be empty".to_string(),
            ));
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

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
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

        let now = Utc::now().timestamp() as u64;

        // Create a new Message with required fields
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: Some(self.transaction_id.clone()),
            pthid: None,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
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

        let transaction_id = body
            .get("transaction_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Validation("Missing or invalid transaction_id".to_string()))?;

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

        let context = body
            .get("@context")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let update_party = Self {
            transaction_id: transaction_id.to_string(),
            party_type: party_type.to_string(),
            party,
            note,
            context,
        };

        update_party.validate()?;

        Ok(update_party)
    }
}

impl_tap_message!(UpdateParty);

/// UpdatePolicies message body (TAIP-7).
///
/// This message type allows agents to update their policies for a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicies {
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    pub policies: Vec<Policy>,
}

impl UpdatePolicies {
    pub fn new(transaction_id: &str, policies: Vec<Policy>) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            policies,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "UpdatePolicies must have a transaction_id".to_string(),
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

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
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

        let now = Utc::now().timestamp() as u64;

        // Create a new Message with required fields
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: Some(self.transaction_id.clone()),
            pthid: None,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
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

impl_tap_message!(UpdatePolicies);

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
    /// * `note` - Optional note
    ///
    /// # Returns
    ///
    /// A new Authorize message body
    fn authorize(&self, note: Option<String>) -> Authorize;

    /// Confirms a relationship between agents, creating a ConfirmRelationship message as a response
    ///
    /// # Arguments
    ///
    /// * `transfer_id` - ID of the transfer related to this message
    /// * `agent_id` - DID of the agent whose relationship is being confirmed
    /// * `for_id` - DID of the entity that the agent acts on behalf of
    /// * `role` - Optional role of the agent in the transaction
    ///
    /// # Returns
    ///
    /// A new ConfirmRelationship message body
    fn confirm_relationship(
        &self,
        transaction_id: String,
        agent_id: String,
        for_id: String,
        role: Option<String>,
    ) -> ConfirmRelationship;

    /// Rejects this message, creating a Reject message as a response
    ///
    /// # Arguments
    ///
    /// * `code` - Rejection code
    /// * `description` - Description of rejection reason
    ///
    /// # Returns
    ///
    /// A new Reject message body
    fn reject(&self, code: String, description: String) -> Reject;

    /// Settles this message, creating a Settle message as a response
    ///
    /// # Arguments
    ///
    /// * `settlement_id` - Settlement ID (CAIP-220 identifier)
    /// * `amount` - Optional amount settled
    ///
    /// # Returns
    ///
    /// A new Settle message body
    fn settle(&self, settlement_id: String, amount: Option<String>) -> Settle;

    /// Updates a party in the transaction, creating an UpdateParty message as a response
    ///
    /// # Arguments
    ///
    /// * `transfer_id` - ID of the transaction this update relates to
    /// * `party_type` - Type of party being updated (e.g., 'originator', 'beneficiary')
    /// * `party` - Updated party information
    /// * `note` - Optional note about the update
    ///
    /// # Returns
    ///
    /// A new UpdateParty message body
    fn update_party(
        &self,
        transaction_id: String,
        party_type: String,
        party: Participant,
        note: Option<String>,
    ) -> UpdateParty;

    /// Updates policies for this message, creating an UpdatePolicies message as a response
    ///
    /// # Arguments
    ///
    /// * `transfer_id` - ID of the transfer being updated
    /// * `policies` - Vector of policies to be applied
    ///
    /// # Returns
    ///
    /// A new UpdatePolicies message body
    fn update_policies(&self, transaction_id: String, policies: Vec<Policy>) -> UpdatePolicies;

    /// Adds agents to this message, creating an AddAgents message as a response
    ///
    /// # Arguments
    ///
    /// * `transfer_id` - ID of the transfer to add agents to
    /// * `agents` - Vector of participants to be added
    ///
    /// # Returns
    ///
    /// A new AddAgents message body
    fn add_agents(&self, transaction_id: String, agents: Vec<Participant>) -> AddAgents;

    /// Replaces an agent in this message, creating a ReplaceAgent message as a response
    ///
    /// # Arguments
    ///
    /// * `transfer_id` - ID of the transfer to replace agent in
    /// * `original` - DID of the original agent to be replaced
    /// * `replacement` - New participant replacing the original agent
    ///
    /// # Returns
    ///
    /// A new ReplaceAgent message body
    fn replace_agent(
        &self,
        transaction_id: String,
        original: String,
        replacement: Participant,
    ) -> ReplaceAgent;

    /// Removes an agent from this message, creating a RemoveAgent message as a response
    ///
    /// # Arguments
    ///
    /// * `transfer_id` - ID of the transfer to remove agent from
    /// * `agent` - DID of the agent to remove
    ///
    /// # Returns
    ///
    /// A new RemoveAgent message body
    fn remove_agent(&self, transaction_id: String, agent: String) -> RemoveAgent;
}

// Implementation of message type conversion for message body types

impl TapMessageBody for Authorize {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#authorize"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transaction ID is required in Authorize".to_string(),
            ));
        }

        Ok(())
    }

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // Create the message
        let message = Message {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: None,
            pthid: None,
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
    }
}

impl_tap_message!(Authorize);

impl TapMessageBody for Reject {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#reject"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transaction ID is required in Reject".to_string(),
            ));
        }

        if self.reason.is_empty() {
            return Err(Error::Validation(
                "Reason is required in Reject".to_string(),
            ));
        }

        Ok(())
    }

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // Create the message
        let message = Message {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: None,
            pthid: None,
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
    }
}

impl_tap_message!(Reject);

impl TapMessageBody for Settle {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#settle"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transaction ID is required in Settle".to_string(),
            ));
        }

        if self.settlement_id.is_empty() {
            return Err(Error::Validation(
                "Settlement ID is required in Settle".to_string(),
            ));
        }

        if let Some(amount) = &self.amount {
            if amount.is_empty() {
                return Err(Error::Validation(
                    "Amount must be a valid number".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // Create the message
        let message = Message {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: Some(self.transaction_id.clone()),
            pthid: None,
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
    }
}

impl_tap_message!(Settle);

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

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // Create the message
        let message = Message {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: None,
            pthid: None,
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
    }
}

impl TapMessageBody for AddAgents {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#addagents"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
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

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // Create the message
        let message = Message {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: Some(self.transaction_id.clone()),
            pthid: None,
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
    }
}

impl_tap_message!(AddAgents);

impl TapMessageBody for ReplaceAgent {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#replaceagent"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
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

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // Create the message
        let message = Message {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: Some(self.transaction_id.clone()),
            pthid: None,
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
    }
}

impl_tap_message!(ReplaceAgent);

impl TapMessageBody for RemoveAgent {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#removeagent"
    }

    fn validate(&self) -> Result<()> {
        if self.transaction_id.is_empty() {
            return Err(Error::Validation(
                "Transfer ID is required in RemoveAgent".to_string(),
            ));
        }

        if self.agent.is_empty() {
            return Err(Error::Validation("Agent DID cannot be empty".to_string()));
        }

        Ok(())
    }

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // Create the message
        let message = Message {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: Some(self.transaction_id.clone()),
            pthid: None,
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
    }
}

impl_tap_message!(RemoveAgent);

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

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // Create the message
        let message = Message {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: None,
            pthid: None,
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
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
    #[serde(rename = "supportedAssets", skip_serializing_if = "Option::is_none")]
    pub supported_assets: Option<Vec<String>>,

    /// Structured invoice information according to TAIP-16
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice: Option<crate::message::invoice::Invoice>,

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

        // Validate amount is a positive number
        match self.amount.parse::<f64>() {
            Ok(amount) if amount <= 0.0 => {
                return Err(Error::Validation("Amount must be positive".to_string()));
            }
            Err(_) => {
                return Err(Error::Validation(
                    "Amount must be a valid number".to_string(),
                ));
            }
            _ => {}
        }

        // Validate merchant
        if self.merchant.id.trim().is_empty() {
            return Err(Error::Validation("Merchant ID is required".to_string()));
        }

        // Validate expiry date format if provided
        if let Some(expiry) = &self.expiry {
            if Utc::now().timestamp()
                > (Utc::now() + chrono::Duration::seconds(expiry.parse::<i64>().unwrap()))
                    .timestamp()
            {
                return Err(Error::Validation(
                    "Expiry must be a valid ISO 8601 timestamp".to_string(),
                ));
            }
        }

        // Validate agents field is not empty
        if self.agents.is_empty() {
            return Err(Error::Validation("Agents cannot be empty".to_string()));
        }

        // Validate invoice if present
        if let Some(invoice) = &self.invoice {
            // Validate the invoice structure
            invoice.validate()?;

            // Validate that invoice total matches payment amount
            if let Ok(amount_f64) = self.amount.parse::<f64>() {
                let difference = (amount_f64 - invoice.total).abs();
                if difference > 0.01 {
                    // Allow a small tolerance for floating point calculations
                    return Err(Error::Validation(format!(
                        "Invoice total ({}) does not match payment amount ({})",
                        invoice.total, amount_f64
                    )));
                }
            }

            // Validate currency consistency if both are present
            if let Some(currency) = &self.currency {
                if currency.to_uppercase() != invoice.currency_code.to_uppercase() {
                    return Err(Error::Validation(format!(
                        "Payment request currency ({}) does not match invoice currency ({})",
                        currency, invoice.currency_code
                    )));
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)] // Used in tests
    fn to_didcomm_with_route<'a, I>(&self, from: Option<&str>, to: I) -> Result<Message>
    where
        I: IntoIterator<Item = &'a str>,
    {
        // First create a message with the sender, automatically extracting agent DIDs
        let mut message = self.to_didcomm(from)?;

        // Override with explicitly provided recipients if any
        let to_vec: Vec<String> = to.into_iter().map(String::from).collect();
        if !to_vec.is_empty() {
            message.to = Some(to_vec);
        }

        // Set the parent thread ID if this message is connected to a previous message
        if let Some(connect_id) = self.connection_id() {
            message.pthid = Some(connect_id.to_string());
        }

        Ok(message)
    }
}

impl Connectable for PaymentRequest {
    fn with_connection(&mut self, connect_id: &str) -> &mut Self {
        // Store the connect_id in metadata
        self.metadata.insert(
            "connect_id".to_string(),
            serde_json::Value::String(connect_id.to_string()),
        );
        self
    }

    fn has_connection(&self) -> bool {
        self.metadata.contains_key("connect_id")
    }

    fn connection_id(&self) -> Option<&str> {
        self.metadata.get("connect_id").and_then(|v| v.as_str())
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

        // Validate amount is a positive number
        match self.amount.parse::<f64>() {
            Ok(amount) if amount <= 0.0 => {
                return Err(Error::Validation("Amount must be positive".to_string()));
            }
            Err(_) => {
                return Err(Error::Validation(
                    "Amount must be a valid number".to_string(),
                ));
            }
            _ => {}
        }

        // Validate merchant
        if self.merchant.id.trim().is_empty() {
            return Err(Error::Validation("Merchant ID is required".to_string()));
        }

        // Validate expiry date format if provided
        if let Some(expiry) = &self.expiry {
            if Utc::now().timestamp()
                > (Utc::now() + chrono::Duration::seconds(expiry.parse::<i64>().unwrap()))
                    .timestamp()
            {
                return Err(Error::Validation(
                    "Expiry must be a valid ISO 8601 timestamp".to_string(),
                ));
            }
        }

        // Validate agents field is not empty
        if self.agents.is_empty() {
            return Err(Error::Validation("Agents cannot be empty".to_string()));
        }

        // Validate invoice if present
        if let Some(invoice) = &self.invoice {
            // Validate the invoice structure
            invoice.validate()?;

            // Validate that invoice total matches payment amount
            if let Ok(amount_f64) = self.amount.parse::<f64>() {
                let difference = (amount_f64 - invoice.total).abs();
                if difference > 0.01 {
                    // Allow a small tolerance for floating point calculations
                    return Err(Error::Validation(format!(
                        "Invoice total ({}) does not match payment amount ({})",
                        invoice.total, amount_f64
                    )));
                }
            }

            // Validate currency consistency if both are present
            if let Some(currency) = &self.currency {
                if currency.to_uppercase() != invoice.currency_code.to_uppercase() {
                    return Err(Error::Validation(format!(
                        "Payment request currency ({}) does not match invoice currency ({})",
                        currency, invoice.currency_code
                    )));
                }
            }
        }

        Ok(())
    }

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Extract agent DIDs directly from the message
        let mut agent_dids = Vec::new();

        // Add merchant DID
        agent_dids.push(self.merchant.id.clone());

        // Add customer DID if present
        if let Some(customer) = &self.customer {
            agent_dids.push(customer.id.clone());
        }

        // Add DIDs from agents array
        for agent in &self.agents {
            agent_dids.push(agent.id.clone());
        }

        // Remove duplicates
        agent_dids.sort();
        agent_dids.dedup();

        // If from_did is provided, remove it from the recipients list to avoid sending to self
        if let Some(from) = from_did {
            agent_dids.retain(|did| did != from);
        }

        let now = Utc::now().timestamp() as u64;

        // Get the connection ID if this message is connected to a previous message
        let pthid = self
            .connection_id()
            .map(|connect_id| connect_id.to_string());

        // Create a new Message with required fields
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: Some(agent_dids),
            thid: None,
            pthid,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
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

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Extract agent DIDs directly from the message
        let mut agent_dids = Vec::new();

        // Add agent DID if present
        if let Some(agent) = &self.agent {
            agent_dids.push(agent.id.clone());
        }

        // Add for_id if it's a DID
        if self.for_id.starts_with("did:") {
            agent_dids.push(self.for_id.clone());
        }

        // Remove duplicates
        agent_dids.sort();
        agent_dids.dedup();

        // If from_did is provided, remove it from the recipients list to avoid sending to self
        if let Some(from) = from_did {
            agent_dids.retain(|did| did != from);
        }

        let now = Utc::now().timestamp() as u64;

        // Connect messages don't have connections, so pthid is always None
        let pthid = None;

        // Create a new Message with required fields
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: Some(agent_dids),
            thid: None,
            pthid,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
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

        match chrono::DateTime::parse_from_rfc3339(&self.expires) {
            Ok(expiry_time) => {
                let expiry_time_utc = expiry_time.with_timezone(&Utc);
                if expiry_time_utc <= Utc::now() {
                    return Err(Error::Validation(
                        "Expires timestamp must be in the future".to_string(),
                    ));
                }
            }
            Err(_) => {
                return Err(Error::Validation(
                    "Expires must be a valid ISO 8601 timestamp string".to_string(),
                ));
            }
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

        match chrono::DateTime::parse_from_rfc3339(&self.expires) {
            Ok(expiry_time) => {
                let expiry_time_utc = expiry_time.with_timezone(&Utc);
                if expiry_time_utc <= Utc::now() {
                    return Err(Error::Validation(
                        "Expires timestamp must be in the future".to_string(),
                    ));
                }
            }
            Err(_) => {
                return Err(Error::Validation(
                    "Expires must be a valid ISO 8601 timestamp string".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // Create the message
        let message = Message {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: None,
            pthid: None,
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
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

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Create a new message with a random ID
        let id = uuid::Uuid::new_v4().to_string();
        let created_time = Utc::now().timestamp() as u64;

        // Create the message
        let message = Message {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: None,
            pthid: None,
            created_time: Some(created_time),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
    }
}

/// DIDComm Presentation format (using present-proof protocol)
///
/// This struct implements the standard DIDComm present-proof protocol format as defined in
/// [DIDComm Messaging Present Proof Protocol 3.0](https://github.com/decentralized-identity/waci-didcomm/tree/main/present-proof).
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
    /// - Verifiable presentations in JSON format must include `@context` and `type` fields
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
/// and the [Present Proof Protocol 3.0](https://github.com/decentralized-identity/waci-didcomm/tree/main/present-proof).
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

    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
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
            from: from_did.map(|s| s.to_string()),
            to: None,
            thid: self.thid.clone(),
            pthid: None,
            created_time: Some(Utc::now().timestamp() as u64),
            expires_time: None,
            extra_headers: HashMap::new(),
            from_prior: None,
            body: serde_json::Value::Object(body),
            attachments: didcomm_attachments,
        };

        Ok(message)
    }
}

/// Implementation of Authorizable for Transfer
impl Authorizable for Transfer {
    /// Authorizes this message, creating an Authorize message as a response
    ///
    /// # Arguments
    ///
    /// * `note` - Optional note
    ///
    /// # Returns
    ///
    /// A new Authorize message body
    fn authorize(&self, note: Option<String>) -> Authorize {
        Authorize {
            transaction_id: self.message_id(),
            note,
        }
    }

    /// Confirms a relationship between agents, creating a ConfirmRelationship message as a response
    ///
    /// # Arguments
    ///
    /// * `transfer_id` - ID of the transfer related to this message
    /// * `agent_id` - DID of the agent whose relationship is being confirmed
    /// * `for_id` - DID of the entity that the agent acts on behalf of
    /// * `role` - Optional role of the agent in the transaction
    ///
    /// # Returns
    ///
    /// A new ConfirmRelationship message body
    fn confirm_relationship(
        &self,
        transaction_id: String,
        agent_id: String,
        for_id: String,
        role: Option<String>,
    ) -> ConfirmRelationship {
        ConfirmRelationship {
            transaction_id,
            agent_id,
            for_id,
            role,
        }
    }

    /// Rejects this message, creating a Reject message as a response
    ///
    /// # Arguments
    ///
    /// * `code` - Rejection code
    /// * `description` - Description of rejection reason
    ///
    /// # Returns
    ///
    /// A new Reject message body
    fn reject(&self, code: String, description: String) -> Reject {
        Reject {
            transaction_id: self.message_id(),
            reason: format!("{}: {}", code, description),
        }
    }

    /// Settles this message, creating a Settle message as a response
    ///
    /// # Arguments
    ///
    /// * `settlement_id` - Settlement ID (CAIP-220 identifier)
    /// * `amount` - Optional amount settled
    ///
    /// # Returns
    ///
    /// A new Settle message body
    fn settle(&self, settlement_id: String, amount: Option<String>) -> Settle {
        Settle {
            transaction_id: self.message_id(),
            settlement_id,
            amount,
        }
    }

    /// Updates a party in the transaction, creating an UpdateParty message as a response
    ///
    /// # Arguments
    ///
    /// * `transfer_id` - ID of the transaction this update relates to
    /// * `party_type` - Type of party being updated (e.g., 'originator', 'beneficiary')
    /// * `party` - Updated party information
    /// * `note` - Optional note about the update
    ///
    /// # Returns
    ///
    /// A new UpdateParty message body
    fn update_party(
        &self,
        transaction_id: String,
        party_type: String,
        party: Participant,
        note: Option<String>,
    ) -> UpdateParty {
        UpdateParty {
            transaction_id,
            party_type,
            party,
            note,
            context: None,
        }
    }

    /// Updates policies for this message, creating an UpdatePolicies message as a response
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - ID of the transaction being updated
    /// * `policies` - Vector of policies to be applied
    ///
    /// # Returns
    ///
    /// A new UpdatePolicies message body
    fn update_policies(&self, transaction_id: String, policies: Vec<Policy>) -> UpdatePolicies {
        UpdatePolicies {
            transaction_id,
            policies,
        }
    }

    /// Adds agents to this message, creating an AddAgents message as a response
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - ID of the transaction to add agents to
    /// * `agents` - Vector of participants to be added
    ///
    /// # Returns
    ///
    /// A new AddAgents message body
    fn add_agents(&self, transaction_id: String, agents: Vec<Participant>) -> AddAgents {
        AddAgents {
            transaction_id,
            agents,
        }
    }

    /// Replaces an agent in this message, creating a ReplaceAgent message as a response
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - ID of the transaction to replace agent in
    /// * `original` - DID of the original agent to be replaced
    /// * `replacement` - New participant replacing the original agent
    ///
    /// # Returns
    ///
    /// A new ReplaceAgent message body
    fn replace_agent(
        &self,
        transaction_id: String,
        original: String,
        replacement: Participant,
    ) -> ReplaceAgent {
        ReplaceAgent {
            transaction_id,
            original,
            replacement,
        }
    }

    /// Removes an agent from this message, creating a RemoveAgent message as a response
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - ID of the transaction to remove agent from
    /// * `agent` - DID of the agent to remove
    ///
    /// # Returns
    ///
    /// A new RemoveAgent message body
    fn remove_agent(&self, transaction_id: String, agent: String) -> RemoveAgent {
        RemoveAgent {
            transaction_id,
            agent,
        }
    }
}

/// Represents a TAP Payment message.
/// This message type is used to initiate a payment request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Payment {
    /// Unique identifier for this payment request.
    pub transaction_id: String,
    /// Identifier for the thread this message belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thid: Option<String>,
    /// Identifier for the parent thread, used for replies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pthid: Option<String>,
    /// The merchant requesting the payment.
    pub merchant: Participant,
    /// The customer making the payment.
    pub customer: Participant,
    /// The asset being transferred (e.g., currency and amount).
    pub asset: AssetId,
    /// The amount requested for payment.
    pub amount: String,
    /// Optional details about the order or transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_details: Option<HashMap<String, serde_json::Value>>,
    /// Timestamp when the payment request was created (RFC3339).
    pub timestamp: String,
    /// Optional expiry time for the payment request (RFC3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
    /// Optional note from the merchant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Optional list of agents involved in processing the payment.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agents: Vec<Participant>,
    /// Optional metadata associated with the payment.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Optional attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
}

impl TapMessageBody for Payment {
    fn message_type() -> &'static str {
        "payment"
    }

    fn validate(&self) -> Result<()> {
        // Validate asset
        if self.asset.namespace().is_empty() || self.asset.reference().is_empty() {
            return Err(Error::Validation("Asset ID is invalid".to_string()));
        }

        // Validate merchant
        if self.merchant.id.is_empty() {
            return Err(Error::Validation("Merchant ID is required".to_string()));
        }

        // Validate customer
        if self.customer.id.is_empty() {
            return Err(Error::Validation("Customer ID is required".to_string()));
        }

        // Validate amount
        if self.amount.is_empty() {
            return Err(Error::Validation("Amount is required".to_string()));
        }
        match self.amount.parse::<f64>() {
            Ok(amount) if amount <= 0.0 => {
                return Err(Error::Validation("Amount must be positive".to_string()));
            }
            Err(_) => {
                return Err(Error::Validation(
                    "Amount must be a valid number".to_string(),
                ));
            }
            _ => {}
        }

        // Validate timestamp format
        if chrono::DateTime::parse_from_rfc3339(&self.timestamp).is_err() {
            return Err(Error::Validation(
                "Timestamp must be a valid RFC3339 string".to_string(),
            ));
        }

        // Validate expires format if present
        if let Some(expiry) = &self.expires {
            if chrono::DateTime::parse_from_rfc3339(expiry).is_err() {
                return Err(Error::Validation(
                    "Expires must be a valid RFC3339 string".to_string(),
                ));
            }
        }

        Ok(())
    }

    // Basic to_didcomm implementation (will be refined later if needed)
    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        let mut agent_dids = vec![self.merchant.id.clone(), self.customer.id.clone()];
        agent_dids.extend(self.agents.iter().map(|a| a.id.clone()));
        agent_dids.sort();
        agent_dids.dedup();

        if let Some(from) = from_did {
            agent_dids.retain(|did| did != from);
        }

        let didcomm_attachments = self.attachments.as_ref().map(|attachments| {
            attachments
                .iter()
                .filter_map(crate::utils::convert_tap_attachment_to_didcomm)
                .collect::<Vec<_>>()
        });

        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to: Some(agent_dids),
            thid: self.thid.clone(),
            pthid: self.pthid.clone(),
            created_time: Some(Utc::now().timestamp() as u64),
            expires_time: self.expires.as_ref().and_then(|exp| {
                chrono::DateTime::parse_from_rfc3339(exp)
                    .ok()
                    .map(|dt| dt.timestamp() as u64)
            }),
            extra_headers: HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: didcomm_attachments,
        };

        Ok(message)
    }
}

impl_tap_message!(Payment);

impl Authorizable for Payment {
    fn authorize(&self, note: Option<String>) -> Authorize {
        Authorize {
            transaction_id: self.message_id(),
            note,
        }
    }

    fn confirm_relationship(
        &self,
        transaction_id: String,
        agent_id: String,
        for_id: String,
        role: Option<String>,
    ) -> ConfirmRelationship {
        ConfirmRelationship {
            transaction_id,
            agent_id,
            for_id,
            role,
        }
    }

    fn reject(&self, code: String, description: String) -> Reject {
        Reject {
            transaction_id: self.message_id(),
            reason: format!("{}: {}", code, description),
        }
    }

    fn settle(&self, settlement_id: String, amount: Option<String>) -> Settle {
        Settle {
            transaction_id: self.message_id(),
            settlement_id,
            amount,
        }
    }

    fn update_party(
        &self,
        transaction_id: String,
        party_type: String,
        party: Participant,
        note: Option<String>,
    ) -> UpdateParty {
        UpdateParty {
            transaction_id,
            party_type,
            party,
            note,
            context: None,
        }
    }

    fn update_policies(&self, transaction_id: String, policies: Vec<Policy>) -> UpdatePolicies {
        UpdatePolicies {
            transaction_id,
            policies,
        }
    }

    fn add_agents(&self, transaction_id: String, agents: Vec<Participant>) -> AddAgents {
        AddAgents {
            transaction_id,
            agents,
        }
    }

    fn replace_agent(
        &self,
        transaction_id: String,
        original: String,
        replacement: Participant,
    ) -> ReplaceAgent {
        ReplaceAgent {
            transaction_id,
            original,
            replacement,
        }
    }

    fn remove_agent(&self, transaction_id: String, agent: String) -> RemoveAgent {
        RemoveAgent {
            transaction_id,
            agent,
        }
    }
}

/// PaymentBuilder
#[derive(Default)]
pub struct PaymentBuilder {
    transaction_id: Option<String>,
    thid: Option<String>,
    pthid: Option<String>,
    merchant: Option<Participant>,
    customer: Option<Participant>,
    asset: Option<AssetId>,
    amount: Option<String>,
    order_details: Option<HashMap<String, serde_json::Value>>,
    timestamp: Option<String>,
    expires: Option<String>,
    note: Option<String>,
    agents: Vec<Participant>,
    metadata: HashMap<String, serde_json::Value>,
    attachments: Option<Vec<Attachment>>,
}

impl PaymentBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn transaction_id(mut self, transaction_id: String) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }

    pub fn thid(mut self, thid: String) -> Self {
        self.thid = Some(thid);
        self
    }

    pub fn pthid(mut self, pthid: String) -> Self {
        self.pthid = Some(pthid);
        self
    }

    pub fn merchant(mut self, merchant: Participant) -> Self {
        self.merchant = Some(merchant);
        self
    }

    pub fn customer(mut self, customer: Participant) -> Self {
        self.customer = Some(customer);
        self
    }

    pub fn asset(mut self, asset: AssetId) -> Self {
        self.asset = Some(asset);
        self
    }

    pub fn amount(mut self, amount: String) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn order_details(mut self, order_details: HashMap<String, serde_json::Value>) -> Self {
        self.order_details = Some(order_details);
        self
    }

    pub fn timestamp(mut self, timestamp: String) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn expires(mut self, expires: String) -> Self {
        self.expires = Some(expires);
        self
    }

    pub fn note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }

    pub fn add_agent(mut self, agent: Participant) -> Self {
        self.agents.push(agent);
        self
    }

    pub fn set_agents(mut self, agents: Vec<Participant>) -> Self {
        self.agents = agents;
        self
    }

    pub fn add_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn set_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn add_attachment(mut self, attachment: Attachment) -> Self {
        self.attachments
            .get_or_insert_with(Vec::new)
            .push(attachment);
        self
    }

    pub fn set_attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.attachments = Some(attachments);
        self
    }

    pub fn build(self) -> Result<Payment> {
        let payment = Payment {
            transaction_id: self
                .transaction_id
                .ok_or_else(|| Error::Validation("Transaction ID is required".to_string()))?,
            thid: self.thid,
            pthid: self.pthid,
            merchant: self
                .merchant
                .ok_or_else(|| Error::Validation("Merchant participant is required".to_string()))?,
            customer: self
                .customer
                .ok_or_else(|| Error::Validation("Customer participant is required".to_string()))?,
            asset: self
                .asset
                .ok_or_else(|| Error::Validation("Asset ID is required".to_string()))?,
            amount: self
                .amount
                .ok_or_else(|| Error::Validation("Amount is required".to_string()))?,
            order_details: self.order_details,
            timestamp: self.timestamp.unwrap_or_else(|| Utc::now().to_rfc3339()),
            expires: self.expires,
            note: self.note,
            agents: self.agents,
            metadata: self.metadata,
            attachments: self.attachments,
        };

        payment.validate()?;

        Ok(payment)
    }
}

/// Helper methods for the Payment struct
impl Payment {
    /// Generates a unique message ID for authorization, rejection, or settlement
    pub fn message_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Requires proof of control from a specific agent
    pub fn require_proof_of_control(
        &self,
        _agent: String,
        _challenge: String,
    ) -> RequireProofOfControl {
        RequireProofOfControl {
            from: None,                   // Placeholder
            from_role: None,              // Placeholder
            from_agent: None,             // Placeholder
            address_id: String::new(),    // Placeholder
            purpose: Some(String::new()), // Placeholder
        }
    }
}
