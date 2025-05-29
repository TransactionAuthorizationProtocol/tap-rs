//! DIDComm Presentation types for TAP messages.
//!
//! This module defines the structure of DIDComm presentation messages used in TAP.

use serde::{Deserialize, Serialize};

use crate::didcomm::Attachment;
use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::message::tap_message_trait::TapMessageBody;
use crate::TapMessage;
use chrono::Utc;

fn default_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// DIDComm Presentation message body.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
pub struct DIDCommPresentation {
    /// Message ID.
    #[serde(default = "default_id")]
    pub id: String,

    /// The format of the presentation (simplified from AttachmentFormat).
    pub formats: Vec<String>,

    /// Attachments containing the presentation data.
    pub attachments: Vec<Attachment>,

    /// Thread ID for this presentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[tap(thread_id)]
    pub thid: Option<String>,
}

impl TapMessageBody for DIDCommPresentation {
    fn message_type() -> &'static str {
        "https://didcomm.org/present-proof/3.0/presentation"
    }

    fn validate(&self) -> Result<()> {
        // Basic validation - ensure we have attachments
        if self.attachments.is_empty() {
            return Err(Error::Validation(
                "Presentation must have at least one attachment".to_string(),
            ));
        }

        // Validate that attachment ids are not empty
        for (i, attachment) in self.attachments.iter().enumerate() {
            if let Some(id) = &attachment.id {
                if id.is_empty() {
                    return Err(Error::Validation(format!(
                        "Attachment {} has an empty ID",
                        i
                    )));
                }
            }
        }

        // Ensure formats are present and not empty
        if self.formats.is_empty() {
            return Err(Error::Validation(
                "Presentation must have at least one format specified".to_string(),
            ));
        }

        // Check attachments for required format field
        for (i, attachment) in self.attachments.iter().enumerate() {
            if attachment.format.is_none() {
                return Err(Error::Validation(format!(
                    "Attachment {} is missing the 'format' field",
                    i
                )));
            }
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
            attachments: None,
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

        // Extract fields from message body as Value
        let body = message.body.clone();
        let mut body_obj = body
            .as_object()
            .ok_or_else(|| Error::SerializationError("Body is not a JSON object".to_string()))?
            .clone();

        // First make sure any message-level attachments are included
        let mut attachments_in_body = if body_obj.contains_key("attachments") {
            match &body_obj["attachments"] {
                serde_json::Value::Array(arr) => arr.clone(),
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };

        // Then add any top-level message attachments
        if let Some(msg_attachments) = &message.attachments {
            // Convert message attachments to value and combine with body attachments
            if let Ok(serde_json::Value::Array(arr)) = serde_json::to_value(msg_attachments) {
                attachments_in_body.extend(arr);
            }
        }

        // Update the body with combined attachments
        body_obj.insert(
            "attachments".to_string(),
            serde_json::Value::Array(attachments_in_body.clone()),
        );

        // Handle missing formats field for backwards compatibility with test vectors
        if !body_obj.contains_key("formats") {
            // Extract formats from attachments if possible
            let mut formats = Vec::new();
            for attachment in &attachments_in_body {
                if let Some(format) = attachment.get("format") {
                    if let Some(format_str) = format.as_str() {
                        formats.push(format_str.to_string());
                    }
                }
            }

            // If we couldn't extract formats, use a default
            if formats.is_empty() {
                formats.push("dif/presentation-exchange/submission@v1.0".to_string());
            }

            body_obj.insert(
                "formats".to_string(),
                serde_json::to_value(formats).unwrap(),
            );
        }

        // Convert the updated body to DIDCommPresentation
        let mut presentation: DIDCommPresentation =
            serde_json::from_value(serde_json::Value::Object(body_obj))
                .map_err(|e| Error::SerializationError(e.to_string()))?;

        // Set thid from message if it's not already set in the presentation
        if presentation.thid.is_none() {
            presentation.thid = message.thid.clone();
        }

        Ok(presentation)
    }
}

impl DIDCommPresentation {
    /// Create a new DIDCommPresentation message.
    pub fn new(formats: Vec<String>, attachments: Vec<Attachment>, thid: Option<String>) -> Self {
        Self {
            id: default_id(),
            formats,
            attachments,
            thid,
        }
    }

    // Import implementation already provided by the TapMessageBody trait
}
