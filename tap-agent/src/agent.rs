use crate::config::AgentConfig;
use crate::crypto::MessagePacker;
use crate::error::{Error, Result};
use crate::key_manager::KeyManager;
use crate::message::SecurityMode;
use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tap_msg::TapMessageBody;

/// Result of a message delivery attempt
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    /// The DID of the recipient
    pub did: String,
    /// The service endpoint URL that was used for delivery
    pub endpoint: String,
    /// HTTP status code if the delivery was successful
    pub status: Option<u16>,
    /// Error message if the delivery failed
    pub error: Option<String>,
}

/// The Agent trait defines the interface for all TAP agents
#[async_trait]
pub trait Agent {
    /// Gets the agent's DID
    fn get_agent_did(&self) -> &str;

    /// Gets the service endpoint URL for a recipient
    async fn get_service_endpoint(&self, to: &str) -> Result<Option<String>>;

    /// Sends a message to one or more recipients
    async fn send_message<T: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &T,
        to: Vec<&str>,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)>;

    /// Receives a message
    async fn receive_message<T: TapMessageBody + DeserializeOwned + Send>(
        &self,
        packed_message: &str,
    ) -> Result<T>;
}

/// The DefaultAgent is a concrete implementation of the Agent trait
/// that uses a configurable message packer for cryptographic operations.
#[derive(Debug)]
pub struct DefaultAgent {
    /// Configuration for the agent
    pub config: AgentConfig,
    /// Message packer for cryptographic operations
    #[allow(dead_code)]
    message_packer: Arc<dyn MessagePacker>,
    /// HTTP client for sending requests
    #[allow(dead_code)]
    http_client: Client,
}

impl DefaultAgent {
    /// Creates a new DefaultAgent with the given configuration and message packer
    pub fn new(
        config: AgentConfig,
        message_packer: impl MessagePacker + 'static,
    ) -> DefaultAgent {
        let timeout = Duration::from_secs(config.timeout_seconds.unwrap_or(30));
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| Client::new());

