use crate::config::AgentConfig;
use crate::crypto::MessagePacker;
use crate::error::{Error, Result};
#[cfg(not(target_arch = "wasm32"))]
use crate::key_manager::KeyManager;
#[cfg(not(target_arch = "wasm32"))]
use crate::message::SecurityMode;
#[cfg(not(target_arch = "wasm32"))]
use crate::message_packing::{KeyManagerPacking, PackOptions, Packable, UnpackOptions, Unpackable};
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
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    #[allow(dead_code)]
    http_client: Client,
}

/// Modern Agent implementation using the new KeyManager and message packing utilities.
#[derive(Debug, Clone)]
pub struct ModernAgent {
    /// Configuration for the agent
    pub config: AgentConfig,
    /// Key Manager for cryptographic operations
    key_manager: Arc<dyn KeyManagerPacking>,
    /// HTTP client for sending requests
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    http_client: Option<Client>,
}

#[cfg(target_arch = "wasm32")]
impl WasmAgent for DefaultAgent {
    fn get_agent_did(&self) -> &str {
        &self.config.agent_did
    }

    fn pack_message<T: TapMessageBody + serde::Serialize>(&self, message: &T) -> Result<String> {
        // Simple mock implementation
        let message_json = serde_json::to_string(message)?;
        Ok(message_json)
    }

