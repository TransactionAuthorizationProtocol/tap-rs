//! Traits for TAP message conversion and validation.
//!
//! This module provides traits for converting between DIDComm messages
//! and TAP-specific message bodies, as well as validation of those bodies.

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::message::policy::Policy;
use crate::message::{
    AddAgents, Authorize, Cancel, ConfirmRelationship, Participant, Reject, RemoveAgent,
    ReplaceAgent, Revert, Settle, UpdateParty, UpdatePolicies,
};
use chrono::Utc;
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
    fn to_didcomm(&self, from: &str) -> Result<PlainMessage> {
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
        agent_dids.retain(|did| did != from);

        // Create the message
        let message = PlainMessage {
            id,
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            from: from.to_string(),
            to: agent_dids,
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

    /// Convert this body to a DIDComm message with a custom routing path.
    ///
    /// This method allows specifying an explicit list of recipient DIDs, overriding
    /// the automatic extraction of participants from the message body.
    ///
    /// # Arguments
    ///
    /// * `from` - The sender DID
    /// * `to` - An iterator of recipient DIDs
    ///
    /// # Returns
    ///
    /// A new DIDComm message with the specified routing
    fn to_didcomm_with_route<'a, I>(&self, from: &str, to: I) -> Result<PlainMessage>
    where
        I: IntoIterator<Item = &'a str>,
    {
        // First create a message with the sender, automatically extracting agent DIDs
        let mut message = self.to_didcomm(from)?;

        // Override with explicitly provided recipients if any
        let to_vec: Vec<String> = to.into_iter().map(String::from).collect();
        if !to_vec.is_empty() {
            message.to = to_vec;
        }

        Ok(message)
    }

    /// Extract this body type from a DIDComm message.
    fn from_didcomm(message: &PlainMessage) -> Result<Self>
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
    fn create_reply<T: TapMessageBody>(&self, body: &T, creator_did: &str) -> Result<PlainMessage> {
        // Create the base message with creator as sender
        let mut message = body.to_didcomm(creator_did)?;

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
            message.to = recipients;
        }

        Ok(message)
    }

    /// Get the thread ID for this message
    fn thread_id(&self) -> Option<&str>;

    /// Get the parent thread ID for this message
    fn parent_thread_id(&self) -> Option<&str>;

    /// Get the message ID for this message
    fn message_id(&self) -> &str;
}

// Implement TapMessage trait for PlainMessage
impl TapMessage for PlainMessage {
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

        // Add sender
        if !self.from.is_empty() {
            participants.push(self.from.clone());
        }

        // Add recipients
        participants.extend(self.to.clone());

