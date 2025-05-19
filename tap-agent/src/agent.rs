//! TAP Agent implementation.
//!
//! This module provides the core Agent functionality for the TAP Protocol:
//! - The `Agent` trait defining core capabilities
//! - The `DefaultAgent` implementation of the trait
//! - Functions for sending and receiving TAP messages with DIDComm

use crate::config::AgentConfig;
use crate::crypto::MessagePacker;
use crate::error::{Error, Result};
use crate::message::SecurityMode;
use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;
use tap_msg::message::tap_message_trait::TapMessageBody;

/// Result of a message delivery attempt to a service endpoint
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    /// The DID that was the target of the delivery
    pub did: String,
    /// The service endpoint URL that was used for delivery
    pub endpoint: String,
    /// The HTTP status code of the delivery
    pub status: Option<u16>,
    /// Error message, if the delivery failed
    pub error: Option<String>,
}

/// A trait for agents that can send and receive TAP messages
///
/// This trait defines the core capabilities of a TAP Agent, including
/// managing identity, sending messages, and receiving messages.
#[async_trait]
pub trait Agent: Debug + Sync + Send {
    /// Get the agent's DID
    ///
    /// Returns the Decentralized Identifier (DID) that identifies this agent
    fn get_agent_did(&self) -> &str;

    /// Send a TAP message to one or more recipients
    ///
    /// This unified method handles:
    /// 1. Serializing the message
    /// 2. Determining appropriate security mode
    /// 3. Packing the message with DIDComm for all recipients
    /// 4. Optionally delivering the message to recipients' service endpoints
    /// 5. Logging the plaintext and packed messages
    ///
    /// # Parameters
    /// * `message` - The message to send, implementing TapMessageBody
    /// * `to` - A vector of recipient DIDs
    /// * `deliver` - Whether to automatically deliver the message to service endpoints
    ///
    /// # Returns
    /// A result containing the packed message and delivery results (if requested)
    async fn send_message<T: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &T,
        to: Vec<&str>,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)>;

    /// Find the service endpoint for a recipient DID
    ///
    /// This method looks up the DID document for the recipient and
    /// extracts the DIDCommMessaging service endpoint if available
    ///
    /// # Parameters
    /// * `to` - The DID of the recipient
    ///
    /// # Returns
    /// The service endpoint URL or None if not found
    async fn get_service_endpoint(&self, to: &str) -> Result<Option<String>>;

    /// Receive and unpack a TAP message
    ///
    /// This method handles:
    /// 1. Unpacking the DIDComm message
    /// 2. Validating the message type
    /// 3. Deserializing to the requested type
    ///
    /// # Parameters
    /// * `packed_message` - The packed message as received
    ///
    /// # Returns
    /// The unpacked message deserialized to type T
    async fn receive_message<T: TapMessageBody + DeserializeOwned + Send>(
        &self,
        packed_message: &str,
    ) -> Result<T>;
}

// Add a compatibility shim for existing code
#[async_trait]
impl<T: Agent + ?Sized> Agent for Arc<T> {
    fn get_agent_did(&self) -> &str {
        (**self).get_agent_did()
    }

    async fn send_message<U: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &U,
        to: Vec<&str>,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)> {
        (**self).send_message(message, to, deliver).await
    }

    async fn receive_message<U: TapMessageBody + DeserializeOwned + Send>(
        &self,
        packed_message: &str,
    ) -> Result<U> {
        (**self).receive_message(packed_message).await
    }

    async fn get_service_endpoint(&self, to: &str) -> Result<Option<String>> {
        (**self).get_service_endpoint(to).await
    }
}

/// Default implementation of the Agent trait
///
/// This implementation provides the standard TAP Agent functionality
/// using DIDComm for secure message exchange.
#[derive(Debug)]
pub struct DefaultAgent {
    /// Configuration for the agent
    config: AgentConfig,
    /// Message packer for handling DIDComm message packing/unpacking
    message_packer: Arc<dyn MessagePacker>,
    /// HTTP client for sending messages to endpoints
    http_client: Client,
}

impl DefaultAgent {
    /// Create a new DefaultAgent with the given configuration and message packer
    ///
    /// # Parameters
    /// * `config` - The agent configuration
    /// * `message_packer` - The message packer for DIDComm operations
    ///
    /// # Returns
    /// A new DefaultAgent instance
    pub fn new(config: AgentConfig, message_packer: Arc<dyn MessagePacker>) -> Self {
        Self {
            config,
            message_packer,
            http_client: Client::new(),
        }
    }

