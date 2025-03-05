//! TAP message types and structures.
//!
//! This module defines the structure of all TAP message types according to the specification.

use chrono;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tap_caip::AssetId;
use crate::message::tap_message_trait::TapMessageBody;
use crate::error::{Error, Result};

/// Represents the type of TAP message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TapMessageType {
    /// Transaction proposal (Transfer in TAIP-3)
    #[serde(rename = "https://tap.rsvp/schema/1.0#Transfer")]
    Transfer,
    /// Identity exchange (related to RequestPresentation/Presentation in TAIP-8)
    #[serde(rename = "https://tap.rsvp/schema/1.0#RequestPresentation")]
    RequestPresentation,
    /// Presentation response (TAIP-8)
    #[serde(rename = "https://tap.rsvp/schema/1.0#Presentation")]
    Presentation,
    /// Authorization response (Authorize in TAIP-4)
    #[serde(rename = "https://tap.rsvp/schema/1.0#Authorize")]
    Authorize,
    /// Rejection response (Reject in TAIP-4)
    #[serde(rename = "https://tap.rsvp/schema/1.0#Reject")]
    Reject,
    /// Settlement notification (Settle in TAIP-4)
    #[serde(rename = "https://tap.rsvp/schema/1.0#Settle")]
    Settle,
    /// Add agents to a transaction (AddAgents in TAIP-5)
    #[serde(rename = "https://tap.rsvp/schema/1.0#AddAgents")]
    AddAgents,
    /// Error message
    #[serde(rename = "https://tap.rsvp/schema/1.0#Error")]
    Error,
    /// Custom message type (for extensibility)
    #[serde(untagged)]
    Custom(String),
}

impl fmt::Display for TapMessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TapMessageType::Transfer => write!(f, "https://tap.rsvp/schema/1.0#Transfer"),
            TapMessageType::RequestPresentation => write!(f, "https://tap.rsvp/schema/1.0#RequestPresentation"),
            TapMessageType::Presentation => write!(f, "https://tap.rsvp/schema/1.0#Presentation"),
            TapMessageType::Authorize => write!(f, "https://tap.rsvp/schema/1.0#Authorize"),
            TapMessageType::Reject => write!(f, "https://tap.rsvp/schema/1.0#Reject"),
            TapMessageType::Settle => write!(f, "https://tap.rsvp/schema/1.0#Settle"),
            TapMessageType::AddAgents => write!(f, "https://tap.rsvp/schema/1.0#AddAgents"),
            TapMessageType::Error => write!(f, "https://tap.rsvp/schema/1.0#Error"),
            TapMessageType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Attachment structure for including files, documents, or other data in TAP messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Unique identifier for the attachment.
    pub id: String,

    /// MIME type of the attachment.
    pub mime_type: String,

    /// Filename (optional).
    pub filename: Option<String>,

    /// Description (optional).
    pub description: Option<String>,

    /// The actual data of the attachment.
    pub data: AttachmentData,
}

/// Representation of attachment data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentData {
    /// Base64-encoded data.
    Base64(String),

    /// JSON data.
    Json(serde_json::Value),

    /// External link to data.
    Links { links: Vec<String> },
}

/// Represents a TAP message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapMessage {
    /// Type of the message.
    #[serde(rename = "type")]
    pub message_type: TapMessageType,

    /// Unique identifier for the message.
    pub id: String,

    /// Version of the TAP protocol.
    pub version: String,

    /// When the message was created (RFC3339 timestamp).
    pub created_time: String,

    /// When the message expires (RFC3339 timestamp, optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_time: Option<String>,

    /// The main content of the message (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,

    /// Attachments to the message (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,

    /// Additional metadata for the message.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,

    /// From DID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_did: Option<String>,

    /// To DID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_did: Option<String>,
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

    /// Amount as a decimal string (to preserve precision).
    #[serde(rename = "amount")]
    pub amount: String,

    /// Agents involved in the transaction.
    pub agents: Vec<Participant>,

    /// Optional settled transaction ID.
    #[serde(skip_serializing_if = "Option::is_none", rename = "settlementId")]
    pub settlement_id: Option<String>,

    /// Optional memo or note for the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Additional metadata for the transaction.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Transfer {
    /// Validates that the transfer proposal contains consistent CAIP identifiers
    ///
    /// # Returns
    ///
    /// Ok(()) if the validation passes, otherwise an Error
    pub fn validate(&self) -> crate::error::Result<()> {
        // TODO: Add CAIP validation
        Ok(())
    }
}

