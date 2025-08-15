//! Out-of-Band (OOB) message support for TAP agents
//!
//! This module implements Out-of-Band invitations according to the DIDComm v2 specification
//! and TAIP-2, allowing agents to share messages through URLs or QR codes.

use crate::error::{Error, Result};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tap_msg::didcomm::{Attachment, AttachmentData, JsonAttachmentData};
use url::Url;

/// Out-of-Band invitation structure following DIDComm v2 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutOfBandInvitation {
    /// Message type - must be "https://didcomm.org/out-of-band/2.0/invitation"
    #[serde(rename = "type")]
    pub type_: String,

    /// Unique identifier for this invitation
    pub id: String,

    /// DID of the sender
    pub from: String,

    /// Message body containing goal and other metadata
    pub body: OutOfBandBody,

    /// Optional attachments containing the actual message content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
}

/// Body of an Out-of-Band invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutOfBandBody {
    /// Goal code indicating the purpose (e.g., "tap.payment", "tap.connect")
    pub goal_code: String,

    /// Human-readable goal description
    pub goal: String,

    /// Accepted message formats
    pub accept: Vec<String>,

    /// Optional additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, Value>,
}

/// Builder for creating Out-of-Band invitations
pub struct OutOfBandBuilder {
    from: String,
    goal_code: String,
    goal: String,
    accept: Vec<String>,
    metadata: HashMap<String, Value>,
    attachments: Vec<Attachment>,
}

impl OutOfBandBuilder {
    /// Create a new builder
    pub fn new(from: &str, goal_code: &str, goal: &str) -> Self {
        Self {
            from: from.to_string(),
            goal_code: goal_code.to_string(),
            goal: goal.to_string(),
            accept: vec!["didcomm/v2".to_string()],
            metadata: HashMap::new(),
            attachments: Vec::new(),
        }
    }

    /// Add an accepted format
    pub fn add_accept(mut self, format: &str) -> Self {
        self.accept.push(format.to_string());
        self
    }

    /// Add metadata
    pub fn add_metadata(mut self, key: &str, value: Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Add a signed message attachment
    pub fn add_signed_attachment(
        mut self,
        id: &str,
        signed_message: &str,
        description: Option<&str>,
    ) -> Self {
        let attachment = Attachment {
            id: Some(id.to_string()),
            description: description.map(|s| s.to_string()),
            media_type: Some("application/didcomm-signed+json".to_string()),
            data: AttachmentData::Json {
                value: JsonAttachmentData {
                    json: serde_json::from_str(signed_message)
                        .unwrap_or_else(|_| Value::String(signed_message.to_string())),
                    jws: None,
                },
            },
            filename: None,
            format: None,
            lastmod_time: None,
            byte_count: None,
        };
        self.attachments.push(attachment);
        self
    }

    /// Add a plain JSON attachment
    pub fn add_json_attachment(
        mut self,
        id: &str,
        json_data: Value,
        description: Option<&str>,
    ) -> Self {
        let attachment = Attachment {
            id: Some(id.to_string()),
            description: description.map(|s| s.to_string()),
            media_type: Some("application/json".to_string()),
            data: AttachmentData::Json {
                value: JsonAttachmentData {
                    json: json_data,
                    jws: None,
                },
            },
            filename: None,
            format: None,
            lastmod_time: None,
            byte_count: None,
        };
        self.attachments.push(attachment);
        self
    }

    /// Build the Out-of-Band invitation
    pub fn build(self) -> OutOfBandInvitation {
        OutOfBandInvitation {
            type_: "https://didcomm.org/out-of-band/2.0/invitation".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            from: self.from,
            body: OutOfBandBody {
                goal_code: self.goal_code,
                goal: self.goal,
                accept: self.accept,
                metadata: self.metadata,
            },
            attachments: if self.attachments.is_empty() {
                None
            } else {
                Some(self.attachments)
            },
        }
    }
}

impl OutOfBandInvitation {
    /// Create a new builder
    pub fn builder(from: &str, goal_code: &str, goal: &str) -> OutOfBandBuilder {
        OutOfBandBuilder::new(from, goal_code, goal)
    }

    /// Encode as a URL with the given base URL
    pub fn to_url(&self, base_url: &str) -> Result<String> {
        let json = serde_json::to_string(self)
            .map_err(|e| Error::Serialization(format!("Failed to serialize OOB: {}", e)))?;

        let encoded = BASE64_URL_SAFE_NO_PAD.encode(json.as_bytes());

        let mut url = Url::parse(base_url)
            .map_err(|e| Error::Validation(format!("Invalid base URL: {}", e)))?;

        url.query_pairs_mut().append_pair("_oob", &encoded);

        Ok(url.to_string())
    }

    /// Create a URL using just the invitation ID (requires pre-published OOB)
    pub fn to_id_url(&self, base_url: &str) -> Result<String> {
        let mut url = Url::parse(base_url)
            .map_err(|e| Error::Validation(format!("Invalid base URL: {}", e)))?;

        url.query_pairs_mut().append_pair("_oobid", &self.id);

        Ok(url.to_string())
    }