    /// Create a new DefaultAgent with a specific HTTP client
    ///
    /// # Parameters
    /// * `config` - The agent configuration
    /// * `message_packer` - The message packer for DIDComm operations
    /// * `http_client` - HTTP client to use for sending messages
    ///
    /// # Returns
    /// A new DefaultAgent instance
    pub fn new_with_client(
        config: AgentConfig,
        message_packer: Arc<dyn MessagePacker>,
        http_client: Client,
    ) -> Self {
        Self {
            config,
            message_packer,
            http_client,
        }
    }

    /// Convenience method to get a service endpoint for a DID
    ///
    /// This is a public wrapper around the get_service_endpoint trait method
    ///
    /// # Parameters
    /// * `did` - The DID to look up the service endpoint for
    ///
    /// # Returns
    /// The service endpoint URL or None if not found
    pub async fn get_did_service_endpoint(&self, did: &str) -> Result<Option<String>> {
        self.get_service_endpoint(did).await
    }

    /// Create a new DefaultAgent with ephemeral did:key and default message packer
    ///
    /// This creates an agent with an Ed25519 did:key that is not persisted.
    /// Useful for testing or short-lived agents.
    ///
    /// # Returns
    /// A tuple containing the new DefaultAgent instance and the generated DID
    pub fn new_ephemeral() -> crate::error::Result<(Self, String)> {
        // Create a key manager
        let key_manager = crate::key_manager::KeyManager::new();

        // Generate an Ed25519 key
        let options = crate::did::DIDGenerationOptions {
            key_type: crate::did::KeyType::Ed25519,
        };

        let key = key_manager.generate_key(options)?;

        // Create a DID resolver
        let did_resolver = Arc::new(crate::did::MultiResolver::default());

        // Create a basic secret resolver for the key
        let mut secret_resolver = crate::crypto::BasicSecretResolver::new();
        let secret = key_manager.generator.create_secret_from_key(&key);
        secret_resolver.add_secret(&key.did, secret);

        // Create a message packer
        let message_packer = Arc::new(crate::crypto::DefaultMessagePacker::new(
            did_resolver,
            Arc::new(secret_resolver),
        ));

        // Create agent configuration with empty parameters
        let config = AgentConfig {
            agent_did: key.did.clone(),
            parameters: std::collections::HashMap::new(),
            security_mode: Some("SIGNED".to_string()),
        };

        // Create the agent
        let agent = Self::new(config, message_packer);

        Ok((agent, key.did))
    }

    /// Send a packed message to a service endpoint via HTTP POST
    ///
    /// # Parameters
    /// * `packed_message` - The packed DIDComm message
    /// * `endpoint` - The service endpoint URL
    ///
    /// # Returns
    /// The HTTP response status code, or error if the request failed
    pub async fn send_to_endpoint(&self, packed_message: &str, endpoint: &str) -> Result<u16> {
        // Send the message to the endpoint via HTTP POST
        let response = self
            .http_client
            .post(endpoint)
            .header("Content-Type", "application/didcomm-encrypted+json")
            .body(packed_message.to_string())
            .send()
            .await
            .map_err(|e| Error::Networking(format!("Failed to send message to endpoint: {}", e)))?;

        // Get the status code
        let status = response.status().as_u16();

        // Log the response status
        println!("Message sent to endpoint {}, status: {}", endpoint, status);

        Ok(status)
    }

    /// Determine the appropriate security mode for a message type
    ///
    /// This method implements TAP protocol rules for which security modes
    /// should be used with different message types:
    /// - Presentation messages use authenticated encryption (AuthCrypt)
    /// - All other messages use digital signatures (Signed)
    ///
    /// If security_mode is specified in the agent config, that takes precedence.
    ///
    /// # Parameters
    /// * `message_type` - The type of the message
    ///
    /// # Returns
    /// The appropriate SecurityMode for the message type
    fn determine_security_mode<T: TapMessageBody>(&self) -> SecurityMode {
        // If security mode is explicitly configured, use that
        if let Some(ref mode) = self.config.security_mode {
            if mode.to_uppercase() == "AUTHCRYPT" {
                return SecurityMode::AuthCrypt;
            } else {
                // Default to Signed for any other value
                return SecurityMode::Signed;
            }
        }

        // Otherwise use type-based rules
        let message_type = T::message_type();
        if message_type == crate::message::PRESENTATION_MESSAGE_TYPE {
            SecurityMode::AuthCrypt
        } else {
            SecurityMode::Signed
        }
    }
}

