//! DIDComm Presentation types for TAP messages.
//!
//! This module defines the structure of DIDComm presentation messages used in TAP.

use serde::{Deserialize, Serialize};

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::message::attachment::{Attachment, AttachmentFormat};
use crate::message::tap_message_trait::TapMessageBody;
use chrono::Utc;

/// DIDComm Presentation message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDCommPresentation {
    /// The format of the presentation.
    pub formats: Vec<AttachmentFormat>,

    /// Attachments containing the presentation data.
    pub attachments: Vec<Attachment>,

    /// Thread ID for this presentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thid: Option<String>,
}

impl TapMessageBody for DIDCommPresentation {
    fn message_type() -> &'static str {
        "https://didcomm.org/present-proof/2.0/presentation"
    }

    fn validate(&self) -> Result<()> {
        // Basic validation - ensure we have attachments
        if self.attachments.is_empty() {
            return Err(Error::Validation(
                "Presentation must have at least one attachment".to_string(),
            ));
        }

        Ok(())
    }

    fn to_didcomm(&self, from: &str) -> Result<PlainMessage> {
        // Serialize the presentation to a JSON value
        let body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        let now = Utc::now().timestamp() as u64;

        // Create a new Message with required fields
        let message = PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: from.to_string(),
            to: Vec::new(), // Recipients will be set separately
            thid: self.thid.clone(),
            pthid: None,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
        };

        Ok(message)
    }

    fn from_didcomm(message: &PlainMessage) -> Result<Self> {
        // Validate message type
        if message.type_ != Self::message_type() {
            return Err(Error::InvalidMessageType(format!(
                "Expected {} but got {}",
                Self::message_type(),
                message.type_
            )));
        }

        // Extract fields from message body
        let presentation: DIDCommPresentation = serde_json::from_value(message.body.clone())
            .map_err(|e| Error::SerializationError(e.to_string()))?;

        Ok(presentation)
    }
}

impl DIDCommPresentation {
    /// Create a new DIDCommPresentation message.
    pub fn new(
        formats: Vec<AttachmentFormat>,
        attachments: Vec<Attachment>,
        thid: Option<String>,
    ) -> Self {
        Self {
            formats,
            attachments,
            thid,
        }
    }

    // Import implementation already provided by the TapMessageBody trait
}
