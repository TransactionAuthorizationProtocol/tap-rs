use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Wrapper for plain message. Provides helpers for message building and packing/unpacking.
/// Adapted from https://github.com/sicpa-dlab/didcomm-rust/blob/main/src/message/message.rs
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct PlainMessage {
    /// Message id. Must be unique to the sender.
    pub id: String,

    /// Optional, if present it must be "application/didcomm-plain+json"
    #[serde(default = "default_typ")]
    pub typ: String,

    /// Message type attribute value MUST be a valid Message Type URI,
    /// that when resolved gives human readable information about the message.
    /// The attribute’s value also informs the content of the message,
    /// or example the presence of other attributes and how they should be processed.
    #[serde(rename = "type")]
    pub type_: String,

    /// Message body.
    pub body: Value,

    /// Sender identifier. The from attribute MUST be a string that is a valid DID
    /// or DID URL (without the fragment component) which identifies the sender of the message.
    pub from: String,

    /// Identifier(s) for recipients. MUST be an array of strings where each element
    /// is a valid DID or DID URL (without the fragment component) that identifies a member
    /// of the message’s intended audience.
    pub to: Vec<String>,

    /// Uniquely identifies the thread that the message belongs to.
    /// If not included the id property of the message MUST be treated as the value of the `thid`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thid: Option<String>,

    /// If the message is a child of a thread the `pthid`
    /// will uniquely identify which thread is the parent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pthid: Option<String>,

    /// Custom message headers.
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extra_headers: HashMap<String, Value>,

    /// The attribute is used for the sender
    /// to express when they created the message, expressed in
    /// UTC Epoch Seconds (seconds since 1970-01-01T00:00:00Z UTC).
    /// This attribute is informative to the recipient, and may be relied on by protocols.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_time: Option<u64>,

    /// The expires_time attribute is used for the sender to express when they consider
    /// the message to be expired, expressed in UTC Epoch Seconds (seconds since 1970-01-01T00:00:00Z UTC).
    /// This attribute signals when the message is considered no longer valid by the sender.
    /// When omitted, the message is considered to have no expiration by the sender.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_time: Option<u64>,

    /// from_prior is a compactly serialized signed JWT containing FromPrior value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_prior: Option<String>,
}

const PLAINTEXT_TYP: &str = "application/didcomm-plain+json";

fn default_typ() -> String {
    PLAINTEXT_TYP.to_string()
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct SignatureHeader {
    pub kid: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Signature {
    pub protected: String,
    pub signature: String,
    pub header: SignatureHeader,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
struct JWS {
    pub payload: String,
    pub signatures: Vec<Signature>,
}

/// Message for out-of-band invitations (TAIP-2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutOfBand {
    /// Goal code for the invitation.
    #[serde(rename = "goal_code")]
    pub goal_code: String,

    /// Invitation message ID.
    pub id: String,

    /// Label for the invitation.
    pub label: String,

    /// Accept option for the invitation.
    pub accept: Option<String>,

    /// The DIDComm services to connect to.
    pub services: Vec<serde_json::Value>,
}
