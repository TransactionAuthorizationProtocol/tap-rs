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
#[cfg(target_arch = "wasm32")]
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(feature = "native")]
use std::time::Duration;
use tap_msg::didcomm::{PlainMessage, PlainMessageExt};
use tap_msg::TapMessageBody;
use tracing::{debug, error, info, warn};

/// Type alias for enhanced agent information: (DID, policies, metadata)
pub type EnhancedAgentInfo = (
    String,
    Vec<String>,
    std::collections::HashMap<String, String>,
);

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
///
/// This trait supports both standalone agent usage and integration with TAP Node.
/// The different receive methods are designed for different usage patterns:
///
/// # Usage Patterns
///
/// ## Node Integration
/// - [`receive_encrypted_message`]: Called by TAP Node for encrypted messages
/// - [`receive_plain_message`]: Called by TAP Node for verified/decrypted messages
///
/// ## Standalone Usage
/// - [`receive_message`]: Handles any message type (plain, signed, encrypted)
///
/// ## Message Sending
/// - [`send_message`]: Sends messages to recipients with optional delivery
///
/// # Examples
///
/// ```rust,no_run
/// use tap_agent::{Agent, TapAgent};
/// use tap_msg::didcomm::PlainMessage;
///
/// async fn process_encrypted_message(agent: &TapAgent, jwe_json: &serde_json::Value) {
///     // This would typically be called by TAP Node
///     if let Err(e) = agent.receive_encrypted_message(jwe_json).await {
///         tracing::error!("Failed to process encrypted message: {}", e);
///     }
/// }
/// ```
#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
#[cfg(not(target_arch = "wasm32"))]
pub trait Agent {
    /// Gets the agent's DID
    fn get_agent_did(&self) -> &str;

    /// Gets the service endpoint URL for a recipient
    ///
    /// This method resolves how to reach a given recipient, which could be:
    /// - A direct URL if `to` is already a URL
    /// - A DID resolution if `to` is a DID
    async fn get_service_endpoint(&self, to: &str) -> Result<Option<String>>;

    /// Sends a message to one or more recipients
    ///
    /// # Parameters
    /// - `message`: The message to send (must implement TapMessageBody)
    /// - `to`: List of recipient DIDs or URLs
    /// - `deliver`: Whether to actually deliver the message or just pack it
    ///
    /// # Returns
    /// - Packed message string
    /// - Vector of delivery results (empty if deliver=false)
    async fn send_message<
        T: TapMessageBody + serde::Serialize + Send + Sync + std::fmt::Debug + 'static,
    >(
        &self,
        message: &T,
        to: Vec<&str>,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)>;

    /// Receives an encrypted message (decrypt and process)
    ///
    /// This method is typically called by TAP Node when routing encrypted
    /// messages to agents. The agent should:
    /// 1. Parse the JWE from the JSON value
    /// 2. Attempt to decrypt using its private keys
    /// 3. Process the resulting PlainMessage
    ///
    /// # Parameters
    /// - `jwe_value`: JSON representation of the encrypted message (JWE)
    async fn receive_encrypted_message(&self, jwe_value: &Value) -> Result<()>;

    /// Receives a plain message (already verified/decrypted)
    ///
    /// This method is called by TAP Node after signature verification
    /// or by other agents after decryption. The message is ready for
    /// business logic processing.
    ///
    /// # Parameters
    /// - `message`: The verified/decrypted PlainMessage
    async fn receive_plain_message(&self, message: PlainMessage) -> Result<()>;

    /// Receives a raw message (for standalone usage - handles any message type)
    ///
    /// This method handles the complete message processing pipeline for
    /// standalone agent usage. It can process:
    /// - Plain messages (passed through)
    /// - Signed messages (signature verified)
    /// - Encrypted messages (decrypted)
    ///
    /// # Parameters
    /// - `raw_message`: JSON string of any message type
    ///
    /// # Returns
    /// - The processed PlainMessage
    async fn receive_message(&self, raw_message: &str) -> Result<PlainMessage>;

