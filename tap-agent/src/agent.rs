use crate::agent_key_manager::{AgentKeyManager, AgentKeyManagerBuilder};
use crate::config::AgentConfig;
#[cfg(all(not(target_arch = "wasm32"), test))]
use crate::did::SyncDIDResolver; // Import SyncDIDResolver trait
use crate::error::{Error, Result};
use crate::key_manager::KeyManager; // Add KeyManager trait
#[cfg(not(target_arch = "wasm32"))]
use crate::message::SecurityMode;
#[cfg(not(target_arch = "wasm32"))]
use crate::message_packing::{PackOptions, Packable, UnpackOptions, Unpackable};
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
        T: TapMessageBody + serde::Serialize + Send + Sync + std::fmt::Debug + 'static,
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

    /// Receives a raw message (can be plain, JWE, or JWS) and returns the unpacked PlainMessage
    async fn receive_raw_message(&self, raw_message: &str) -> Result<PlainMessage>;
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

/// TapAgent implementation using the AgentKeyManager for cryptographic operations.
#[derive(Debug, Clone)]
pub struct TapAgent {
    /// Configuration for the agent
    pub config: AgentConfig,
    /// Key Manager for cryptographic operations
    key_manager: Arc<AgentKeyManager>,
    /// DID Resolver for resolving DIDs to service endpoints
    #[cfg(all(not(target_arch = "wasm32"), test))]
    resolver: Option<Arc<dyn SyncDIDResolver>>,
    /// HTTP client for sending requests
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    http_client: Option<Client>,
}

impl TapAgent {
    /// Returns a reference to the agent's key manager
    pub fn key_manager(&self) -> &Arc<AgentKeyManager> {
        &self.key_manager
    }

    /// Creates a new TapAgent with the given configuration and AgentKeyManager
    pub fn new(config: AgentConfig, key_manager: Arc<AgentKeyManager>) -> Self {
        #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
        {
            let timeout = Duration::from_secs(config.timeout_seconds.unwrap_or(30));
            let client = Client::builder().timeout(timeout).build().ok();

            #[cfg(test)]
            let agent = TapAgent {
                config,
                key_manager,
                resolver: None,
                http_client: client,
            };

            #[cfg(not(test))]
            let agent = TapAgent {
                config,
                key_manager,
                http_client: client,
            };

            agent
        }

        #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
        {
            #[cfg(all(not(target_arch = "wasm32"), test))]
            let agent = TapAgent {
                config,
                key_manager,
                resolver: None,
            };

            #[cfg(all(not(target_arch = "wasm32"), not(test)))]
            let agent = TapAgent {
                config,
                key_manager,
            };

            #[cfg(target_arch = "wasm32")]
            let agent = TapAgent {
                config,
                key_manager,
            };

            agent
        }
    }