impl TapMessageBody for Transfer {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#Transfer"
    }
    
    fn validate(&self) -> Result<()> {
        // Delegate to the existing validate method
        self.validate()
    }
}

/// Participant structure for participants in a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// DID of the participant.
    #[serde(rename = "@id")]
    pub id: String,

    /// Role of the participant in the transaction (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// Request presentation message body (TAIP-8).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPresentation {
    /// Unique identifier for the presentation request.
    pub presentation_id: String,

    /// Information about the requested presentation.
    pub credentials: Vec<serde_json::Value>,

    /// Optional comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Expiration time for the request (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_time: Option<String>,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TapMessageBody for RequestPresentation {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#RequestPresentation"
    }
    
    fn validate(&self) -> Result<()> {
        // Basic validation logic
        if self.presentation_id.is_empty() {
            return Err(Error::Validation("Presentation ID is required".to_string()));
        }
        
        if self.credentials.is_empty() {
            return Err(Error::Validation("At least one credential request is required".to_string()));
        }
        
        Ok(())
    }
}

/// Presentation message body (TAIP-8).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    /// Presentation ID that relates to a RequestPresentation.
    pub presentation_id: String,

    /// Requested presentation information.
    pub credentials: Vec<serde_json::Value>,

    /// Optional comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TapMessageBody for Presentation {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#Presentation"
    }
    
    fn validate(&self) -> Result<()> {
        // Basic validation logic
        if self.presentation_id.is_empty() {
            return Err(Error::Validation("Presentation ID is required".to_string()));
        }
        
        if self.credentials.is_empty() {
            return Err(Error::Validation("Credentials are required".to_string()));
        }
        
        Ok(())
    }
}


/// Authorize message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorize {
    /// The ID of the transfer being authorized.
    pub transfer_id: String,

    /// Optional note about the authorization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TapMessageBody for Authorize {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#Authorize"
    }
    
    fn validate(&self) -> Result<()> {
        // Basic validation logic
        if self.transfer_id.is_empty() {
            return Err(Error::Validation("Transfer ID is required".to_string()));
        }
        
        Ok(())
    }
}

/// Reject message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reject {
    /// The ID of the transfer being rejected.
    pub transfer_id: String,

    /// Reason code for the rejection.
    pub code: String,

    /// Human-readable description of the rejection reason.
    pub description: String,

    /// Optional note about the rejection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TapMessageBody for Reject {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#Reject"
    }
    
    fn validate(&self) -> Result<()> {
        // Basic validation logic
        if self.transfer_id.is_empty() {
            return Err(Error::Validation("Transfer ID is required".to_string()));
        }
        
        if self.code.is_empty() {
            return Err(Error::Validation("Rejection code is required".to_string()));
        }
        
        Ok(())
    }
}

/// Settle message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settle {
    /// The ID of the transfer being settled.
    pub transfer_id: String,

    /// Transaction ID on the external ledger.
    pub transaction_id: String,
    
    /// Optional transaction hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,

    /// Block height of the transaction (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_height: Option<u64>,

    /// Optional note about the settlement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TapMessageBody for Settle {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#Settle"
    }
    
    fn validate(&self) -> Result<()> {
        // Basic validation logic
        if self.transfer_id.is_empty() {
            return Err(Error::Validation("Transfer ID is required".to_string()));
        }
        
        if self.transaction_id.is_empty() {
            return Err(Error::Validation("Transaction ID is required".to_string()));
        }
        
        Ok(())
    }
}

/// AddAgents message body (TAIP-5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAgents {
    /// The ID of the transfer to add agents to.
    pub transfer_id: String,

    /// Agents to add to the transaction.
    pub agents: Vec<Participant>,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TapMessageBody for AddAgents {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#AddAgents"
    }
    
    fn validate(&self) -> Result<()> {
        // Basic validation logic
        if self.transfer_id.is_empty() {
            return Err(Error::Validation("Transfer ID is required".to_string()));
        }
        
        if self.agents.is_empty() {
            return Err(Error::Validation("At least one agent must be specified".to_string()));
        }
        
        // Validate that all agents have a valid ID
        for agent in &self.agents {
            if agent.id.is_empty() {
                return Err(Error::Validation("Agent ID is required for all agents".to_string()));
            }
        }
        
        Ok(())
    }
}

/// Error message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Error code.
    pub code: String,

    /// Human-readable description of the error.
    pub description: String,
    
    /// The ID of the message that caused the error (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_message_id: Option<String>,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TapMessageBody for ErrorBody {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#Error"
    }
    
    fn validate(&self) -> Result<()> {
        // Basic validation logic
        if self.code.is_empty() {
            return Err(Error::Validation("Error code is required".to_string()));
        }
        
        if self.description.is_empty() {
            return Err(Error::Validation("Error description is required".to_string()));
        }
        
        Ok(())
    }
}

