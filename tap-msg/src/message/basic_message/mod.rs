//! Basic Message Protocol Implementation
//!
//! Implementation of the DIDComm Basic Message 2.0 protocol as specified at:
//! https://didcomm.org/basicmessage/2.0/
//!
//! The Basic Message protocol provides simple, human-readable messaging
//! capabilities between DIDComm agents.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tap_msg_derive::TapMessage;

pub const BASIC_MESSAGE_TYPE: &str = "https://didcomm.org/basicmessage/2.0/message";

/// Basic Message for simple text communication between agents
///
/// The Basic Message protocol allows agents to send simple text messages
/// to each other. This is useful for human-readable communication and
/// debugging purposes.
#[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
#[tap(
    message_type = "https://didcomm.org/basicmessage/2.0/message",
    custom_validation
)]
pub struct BasicMessage {
    /// The content of the message
    pub content: String,

    /// Optional locale for the message content (e.g., "en", "es", "fr")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    /// Optional timestamp when the message was sent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_time: Option<u64>,

    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl BasicMessage {
    /// Create a new Basic Message with content
    pub fn new(content: String) -> Self {
        Self {
            content,
            locale: None,
            sent_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            metadata: HashMap::new(),
        }
    }

    /// Create a Basic Message with content and locale
    pub fn with_locale(content: String, locale: String) -> Self {
        Self {
            content,
            locale: Some(locale),
            sent_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            metadata: HashMap::new(),
        }
    }

    /// Set the sent time
    pub fn sent_time(mut self, timestamp: u64) -> Self {
        self.sent_time = Some(timestamp);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get the content of the message
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// Get the locale of the message
    pub fn get_locale(&self) -> Option<&str> {
        self.locale.as_deref()
    }

    /// Get the sent time
    pub fn get_sent_time(&self) -> Option<u64> {
        self.sent_time
    }
}

impl BasicMessage {
    /// Custom validation for Basic Message
    pub fn validate_basicmessage(&self) -> Result<()> {
        if self.content.is_empty() {
            return Err(Error::Validation(
                "Basic message content cannot be empty".to_string(),
            ));
        }

        // Validate comment length if present
        if self.content.len() > 10000 {
            return Err(Error::Validation(
                "Basic message content exceeds maximum length of 10000 characters".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_message_creation() {
        let message = BasicMessage::new("Hello, world!".to_string());
        assert_eq!(message.content, "Hello, world!");
        assert!(message.locale.is_none());
        assert!(message.sent_time.is_some());
        assert!(message.metadata.is_empty());
    }

    #[test]
    fn test_basic_message_with_locale() {
        let message = BasicMessage::with_locale("Hola, mundo!".to_string(), "es".to_string());
        assert_eq!(message.content, "Hola, mundo!");
        assert_eq!(message.locale, Some("es".to_string()));
        assert!(message.sent_time.is_some());
    }

    #[test]
    fn test_basic_message_with_metadata() {
        let message = BasicMessage::new("Test message".to_string())
            .with_metadata("priority".to_string(), serde_json::json!("high"))
            .with_metadata("category".to_string(), serde_json::json!("alert"));

        assert_eq!(message.metadata.len(), 2);
        assert_eq!(
            message.metadata.get("priority"),
            Some(&serde_json::json!("high"))
        );
        assert_eq!(
            message.metadata.get("category"),
            Some(&serde_json::json!("alert"))
        );
    }

    #[test]
    fn test_basic_message_getters() {
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        let message = BasicMessage::new("Test".to_string()).sent_time(timestamp);

        assert_eq!(message.get_content(), "Test");
        assert_eq!(message.get_locale(), None);
        assert_eq!(message.get_sent_time(), Some(timestamp));
    }

    #[test]
    fn test_basic_message_serialization() {
        let message = BasicMessage::with_locale("Test message".to_string(), "en".to_string())
            .with_metadata("test_key".to_string(), serde_json::json!("test_value"));

        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: BasicMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(message.content, deserialized.content);
        assert_eq!(message.locale, deserialized.locale);
        assert_eq!(message.sent_time, deserialized.sent_time);
        assert_eq!(message.metadata, deserialized.metadata);
    }

    #[test]
    fn test_basic_message_validation() {
        let message = BasicMessage::new("Test".to_string());
        assert!(message.validate_basicmessage().is_ok());

        let empty_message = BasicMessage {
            content: "".to_string(),
            locale: None,
            sent_time: None,
            metadata: HashMap::new(),
        };
        assert!(empty_message.validate_basicmessage().is_err());

        let long_message = BasicMessage::new("a".repeat(10001));
        assert!(long_message.validate_basicmessage().is_err());
    }

    #[test]
    fn test_message_type() {
        use crate::message::tap_message_trait::TapMessageBody;
        assert_eq!(BasicMessage::message_type(), BASIC_MESSAGE_TYPE);
    }
}
