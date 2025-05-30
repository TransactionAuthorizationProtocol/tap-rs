//! Presentation and RequestPresentation message types for the Transaction Authorization Protocol.
//!
//! This module defines the Presentation and RequestPresentation message types, which
//! are used for requesting and submitting verifiable credentials in the TAP protocol.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{TapMessage, TapMessageBody};

/// Request Presentation message body (TAIP-10).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage, TapMessageBody)]
#[tap(message_type = "https://tap.rsvp/schema/1.0#RequestPresentation")]
pub struct RequestPresentation {
    /// Transfer ID that this request is related to.
    #[tap(transaction_id)]
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

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Presentation message body (TAIP-8, TAIP-10).
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage, TapMessageBody)]
#[tap(message_type = "https://didcomm.org/present-proof/3.0/presentation")]
pub struct Presentation {
    /// Challenge from the request.
    pub challenge: String,

    /// Credential data.
    pub credentials: Vec<serde_json::Value>,

    /// Transfer ID that this presentation is related to (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tap(optional_transaction_id)]
    pub transaction_id: Option<String>,

    /// Identifier for this presentation (used for message_id)
    pub id: String,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Presentation {
    /// Create a new Presentation
    pub fn new(
        challenge: String,
        credentials: Vec<serde_json::Value>,
        transaction_id: Option<String>,
    ) -> Self {
        Self {
            challenge,
            credentials,
            transaction_id,
            id: uuid::Uuid::new_v4().to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the presentation
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}
