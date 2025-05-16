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

    /// Send a TAP message to a recipient
    ///
    /// This method handles:
    /// 1. Serializing the message
    /// 2. Determining appropriate security mode 
    /// 3. Packing the message with DIDComm
    ///
    /// # Parameters
    /// * `message` - The message to send, implementing TapMessageBody
    /// * `to` - The DID of the recipient
    ///
    /// # Returns
    /// The packed message as a string
    async fn send_message<T: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &T,
        to: &str,
    ) -> Result<String>;
    
    /// Send a TAP message to a recipient with delivery option
    ///
    /// This method handles:
    /// 1. Serializing the message
    /// 2. Determining appropriate security mode 
    /// 3. Packing the message with DIDComm
    /// 4. Optionally sending the message to service endpoints if available
    ///
    /// # Parameters
    /// * `message` - The message to send, implementing TapMessageBody
    /// * `to` - The DID of the recipient
    /// * `deliver` - Whether to automatically deliver the message to service endpoints if available
    ///
    /// # Returns
    /// A result containing the packed message and a vector of service delivery results
    async fn send_message_with_delivery<T: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &T,
        to: &str,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)>;
    
    /// Send a TAP message to multiple recipients with delivery option
    ///
    /// # Parameters
    /// * `message` - The message to send, implementing TapMessageBody
    /// * `to` - A list of DIDs to send the message to
    /// * `deliver` - Whether to automatically deliver the message to service endpoints if available
    ///
    /// # Returns
    /// A result containing the packed message and a vector of service delivery results
    async fn send_message_to_many<T: TapMessageBody + serde::Serialize + Send + Sync>(
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
        to: &str,
    ) -> Result<String> {
        (**self).send_message(message, to).await
    }
    
    async fn send_message_with_delivery<U: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &U,
        to: &str,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)> {
        (**self).send_message_with_delivery(message, to, deliver).await
    }
    
    async fn send_message_to_many<U: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &U,
        to: Vec<&str>,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)> {
        (**self).send_message_to_many(message, to, deliver).await
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
        let response = self.http_client
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
    /// # Parameters
    /// * `message_type` - The type of the message
    ///
    /// # Returns
    /// The appropriate SecurityMode for the message type
    fn determine_security_mode<T: TapMessageBody>(&self) -> SecurityMode {
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
            for service in &doc.service {
                // Check the service type and extract the URI based on the type
                match &service.service_endpoint {
                    didcomm::did::ServiceKind::DIDCommMessaging { value } => {
                        // For DIDCommMessaging, return the URI directly
                        return Ok(Some(value.uri.clone()));
                    },
                    didcomm::did::ServiceKind::Other { value } => {
                        // For other services, try to extract a service endpoint from the value
                        if let Some(endpoint) = value.get("serviceEndpoint").and_then(|v| v.as_str()) {
                            return Ok(Some(endpoint.to_string()));
                        } else if let Some(uri) = value.get("uri").and_then(|v| v.as_str()) {
                            return Ok(Some(uri.to_string()));
                        } else {
                            // If we can't find a specific field, use the whole value as a string
                            return Ok(Some(format!("{}", value)));
                        }
                    }
                }
            }
        }
        
        // No service endpoint found
        Ok(None)
    }

    async fn send_message<T: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &T,
        to: &str,
    ) -> Result<String> {
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

        // Look up service endpoint before sending (only for logging, not delivery)
        if let Ok(Some(endpoint)) = self.get_service_endpoint(to).await {
            // Log the found service endpoint - this can be helpful for debugging in both production and tests
            println!("Found service endpoint for {}: {}", to, endpoint);
            println!("Message will be delivered to this endpoint");
        }

        // Use message packer to pack the message
        let packed = self
            .message_packer
            .pack_message(&message_obj, to, Some(self.get_agent_did()), security_mode)
            .await?;

        Ok(packed)
    }
    
    async fn send_message_with_delivery<T: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &T,
        to: &str,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)> {
        // First pack the message
        let packed = self.send_message(message, to).await?;
        
        // If delivery is not requested, just return the packed message
        if !deliver {
            return Ok((packed, Vec::new()));
        }
        
        // Try to deliver the message
        let mut delivery_results = Vec::new();
        
        match self.get_service_endpoint(to).await {
            Ok(Some(endpoint)) => {
                println!("Found service endpoint for {}: {}", to, endpoint);
                
                // Extract message ID for logging
                let message_id = match serde_json::from_str::<serde_json::Value>(&packed) {
                    Ok(json) => json.get("id").and_then(|id| id.as_str()).map(String::from).unwrap_or_else(|| "unknown".to_string()),
                    Err(_) => "unknown".to_string(),
                };
                
                // Attempt to deliver the message
                match self.send_to_endpoint(&packed, &endpoint).await {
                    Ok(status) => {
                        // Log success
                        println!("Delivered message {} to {} at {}", message_id, to, endpoint);
                        
                        delivery_results.push(DeliveryResult {
                            did: to.to_string(),
                            endpoint: endpoint.clone(),
                            status: Some(status),
                            error: None,
                        });
                    },
                    Err(e) => {
                        // Log error but don't fail
                        let error_msg = format!("Failed to deliver message {} to {} at {}: {}", 
                                               message_id, to, endpoint, e);
                        println!("{}", error_msg);
                        
                        delivery_results.push(DeliveryResult {
                            did: to.to_string(),
                            endpoint: endpoint.clone(),
                            status: None, 
                            error: Some(error_msg),
                        });
                    }
                }
            },
            Ok(None) => {
                // Just log a message but don't add an error result
                println!("No service endpoint found for {}, skipping delivery", to);
            },
            Err(e) => {
                // Log error but don't fail
                let error_msg = format!("Failed to resolve service endpoint for {}: {}", to, e);
                println!("{}", error_msg);
            }
        }
        
        Ok((packed, delivery_results))
    }
    
    async fn send_message_to_many<T: TapMessageBody + serde::Serialize + Send + Sync>(
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

        // Use the first recipient for packing - DIDComm will include all recipients
        let packed = self
            .message_packer
            .pack_message(&message_obj, to[0], Some(self.get_agent_did()), security_mode)
            .await?;
        
        // If delivery is not requested, just return the packed message
        if !deliver {
            return Ok((packed, Vec::new()));
        }
        
        // Try to deliver the message to each recipient's service endpoint
        let mut delivery_results = Vec::new();
        
        for recipient in &to {
            match self.get_service_endpoint(recipient).await {
                Ok(Some(endpoint)) => {
                    println!("Found service endpoint for {}: {}", recipient, endpoint);
                    
                    // Extract message ID for logging
                    let message_id = match serde_json::from_str::<serde_json::Value>(&packed) {
                        Ok(json) => json.get("id").and_then(|id| id.as_str()).map(String::from).unwrap_or_else(|| "unknown".to_string()),
                        Err(_) => "unknown".to_string(),
                    };
                    
                    // Attempt to deliver the message
                    match self.send_to_endpoint(&packed, &endpoint).await {
                        Ok(status) => {
                            // Log success
                            println!("Delivered message {} to {} at {}", message_id, recipient, endpoint);
                            
                            delivery_results.push(DeliveryResult {
                                did: recipient.to_string(),
                                endpoint: endpoint.clone(),
                                status: Some(status),
                                error: None,
                            });
                        },
                        Err(e) => {
                            // Log error but don't fail
                            let error_msg = format!("Failed to deliver message {} to {} at {}: {}", 
                                                  message_id, recipient, endpoint, e);
                            println!("{}", error_msg);
                            
                            delivery_results.push(DeliveryResult {
                                did: recipient.to_string(),
                                endpoint: endpoint.clone(),
                                status: None, 
                                error: Some(error_msg),
                            });
                        }
                    }
                },
                Ok(None) => {
                    // Just log a message but don't add an error result
                    println!("No service endpoint found for {}, skipping delivery", recipient);
                },
                Err(e) => {
                    // Log error but don't fail
                    let error_msg = format!("Failed to resolve service endpoint for {}: {}", recipient, e);
                    println!("{}", error_msg);
                }
            }
        }

        Ok((packed, delivery_results))
    }

    async fn receive_message<T: TapMessageBody + DeserializeOwned + Send>(
        &self,
        packed_message: &str,
    ) -> Result<T> {
        // Unpack the message
        let message_value: Value = self
            .message_packer
            .unpack_message_value(packed_message)
            .await?;

        // Get the message type from the unpacked message
        let message_type = message_value
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| Error::Validation("Message missing 'type' field".to_string()))?;

        // Validate the message type
        if message_type != T::message_type() {
            return Err(Error::Validation(format!(
                "Expected message type {} but got {}",
                T::message_type(),
                message_type
            )));
        }

        // Check if we need to convert to a DIDComm message first
        if let Some(id) = message_value.get("id") {
            // This appears to be a proper DIDComm message already
            // Create a DIDComm message with the required fields
            let didcomm_message = didcomm::Message {
                id: id.as_str().unwrap_or("").to_string(),
                typ: "application/didcomm-plain+json".to_string(),
                type_: message_type.to_string(),
                // Use the entire message as the body, not just the "body" field
                body: message_value.clone(),
                from: message_value
                    .get("from")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                to: message_value
                    .get("to")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
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
            message.validate().map_err(|e| {
                Error::Validation(format!(
                    "Message validation failed for type {}: {}",
                    T::message_type(),
                    e
                ))
            })?;

            Ok(message)
        } else {
            // This might be just the message body directly, try to deserialize
            let message = serde_json::from_value::<T>(message_value).map_err(|e| {
                Error::Serialization(format!("Failed to deserialize message: {}", e))
            })?;

            // Validate the message
            message.validate().map_err(|e| {
                Error::Validation(format!(
                    "Message validation failed for type {}: {}",
                    T::message_type(),
                    e
                ))
            })?;

            Ok(message)
        }
    }
}
