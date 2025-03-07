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

    /// Convert this body to a DIDComm message.
    fn to_didcomm(&self) -> Result<Message> {
        // Create a JSON representation of self with explicit type field
        let mut body_json =
            serde_json::to_value(self).map_err(|e| Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field with the message type
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        let now = get_current_time()?;

        // Create a new Message with required fields
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
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
    ///
    /// According to TAP requirements:
    /// - The `from` field should always be the creator's DID
    /// - The `to` field should include DIDs of all agents involved except the creator
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

    /// Create a reply message to an existing message.
    ///
    /// This method helps create a response that maintains thread correlation with the original message.
    /// According to TAP requirements:
    /// - Responses should include DIDs of all agents involved except for the creator of the response
    /// - The thread ID should be preserved to maintain message correlation
    ///
    /// # Arguments
    ///
    /// * `original` - The original message to reply to
    /// * `creator_did` - DID of the creator of this reply (will be set as the `from` value)
    /// * `participant_dids` - DIDs of all participants in the thread
    ///                       (creator_did will be filtered out automatically for the `to` field)
    ///
    /// # Returns
    ///
    /// A new DIDComm message that is properly linked to the original message
    fn create_reply(
        &self,
        original: &Message,
        creator_did: &str,
        participant_dids: &[&str],
    ) -> Result<Message> {
        let mut message = self.to_didcomm()?;

        // Set the thread ID to maintain the conversation thread
        if let Some(thread_id) = original.thid.as_ref() {
            message.thid = Some(thread_id.clone());
        } else {
            // If no thread ID exists, use the original message ID as the thread ID
            message.thid = Some(original.id.clone());
        }

        // Set the parent thread ID if this thread is part of a larger transaction
        if let Some(parent_thread_id) = original.pthid.as_ref() {
            message.pthid = Some(parent_thread_id.clone());
        }

        // Set the creator as the sender
        message.from = Some(creator_did.to_string());

        // Set recipients to all participants except the creator
        let recipients: Vec<String> = participant_dids
            .iter()
            .filter(|&&did| did != creator_did)
            .map(|&did| did.to_string())
            .collect();

        if !recipients.is_empty() {
            message.to = Some(recipients);
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

        // Create a copy of the message body that we can modify
        let mut body_json = message.body.clone();

        // Ensure the @type field is present for deserialization
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field to ensure it's present
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Extract and deserialize the body
        let body = serde_json::from_value(body_json).map_err(|e| {
            Error::SerializationError(format!("Failed to deserialize message body: {}", e))
        })?;

        Ok(body)
    }
}

/// Trait for types that can be represented as TAP messages.
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

    /// Extract a specific message body type from this message.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the body to extract, must implement `TapMessageBody`
    ///
    /// # Returns
    ///
    /// The message body if the type matches T, otherwise an error
    fn body_as<T: TapMessageBody>(&self) -> Result<T>;

    /// Get all participant DIDs from this message.
    ///
    /// This includes all DIDs that are involved in the message, such as sender, recipients, and
    /// any other participants mentioned in the message body.
    ///
    /// # Returns
    ///
    /// List of participant DIDs
    fn get_all_participants(&self) -> Vec<String>;

    /// Create a reply to this message.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The TAP message body type to create
    ///
    /// # Arguments
    ///
    /// * `body` - The reply message body
    /// * `creator_did` - DID of the creator of this reply
    ///
    /// # Returns
    ///
    /// A new DIDComm message that is properly linked to this message as a reply
    fn create_reply<T: TapMessageBody>(&self, body: &T, creator_did: &str) -> Result<Message>;
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
            return Err(Error::Validation(
                "Message must have a non-empty ID".to_string(),
            ));
        }

        // Check created time
        if self.created_time.is_none() {
            return Err(Error::Validation(
                "Message must have a created timestamp".to_string(),
            ));
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

        // Create a copy of the body that we can modify
        let mut body_json = self.body.clone();

        // Debug: Print the body JSON before modification
        println!(
            "DEBUG: Body JSON before: {}",
            serde_json::to_string_pretty(&body_json).unwrap()
        );

        // Ensure the @type field is present for deserialization
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field to ensure it's present
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(T::message_type().to_string()),
            );
        }

        // Debug: Print the body JSON after modification
        println!(
            "DEBUG: Body JSON after: {}",
            serde_json::to_string_pretty(&body_json).unwrap()
        );

        // Debug: Print the expected struct type
        println!(
            "DEBUG: Deserializing to type: {}",
            std::any::type_name::<T>()
        );

        // Try direct string-based deserialization
        let json_str = serde_json::to_string(&body_json).unwrap();
        println!("DEBUG: JSON string for deserialization: {}", json_str);

        match serde_json::from_str::<T>(&json_str) {
            Ok(body) => {
                println!("DEBUG: String-based deserialization succeeded");
                return Ok(body);
            }
            Err(e) => {
                println!("DEBUG: String-based deserialization failed: {}", e);
                // Fall through to try the value-based approach
            }
        }

        // Extract and deserialize the body using value-based approach
        match serde_json::from_value::<T>(body_json) {
            Ok(body) => {
                println!("DEBUG: Value-based deserialization succeeded");
                Ok(body)
            }
            Err(e) => {
                println!("DEBUG: Value-based deserialization failed: {}", e);
                Err(Error::SerializationError(format!(
                    "Failed to deserialize message body: {}",
                    e
                )))
            }
        }
    }

    fn get_all_participants(&self) -> Vec<String> {
        let mut participants = Vec::new();

        // Add sender if present
        if let Some(from) = &self.from {
            participants.push(from.clone());
        }

        // Add recipients if present
        if let Some(to) = &self.to {
            participants.extend(to.clone());
        }

        participants
    }

    fn create_reply<T: TapMessageBody>(&self, body: &T, creator_did: &str) -> Result<Message> {
        // Get all participants from this message
        let all_participants = self.get_all_participants();

        // Convert to &str for the body.create_reply call
        let participant_refs: Vec<&str> = all_participants.iter().map(AsRef::as_ref).collect();

        // Create the reply using the body's method
        body.create_reply(self, creator_did, &participant_refs)
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
pub fn create_tap_message<T: TapMessageBody>(
    body: &T,
    id: Option<String>,
    from_did: Option<&str>,
    to_dids: &[&str],
) -> Result<Message> {
    // Create the base message from the body
    let mut message = body.to_didcomm()?;

    // Set custom ID if provided
    if let Some(custom_id) = id {
        message.id = custom_id;
    }

    // Set sender and recipients
    if let Some(sender) = from_did {
        message.from = Some(sender.to_string());
    }

    if !to_dids.is_empty() {
        message.to = Some(to_dids.iter().map(|&s| s.to_string()).collect());
    }

    Ok(message)
}