        DefaultAgent {
            config,
            message_packer: Arc::new(message_packer),
            http_client: client,
        }
    }

    /// Creates a new DefaultAgent with the given configuration and a default message packer
    pub fn new_with_default_packer(
        config: AgentConfig,
        key_manager: Arc<dyn crate::key_manager::KeyManager>,
    ) -> DefaultAgent {
        let resolver = key_manager.secret_resolver();
        let message_packer =
            crate::crypto::DefaultMessagePacker::new_with_default_resolver(Arc::new(resolver), config.debug);
        DefaultAgent::new(config, message_packer)
    }

    /// Creates a new DefaultAgent builder with the given agent DID
    pub fn builder(agent_did: impl Into<String>) -> DefaultAgentBuilder {
        DefaultAgentBuilder::new(agent_did.into())
    }
    
    /// Creates a new ephemeral agent with a randomly generated DID
    /// 
    /// This is a helper method to aid with migration from the old API.
    pub fn new_ephemeral() -> Result<(Self, String)> {
        // Create a new key manager
        let key_manager = Arc::new(crate::key_manager::DefaultKeyManager::new());
        
        // Generate a random Ed25519 key
        let key = key_manager.generate_key(crate::did::DIDGenerationOptions {
            key_type: crate::did::KeyType::Ed25519,
        })?;
        
        // Get the DID from the key
        let did = key.did.clone();
        
        // Create a config with the DID
        let config = crate::config::AgentConfig {
            agent_did: did.clone(),
            security_mode: Some("SIGNED".to_string()),
            debug: true,
            timeout_seconds: Some(30),
            parameters: std::collections::HashMap::new(),
        };
        
        // Create a new agent
        let agent = Self::new_with_default_packer(config, key_manager);
        
        Ok((agent, did))
    }

    /// Send a message to a specific endpoint
    ///
    /// # Parameters
    /// * `packed_message` - The packed message to send
    /// * `endpoint` - The endpoint URL to send the message to
    ///
    /// # Returns
    /// The HTTP response status code, or error if the request failed
    #[cfg(not(target_arch = "wasm32"))]
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
    
    #[cfg(target_arch = "wasm32")]
    pub async fn send_to_endpoint(&self, packed_message: &str, endpoint: &str) -> Result<u16> {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, RequestMode, Response};
        
        // Create request options
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);
        opts.body(Some(&JsValue::from_str(packed_message)));
        
        // Create the request
        let request = Request::new_with_str_and_init(endpoint, &opts)
            .map_err(|e| Error::Networking(format!("Failed to create request: {:?}", e)))?;
            
        request.headers().set("Content-Type", "application/didcomm-encrypted+json")
            .map_err(|e| Error::Networking(format!("Failed to set headers: {:?}", e)))?;
            
        // Get the window object
        let window = web_sys::window()
            .ok_or_else(|| Error::Networking("No window object available".to_string()))?;
            
        // Send the request
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| Error::Networking(format!("Failed to fetch: {:?}", e)))?;
            
        // Convert response to Response object
        let resp: Response = resp_value.dyn_into()
            .map_err(|_| Error::Networking("Failed to convert response".to_string()))?;
            
        // Get status code
        let status = resp.status();
        
        // Log the response
        web_sys::console::log_1(&JsValue::from_str(&format!("Message sent to endpoint {}, status: {}", endpoint, status)));
        
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

    #[cfg(not(target_arch = "wasm32"))]
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
    
    #[cfg(target_arch = "wasm32")]
    async fn get_service_endpoint(&self, to: &str) -> Result<Option<String>> {
        // WASM-specific implementation for DID resolution
        // This simplified version just returns the DID as an endpoint for testing purposes
        // In a real implementation, this would call into JavaScript to resolve the DID
        
        // For WASM, we'll create a mock endpoint for now
        if to.starts_with("did:") {
            // Create a mock endpoint URL for testing
            // In a real implementation, this would call the resolver
            let endpoint = format!("https://example.com/agents/{}", to.replace(":", "-"));
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
                &format!("Mock endpoint for {}: {}", to, endpoint)
            ));
            return Ok(Some(endpoint));
        }
        
        // If not a DID, might be a direct URL
        if to.starts_with("http://") || to.starts_with("https://") {
            return Ok(Some(to.to_string()));
        }
        
        // No endpoint found
        Ok(None)
    }

    #[cfg(not(target_arch = "wasm32"))]
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
    
    #[cfg(target_arch = "wasm32")]
    async fn send_message<T: TapMessageBody + serde::Serialize + Send + Sync>(
        &self,
        message: &T,
        to: Vec<&str>,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)> {
        use wasm_bindgen::JsValue;
        use web_sys::console;
        
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

        // Log the plaintext message
        console::log_1(&JsValue::from_str("==== SENDING TAP MESSAGE ===="));
        console::log_1(&JsValue::from_str(&format!("Message Type: {}", T::message_type())));
        console::log_1(&JsValue::from_str(&format!("Recipients: {:?}", to)));
        console::log_1(&JsValue::from_str(&format!(
            "PLAINTEXT CONTENT: {}",
            serde_json::to_string_pretty(&message_obj).unwrap_or_else(|_| message_obj.to_string())
        )));

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
        console::log_1(&JsValue::from_str(&format!("Security Mode: {:?}", security_mode)));

        // For each recipient, look up service endpoint before sending
        for recipient in &to {
            if let Ok(Some(endpoint)) = self.get_service_endpoint(recipient).await {
                console::log_1(&JsValue::from_str(
                    &format!("Found service endpoint for {}: {}", recipient, endpoint)
                ));
            }
        }

        // Use message packer to pack the message for all recipients
        let packed = self
            .message_packer
            .pack_message(&message_obj, &to, Some(self.get_agent_did()), security_mode)
            .await?;

        // Log the packed message with clear separation and formatting
        console::log_1(&JsValue::from_str("--- PACKED MESSAGE ---"));
        let formatted_msg = serde_json::from_str::<serde_json::Value>(&packed)
            .map(|v| serde_json::to_string_pretty(&v).unwrap_or(packed.clone()))
            .unwrap_or(packed.clone());
        console::log_1(&JsValue::from_str(&formatted_msg));

        // If delivery is not requested, just return the packed message
        if !deliver {
            return Ok((packed, Vec::new()));
        }

        // Try to deliver the message to each recipient's service endpoint
        let mut delivery_results = Vec::new();

        for recipient in &to {
            match self.get_service_endpoint(recipient).await {
                Ok(Some(endpoint)) => {
                    console::log_1(&JsValue::from_str(
                        &format!("Delivering message to {} at {}", recipient, endpoint)
                    ));

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
                            // Log success
                            console::log_1(&JsValue::from_str(
                                &format!("✅ Delivered message {} to {} at {}", message_id, recipient, endpoint)
                            ));

                            delivery_results.push(DeliveryResult {
                                did: recipient.to_string(),
                                endpoint: endpoint.clone(),
                                status: Some(status),
                                error: None,
                            });
                        }
                        Err(e) => {
                            // Log error but don't fail
                            let error_msg = format!(
                                "Failed to deliver message {} to {} at {}: {}",
                                message_id, recipient, endpoint, e
                            );
                            console::log_1(&JsValue::from_str(&format!("❌ {}", error_msg)));

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
                    console::log_1(&JsValue::from_str(
                        &format!("⚠️ No service endpoint found for {}, skipping delivery", recipient)
                    ));
                }
                Err(e) => {
                    // Log error but don't fail
                    let error_msg = format!(
                        "Failed to resolve service endpoint for {}: {}",
                        recipient, e
                    );
                    console::log_1(&JsValue::from_str(&format!("❌ {}", error_msg)));
                }
            }
        }

        Ok((packed, delivery_results))
    }

    #[cfg(not(target_arch = "wasm32"))]
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

        // Deserialize the message into the expected type
        serde_json::from_value(message_value.clone())
            .map_err(|e| Error::Serialization(format!("Failed to deserialize message: {}", e)))
    }
    
    #[cfg(target_arch = "wasm32")]
    async fn receive_message<T: TapMessageBody + DeserializeOwned + Send>(
        &self,
        packed_message: &str,
    ) -> Result<T> {
        use wasm_bindgen::JsValue;
        use web_sys::console;
        
        // Log the received packed message
        console::log_1(&JsValue::from_str("==== RECEIVING TAP MESSAGE ===="));
        console::log_1(&JsValue::from_str("--- PACKED MESSAGE ---"));
        
        let formatted_msg = serde_json::from_str::<serde_json::Value>(packed_message)
            .map(|v| serde_json::to_string_pretty(&v).unwrap_or(packed_message.to_string()))
            .unwrap_or(packed_message.to_string());
        console::log_1(&JsValue::from_str(&formatted_msg));

        // Unpack the message
        let message_value: Value = self
            .message_packer
            .unpack_message_value(packed_message)
            .await?;

        // Log the unpacked message value
        console::log_1(&JsValue::from_str("--- UNPACKED CONTENT ---"));
        let pretty_value = serde_json::to_string_pretty(&message_value)
            .unwrap_or_else(|_| message_value.to_string());
        console::log_1(&JsValue::from_str(&pretty_value));

        // Get the message type from the unpacked message
        let message_type = message_value
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| Error::Validation("Message missing 'type' field".to_string()))?;

        // Validate the message type
        if message_type != T::message_type() {
            console::log_1(&JsValue::from_str(&format!(
                "❌ Message type validation failed: expected {}, got {}",
                T::message_type(),
                message_type
            )));
            return Err(Error::Validation(format!(
                "Expected message type {} but got {}",
                T::message_type(),
                message_type
            )));
        }
        console::log_1(&JsValue::from_str(&format!("✅ Message type validation passed: {}", message_type)));

        // Deserialize the message into the expected type
        serde_json::from_value(message_value.clone())
            .map_err(|e| Error::Serialization(format!("Failed to deserialize message: {}", e)))
    }
}

/// Builder for DefaultAgent instance
#[derive(Debug, Clone)]
pub struct DefaultAgentBuilder {
    agent_did: String,
    debug: bool,
    timeout_seconds: Option<u64>,
    security_mode: Option<String>,
}

impl DefaultAgentBuilder {
    /// Creates a new DefaultAgentBuilder with the given agent DID
    pub fn new(agent_did: String) -> Self {
        DefaultAgentBuilder {
            agent_did,
            debug: false,
            timeout_seconds: None,
            security_mode: None,
        }
    }

    /// Sets the debug flag
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Sets the timeout in seconds
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }

    /// Sets the security mode
    pub fn with_security_mode(mut self, security_mode: String) -> Self {
        self.security_mode = Some(security_mode);
        self
    }

    /// Builds a DefaultAgent with the given key manager
    pub fn build(self, key_manager: Arc<dyn crate::key_manager::KeyManager>) -> DefaultAgent {
        let config = AgentConfig {
            agent_did: self.agent_did,
            debug: self.debug,
            timeout_seconds: self.timeout_seconds,
            security_mode: self.security_mode,
            parameters: std::collections::HashMap::new(),
        };

        DefaultAgent::new_with_default_packer(config, key_manager)
    }
}