    /// Send a strongly-typed message
    ///
    /// # Parameters
    /// - `message`: The typed message to send
    /// - `deliver`: Whether to actually deliver the message
    ///
    /// # Returns
    /// - Packed message string
    /// - Vector of delivery results (empty if deliver=false)
    async fn send_typed<T: TapMessageBody + Send + Sync + std::fmt::Debug + 'static>(
        &self,
        message: PlainMessage<T>,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)> {
        // Convert to plain message and use existing send infrastructure
        let plain_message = message.to_plain_message()?;
        let to_vec: Vec<&str> = plain_message.to.iter().map(|s| s.as_str()).collect();

        // Extract the body and send using the existing method
        let body = serde_json::from_value::<T>(plain_message.body)?;
        self.send_message(&body, to_vec, deliver).await
    }

    /// Send a message with MessageContext support for automatic routing
    ///
    /// This method uses MessageContext to automatically extract participants
    /// and routing hints for improved message delivery.
    ///
    /// # Parameters
    /// - `message`: The message body that implements both TapMessageBody and MessageContext
    /// - `deliver`: Whether to actually deliver the message
    ///
    /// # Returns
    /// - Packed message string
    /// - Vector of delivery results (empty if deliver=false)
    async fn send_with_context<T>(
        &self,
        message: &T,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)>
    where
        T: TapMessageBody
            + tap_msg::message::MessageContext
            + Send
            + Sync
            + std::fmt::Debug
            + 'static,
    {
        // Extract participants using MessageContext
        let participant_dids = message.participant_dids();
        let recipients: Vec<&str> = participant_dids
            .iter()
            .map(|s| s.as_str())
            .filter(|&did| did != self.get_agent_did()) // Don't send to self
            .collect();

        // Get routing hints for enhanced delivery
        let _routing_hints = message.routing_hints();

        // TODO: Use routing_hints to optimize delivery
        // For now, just use the standard send_message method
        self.send_message(message, recipients, deliver).await
    }

    /// Send a typed message with automatic context routing
    ///
    /// # Parameters
    /// - `message`: The typed message with MessageContext support
    /// - `deliver`: Whether to actually deliver the message
    ///
    /// # Returns
    /// - Packed message string
    /// - Vector of delivery results (empty if deliver=false)
    async fn send_typed_with_context<T>(
        &self,
        message: PlainMessage<T>,
        deliver: bool,
    ) -> Result<(String, Vec<DeliveryResult>)>
    where
        T: TapMessageBody
            + tap_msg::message::MessageContext
            + Send
            + Sync
            + std::fmt::Debug
            + 'static,
    {
        // Use the enhanced participant extraction
        let participants = message.extract_participants_with_context();
        let _recipients: Vec<&str> = participants
            .iter()
            .map(|s| s.as_str())
            .filter(|&did| did != self.get_agent_did()) // Don't send to self
            .collect();

        // Get routing hints
        let _routing_hints = message.routing_hints();

        // Extract the body and send using the context-aware method
        let body = message.body;
        self.send_with_context(&body, deliver).await
    }

    /// Receive and parse a typed message
    ///
    /// # Parameters
    /// - `raw_message`: The raw message string
    ///
    /// # Type Parameters
    /// - `T`: The expected message body type
    ///
    /// # Returns
    /// - The typed message if parsing succeeds
    async fn receive_typed<T: TapMessageBody>(&self, raw_message: &str) -> Result<PlainMessage<T>> {
        let plain_message = self.receive_message(raw_message).await?;
        plain_message
            .parse_as()
            .map_err(|e| Error::Serialization(e.to_string()))
    }
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

    /// Internal method to process a PlainMessage
    async fn process_message_internal(&self, message: PlainMessage) -> Result<()> {
        // This is where actual message processing logic would go
        // For now, just log that we processed it
        debug!(
            "Processing message: {} of type {}",
            message.id, message.type_
        );

        // TODO: Add actual message processing logic here
        // This could include:
        // - Validating the message against policies
        // - Updating internal state
        // - Triggering workflows
        // - Generating responses

        Ok(())
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

    /// Get the signing key ID for this agent
    ///
    /// Resolves the DID document and returns the first authentication verification method ID
    pub async fn get_signing_kid(&self) -> Result<String> {
        let did = &self.config.agent_did;

        // Try to get the DID document from our key manager first
        if let Ok(agent_key) = self.key_manager.get_generated_key(did) {
            // Get the first authentication method from the DID document
            if let Some(auth_method_id) = agent_key.did_doc.authentication.first() {
                return Ok(auth_method_id.clone());
            }

            // Fallback to first verification method
            if let Some(vm) = agent_key.did_doc.verification_method.first() {
                return Ok(vm.id.clone());
            }
        }

        // Fallback to guessing based on DID method (for backward compatibility)
        if did.starts_with("did:key:") {
            let multibase = did.strip_prefix("did:key:").unwrap_or("");
            Ok(format!("{}#{}", did, multibase))
        } else if did.starts_with("did:web:") {
            Ok(format!("{}#keys-1", did))
        } else {
            Ok(format!("{}#key-1", did))
        }
    }

    /// Get the encryption key ID for a recipient
    ///
    /// Resolves the DID document and returns the appropriate key agreement method ID
    pub async fn get_encryption_kid(&self, recipient_did: &str) -> Result<String> {
        if recipient_did == self.config.agent_did {
            // If asking for our own encryption key, get it from our DID document
            if let Ok(agent_key) = self.key_manager.get_generated_key(recipient_did) {
                // Look for key agreement methods first
                if let Some(agreement_method_id) = agent_key.did_doc.key_agreement.first() {
                    return Ok(agreement_method_id.clone());
                }

                // Fallback to authentication method (for keys that do both)
                if let Some(auth_method_id) = agent_key.did_doc.authentication.first() {
                    return Ok(auth_method_id.clone());
                }

                // Fallback to first verification method
                if let Some(vm) = agent_key.did_doc.verification_method.first() {
                    return Ok(vm.id.clone());
                }
            }

            // Final fallback to signing key
            return self.get_signing_kid().await;
        }

        // For external recipients, try to resolve their DID document
        #[cfg(all(not(target_arch = "wasm32"), test))]
        if let Some(resolver) = &self.resolver {
            if let Ok(Some(did_doc)) = resolver.resolve(recipient_did).await {
                // Look for key agreement methods first
                if let Some(agreement_method_id) = did_doc.key_agreement.first() {
                    return Ok(agreement_method_id.clone());
                }

                // Fallback to authentication method
                if let Some(auth_method_id) = did_doc.authentication.first() {
                    return Ok(auth_method_id.clone());
                }

                // Fallback to first verification method
                if let Some(vm) = did_doc.verification_method.first() {
                    return Ok(vm.id.clone());
                }
            }
        }

        // Fallback to guessing based on DID method (for backward compatibility)
        if recipient_did.starts_with("did:key:") {
            let multibase = recipient_did.strip_prefix("did:key:").unwrap_or("");
            Ok(format!("{}#{}", recipient_did, multibase))
        } else if recipient_did.starts_with("did:web:") {
            Ok(format!("{}#keys-1", recipient_did))
        } else {
            Ok(format!("{}#key-1", recipient_did))
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
        debug!("Message sent to endpoint {}, status: {}", endpoint, status);

        Ok(status)
    }

    #[cfg(any(not(feature = "native"), target_arch = "wasm32"))]
    pub async fn send_to_endpoint(&self, _packed_message: &str, _endpoint: &str) -> Result<u16> {
        // Feature not enabled or WASM doesn't have http_client
        Err(crate::error::Error::NotImplemented(
            "HTTP client not available".to_string(),
        ))
    }

    /// Create an agent with enhanced configuration (policies and metadata)
    pub async fn create_enhanced_agent(
        agent_id: String,
        policies: Vec<String>,
        metadata: std::collections::HashMap<String, String>,
        save_to_storage: bool,
    ) -> Result<(Self, String)> {
        Self::create_enhanced_agent_with_path(agent_id, policies, metadata, save_to_storage, None)
            .await
    }

    /// Create an agent with enhanced configuration (policies and metadata) with custom storage path
    pub async fn create_enhanced_agent_with_path(
        agent_id: String,
        policies: Vec<String>,
        metadata: std::collections::HashMap<String, String>,
        save_to_storage: bool,
        storage_path: Option<PathBuf>,
    ) -> Result<(Self, String)> {
        use crate::did::{DIDGenerationOptions, KeyType};
        use crate::storage::KeyStorage;

        // Create a key manager and generate a key without saving to storage
        let key_manager = AgentKeyManager::new();
        let generated_key = key_manager.generate_key_without_save(DIDGenerationOptions {
            key_type: KeyType::Ed25519,
        })?;

        // Create a config with the provided agent ID
        let config = AgentConfig::new(agent_id.clone()).with_debug(true);

        // Add the generated key to the key manager with the custom DID
        // Use add_key_without_save to prevent automatic storage write
        let mut custom_generated_key = generated_key.clone();
        custom_generated_key.did = agent_id.clone();
        key_manager.add_key_without_save(&custom_generated_key)?;

        // Create the agent
        #[cfg(all(not(target_arch = "wasm32"), test))]
        let agent = {
            let resolver = Arc::new(crate::did::MultiResolver::default());
            Self::new_with_resolver(config, Arc::new(key_manager), resolver)
        };

        #[cfg(all(not(target_arch = "wasm32"), not(test)))]
        let agent = Self::new(config, Arc::new(key_manager));

        #[cfg(target_arch = "wasm32")]
        let agent = Self::new(config, Arc::new(key_manager));

        if save_to_storage {
            // Save to key storage
            let mut key_storage = if let Some(path) = &storage_path {
                KeyStorage::load_from_path(path)?
            } else {
                KeyStorage::load_default()?
            };

            // Convert the generated key to a stored key
            let mut stored_key = KeyStorage::from_generated_key(&custom_generated_key);
            stored_key.label = format!("agent-{}", agent_id.split(':').last().unwrap_or("agent"));

            key_storage.add_key(stored_key);

            if let Some(path) = &storage_path {
                key_storage.save_to_path(path)?;
            } else {
                key_storage.save_default()?;
            }

            // Create agent directory with policies and metadata
            key_storage.create_agent_directory(&agent_id, &policies, &metadata)?;
        }

        Ok((agent, agent_id))
    }

    /// Load an enhanced agent from storage with policies and metadata
    pub async fn load_enhanced_agent(
        did: &str,
    ) -> Result<(Self, Vec<String>, std::collections::HashMap<String, String>)> {
        use crate::storage::KeyStorage;

        // Load key storage
        let key_storage = KeyStorage::load_default()?;

        // Check if the key exists in storage
        let agent = if key_storage.keys.contains_key(did) {
            // Load agent from stored keys
            Self::from_stored_keys(Some(did.to_string()), true).await?
        } else {
            // If key doesn't exist in storage, create an ephemeral agent
            // This is for test scenarios where agents are created but not persisted
            let (mut agent, _) = Self::from_ephemeral_key().await?;
            agent.config.agent_did = did.to_string();
            agent
        };

        // Load policies and metadata from agent directory
        let policies = key_storage.load_agent_policies(did).unwrap_or_default();
        let metadata = key_storage.load_agent_metadata(did).unwrap_or_default();

        Ok((agent, policies, metadata))
    }

    /// List all enhanced agents with their policies and metadata
    pub fn list_enhanced_agents() -> Result<Vec<EnhancedAgentInfo>> {
        Self::list_enhanced_agents_with_path(None)
    }

    /// List all enhanced agents with their policies and metadata with custom storage path
    pub fn list_enhanced_agents_with_path(
        storage_path: Option<PathBuf>,
    ) -> Result<Vec<EnhancedAgentInfo>> {
        use crate::storage::KeyStorage;
        use std::fs;

        let key_storage = if let Some(path) = &storage_path {
            KeyStorage::load_from_path(path)?
        } else {
            KeyStorage::load_default()?
        };
        let mut agents = Vec::new();

        // Get TAP directory
        let tap_dir = if let Some(path) = &storage_path {
            // For custom paths, the tap directory is the parent of the keys.json file
            path.parent()
                .ok_or_else(|| Error::Storage("Invalid storage path".to_string()))?
                .to_path_buf()
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| Error::Storage("Could not determine home directory".to_string()))?;
            home.join(crate::storage::DEFAULT_TAP_DIR)
        };

        if !tap_dir.exists() {
            return Ok(agents);
        }

        // Scan for agent directories
        for entry in fs::read_dir(&tap_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Skip known non-agent directories
                if dir_name == "keys.json" || dir_name.is_empty() {
                    continue;
                }

                // Convert sanitized DID back to original format
                let did = dir_name.replace('_', ":");

                // Try to load policies and metadata
                let policies = key_storage.load_agent_policies(&did).unwrap_or_default();
                let metadata = key_storage.load_agent_metadata(&did).unwrap_or_default();

                // Only include if there are policies or metadata (indicating an enhanced agent)
                if !policies.is_empty() || !metadata.is_empty() {
                    agents.push((did, policies, metadata));
                }
            }
        }

        Ok(agents)
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
        debug!("\n==== SENDING TAP MESSAGE ====");
        debug!("Message Type: {}", T::message_type());
        debug!("Recipients: {:?}", to);

        // Convert the TapMessageBody to a PlainMessage with explicit routing
        let plain_message =
            message.to_didcomm_with_route(self.get_agent_did(), to.iter().copied())?;

        // Determine the appropriate security mode
        let security_mode = self.determine_security_mode::<T>();
        debug!("Security Mode: {:?}", security_mode);

        // For each recipient, look up service endpoint before sending
        for recipient in &to {
            if let Ok(Some(endpoint)) = self.get_service_endpoint(recipient).await {
                debug!("Found service endpoint for {}: {}", recipient, endpoint);
            }
        }

        // Get the appropriate key IDs
        let sender_kid = self.get_signing_kid().await?;
        let recipient_kid = if to.len() == 1 && security_mode == SecurityMode::AuthCrypt {
            Some(self.get_encryption_kid(to[0]).await?)
        } else {
            None
        };

        // Create pack options for the plaintext message
        let pack_options = PackOptions {
            security_mode,
            sender_kid: Some(sender_kid),
            recipient_kid,
        };

        // Pack the plain message using the Packable trait
        let packed = plain_message.pack(&*self.key_manager, pack_options).await?;

        // Log the packed message
        debug!("--- PACKED MESSAGE ---");
        debug!(
            "{}",
            serde_json::from_str::<serde_json::Value>(&packed)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or(packed.clone()))
                .unwrap_or(packed.clone())
        );
        debug!("=====================");

        // If delivery is not requested, just return the packed message
        if !deliver {
            return Ok((packed, Vec::new()));
        }

        // Try to deliver the message to each recipient's service endpoint
        let mut delivery_results = Vec::new();

        for recipient in &to {
            match self.get_service_endpoint(recipient).await {
                Ok(Some(endpoint)) => {
                    debug!("Delivering message to {} at {}", recipient, endpoint);

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
                            info!(
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
                            error!("❌ {}", error_msg);

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
                    warn!(
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
                    error!("❌ {}", error_msg);
                }
            }
        }

        Ok((packed, delivery_results))
    }

    async fn receive_encrypted_message(&self, jwe_value: &Value) -> Result<()> {
        // Log the received encrypted message
        debug!("\n==== RECEIVING ENCRYPTED MESSAGE ====");
        debug!("Agent DID: {}", self.get_agent_did());

        // Parse as JWE
        let jwe: crate::message::Jwe = serde_json::from_value(jwe_value.clone())
            .map_err(|e| Error::Serialization(format!("Failed to parse JWE: {}", e)))?;

        // Get our encryption key ID
        let our_kid = self.get_signing_kid().await.ok();

        // Create unpack options
        let unpack_options = UnpackOptions {
            expected_security_mode: SecurityMode::AuthCrypt,
            expected_recipient_kid: our_kid,
            require_signature: false,
        };

        // Decrypt the message
        let plain_message =
            crate::message::Jwe::unpack(&jwe, &*self.key_manager, unpack_options).await?;

        // Process the decrypted message
        self.process_message_internal(plain_message).await
    }

    async fn receive_plain_message(&self, message: PlainMessage) -> Result<()> {
        // Process already verified/decrypted message
        debug!("\n==== RECEIVING PLAIN MESSAGE ====");
        debug!("Message ID: {}", message.id);
        debug!("Message Type: {}", message.type_);

        self.process_message_internal(message).await
    }

    async fn receive_message(&self, raw_message: &str) -> Result<PlainMessage> {
        // Log the received raw message
        debug!("\n==== RECEIVING RAW MESSAGE ====");
        debug!("Agent DID: {}", self.get_agent_did());

        // First try to parse as JSON to determine message type
        let json_value: Value = serde_json::from_str(raw_message)
            .map_err(|e| Error::Serialization(format!("Failed to parse message as JSON: {}", e)))?;

        // Check if it's an encrypted message (JWE) or signed message (JWS)
        let is_encrypted =
            json_value.get("protected").is_some() && json_value.get("recipients").is_some();
        let is_signed =
            json_value.get("payload").is_some() && json_value.get("signatures").is_some();

        debug!(
            "Message type detection: encrypted={}, signed={}",
            is_encrypted, is_signed
        );

        if is_signed {
            debug!("Detected signed message");
            debug!("--- SIGNED MESSAGE ---");
            debug!(
                "{}",
                serde_json::to_string_pretty(&json_value).unwrap_or(raw_message.to_string())
            );
            debug!("---------------------");

            // Parse as JWS
            let jws: crate::message::Jws = serde_json::from_value(json_value)
                .map_err(|e| Error::Serialization(format!("Failed to parse JWS: {}", e)))?;

            // Verify using our resolver
            #[cfg(test)]
            let plain_message = if let Some(resolver) = &self.resolver {
                crate::verification::verify_jws(&jws, &**resolver).await?
            } else {
                // Fallback to unpacking with key manager for test compatibility
                let unpack_options = UnpackOptions {
                    expected_security_mode: SecurityMode::Signed,
                    expected_recipient_kid: None,
                    require_signature: true,
                };
                crate::message::Jws::unpack(&jws, &*self.key_manager, unpack_options).await?
            };

            #[cfg(not(test))]
            let plain_message = {
                // In production, we need a resolver - for now use unpacking
                let unpack_options = UnpackOptions {
                    expected_security_mode: SecurityMode::Signed,
                    expected_recipient_kid: None,
                    require_signature: true,
                };
                crate::message::Jws::unpack(&jws, &*self.key_manager, unpack_options).await?
            };

            // Log the unpacked message
            debug!("--- UNPACKED CONTENT ---");
            debug!(
                "{}",
                serde_json::to_string_pretty(&plain_message)
                    .unwrap_or_else(|_| format!("{:?}", plain_message))
            );
            debug!("------------------------");

            Ok(plain_message)
        } else if is_encrypted {
            debug!("Detected encrypted message");
            debug!("--- ENCRYPTED MESSAGE ---");
            debug!(
                "{}",
                serde_json::to_string_pretty(&json_value).unwrap_or(raw_message.to_string())
            );
            debug!("---------------------");

            // Get our encryption key ID
            let our_kid = self.get_signing_kid().await.ok();

            // Create unpack options
            let unpack_options = UnpackOptions {
                expected_security_mode: SecurityMode::AuthCrypt,
                expected_recipient_kid: our_kid,
                require_signature: false,
            };

            debug!("Unpacking with options: {:?}", unpack_options);

            // Unpack the message
            let plain_message: PlainMessage =
                match String::unpack(&raw_message.to_string(), &*self.key_manager, unpack_options)
                    .await
                {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to unpack message: {}", e);
                        return Err(e);
                    }
                };

            // Log the unpacked message
            debug!("--- UNPACKED CONTENT ---");
            debug!(
                "{}",
                serde_json::to_string_pretty(&plain_message)
                    .unwrap_or_else(|_| format!("{:?}", plain_message))
            );
            debug!("------------------------");

            Ok(plain_message)
        } else {
            // It's already a plain message
            debug!("Detected plain message");
            debug!("--- PLAIN MESSAGE ---");
            debug!(
                "{}",
                serde_json::to_string_pretty(&json_value).unwrap_or(raw_message.to_string())
            );
            debug!("---------------------");

            // Parse directly as PlainMessage
            serde_json::from_str::<PlainMessage>(raw_message)
                .map_err(|e| Error::Serialization(format!("Failed to parse PlainMessage: {}", e)))
        }
    }
}