    /// Creates a new TapAgent with the given configuration, key manager, and DID resolver
    #[cfg(all(not(target_arch = "wasm32"), test))]
    pub fn new_with_resolver(
        config: AgentConfig,
        key_manager: Arc<AgentKeyManager>,
        resolver: Arc<dyn SyncDIDResolver>,
    ) -> Self {
        #[cfg(feature = "native")]
        {
            let timeout = Duration::from_secs(config.timeout_seconds.unwrap_or(30));
            let client = Client::builder().timeout(timeout).build().ok();

            TapAgent {
                config,
                key_manager,
                resolver: Some(resolver),
                http_client: client,
            }
        }

        #[cfg(not(feature = "native"))]
        {
            TapAgent {
                config,
                key_manager,
                resolver: Some(resolver),
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
        use crate::did::{DIDGenerationOptions, KeyType};

        // Create a key manager
        let key_manager = AgentKeyManager::new();

        // Generate a key
        let key = key_manager.generate_key(DIDGenerationOptions {
            key_type: KeyType::Ed25519,
        })?;

        // Create a config with the new DID
        let config = AgentConfig::new(key.did.clone()).with_debug(true);

        // Create the agent
        #[cfg(all(not(target_arch = "wasm32"), test))]
        {
            // Create a default resolver
            let resolver = Arc::new(crate::did::MultiResolver::default());
            let agent = Self::new_with_resolver(config, Arc::new(key_manager), resolver);
            Ok((agent, key.did))
        }

        #[cfg(all(not(target_arch = "wasm32"), not(test)))]
        {
            let agent = Self::new(config, Arc::new(key_manager));
            Ok((agent, key.did))
        }

        #[cfg(target_arch = "wasm32")]
        {
            let agent = Self::new(config, Arc::new(key_manager));
            Ok((agent, key.did))
        }
    }

    /// Creates a new TapAgent from stored keys
    ///
    /// This function uses the AgentKeyManagerBuilder to load keys from storage
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
        use crate::storage::KeyStorage;

        // Load keys from storage
        let key_manager_builder = AgentKeyManagerBuilder::new().load_from_default_storage();
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
        #[cfg(all(not(target_arch = "wasm32"), test))]
        {
            // Create a default resolver
            let resolver = Arc::new(crate::did::MultiResolver::default());
            Ok(TapAgent::new_with_resolver(
                config,
                Arc::new(key_manager),
                resolver,
            ))
        }

        #[cfg(all(not(target_arch = "wasm32"), not(test)))]
        {
            Ok(TapAgent::new(config, Arc::new(key_manager)))
        }

        #[cfg(target_arch = "wasm32")]
        {
            Ok(TapAgent::new(config, Arc::new(key_manager)))
        }
    }

    /// Creates a new TapAgent from an existing private key
    ///
    /// This function creates a new TapAgent using a provided private key,
    /// which can be useful for integrating with external key management systems
    /// or when keys are generated outside the TAP agent.
    ///
    /// # Arguments
    ///
    /// * `private_key` - The private key bytes
    /// * `key_type` - The type of key (Ed25519, P256, or Secp256k1)
    /// * `debug` - Whether to enable debug mode
    ///
    /// # Returns
    ///
    /// A Result containing either the created agent or an error
    pub async fn from_private_key(
        private_key: &[u8],
        key_type: crate::did::KeyType,
        debug: bool,
    ) -> Result<(Self, String)> {
        use crate::did::{DIDKeyGenerator, GeneratedKey};
        use crate::did::{VerificationMaterial, VerificationMethod, VerificationMethodType};
        use curve25519_dalek::edwards::CompressedEdwardsY;
        use multibase::{encode, Base};

        // Create a key manager to hold our key
        let key_manager = AgentKeyManager::new();

        // Generate the appropriate key and DID based on the key type
        let generated_key = match key_type {
            crate::did::KeyType::Ed25519 => {
                if private_key.len() != 32 {
                    return Err(Error::Validation(format!(
                        "Invalid Ed25519 private key length: {}, expected 32 bytes",
                        private_key.len()
                    )));
                }

                // For Ed25519, we need to derive the public key from the private key
                let mut private_key_bytes = [0u8; 32];
                private_key_bytes.copy_from_slice(&private_key[0..32]);

                let signing_key = ed25519_dalek::SigningKey::from_bytes(&private_key_bytes);

                // Get the public key
                let verifying_key = ed25519_dalek::VerifyingKey::from(&signing_key);
                let public_key = verifying_key.to_bytes().to_vec();

                // Create did:key identifier
                // Multicodec prefix for Ed25519: 0xed01
                let mut prefixed_key = vec![0xed, 0x01];
                prefixed_key.extend_from_slice(&public_key);

                // Encode the key with multibase (base58btc with 'z' prefix)
                let multibase_encoded = encode(Base::Base58Btc, &prefixed_key);
                let did = format!("did:key:{}", multibase_encoded);

                // Create the verification method ID
                let vm_id = format!("{}#{}", did, multibase_encoded);

                // Create the verification method
                let verification_method = VerificationMethod {
                    id: vm_id.clone(),
                    type_: VerificationMethodType::Ed25519VerificationKey2018,
                    controller: did.clone(),
                    verification_material: VerificationMaterial::Multibase {
                        public_key_multibase: multibase_encoded.clone(),
                    },
                };

                // Create X25519 key for key agreement - Implement the ed25519_to_x25519 conversion directly
                let x25519_method_and_agreement = {
                    // Only Ed25519 public keys must be exactly 32 bytes
                    if public_key.len() != 32 {
                        None
                    } else {
                        // Try to create a CompressedEdwardsY from the bytes
                        let edwards_y = match CompressedEdwardsY::from_slice(&public_key) {
                            Ok(point) => point,
                            Err(_) => {
                                return Err(Error::Cryptography(
                                    "Failed to create Edwards point".to_string(),
                                ))
                            }
                        };

                        // Try to decompress to get the Edwards point
                        let edwards_point = match edwards_y.decompress() {
                            Some(point) => point,
                            None => {
                                return Err(Error::Cryptography(
                                    "Failed to decompress Edwards point".to_string(),
                                ))
                            }
                        };

                        // Convert to Montgomery form
                        let montgomery_point = edwards_point.to_montgomery();

                        // Get the raw bytes representation of the X25519 key
                        let x25519_key = montgomery_point.to_bytes();

                        // Prefix for X25519: 0xEC01
                        let mut x25519_prefixed = vec![0xEC, 0x01];
                        x25519_prefixed.extend_from_slice(&x25519_key);

                        // Encode the prefixed X25519 key with multibase
                        let x25519_multibase = encode(Base::Base58Btc, &x25519_prefixed);

                        // Create the X25519 verification method ID
                        let x25519_vm_id = format!("{}#{}", did, x25519_multibase);

                        // Create the X25519 verification method
                        let x25519_verification_method = VerificationMethod {
                            id: x25519_vm_id.clone(),
                            type_: VerificationMethodType::X25519KeyAgreementKey2019,
                            controller: did.clone(),
                            verification_material: VerificationMaterial::Multibase {
                                public_key_multibase: x25519_multibase,
                            },
                        };

                        Some((x25519_verification_method, x25519_vm_id))
                    }
                };

                // Build verification methods array
                let mut verification_methods = vec![verification_method.clone()];
                let mut key_agreement = Vec::new();

                if let Some((x25519_vm, x25519_id)) = x25519_method_and_agreement {
                    verification_methods.push(x25519_vm);
                    key_agreement.push(x25519_id);
                }

                // Create the DID document
                let did_doc = crate::did::DIDDoc {
                    id: did.clone(),
                    verification_method: verification_methods,
                    authentication: vec![vm_id],
                    key_agreement,
                    assertion_method: Vec::new(),
                    capability_invocation: Vec::new(),
                    capability_delegation: Vec::new(),
                    service: Vec::new(),
                };

                // Create a GeneratedKey with all necessary fields
                GeneratedKey {
                    key_type: crate::did::KeyType::Ed25519,
                    did: did.clone(),
                    public_key,
                    private_key: private_key.to_vec(),
                    did_doc,
                }
            }
            crate::did::KeyType::P256 => {
                if private_key.len() != 32 {
                    return Err(Error::Validation(format!(
                        "Invalid P-256 private key length: {}, expected 32 bytes",
                        private_key.len()
                    )));
                }

                // For P-256, create a signing key from the private key
                let signing_key = match p256::ecdsa::SigningKey::from_slice(private_key) {
                    Ok(key) => key,
                    Err(e) => {
                        return Err(Error::Cryptography(format!(
                            "Failed to create P-256 signing key: {:?}",
                            e
                        )))
                    }
                };

                // Get the public key in uncompressed form
                let public_key = signing_key
                    .verifying_key()
                    .to_encoded_point(false)
                    .to_bytes()
                    .to_vec();

                // Create did:key identifier
                // Multicodec prefix for P-256: 0x1200
                let mut prefixed_key = vec![0x12, 0x00];
                prefixed_key.extend_from_slice(&public_key);

                // Encode the key with multibase (base58btc with 'z' prefix)
                let multibase_encoded = encode(Base::Base58Btc, &prefixed_key);
                let did = format!("did:key:{}", multibase_encoded);

                // Create the verification method ID
                let vm_id = format!("{}#{}", did, multibase_encoded);

                // Create the verification method
                let verification_method = VerificationMethod {
                    id: vm_id.clone(),
                    type_: VerificationMethodType::EcdsaSecp256k1VerificationKey2019, // Using the available type
                    controller: did.clone(),
                    verification_material: VerificationMaterial::Multibase {
                        public_key_multibase: multibase_encoded.clone(),
                    },
                };

                // Create the DID document
                let did_doc = crate::did::DIDDoc {
                    id: did.clone(),
                    verification_method: vec![verification_method],
                    authentication: vec![vm_id],
                    key_agreement: Vec::new(),
                    assertion_method: Vec::new(),
                    capability_invocation: Vec::new(),
                    capability_delegation: Vec::new(),
                    service: Vec::new(),
                };

                // Create a GeneratedKey with all necessary fields
                GeneratedKey {
                    key_type: crate::did::KeyType::P256,
                    did: did.clone(),
                    public_key,
                    private_key: private_key.to_vec(),
                    did_doc,
                }
            }
            crate::did::KeyType::Secp256k1 => {
                if private_key.len() != 32 {
                    return Err(Error::Validation(format!(
                        "Invalid Secp256k1 private key length: {}, expected 32 bytes",
                        private_key.len()
                    )));
                }

                // For Secp256k1, create a signing key from the private key
                let signing_key = match k256::ecdsa::SigningKey::from_slice(private_key) {
                    Ok(key) => key,
                    Err(e) => {
                        return Err(Error::Cryptography(format!(
                            "Failed to create Secp256k1 signing key: {:?}",
                            e
                        )))
                    }
                };

                // Get the public key in uncompressed form
                let public_key = signing_key
                    .verifying_key()
                    .to_encoded_point(false)
                    .to_bytes()
                    .to_vec();

                // Create did:key identifier
                // Multicodec prefix for Secp256k1: 0xe701
                let mut prefixed_key = vec![0xe7, 0x01];
                prefixed_key.extend_from_slice(&public_key);

                // Encode the key with multibase (base58btc with 'z' prefix)
                let multibase_encoded = encode(Base::Base58Btc, &prefixed_key);
                let did = format!("did:key:{}", multibase_encoded);

                // Create the verification method ID
                let vm_id = format!("{}#{}", did, multibase_encoded);

                // Create the verification method
                let verification_method = VerificationMethod {
                    id: vm_id.clone(),
                    type_: VerificationMethodType::EcdsaSecp256k1VerificationKey2019,
                    controller: did.clone(),
                    verification_material: VerificationMaterial::Multibase {
                        public_key_multibase: multibase_encoded.clone(),
                    },
                };

                // Create the DID document
                let did_doc = crate::did::DIDDoc {
                    id: did.clone(),
                    verification_method: vec![verification_method],
                    authentication: vec![vm_id],
                    key_agreement: Vec::new(),
                    assertion_method: Vec::new(),
                    capability_invocation: Vec::new(),
                    capability_delegation: Vec::new(),
                    service: Vec::new(),
                };

                // Create a GeneratedKey with all necessary fields
                GeneratedKey {
                    key_type: crate::did::KeyType::Secp256k1,
                    did: did.clone(),
                    public_key,
                    private_key: private_key.to_vec(),
                    did_doc,
                }
            }
        };

        // Create secret from the generated key and use it to add to the key manager
        let did_generator = DIDKeyGenerator::new();
        let _secret = did_generator.create_secret_from_key(&generated_key);

        // Add the key to the key manager
        key_manager.add_key(&generated_key)?;

        // Create a config with the new DID
        let config = AgentConfig::new(generated_key.did.clone()).with_debug(debug);

        // Create the agent
        #[cfg(all(not(target_arch = "wasm32"), test))]
        {
            // Create a default resolver
            let resolver = Arc::new(crate::did::MultiResolver::default());
            let agent = Self::new_with_resolver(config, Arc::new(key_manager), resolver);
            Ok((agent, generated_key.did))
        }

        #[cfg(all(not(target_arch = "wasm32"), not(test)))]
        {
            let agent = Self::new(config, Arc::new(key_manager));
            Ok((agent, generated_key.did))
        }

        #[cfg(target_arch = "wasm32")]
        {
            let agent = Self::new(config, Arc::new(key_manager));
            Ok((agent, generated_key.did))
        }
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
    #[cfg(not(target_arch = "wasm32"))]
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
        // If it's a URL, return it directly
        if to.starts_with("http://") || to.starts_with("https://") {
            return Ok(Some(to.to_string()));
        }

        // If it's a DID, try to find a service endpoint using the resolver
        if to.starts_with("did:") {
            // Use the DID resolver from the AgentKeyManager to get the service endpoints
            // For now, we'll use a simple approach that looks for DIDCommMessaging or Web service types

            // For testing purposes, attempt to check if TapAgent has a resolver field
            #[cfg(test)]
            if let Some(resolver) = self.resolver.as_ref() {
                if let Ok(Some(did_doc)) = resolver.resolve(to).await {
                    // Look for services of type DIDCommMessaging first
                    if let Some(service) = did_doc
                        .service
                        .iter()
                        .find(|s| s.type_ == "DIDCommMessaging")
                    {
                        return Ok(Some(service.service_endpoint.clone()));
                    }

                    // Then try Web type
                    if let Some(service) = did_doc.service.iter().find(|s| s.type_ == "Web") {
                        return Ok(Some(service.service_endpoint.clone()));
                    }

                    // No matching service found in DID doc
                    if !did_doc.service.is_empty() {
                        // Use the first service as fallback
                        return Ok(Some(did_doc.service[0].service_endpoint.clone()));
                    }
                }
            }

            // Fallback to a placeholder URL if no resolver is available or no service found
            return Ok(Some(format!(
                "https://example.com/did/{}",
                to.replace(":", "_")
            )));
        }

        // No service endpoint found
        Ok(None)
    }

    async fn send_message<
        T: TapMessageBody + serde::Serialize + Send + Sync + std::fmt::Debug + 'static,
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

        // Convert the TapMessageBody to a PlainMessage with explicit routing
        let plain_message =
            message.to_didcomm_with_route(self.get_agent_did(), to.iter().copied())?;

        // Determine the appropriate security mode
        let security_mode = self.determine_security_mode::<T>();
        println!("Security Mode: {:?}", security_mode);

        // For each recipient, look up service endpoint before sending
        for recipient in &to {
            if let Ok(Some(endpoint)) = self.get_service_endpoint(recipient).await {
                println!("Found service endpoint for {}: {}", recipient, endpoint);
            }
        }

        // Create pack options for the plaintext message
        let pack_options = PackOptions {
            security_mode,
            sender_kid: Some(format!("{}#keys-1", self.get_agent_did())),
            recipient_kid: if to.len() == 1 {
                Some(format!("{}#keys-1", to[0]))
            } else {
                None
            },
        };

        // Pack the plain message using the Packable trait
        let packed = plain_message.pack(&*self.key_manager, pack_options).await?;

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
        .await?;

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

    async fn receive_raw_message(&self, raw_message: &str) -> Result<PlainMessage> {
        // Log the received raw message
        println!("\n==== RECEIVING RAW MESSAGE ====");
        println!("Agent DID: {}", self.get_agent_did());

        // First try to parse as JSON to determine message type
        let json_value: Value = serde_json::from_str(raw_message)
            .map_err(|e| Error::Serialization(format!("Failed to parse message as JSON: {}", e)))?;

        // Check if it's an encrypted message (JWE) or signed message (JWS)
        let is_encrypted =
            json_value.get("protected").is_some() && json_value.get("recipients").is_some();
        let is_signed =
            json_value.get("protected").is_some() && json_value.get("signatures").is_some();

        println!(
            "Message type detection: encrypted={}, signed={}",
            is_encrypted, is_signed
        );

        if is_encrypted || is_signed {
            println!(
                "Detected {} message",
                if is_encrypted { "encrypted" } else { "signed" }
            );
            println!("--- ENCRYPTED/SIGNED MESSAGE ---");
            println!(
                "{}",
                serde_json::to_string_pretty(&json_value).unwrap_or(raw_message.to_string())
            );
            println!("---------------------");

            // Create unpack options
            let unpack_options = UnpackOptions {
                expected_security_mode: SecurityMode::Any,
                expected_recipient_kid: Some(format!("{}#keys-1", self.get_agent_did())),
                require_signature: false,
            };

            println!("Unpacking with options: {:?}", unpack_options);

            // Unpack the message
            let plain_message: PlainMessage =
                match String::unpack(&raw_message.to_string(), &*self.key_manager, unpack_options)
                    .await
                {
                    Ok(msg) => msg,
                    Err(e) => {
                        println!("Failed to unpack message: {}", e);
                        return Err(e);
                    }
                };

            // Log the unpacked message
            println!("--- UNPACKED CONTENT ---");
            println!(
                "{}",
                serde_json::to_string_pretty(&plain_message)
                    .unwrap_or_else(|_| format!("{:?}", plain_message))
            );
            println!("------------------------");

            Ok(plain_message)
        } else {
            // It's already a plain message
            println!("Detected plain message");
            println!("--- PLAIN MESSAGE ---");
            println!(
                "{}",
                serde_json::to_string_pretty(&json_value).unwrap_or(raw_message.to_string())
            );
            println!("---------------------");

            // Parse directly as PlainMessage
            serde_json::from_str::<PlainMessage>(raw_message)
                .map_err(|e| Error::Serialization(format!("Failed to parse PlainMessage: {}", e)))
        }
    }
}
