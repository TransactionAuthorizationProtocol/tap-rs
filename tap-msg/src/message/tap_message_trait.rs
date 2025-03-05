//! Traits for TAP message conversion and validation.
//!
//! This module provides traits for converting between DIDComm messages
//! and TAP-specific message bodies, as well as validation of those bodies.

use crate::error::{Error, Result};
use crate::utils::get_current_time;
use didcomm::Message;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// A trait for TAP message body types that can be serialized to and deserialized from DIDComm messages.
pub trait TapMessageBody: Serialize + DeserializeOwned {
    /// Get the message type string for this body type.
    fn message_type() -> &'static str;

    /// Validate the message body.
    fn validate(&self) -> Result<()>;

    /// Convert this body to a DIDComm message with no sender or recipients.
    fn to_didcomm(&self) -> Result<Message> {
        let body = serde_json::to_value(self)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
        
        // Get current time as u64 seconds since Unix epoch
        let now = get_current_time()?;
            
        // Create a new Message with required fields
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body,
            from: None,
            to: None,
            thid: None,
            pthid: None,
            extra_headers: std::collections::HashMap::new(),
            created_time: Some(now),
            expires_time: None,
            from_prior: None,
            attachments: None,
        };

        Ok(message)
    }

    /// Convert this body to a DIDComm message with a sender and multiple recipients.
    fn to_didcomm_with_route<'a, I>(&self, from: Option<&str>, to: I) -> Result<Message>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut message = self.to_didcomm()?;
        
        // Set the sender if provided
        if let Some(sender) = from {
            message.from = Some(sender.to_string());
        }
        
        // Set the recipients
        let to_vec: Vec<String> = to.into_iter().map(String::from).collect();
        if !to_vec.is_empty() {
            message.to = Some(to_vec);
        }
        
        Ok(message)
    }

    /// Extract this body type from a DIDComm message.
    fn from_didcomm(message: &Message) -> Result<Self> {
        // Verify that this is the correct message type
        if let Some(type_) = message.get_tap_type() {
            if type_ != Self::message_type() {
                return Err(Error::InvalidMessageType(format!(
                    "Expected message type {}, but found {}",
                    Self::message_type(),
                    type_
                )));
            }
        } else {
            return Err(Error::InvalidMessageType(format!(
                "Message is not a TAP message or missing type, expected {}",
                Self::message_type()
            )));
        }
        
        // Extract and deserialize the body
        let body = serde_json::from_value(message.body.clone())
            .map_err(|e| Error::SerializationError(format!("Failed to deserialize message body: {}", e)))?;
        
        Ok(body)
    }
}

/// A trait for working with TAP messages using the DIDComm Message struct directly.
///
/// This trait provides utility methods for working with DIDComm Messages in the context of the TAP protocol.
pub trait TapMessage {
    /// Validates a TAP message.
    ///
    /// This method checks if the message meets all the requirements
    /// for a valid TAP message, including:
    ///
    /// - Having a valid ID
    /// - Having a valid created timestamp
    /// - Having a valid TAP message type
    /// - Having a valid body structure
    ///
    /// # Returns
    ///
    /// `Ok(())` if the message is valid, otherwise an error
    fn validate(&self) -> Result<()>;

    /// Checks if this message is a TAP message.
    fn is_tap_message(&self) -> bool;

    /// Gets the TAP message type from this message.
    fn get_tap_type(&self) -> Option<String>;

    /// Extracts the message body as a specific TAP body type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The TAP message body type to extract
    ///
    /// # Returns
    ///
    /// The message body if the type matches T, otherwise an error
    fn body_as<T: TapMessageBody>(&self) -> Result<T>;
}

// Implement TapMessage trait for didcomm::Message
impl TapMessage for Message {
    fn validate(&self) -> Result<()> {
        // Check if it's a TAP message first
        if !self.is_tap_message() {
            return Err(Error::Validation("Not a TAP message".to_string()));
        }
        
        // Check ID
        if self.id.is_empty() {
            return Err(Error::Validation("Message must have a non-empty ID".to_string()));
        }
        
        // Check created time
        if self.created_time.is_none() {
            return Err(Error::Validation("Message must have a created timestamp".to_string()));
        }
        
        // Body validation is type-specific and handled during body extraction
        Ok(())
    }
    
    fn is_tap_message(&self) -> bool {
        self.type_.starts_with("https://tap.rsvp/schema/1.0#")
    }
    
    fn get_tap_type(&self) -> Option<String> {
        if self.is_tap_message() {
            Some(self.type_.clone())
        } else {
            None
        }
    }
    
    fn body_as<T: TapMessageBody>(&self) -> Result<T> {
        // Check if the message type matches the expected type
        if self.type_ != T::message_type() {
            return Err(Error::Validation(format!(
                "Message type mismatch. Expected {}, got {}",
                T::message_type(),
                self.type_
            )));
        }
        
        // Extract and deserialize the body
        let body = serde_json::from_value(self.body.clone())
            .map_err(|e| Error::SerializationError(format!("Failed to deserialize message body: {}", e)))?;
        
        Ok(body)
    }
}

/// Creates a new TAP message from a message body.
///
/// This function constructs a DIDComm message with the appropriate
/// TAP message type and body.
///
/// # Arguments
///
/// * `body` - The message body to include
/// * `id` - Optional message ID (will be generated if None)
/// * `from_did` - Optional sender DID
/// * `to_dids` - Recipient DIDs
///
/// # Returns
///
/// A DIDComm message object ready for further processing
pub async fn create_tap_message<T: TapMessageBody>(
    body: &T,
    id: Option<String>,
    from_did: Option<&str>,
    to_dids: &[&str],
) -> Result<Message> {
    // Convert the message body to a DIDComm message
    let mut message = body.to_didcomm()?;
    
    // Set custom ID if provided
    if let Some(custom_id) = id {
        message.id = custom_id;
    }
    
    // Set the sender if provided
    if let Some(from) = from_did {
        message.from = Some(from.to_string());
    }
    
    // Set the recipients if provided
    if !to_dids.is_empty() {
        message.to = Some(to_dids.iter().map(|&s| s.to_string()).collect());
    }
    
    Ok(message)
}
