//! Traits for TAP message conversion and validation.
//!
//! This module provides traits for converting between DIDComm messages
//! and TAP-specific message bodies, as well as validation of those bodies.

use crate::error::{Error, Result};
use chrono::Utc;
use didcomm::Message;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// A trait for TAP message body types that can be serialized to and deserialized from DIDComm messages.
pub trait TapMessageBody: Serialize + DeserializeOwned + Send + Sync {
    /// Get the message type string for this body type.
    fn message_type() -> &'static str
    where
        Self: Sized;

    /// Validate the message body.
    fn validate(&self) -> Result<()>;

    /// Convert this body to a DIDComm message.
    fn to_didcomm(&self, from_did: Option<&str>) -> Result<Message> {
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

        // Create a unique ID for the message
        let id = uuid::Uuid::new_v4().to_string();

        // Get current timestamp in milliseconds since Unix epoch
        let now = Utc::now().timestamp_millis() as u64;

        // Extract agent DIDs directly from the message body
        let mut agent_dids = Vec::new();

        // Extract from the body JSON
        if let Some(body_obj) = body_json.as_object() {
            // Extract from originator
            if let Some(originator) = body_obj.get("originator") {
                if let Some(originator_obj) = originator.as_object() {
                    if let Some(id) = originator_obj.get("id") {
                        if let Some(id_str) = id.as_str() {
                            if id_str.starts_with("did:") {
                                agent_dids.push(id_str.to_string());
                            }
                        }
                    }
                }
            }

            // Extract from beneficiary
            if let Some(beneficiary) = body_obj.get("beneficiary") {
                if let Some(beneficiary_obj) = beneficiary.as_object() {
                    if let Some(id) = beneficiary_obj.get("id") {
                        if let Some(id_str) = id.as_str() {
                            if id_str.starts_with("did:") {
                                agent_dids.push(id_str.to_string());
                            }
                        }
                    }
                }
            }

            // Extract from agents array
            if let Some(agents) = body_obj.get("agents") {
                if let Some(agents_array) = agents.as_array() {
                    for agent in agents_array {
                        if let Some(agent_obj) = agent.as_object() {
                            if let Some(id) = agent_obj.get("id") {
                                if let Some(id_str) = id.as_str() {
                                    if id_str.starts_with("did:") {
                                        agent_dids.push(id_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove duplicates
        agent_dids.sort();
        agent_dids.dedup();

        // If from_did is provided, remove it from the recipients list to avoid sending to self
        if let Some(from) = from_did {
            agent_dids.retain(|did| did != from);
        }

        // Always set the 'to' field, even if it's an empty list
        let to = Some(agent_dids);

        // Create the message
        let message = Message {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from_did.map(|s| s.to_string()),
            to,
            thid: None,
            pthid: None,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            body: body_json,
            attachments: None,
        };

        Ok(message)
    }

    /// Convert this body to a DIDComm message with a sender and multiple recipients.
    ///
    /// According to TAP requirements:
    /// - The `from` field should always be the creator's DID
    /// - The `to` field should include DIDs of all agents involved except the creator
    ///
    /// Note: This method now directly uses the enhanced to_didcomm implementation
    /// which automatically extracts agent DIDs. The explicit 'to' parameter allows
    /// overriding the automatically extracted recipients when needed.
    #[allow(dead_code)] // Used in tests but not in production code
    fn to_didcomm_with_route<'a, I>(&self, from: Option<&str>, to: I) -> Result<Message>
    where
        I: IntoIterator<Item = &'a str>,
    {
        // First create a message with the sender, automatically extracting agent DIDs
        let mut message = self.to_didcomm(from)?;

        // Override with explicitly provided recipients if any
        let to_vec: Vec<String> = to.into_iter().map(String::from).collect();
        if !to_vec.is_empty() {
            message.to = Some(to_vec);
        }

        Ok(message)
    }

    /// Extract this body type from a DIDComm message.
    fn from_didcomm(message: &Message) -> Result<Self>
    where
        Self: Sized,
    {
        // Verify that this is the correct message type
        if message.type_ != Self::message_type() {
            return Err(Error::InvalidMessageType(format!(
                "Expected message type {}, but found {}",
                Self::message_type(),
                message.type_
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

/// A trait for messages that can be connected to a prior Connect message.
///
/// This trait provides functionality for linking messages to a previous Connect message,
/// enabling the building of message chains in the TAP protocol.
pub trait Connectable {
    /// Connect this message to a prior Connect message by setting the parent thread ID (pthid).
    ///
    /// # Arguments
    ///
    /// * `connect_id` - The ID of the Connect message to link to
    ///
    /// # Returns
    ///
    /// Self reference for fluent interface chaining
    fn with_connection(&mut self, connect_id: &str) -> &mut Self;

    /// Check if this message is connected to a prior Connect message.
    ///
    /// # Returns
    ///
    /// `true` if this message has a connection (pthid) set, `false` otherwise
    fn has_connection(&self) -> bool;

    /// Get the connection ID if present.
    ///
    /// # Returns
    ///
    /// The Connect message ID this message is connected to, or None if not connected
    fn connection_id(&self) -> Option<&str>;
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
    fn create_reply<T: TapMessageBody>(&self, body: &T, creator_did: &str) -> Result<Message> {
        // Create the base message with creator as sender
        let mut message = body.to_didcomm(Some(creator_did))?;

        // Set the thread ID to maintain the conversation thread
        if let Some(thread_id) = self.thread_id() {
            message.thid = Some(thread_id.to_string());
        } else {
            // If no thread ID exists, use the original message ID as the thread ID
            message.thid = Some(self.message_id().to_string());
        }

        // Set the parent thread ID if this thread is part of a larger transaction
        if let Some(parent_thread_id) = self.parent_thread_id() {
            message.pthid = Some(parent_thread_id.to_string());
        }

        // Get all participants from the message
        let participant_dids = self.get_all_participants();

        // Set recipients to all participants except the creator
        let recipients: Vec<String> = participant_dids
            .into_iter()
            .filter(|did| did != creator_did)
            .collect();

        if !recipients.is_empty() {
            message.to = Some(recipients);
        }

        Ok(message)
    }

    /// Get the message type for this message
    fn message_type(&self) -> &'static str;

    /// Get the thread ID for this message
    fn thread_id(&self) -> Option<&str>;

    /// Get the parent thread ID for this message
    fn parent_thread_id(&self) -> Option<&str>;

    /// Get the message ID for this message
    fn message_id(&self) -> &str;
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

        // Ensure the @type field is present for deserialization
        if let Some(body_obj) = body_json.as_object_mut() {
            // Add or update the @type field to ensure it's present
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(T::message_type().to_string()),
            );
        }

        // Extract and deserialize the body
        let body = serde_json::from_value(body_json).map_err(|e| {
            Error::SerializationError(format!("Failed to deserialize message body: {}", e))
        })?;

        Ok(body)
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

    fn message_type(&self) -> &'static str {
        match self.type_.as_str() {
            "https://tap.rsvp/schema/1.0#transfer" => "https://tap.rsvp/schema/1.0#transfer",
            "https://tap.rsvp/schema/1.0#payment-request" => {
                "https://tap.rsvp/schema/1.0#payment-request"
            }
            "https://tap.rsvp/schema/1.0#connect" => "https://tap.rsvp/schema/1.0#connect",
            "https://tap.rsvp/schema/1.0#authorize" => "https://tap.rsvp/schema/1.0#authorize",
            "https://tap.rsvp/schema/1.0#reject" => "https://tap.rsvp/schema/1.0#reject",
            "https://tap.rsvp/schema/1.0#settle" => "https://tap.rsvp/schema/1.0#settle",
            "https://tap.rsvp/schema/1.0#update-party" => {
                "https://tap.rsvp/schema/1.0#update-party"
            }
            "https://tap.rsvp/schema/1.0#update-policies" => {
                "https://tap.rsvp/schema/1.0#update-policies"
            }
            "https://didcomm.org/present-proof/3.0/presentation" => {
                "https://didcomm.org/present-proof/3.0/presentation"
            }
            _ => "unknown",
        }
    }

    fn thread_id(&self) -> Option<&str> {
        self.thid.as_deref()
    }

    fn parent_thread_id(&self) -> Option<&str> {
        self.pthid.as_deref()
    }

    fn message_id(&self) -> &str {
        &self.id
    }
}

// Implement Connectable trait for Message
impl Connectable for Message {
    fn with_connection(&mut self, connect_id: &str) -> &mut Self {
        self.pthid = Some(connect_id.to_string());
        self
    }

    fn has_connection(&self) -> bool {
        self.pthid.is_some()
    }

    fn connection_id(&self) -> Option<&str> {
        self.pthid.as_deref()
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
/// * `to_dids` - Recipient DIDs (will override automatically extracted agent DIDs)
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
    // Create the base message from the body, passing the from_did
    let mut message = body.to_didcomm(from_did)?;

    // Set custom ID if provided
    if let Some(custom_id) = id {
        message.id = custom_id;
    }

    // Override with explicitly provided recipients if any
    if !to_dids.is_empty() {
        message.to = Some(to_dids.iter().map(|&s| s.to_string()).collect());
    }

    Ok(message)
}