/// Trait for validating TAP message structures.
pub trait Validate {
    /// Validates the structure and content of a TAP message.
    fn validate(&self) -> crate::error::Result<()>;
}

impl TapMessage {
    /// Create a new TAP message with default values.
    pub fn new(message_type: TapMessageType) -> Self {
        Self {
            message_type,
            id: uuid::Uuid::new_v4().to_string(),
            version: "1.0".to_string(),
            created_time: chrono::Utc::now().to_rfc3339(),
            expires_time: None,
            body: None,
            attachments: None,
            metadata: HashMap::new(),
            from_did: None,
            to_did: None,
        }
    }

    /// Set the ID of the message
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set the body of the message
    pub fn with_body<T: Serialize>(mut self, body: &T) -> Self {
        self.body = serde_json::to_value(body).ok();
        self
    }

    /// Set the attachments of the message
    pub fn with_attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.attachments = Some(attachments);
        self
    }

    /// Set the expires time of the message
    pub fn with_expires_time(mut self, expires_time: impl Into<String>) -> Self {
        self.expires_time = Some(expires_time.into());
        self
    }

    /// Convert the body to the specified type, or return an error if conversion fails.
    pub fn body_as<T: DeserializeOwned>(&self) -> crate::error::Result<T> {
        match &self.body {
            None => Err(crate::error::Error::Validation(
                "Message body is missing".to_string(),
            )),
            Some(body) => serde_json::from_value(body.clone())
                .map_err(|e| crate::error::Error::SerializationError(e.to_string())),
        }
    }
}

/// Builder for TAP messages.
#[derive(Debug, Clone, Default)]
pub struct TapMessageBuilder {
    id: Option<String>,
    message_type: Option<TapMessageType>,
    body: Option<serde_json::Value>,
    from: Option<String>,
    to: Option<String>,
    attachments: Option<Vec<Attachment>>,
    expires_time: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl TapMessageBuilder {
    /// Create a new message builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the ID of the message.
    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the message type.
    pub fn message_type(mut self, message_type: TapMessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }

    /// Set the type field (legacy method).
    #[deprecated(since = "0.2.0", note = "Use message_type instead")]
    pub fn type_field(mut self, message_type: TapMessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }

    /// Set the body of the message.
    pub fn body<T: Serialize>(mut self, body: T) -> Self {
        self.body = serde_json::to_value(body).ok();
        self
    }

    /// Set the from DID.
    pub fn from_did<S: Into<String>>(mut self, from: Option<S>) -> Self {
        self.from = from.map(|s| s.into());
        self
    }

    /// Set the from field (legacy method).
    #[deprecated(since = "0.2.0", note = "Use from_did instead")]
    pub fn from<S: Into<String>>(mut self, from: S) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Set the to DID.
    pub fn to_did<S: Into<String>>(mut self, to: Option<S>) -> Self {
        self.to = to.map(|s| s.into());
        self
    }

    /// Set the to field (legacy method).
    #[deprecated(since = "0.2.0", note = "Use to_did instead")]
    pub fn to<S: Into<String>>(mut self, to: S) -> Self {
        self.to = Some(to.into());
        self
    }

    /// Set the attachments.
    pub fn attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.attachments = Some(attachments);
        self
    }

    /// Set the expiration time.
    pub fn expires_time<S: Into<String>>(mut self, expires_time: S) -> Self {
        self.expires_time = Some(expires_time.into());
        self
    }

    /// Add metadata.
    pub fn metadata<K: Into<String>, V: Serialize>(mut self, key: K, value: V) -> Self {
        if let Ok(value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), value);
        }
        self
    }

    /// Build the message.
    pub fn build(self) -> crate::error::Result<TapMessage> {
        if self.id.is_none() {
            return Err(crate::error::Error::Validation(
                "Message ID is required".to_string(),
            ));
        }

        if self.message_type.is_none() {
            return Err(crate::error::Error::Validation(
                "Message type is required".to_string(),
            ));
        }

        let now = chrono::Utc::now();

        Ok(TapMessage {
            id: self.id.unwrap(),
            message_type: self.message_type.unwrap(),
            version: "1.0".to_string(),
            created_time: now.to_rfc3339(),
            expires_time: self.expires_time,
            body: self.body,
            attachments: self.attachments,
            metadata: self.metadata,
            from_did: self.from,
            to_did: self.to,
        })
    }
}
