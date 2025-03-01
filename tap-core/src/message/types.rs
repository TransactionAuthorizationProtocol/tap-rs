//! TAP message types and structures.
//!
//! This module defines the structure of all TAP message types according to the specification.

use chrono;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid;

/// Represents the type of TAP message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TapMessageType {
    TransactionProposal,
    IdentityExchange,
    TravelRuleInfo,
    AuthorizationResponse,
    Error,
    #[serde(untagged)]
    Custom(String),
}

impl fmt::Display for TapMessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TapMessageType::TransactionProposal => write!(f, "transaction-proposal"),
            TapMessageType::IdentityExchange => write!(f, "identity-exchange"),
            TapMessageType::TravelRuleInfo => write!(f, "travel-rule-info"),
            TapMessageType::AuthorizationResponse => write!(f, "authorization-response"),
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
}

/// Transaction proposal message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionProposalBody {
    /// Unique identifier for the transaction.
    pub transaction_id: String,

    /// Network identifier (CAIP-2 format).
    pub network: String,

    /// Sender account address (CAIP-10 format).
    pub sender: String,

    /// Recipient account address (CAIP-10 format).
    pub recipient: String,

    /// Asset identifier (CAIP-19 format).
    pub asset: String,

    /// Amount of the asset (as a string to preserve precision).
    pub amount: String,

    /// Optional memo or note for the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,

    /// Optional reference to an external transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_reference: Option<String>,

    /// Additional metadata for the transaction.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Identity exchange message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityExchangeBody {
    /// DID of the entity.
    pub entity_did: String,

    /// Optional name of the entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_name: Option<String>,

    /// Optional verification method ID for the entity's DID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_method_id: Option<String>,

    /// Optional key agreement method ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_agreement_id: Option<String>,

    /// Additional metadata for the identity.
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

/// Authorization response message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResponseBody {
    /// Transaction ID this response is related to.
    pub transaction_id: String,

    /// Whether the transaction is authorized.
    pub authorized: bool,

    /// Optional reason for the authorization decision.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Additional metadata for the authorization response.
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

// The fmt::Display implementation for TapMessageType is already defined at the top of the file