    fn unpack_message<T: TapMessageBody + DeserializeOwned>(
        &self,
        packed_message: &str,
    ) -> Result<T> {
        // Simple mock implementation
        let message: T = serde_json::from_str(packed_message)?;
        Ok(message)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl DefaultAgent {
    /// Creates a new DefaultAgent with the given configuration and message packer
    pub fn new(config: AgentConfig, message_packer: impl MessagePacker + 'static) -> DefaultAgent {
        #[cfg(feature = "native")]
        {
            let timeout = Duration::from_secs(config.timeout_seconds.unwrap_or(30));
            let client = Client::builder()
                .timeout(timeout)
                .build()
                .unwrap_or_else(|_| Client::new());

            // Since we're in #[cfg(feature = "native")] block, http_client is required
            DefaultAgent {
                config,
                message_packer: Arc::new(message_packer),
                http_client: client,
            }
        }

        #[cfg(not(feature = "native"))]
        {
            DefaultAgent {
                config,
                message_packer: Arc::new(message_packer),
            }
        }
    }

    /// Creates a new DefaultAgent with the given configuration and a default message packer
    pub fn new_with_default_packer(
        config: AgentConfig,
        key_manager: Arc<dyn crate::key_manager::KeyManager>,
    ) -> DefaultAgent {
        let resolver = key_manager.secret_resolver();
        let message_packer = crate::crypto::DefaultMessagePacker::new_with_default_resolver(
            Arc::new(resolver),
            config.debug,
        );
        DefaultAgent::new(config, message_packer)
    }

    /// Creates a new DefaultAgent from stored keys
    ///
    /// This function checks for stored keys in the default location (~/.tap/keys.json)
    /// and creates an agent using the default key if available.
    ///
    /// # Arguments
    ///
    /// * `did` - Optional DID to use. If None, the default DID from storage is used.
    /// * `debug` - Whether to enable debug mode
    ///
    /// # Returns
    ///
    /// A Result containing either the created agent or an error if no keys are available
    pub fn from_stored_keys(did: Option<String>, debug: bool) -> Result<Self> {
        use crate::storage::KeyStorage;

        // Try to load key storage
        let storage = match KeyStorage::load_default() {
            Ok(storage) => storage,
            Err(e) => {
                return Err(Error::Storage(format!(
                    "Failed to load keys from storage: {}",
                    e
                )));
            }
        };

        // Check if storage has any keys
        if storage.keys.is_empty() {
            return Err(Error::Storage(
                "No keys found in storage. Generate keys first with 'tap-agent-cli generate --save'".to_string(),
            ));
        }

        // Get the key to use (either specified DID or default)
        let key_did = match did {
            Some(d) => {
                if !storage.keys.contains_key(&d) {
                    return Err(Error::Storage(format!(
                        "Key with DID '{}' not found in storage",
                        d
                    )));
                }
                d
            }
            None => {
                // Use default key or first available
                match &storage.default_did {
                    Some(d) => d.clone(),
                    None => {
                        // Use the first key in storage
                        storage.keys.keys().next().unwrap().clone()
                    }
                }
            }
        };

        // Get the key
        let stored_key = storage.keys.get(&key_did).unwrap();

        // Create a BasicSecretResolver with the key
        let mut secret_resolver = crate::crypto::BasicSecretResolver::new();
        let secret = KeyStorage::to_secret(stored_key);
        secret_resolver.add_secret(&key_did, secret);

        // Create agent config
        let config = AgentConfig::new(key_did).with_debug(debug);

        // Create a message packer with the secret resolver
        let did_resolver = Arc::new(crate::did::MultiResolver::default());
        let message_packer = crate::crypto::DefaultMessagePacker::new(
            did_resolver,
            Arc::new(secret_resolver),
            debug,
        );

        // Create and return the agent
        Ok(DefaultAgent::new(config, message_packer))
    }

    /// Tries to create an agent from stored keys, or creates an ephemeral agent if none are available
    ///
    /// This function first attempts to create an agent using stored keys.
    /// If that fails, it creates a new ephemeral agent with a generated key.
    ///
    /// # Arguments
    ///
    /// * `did` - Optional DID to use. If None, the default DID from storage is used.
    /// * `debug` - Whether to enable debug mode
    ///
    /// # Returns
    ///
    /// The created agent, either from storage or as a new ephemeral agent
    pub fn from_stored_or_ephemeral(did: Option<String>, debug: bool) -> Self {
        use base64::Engine;

        match Self::from_stored_keys(did, debug) {
            Ok(agent) => agent,
            Err(_) => {
                // Create an ephemeral agent
                let generator = crate::did::DIDKeyGenerator::new();
                let options = crate::did::DIDGenerationOptions::default();

                // Generate a new key
                let key = match generator.generate_did(options) {
                    Ok(k) => k,
                    Err(e) => {
                        // This should rarely happen, but log it
                        eprintln!("Warning: Failed to generate ephemeral key: {}", e);
                        // Use a hardcoded key for emergency fallback
                        crate::did::GeneratedKey {
                            did: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
                                .to_string(),
                            key_type: crate::did::KeyType::Ed25519,
                            private_key: vec![],
                            public_key: vec![],
                            did_doc: crate::did::DIDDoc {
                                id: "".to_string(),
                                verification_method: vec![],
                                authentication: vec![],
                                key_agreement: vec![],
                                assertion_method: vec![],
                                capability_invocation: vec![],
                                capability_delegation: vec![],
                                service: vec![],
                            },
                        }
                    }
                };

                // Create a BasicSecretResolver with the key
                let mut secret_resolver = crate::crypto::BasicSecretResolver::new();

                // Create a JWK secret
                let secret_material = crate::key_manager::SecretMaterial::JWK {
                    private_key_jwk: serde_json::json!({
                        "kty": "OKP",
                        "crv": "Ed25519",
                        "x": base64::engine::general_purpose::STANDARD.encode(&key.public_key),
                        "d": base64::engine::general_purpose::STANDARD.encode(&key.private_key),
                        "kid": format!("{}#keys-1", key.did)
                    }),
                };

                let secret = crate::key_manager::Secret {
                    id: key.did.clone(),
                    type_: crate::key_manager::SecretType::JsonWebKey2020,
                    secret_material,
                };

                secret_resolver.add_secret(&key.did, secret);

                // Create agent config
                let config = AgentConfig::new(key.did).with_debug(debug);

                // Create a message packer with the secret resolver
                let did_resolver = Arc::new(crate::did::MultiResolver::default());
                let message_packer = crate::crypto::DefaultMessagePacker::new(
                    did_resolver,
                    Arc::new(secret_resolver),
                    debug,
                );

                // Create and return the agent
                DefaultAgent::new(config, message_packer)
            }
        }
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
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
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

    #[cfg(any(not(feature = "native"), target_arch = "wasm32"))]
    pub async fn send_to_endpoint(&self, _packed_message: &str, _endpoint: &str) -> Result<u16> {
        // Feature not enabled or WASM doesn't have http_client
        Err(crate::error::Error::NotImplemented(
            "HTTP client not available".to_string(),
        ))
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

impl ModernAgent {
    /// Creates a new ModernAgent with the given configuration and key manager
    pub fn new(config: AgentConfig, key_manager: Arc<dyn KeyManagerPacking>) -> Self {
        #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
        {
            let timeout = Duration::from_secs(config.timeout_seconds.unwrap_or(30));
            let client = Client::builder().timeout(timeout).build().ok();

            ModernAgent {
                config,
                key_manager,
                http_client: client,
            }
        }

        #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
        {
            ModernAgent {
                config,
                key_manager,
            }
        }
    }

    /// Creates a new ModernAgent from stored keys
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
        use crate::key_manager::KeyManagerBuilder;
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
        Ok(ModernAgent::new(config, Arc::new(key_manager)))
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
impl Agent for ModernAgent {
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

    async fn send_message<T: TapMessageBody + serde::Serialize + Send + Sync>(
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
        let packed = message
            .pack(&*self.key_manager, pack_options)
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

#[async_trait]
#[cfg(not(target_arch = "wasm32"))]
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
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                "Mock endpoint for {}: {}",
                to, endpoint
            )));
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
        console::log_1(&JsValue::from_str(&format!(
            "Message Type: {}",
            T::message_type()
        )));
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
        console::log_1(&JsValue::from_str(&format!(
            "Security Mode: {:?}",
            security_mode
        )));

        // For each recipient, look up service endpoint before sending
        for recipient in &to {
            if let Ok(Some(endpoint)) = self.get_service_endpoint(recipient).await {
                console::log_1(&JsValue::from_str(&format!(
                    "Found service endpoint for {}: {}",
                    recipient, endpoint
                )));
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
                    console::log_1(&JsValue::from_str(&format!(
                        "Delivering message to {} at {}",
                        recipient, endpoint
                    )));

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
                            console::log_1(&JsValue::from_str(&format!(
                                "✅ Delivered message {} to {} at {}",
                                message_id, recipient, endpoint
                            )));

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
                    console::log_1(&JsValue::from_str(&format!(
                        "⚠️ No service endpoint found for {}, skipping delivery",
                        recipient
                    )));
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
        console::log_1(&JsValue::from_str(&format!(
            "✅ Message type validation passed: {}",
            message_type
        )));

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

/// Builder for ModernAgent instance
#[derive(Debug, Clone)]
pub struct ModernAgentBuilder {
    agent_did: String,
    debug: bool,
    timeout_seconds: Option<u64>,
    security_mode: Option<String>,
}

impl ModernAgentBuilder {
    /// Creates a new ModernAgentBuilder with the given agent DID
    pub fn new(agent_did: String) -> Self {
        ModernAgentBuilder {
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

    /// Builds a ModernAgent with the given key manager
    pub fn build(self, key_manager: Arc<dyn KeyManagerPacking>) -> ModernAgent {
        let config = AgentConfig {
            agent_did: self.agent_did,
            debug: self.debug,
            timeout_seconds: self.timeout_seconds,
            security_mode: self.security_mode,
            parameters: std::collections::HashMap::new(),
        };

        ModernAgent::new(config, key_manager)
    }
}
