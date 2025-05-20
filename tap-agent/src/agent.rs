use crate::config::AgentConfig;
use crate::error::{Error, Result};
#[cfg(not(target_arch = "wasm32"))]
use crate::message::SecurityMode;
#[cfg(not(target_arch = "wasm32"))]
use crate::message_packing::{KeyManagerPacking, PackOptions, UnpackOptions, Unpackable};
use async_trait::async_trait;
#[cfg(feature = "native")]
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
#[cfg(feature = "native")]
use std::time::Duration;
use tap_msg::didcomm::PlainMessage;
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
#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
#[cfg(not(target_arch = "wasm32"))]
pub trait Agent {
    /// Gets the agent's DID
    fn get_agent_did(&self) -> &str;

    /// Gets the service endpoint URL for a recipient
    async fn get_service_endpoint(&self, to: &str) -> Result<Option<String>>;

    /// Sends a message to one or more recipients
    async fn send_message<
        T: TapMessageBody + serde::Serialize + Send + Sync + std::fmt::Debug + PartialEq + 'static,
    >(
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

/// A simplified Agent trait for WASM with relaxed bounds
#[cfg(target_arch = "wasm32")]
pub trait WasmAgent {
    /// Gets the agent's DID
    fn get_agent_did(&self) -> &str;

    /// Pack a message for delivery
    fn pack_message<T: TapMessageBody + serde::Serialize>(&self, message: &T) -> Result<String>;

    /// Unpack a received message
    fn unpack_message<T: TapMessageBody + DeserializeOwned>(
        &self,
        packed_message: &str,
    ) -> Result<T>;
}


/// TapAgent implementation using the KeyManager and message packing utilities.
#[derive(Debug, Clone)]
pub struct TapAgent {
    /// Configuration for the agent
    pub config: AgentConfig,
    /// Key Manager for cryptographic operations
    key_manager: Arc<dyn KeyManagerPacking>,
    /// HTTP client for sending requests
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    http_client: Option<Client>,
}


impl TapAgent {
    /// Creates a new TapAgent with the given configuration and key manager
    pub fn new(config: AgentConfig, key_manager: Arc<dyn KeyManagerPacking>) -> Self {
        #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
        {
            let timeout = Duration::from_secs(config.timeout_seconds.unwrap_or(30));
            let client = Client::builder().timeout(timeout).build().ok();

            TapAgent {
                config,
                key_manager,
                http_client: client,
            }
        }

        #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
        {
            TapAgent {
                config,
                key_manager,
            }
        }
    }
    
    /// Creates a new TapAgent with an ephemeral key
    ///
    /// This function generates a new DID key for temporary use.
    /// The key is not persisted to storage and will be lost when the agent is dropped.
    ///
    /// # Returns
    ///
    /// A tuple containing the TapAgent and the DID that was generated
    pub async fn from_ephemeral_key() -> crate::error::Result<(Self, String)> {
        use crate::did::{DIDGenerationOptions, DIDKeyGenerator, KeyType};
        use crate::key_manager::{DefaultKeyManager, KeyManager};
        
        // Create a key manager
        let key_manager = DefaultKeyManager::new();
        
        // Generate a key
        let key = key_manager.generate_key(DIDGenerationOptions {
            key_type: KeyType::Ed25519,
        })?;
        
        // Create a config with the new DID
        let config = AgentConfig::new(key.did.clone())
            .with_debug(true);
            
        // Create the agent
        let agent = Self::new(config, Arc::new(key_manager));
        
        Ok((agent, key.did))
    }

    /// Creates a new TapAgent from stored keys
    ///
    /// This function uses the KeyManagerBuilder to load keys from storage
    ///
    /// # Arguments
    ///
    /// * `did` - Optional DID to use. If None, the default DID from storage is used.
    /// * `debug` - Whether to enable debug mode
    ///
    /// # Returns
    ///
    /// A Result containing either the created agent or an error if no keys are available
    pub async fn from_stored_keys(did: Option<String>, debug: bool) -> Result<Self> {
        use crate::key_manager::{KeyManager, KeyManagerBuilder};
        use crate::storage::KeyStorage;

        // Load keys from storage
        let key_manager_builder = KeyManagerBuilder::new().load_from_default_storage();
        let key_manager = key_manager_builder.build()?;

        // Get the DIDs available in the key manager
        let dids = key_manager.list_keys()?;
        if dids.is_empty() {
            return Err(Error::Storage(
                "No keys found in storage. Generate keys first with 'tap-agent-cli generate --save'".to_string(),
            ));
        }

        // Get the DID to use
        let agent_did = if let Some(specified_did) = did {
            if !dids.contains(&specified_did) {
                return Err(Error::Storage(format!(
                    "Key with DID '{}' not found in storage",
                    specified_did
                )));
            }
            specified_did
        } else {
            // Try to get the default DID from storage
            let storage = KeyStorage::load_default()?;
            storage.default_did.unwrap_or_else(|| dids[0].clone())
        };

        // Create agent config
        let config = AgentConfig::new(agent_did).with_debug(debug);

        // Create the agent
        Ok(TapAgent::new(config, Arc::new(key_manager)))
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

    /// Send a message to a specific endpoint
    ///
    /// # Parameters
    /// * `packed_message` - The packed message to send
    /// * `endpoint` - The endpoint URL to send the message to
    ///
    /// # Returns
    /// The HTTP response status code, or error if the request failed
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    pub async fn send_to_endpoint(&self, packed_message: &str, endpoint: &str) -> Result<u16> {
        // Get HTTP client
        let client = self
            .http_client
            .as_ref()
            .ok_or_else(|| Error::Networking("HTTP client not available".to_string()))?;

        // Send the message to the endpoint via HTTP POST
        let response = client
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

    #[cfg(any(not(feature = "native"), target_arch = "wasm32"))]
    pub async fn send_to_endpoint(&self, _packed_message: &str, _endpoint: &str) -> Result<u16> {
        // Feature not enabled or WASM doesn't have http_client
        Err(crate::error::Error::NotImplemented(
            "HTTP client not available".to_string(),
        ))
    }
}

#[async_trait]
#[cfg(not(target_arch = "wasm32"))]
impl crate::agent::Agent for TapAgent {
    fn get_agent_did(&self) -> &str {
        &self.config.agent_did
    }

    async fn get_service_endpoint(&self, to: &str) -> Result<Option<String>> {
        // For now, we'll use a simple approach that assumes the DID is a URL
        // In a real implementation, we would use a DID Resolver to get the service endpoint

        // If it's a URL, return it directly
        if to.starts_with("http://") || to.starts_with("https://") {
            return Ok(Some(to.to_string()));
        }

        // If it's a DID, try to find a service endpoint
        if to.starts_with("did:") {
            // Simulate a service endpoint for now
            return Ok(Some(format!(
                "https://example.com/did/{}",
                to.replace(":", "_")
            )));
        }

        // No service endpoint found
        Ok(None)
    }

    async fn send_message<
        T: TapMessageBody + serde::Serialize + Send + Sync + std::fmt::Debug + PartialEq + 'static,
    >(
        &self,
        message: &T,
        to: Vec<&str>,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)> {
        if to.is_empty() {
            return Err(Error::Validation("No recipients specified".to_string()));
        }

        // Log the plaintext message
        println!("\n==== SENDING TAP MESSAGE ====");
        println!("Message Type: {}", T::message_type());
        println!("Recipients: {:?}", to);
        println!(
            "--- PLAINTEXT CONTENT ---\n{}",
            serde_json::to_string_pretty(message).unwrap_or_else(|_| format!("{:?}", message))
        );

        // Determine the appropriate security mode
        let security_mode = self.determine_security_mode::<T>();
        println!("Security Mode: {:?}", security_mode);

        // For each recipient, look up service endpoint before sending
        for recipient in &to {
            if let Ok(Some(endpoint)) = self.get_service_endpoint(recipient).await {
                println!("Found service endpoint for {}: {}", recipient, endpoint);
            }
        }

        // Use the Packable trait to pack the message
        // Create pack options
        let pack_options = PackOptions {
            security_mode,
            sender_kid: Some(format!("{}#keys-1", self.get_agent_did())),
            recipient_kid: if to.len() == 1 {
                Some(format!("{}#keys-1", to[0]))
            } else {
                None
            },
        };

        // Pack the message
        // Use the pack_any helper function instead of trait method
        let packed =
            crate::message_packing::pack_any(message, self.key_manager.as_ref(), pack_options)
                .await
                .map_err(|e| Error::Cryptography(format!("Failed to pack message: {}", e)))?;

        // Log the packed message
        println!("--- PACKED MESSAGE ---");
        println!(
            "{}",
            serde_json::from_str::<Value>(&packed)
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
                    let message_id = match serde_json::from_str::<Value>(&packed) {
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
                            // Log error but don't fail
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
                    println!(
                        "⚠️ No service endpoint found for {}, skipping delivery",
                        recipient
                    );
                }
                Err(e) => {
                    // Log error but don't fail
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

    async fn receive_message<T: TapMessageBody + DeserializeOwned + Send>(
        &self,
        packed_message: &str,
    ) -> Result<T> {
        // Log the received packed message
        println!("\n==== RECEIVING TAP MESSAGE ====");
        println!("--- PACKED MESSAGE ---");
        println!(
            "{}",
            serde_json::from_str::<Value>(packed_message)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or(packed_message.to_string()))
                .unwrap_or(packed_message.to_string())
        );
        println!("---------------------");

        // Create unpack options
        let unpack_options = UnpackOptions {
            expected_security_mode: SecurityMode::Any,
            expected_recipient_kid: Some(format!("{}#keys-1", self.get_agent_did())),
            require_signature: false,
        };

        // Unpack the message using the Unpackable trait
        let plain_message: PlainMessage = String::unpack(
            &packed_message.to_string(),
            &*self.key_manager,
            unpack_options,
        )
        .await
        .map_err(|e| Error::Cryptography(format!("Failed to unpack message: {}", e)))?;

        // Log the unpacked message
        println!("--- UNPACKED CONTENT ---");
        println!(
            "{}",
            serde_json::to_string_pretty(&plain_message)
                .unwrap_or_else(|_| format!("{:?}", plain_message))
        );
        println!("------------------------");

        // Get the message type from the unpacked message
        let message_type = &plain_message.type_;

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

        // Deserialize the message body into the expected type
        serde_json::from_value::<T>(plain_message.body.clone())
            .map_err(|e| Error::Serialization(format!("Failed to deserialize message: {}", e)))
    }
}



/// Builder for TapAgent instance
#[derive(Debug, Clone)]
pub struct AgentBuilder {
    agent_did: String,
    debug: bool,
    timeout_seconds: Option<u64>,
    security_mode: Option<String>,
}

impl AgentBuilder {
    /// Creates a new AgentBuilder with the given agent DID
    pub fn new(agent_did: String) -> Self {
        AgentBuilder {
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

    /// Builds a TapAgent with the given key manager
    pub fn build(self, key_manager: Arc<dyn KeyManagerPacking>) -> TapAgent {
        let config = AgentConfig {
            agent_did: self.agent_did,
            debug: self.debug,
            timeout_seconds: self.timeout_seconds,
            security_mode: self.security_mode,
            parameters: std::collections::HashMap::new(),
        };

        TapAgent::new(config, key_manager)
    }
}