#[async_trait]
impl Agent for DefaultAgent {
    fn get_agent_did(&self) -> &str {
        &self.config.agent_did
    }

    async fn get_service_endpoint(&self, to: &str) -> Result<Option<String>> {
        // Get the recipient's DID document
        let did_doc = self.message_packer.resolve_did_doc(to).await?;

        // Look for service endpoints
        if let Some(doc) = did_doc {
            // First pass: Look for DIDCommMessaging services specifically
            // Try to find a DIDCommMessaging service first
            if let Some(service) = doc.service.iter().find(|s| s.type_ == "DIDCommMessaging") {
                // For DIDCommMessaging, return the service_endpoint directly
                return Ok(Some(service.service_endpoint.clone()));
            }

            // If no DIDCommMessaging service found, look for other service types
            // If no DIDCommMessaging service found, look for any service endpoint
            if let Some(service) = doc.service.first() {
                // Use the service endpoint directly for any service type
                return Ok(Some(service.service_endpoint.clone()));
            }
        }

        // No service endpoint found
        Ok(None)
    }

    async fn send_message<T: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &T,
        to: Vec<&str>,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)> {
        if to.is_empty() {
            return Err(Error::Validation("No recipients specified".to_string()));
        }

        // Create the message object with proper type
        let mut message_obj = serde_json::to_value(message)
            .map_err(|e| Error::Serialization(format!("Failed to serialize message: {}", e)))?;

        // Ensure message has a type field
        if message_obj.get("type").is_none() {
            if let serde_json::Value::Object(ref mut obj) = message_obj {
                obj.insert(
                    "type".to_string(),
                    serde_json::Value::String(T::message_type().to_string()),
                );
            }
        }

        // Log the plaintext message with clear formatting
        println!("\n==== SENDING TAP MESSAGE ====");
        println!("Message Type: {}", T::message_type());
        println!("Recipients: {:?}", to);
        println!(
            "--- PLAINTEXT CONTENT ---\n{}",
            serde_json::to_string_pretty(&message_obj).unwrap_or_else(|_| message_obj.to_string())
        );
        println!("-------------------------");

        // Validate the message
        message.validate().map_err(|e| {
            Error::Validation(format!(
                "Message validation failed for type {}: {}",
                T::message_type(),
                e
            ))
        })?;

        // Determine the appropriate security mode
        let security_mode = self.determine_security_mode::<T>();
        println!("Security Mode: {:?}", security_mode);

        // For each recipient, look up service endpoint before sending (only for logging, not delivery)
        for recipient in &to {
            if let Ok(Some(endpoint)) = self.get_service_endpoint(recipient).await {
                // Log the found service endpoint - this can be helpful for debugging
                println!("Found service endpoint for {}: {}", recipient, endpoint);
            }
        }

        // Use message packer to pack the message for all recipients
        let packed = self
            .message_packer
            .pack_message(&message_obj, &to, Some(self.get_agent_did()), security_mode)
            .await?;

        // Log the packed message with clear separation and formatting
        println!("--- PACKED MESSAGE ---");
        println!(
            "{}",
            serde_json::from_str::<serde_json::Value>(&packed)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or(packed.clone()))
                .unwrap_or(packed.clone())
        );
        println!("=====================");

        // If delivery is not requested, just return the packed message
        if !deliver {
            return Ok((packed, Vec::new()));
        }

        // Try to deliver the message to each recipient's service endpoint
        let mut delivery_results = Vec::new();

        for recipient in &to {
            match self.get_service_endpoint(recipient).await {
                Ok(Some(endpoint)) => {
                    println!("Delivering message to {} at {}", recipient, endpoint);

                    // Extract message ID for logging
                    let message_id = match serde_json::from_str::<serde_json::Value>(&packed) {
                        Ok(json) => json
                            .get("id")
                            .and_then(|id| id.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| "unknown".to_string()),
                        Err(_) => "unknown".to_string(),
                    };

                    // Attempt to deliver the message
                    match self.send_to_endpoint(&packed, &endpoint).await {
                        Ok(status) => {
                            // Log success with clear formatting
                            println!(
                                "✅ Delivered message {} to {} at {}",
                                message_id, recipient, endpoint
                            );

                            delivery_results.push(DeliveryResult {
                                did: recipient.to_string(),
                                endpoint: endpoint.clone(),
                                status: Some(status),
                                error: None,
                            });
                        }
                        Err(e) => {
                            // Log error with clear formatting but don't fail
                            let error_msg = format!(
                                "Failed to deliver message {} to {} at {}: {}",
                                message_id, recipient, endpoint, e
                            );
                            println!("❌ {}", error_msg);

                            delivery_results.push(DeliveryResult {
                                did: recipient.to_string(),
                                endpoint: endpoint.clone(),
                                status: None,
                                error: Some(error_msg),
                            });
                        }
                    }
                }
                Ok(None) => {
                    // Log with clear formatting but don't add an error result
                    println!(
                        "⚠️ No service endpoint found for {}, skipping delivery",
                        recipient
                    );
                }
                Err(e) => {
                    // Log error with clear formatting but don't fail
                    let error_msg = format!(
                        "Failed to resolve service endpoint for {}: {}",
                        recipient, e
                    );
                    println!("❌ {}", error_msg);
                }
            }
        }

        Ok((packed, delivery_results))
    }

    // Main send_message implementation handles all cases

    async fn receive_message<T: TapMessageBody + DeserializeOwned + Send>(
        &self,
        packed_message: &str,
    ) -> Result<T> {
        // Log the received packed message with clear formatting
        println!("\n==== RECEIVING TAP MESSAGE ====");
        println!("--- PACKED MESSAGE ---");
        println!(
            "{}",
            serde_json::from_str::<serde_json::Value>(packed_message)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or(packed_message.to_string()))
                .unwrap_or(packed_message.to_string())
        );
        println!("---------------------");

        // Unpack the message
        let message_value: Value = self
            .message_packer
            .unpack_message_value(packed_message)
            .await?;

        // Log the unpacked message value with clear formatting
        println!("--- UNPACKED CONTENT ---");
        println!(
            "{}",
            serde_json::to_string_pretty(&message_value)
                .unwrap_or_else(|_| message_value.to_string())
        );
        println!("------------------------");

        // Get the message type from the unpacked message
        let message_type = message_value
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| Error::Validation("Message missing 'type' field".to_string()))?;

        // Validate the message type
        if message_type != T::message_type() {
            println!(
                "❌ Message type validation failed: expected {}, got {}",
                T::message_type(),
                message_type
            );
            return Err(Error::Validation(format!(
                "Expected message type {} but got {}",
                T::message_type(),
                message_type
            )));
        }
        println!("✅ Message type validation passed: {}", message_type);

        // Check if we need to convert to a DIDComm message first
        if let Some(id) = message_value.get("id") {
            // This appears to be a proper DIDComm message already
            // Create a DIDComm message with the required fields
            let didcomm_message = tap_msg::didcomm::PlainMessage {
                id: id.as_str().unwrap_or("").to_string(),
                typ: "application/didcomm-plain+json".to_string(),
                type_: message_type.to_string(),
                // Use the entire message as the body, not just the "body" field
                body: message_value.clone(),
                from: message_value
                    .get("from")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                to: message_value
                    .get("to")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                thid: None,
                pthid: None,
                extra_headers: Default::default(),
                created_time: None,
                expires_time: None,
                from_prior: None,
                attachments: None,
            };

            // Convert to the requested type using the TapMessageBody trait
            let message = T::from_didcomm(&didcomm_message)
                .map_err(|e| Error::Validation(format!("Failed to convert message: {}", e)))?;

            // Validate the message
            match message.validate() {
                Ok(_) => {
                    println!("✅ Message content validation passed");
                    println!("==== MESSAGE PROCESSING COMPLETE ====\n");
                    Ok(message)
                }
                Err(e) => {
                    println!("❌ Message content validation failed: {}", e);
                    Err(Error::Validation(format!(
                        "Message validation failed for type {}: {}",
                        T::message_type(),
                        e
                    )))
                }
            }
        } else {
            // This might be just the message body directly, try to deserialize
            let message = serde_json::from_value::<T>(message_value).map_err(|e| {
                Error::Serialization(format!("Failed to deserialize message: {}", e))
            })?;

            // Validate the message
            match message.validate() {
                Ok(_) => {
                    println!("✅ Message content validation passed");
                    println!("==== MESSAGE PROCESSING COMPLETE ====\n");
                    Ok(message)
                }
                Err(e) => {
                    println!("❌ Message content validation failed: {}", e);
                    Err(Error::Validation(format!(
                        "Message validation failed for type {}: {}",
                        T::message_type(),
                        e
                    )))
                }
            }
        }
    }
}
