//! TAP message types and structures.
//!
//! This module defines the structure of all TAP message types according to the specification.

use chrono;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tap_caip::AssetId;

/// Represents the type of TAP message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TapMessageType {
    /// Transaction proposal (Transfer in TAIP-3)
    #[serde(rename = "TAP_TRANSFER")]
    Transfer,
    /// Identity exchange (related to RequestPresentation/Presentation in TAIP-8)
    #[serde(rename = "TAP_REQUEST_PRESENTATION")]
    RequestPresentation,
    /// Presentation response (TAIP-8)
    #[serde(rename = "TAP_PRESENTATION")]
    Presentation,
    /// Authorization response (Authorize in TAIP-4)
    #[serde(rename = "TAP_AUTHORIZE")]
    Authorize,
    /// Rejection response (Reject in TAIP-4)
    #[serde(rename = "TAP_REJECT")]
    Reject,
    /// Settlement notification (Settle in TAIP-4)
    #[serde(rename = "TAP_SETTLE")]
    Settle,
    /// Add agents to a transaction (AddAgents in TAIP-5)
    #[serde(rename = "TAP_ADD_AGENTS")]
    AddAgents,
    /// Error message
    #[serde(rename = "TAP_ERROR")]
    Error,
    /// Custom message type (for extensibility)
    #[serde(untagged)]
    Custom(String),
}

impl fmt::Display for TapMessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TapMessageType::Transfer => write!(f, "transfer"),
            TapMessageType::RequestPresentation => write!(f, "request-presentation"),
            TapMessageType::Presentation => write!(f, "presentation"),
            TapMessageType::Authorize => write!(f, "authorize"),
            TapMessageType::Reject => write!(f, "reject"),
            TapMessageType::Settle => write!(f, "settle"),
            TapMessageType::AddAgents => write!(f, "add-agents"),
            TapMessageType::Error => write!(f, "error"),
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
pub struct TransferBody {
    /// Network asset identifier (CAIP-19 format).
    pub asset: AssetId,

    /// Originator information.
    #[serde(rename = "originator")]
    pub originator: Agent,

    /// Beneficiary information (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beneficiary: Option<Agent>,

    /// Amount in subunits (as a string to preserve precision).
    #[serde(rename = "amountSubunits")]
    pub amount_subunits: String,

    /// Agents involved in the transaction.
    pub agents: Vec<Agent>,

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

/// Agent structure for participants in a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// DID of the agent.
    #[serde(rename = "@id")]
    pub id: String,

    /// Role of the agent in the transaction (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

impl TransferBody {
    /// Validates that the transfer proposal contains consistent CAIP identifiers
    ///
    /// # Returns
    ///
    /// Ok(()) if the validation passes, otherwise an Error
    pub fn validate(&self) -> crate::error::Result<()> {
        // Additional validation logic could be added here
        Ok(())
    }
}

/// Request presentation message body (TAIP-8).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPresentationBody {
    /// From agent DID.
    #[serde(rename = "fromAgent")]
    pub from_agent: String,

    /// Request presentation about specific DIDs (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<Vec<String>>,

    /// Request presentation about a specific party (optional).
    #[serde(skip_serializing_if = "Option::is_none", rename = "aboutParty")]
    pub about_party: Option<String>,

    /// Request presentation about a specific agent (optional).
    #[serde(skip_serializing_if = "Option::is_none", rename = "aboutAgent")]
    pub about_agent: Option<String>,

    /// URL to a presentation definition.
    #[serde(rename = "presentationDefinition")]
    pub presentation_definition: String,

    /// Purpose of the request.
    pub purpose: String,

    /// Additional metadata for the request.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Presentation message body (TAIP-8).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationBody {
    /// Presentation submission (as a generic JSON value).
    #[serde(rename = "presentationSubmission")]
    pub presentation_submission: serde_json::Value,

    /// Verifiable presentation (as a generic JSON value).
    #[serde(rename = "verifiablePresentation")]
    pub verifiable_presentation: serde_json::Value,

    /// Additional metadata for the presentation.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Travel rule information message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelRuleInfoBody {
    /// Transaction ID this information is related to.
    pub transaction_id: String,

    /// Type of travel rule information.
    pub information_type: String,

    /// The travel rule information content.
    pub content: serde_json::Value,

    /// Additional metadata for the travel rule information.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Authorize message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizeBody {
    /// Agent identifier.
    #[serde(rename = "@id")]
    pub id: String,

    /// Transaction in context.
    pub transaction: String,

    /// Additional metadata for the authorization.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Reject message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectBody {
    /// Agent identifier.
    #[serde(rename = "@id")]
    pub id: String,

    /// Transaction in context.
    pub transaction: String,

    /// Reason for rejection.
    pub reason: String,

    /// Additional metadata for the rejection.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Settle message body (TAIP-4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettleBody {
    /// Agent identifier.
    #[serde(rename = "@id")]
    pub id: String,

    /// Transaction in context.
    pub transaction: String,

    /// Settlement ID.
    #[serde(rename = "settlementId")]
    pub settlement_id: String,

    /// Timestamp of the settlement.
    pub timestamp: String,

    /// Additional metadata for the settlement.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// AddAgents message body (TAIP-5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAgentsBody {
    /// Transaction in context.
    pub transaction: String,

    /// Agents to add to the transaction.
    pub agents: Vec<Agent>,

    /// Additional metadata for the add agents operation.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Error message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Error code.
    pub code: String,

    /// Error message.
    pub message: String,

    /// Optional transaction ID this error is related to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,

    /// Additional metadata for the error.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
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