    /// Parse an Out-of-Band invitation from a URL
    pub fn from_url(url_str: &str) -> Result<Self> {
        let url = Url::parse(url_str)
            .map_err(|e| Error::Validation(format!("Invalid URL: {}", e)))?;

        // Look for _oob parameter
        for (key, value) in url.query_pairs() {
            if key == "_oob" {
                let decoded = BASE64_URL_SAFE_NO_PAD
                    .decode(value.as_bytes())
                    .map_err(|e| Error::Validation(format!("Invalid base64 encoding: {}", e)))?;

                let json_str = String::from_utf8(decoded)
                    .map_err(|e| Error::Validation(format!("Invalid UTF-8: {}", e)))?;

                return serde_json::from_str(&json_str)
                    .map_err(|e| Error::Serialization(format!("Failed to parse OOB: {}", e)));
            }
        }

        Err(Error::Validation(
            "No _oob parameter found in URL".to_string(),
        ))
    }

    /// Get the first signed attachment if present
    pub fn get_signed_attachment(&self) -> Option<&Attachment> {
        self.attachments.as_ref()?.iter().find(|attachment| {
            attachment
                .media_type
                .as_ref()
                .map(|mt| mt.contains("didcomm-signed"))
                .unwrap_or(false)
        })
    }

    /// Extract the JSON data from an attachment
    pub fn extract_attachment_json(&self, attachment_id: &str) -> Option<&Value> {
        let attachments = self.attachments.as_ref()?;
        let attachment = attachments
            .iter()
            .find(|a| a.id.as_deref() == Some(attachment_id))?;

        match &attachment.data {
            AttachmentData::Json { value } => Some(&value.json),
            _ => None,
        }
    }

    /// Check if this is a payment invitation
    pub fn is_payment_invitation(&self) -> bool {
        self.body.goal_code == "tap.payment"
    }

    /// Check if this is a connection invitation
    pub fn is_connection_invitation(&self) -> bool {
        self.body.goal_code == "tap.connect"
    }

    /// Validate the Out-of-Band invitation structure
    pub fn validate(&self) -> Result<()> {
        // Check type
        if self.type_ != "https://didcomm.org/out-of-band/2.0/invitation" {
            return Err(Error::Validation(format!(
                "Invalid type: expected https://didcomm.org/out-of-band/2.0/invitation, got {}",
                self.type_
            )));
        }

        // Check goal_code format - must be valid format
        if self.body.goal_code.contains('.') {
            if self.body.goal_code.starts_with("tap.") {
                let valid_codes = ["tap.payment", "tap.connect", "tap.transfer"];
                if !valid_codes.contains(&self.body.goal_code.as_str()) {
                    return Err(Error::Validation(format!(
                        "Invalid TAP goal code: {}",
                        self.body.goal_code
                    )));
                }
            } else {
                // Reject unknown namespaced goal codes
                return Err(Error::Validation(format!(
                    "Unknown goal code namespace: {}",
                    self.body.goal_code
                )));
            }
        }

        // Check that accept includes didcomm/v2
        if !self.body.accept.contains(&"didcomm/v2".to_string()) {
            return Err(Error::Validation(
                "Out-of-Band invitation must accept didcomm/v2".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_oob_builder() {
        let oob = OutOfBandInvitation::builder(
            "did:example:alice",
            "tap.payment",
            "Process payment request",
        )
        .add_metadata("amount", json!("100.00"))
        .build();

        assert_eq!(oob.from, "did:example:alice");
        assert_eq!(oob.body.goal_code, "tap.payment");
        assert_eq!(oob.body.goal, "Process payment request");
        assert!(oob.body.accept.contains(&"didcomm/v2".to_string()));
    }

    #[test]
    fn test_oob_url_encoding() {
        let oob = OutOfBandInvitation::builder(
            "did:example:alice",
            "tap.payment",
            "Process payment request",
        )
        .build();

        let url = oob.to_url("https://example.com/pay").unwrap();
        assert!(url.starts_with("https://example.com/pay?_oob="));

        // Test round trip
        let parsed_oob = OutOfBandInvitation::from_url(&url).unwrap();
        assert_eq!(parsed_oob.from, oob.from);
        assert_eq!(parsed_oob.body.goal_code, oob.body.goal_code);
    }

    #[test]
    fn test_oob_validation() {
        let mut oob = OutOfBandInvitation::builder(
            "did:example:alice",
            "tap.payment",
            "Process payment request",
        )
        .build();

        // Valid OOB should pass
        assert!(oob.validate().is_ok());

        // Invalid type should fail
        oob.type_ = "invalid-type".to_string();
        assert!(oob.validate().is_err());

        // Reset type and test invalid goal code
        oob.type_ = "https://didcomm.org/out-of-band/2.0/invitation".to_string();
        oob.body.goal_code = "tap.invalid".to_string();
        assert!(oob.validate().is_err());
    }

    #[test]
    fn test_signed_attachment() {
        let signed_jws = r#"{"payload":"eyJ0ZXN0IjoidmFsdWUifQ","signatures":[{"signature":"test"}]}"#;

        let oob = OutOfBandInvitation::builder(
            "did:example:alice",
            "tap.payment",
            "Process payment request",
        )
        .add_signed_attachment("payment-1", signed_jws, Some("Payment request"))
        .build();

        assert!(oob.attachments.is_some());
        let attachment = oob.get_signed_attachment().unwrap();
        assert_eq!(attachment.id.as_deref(), Some("payment-1"));
        assert_eq!(
            attachment.media_type.as_deref(),
            Some("application/didcomm-signed+json")
        );
    }
}