        participants
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

// Implement Connectable trait for PlainMessage
impl Connectable for PlainMessage {
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

/// Helper function to convert an untyped PlainMessage to a typed PlainMessage.
///
/// This is used internally to convert the result of create_reply to a typed message.
pub fn typed_plain_message<T: TapMessageBody>(reply: PlainMessage, body: T) -> PlainMessage<T> {
    PlainMessage {
        id: reply.id,
        typ: reply.typ,
        type_: reply.type_,
        body,
        from: reply.from,
        to: reply.to,
        thid: reply.thid,
        pthid: reply.pthid,
        extra_headers: reply.extra_headers,
        created_time: reply.created_time,
        expires_time: reply.expires_time,
        from_prior: reply.from_prior,
        attachments: reply.attachments,
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
/// * `from_did` - Sender DID
/// * `to_dids` - Recipient DIDs (will override automatically extracted agent DIDs)
///
/// # Returns
///
/// A DIDComm message object ready for further processing
pub fn create_tap_message<T: TapMessageBody>(
    body: &T,
    id: Option<String>,
    from_did: &str,
    to_dids: &[&str],
) -> Result<PlainMessage> {
    // Create the base message from the body, passing the from_did
    let mut message = body.to_didcomm(from_did)?;

    // Set custom ID if provided
    if let Some(custom_id) = id {
        message.id = custom_id;
    }

    // Override with explicitly provided recipients if any
    if !to_dids.is_empty() {
        message.to = to_dids.iter().map(|&s| s.to_string()).collect();
    }

    Ok(message)
}

/// Authorizable trait for TAIP-4 authorization messages.
///
/// This trait provides methods for creating authorization-related messages
/// as defined in TAIP-4, excluding the Settle message which is handled
/// separately in the Transaction trait.
pub trait Authorizable: TapMessage {
    /// Create an Authorize message for this object (TAIP-4).
    ///
    /// # Arguments
    /// * `creator_did` - The DID of the agent creating this authorization
    /// * `settlement_address` - Optional settlement address in CAIP-10 format
    /// * `expiry` - Optional expiry timestamp in ISO 8601 format
    /// * `note` - Optional note
    fn authorize(
        &self,
        creator_did: &str,
        settlement_address: Option<&str>,
        expiry: Option<&str>,
        note: Option<&str>,
    ) -> PlainMessage<Authorize>;

    /// Create a Cancel message for this object (TAIP-4).
    ///
    /// # Arguments
    /// * `creator_did` - The DID of the agent creating this cancellation
    /// * `by` - The party wishing to cancel (e.g., "originator" or "beneficiary")
    /// * `reason` - Optional reason for cancellation
    fn cancel(&self, creator_did: &str, by: &str, reason: Option<&str>) -> PlainMessage<Cancel>;

    /// Create a Reject message for this object (TAIP-4).
    ///
    /// # Arguments
    /// * `creator_did` - The DID of the agent creating this rejection
    /// * `reason` - Reason for rejection
    fn reject(&self, creator_did: &str, reason: &str) -> PlainMessage<Reject>;
}

/// Transaction trait for managing transaction lifecycle operations.
///
/// This trait provides methods for transaction processing operations
/// as defined in TAIPs 5-9, including agent management, party updates,
/// policy management, and settlement.
pub trait Transaction: TapMessage {
    /// Create a Settle message for this object (TAIP-4).
    ///
    /// # Arguments
    /// * `creator_did` - The DID of the agent creating this settlement
    /// * `settlement_id` - CAIP-220 identifier of the underlying settlement transaction
    /// * `amount` - Optional amount settled (must be <= original amount)
    fn settle(
        &self,
        creator_did: &str,
        settlement_id: &str,
        amount: Option<&str>,
    ) -> PlainMessage<Settle>;

    /// Create a Revert message for this object (TAIP-4).
    ///
    /// # Arguments
    /// * `creator_did` - The DID of the agent creating this reversal request
    /// * `settlement_address` - CAIP-10 format address to return funds to
    /// * `reason` - Reason for reversal request
    fn revert(
        &self,
        creator_did: &str,
        settlement_address: &str,
        reason: &str,
    ) -> PlainMessage<Revert>;

    /// Create an AddAgents message for this object (TAIP-5).
    ///
    /// # Arguments
    /// * `creator_did` - The DID of the agent creating this addition
    /// * `agents` - List of agents to add
    fn add_agents(&self, creator_did: &str, agents: Vec<Participant>) -> PlainMessage<AddAgents>;

    /// Create a ReplaceAgent message for this object (TAIP-5).
    ///
    /// # Arguments
    /// * `creator_did` - The DID of the agent creating this replacement
    /// * `original_agent` - The agent DID to replace
    /// * `replacement` - The replacement agent participant
    fn replace_agent(
        &self,
        creator_did: &str,
        original_agent: &str,
        replacement: Participant,
    ) -> PlainMessage<ReplaceAgent>;

    /// Create a RemoveAgent message for this object (TAIP-5).
    ///
    /// # Arguments
    /// * `creator_did` - The DID of the agent creating this removal
    /// * `agent` - The agent DID to remove
    fn remove_agent(&self, creator_did: &str, agent: &str) -> PlainMessage<RemoveAgent>;

    /// Create an UpdateParty message for this object (TAIP-6).
    ///
    /// # Arguments
    /// * `creator_did` - The DID of the agent creating this update
    /// * `party_type` - The type of party ("originator", "beneficiary", etc.)
    /// * `party` - The party information to update
    fn update_party(
        &self,
        creator_did: &str,
        party_type: &str,
        party: Participant,
    ) -> PlainMessage<UpdateParty>;

    /// Create an UpdatePolicies message for this object (TAIP-7).
    ///
    /// # Arguments
    /// * `creator_did` - The DID of the agent creating this update
    /// * `policies` - New policies to apply
    fn update_policies(
        &self,
        creator_did: &str,
        policies: Vec<Policy>,
    ) -> PlainMessage<UpdatePolicies>;

    /// Create a ConfirmRelationship message for this object (TAIP-9).
    ///
    /// # Arguments
    /// * `creator_did` - The DID of the agent creating this confirmation
    /// * `agent_did` - The agent DID confirming their relationship
    /// * `relationship_type` - The type of relationship being confirmed
    fn confirm_relationship(
        &self,
        creator_did: &str,
        agent_did: &str,
        relationship_type: &str,
    ) -> PlainMessage<ConfirmRelationship>;
}
