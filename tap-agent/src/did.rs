//! DID resolution and generation functionality for the TAP Agent.
//!
//! This module provides a multi-resolver for Decentralized Identifiers (DIDs)
//! that integrates with the didcomm library's DID resolution system. The multi-resolver
//! currently supports the did:key method, with the architecture allowing for additional
//! methods to be added in the future.
//!
//! It also provides functionality to generate new DIDs with different cryptographic curves.

use crate::key_manager::{Secret, SecretMaterial, SecretType};
use async_trait::async_trait;
use base64::Engine;
use curve25519_dalek::edwards::CompressedEdwardsY;
use ed25519_dalek::{SigningKey as Ed25519SigningKey, VerifyingKey as Ed25519VerifyingKey};
use k256::ecdsa::SigningKey as Secp256k1SigningKey;
use multibase::{decode, encode, Base};
use p256::ecdsa::SigningKey as P256SigningKey;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use tracing::{debug, warn};

use crate::error::{Error, Result};

/// DID Document
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DIDDoc {
    /// DID that this document describes
    pub id: String,

    /// List of verification methods
    pub verification_method: Vec<VerificationMethod>,

    /// List of authentication verification method references (id strings)
    pub authentication: Vec<String>,

    /// List of key agreement verification method references (id strings)
    pub key_agreement: Vec<String>,

    /// List of assertion method verification method references (id strings)
    #[serde(default)]
    pub assertion_method: Vec<String>,

    /// List of capability invocation verification method references (id strings)
    #[serde(default)]
    pub capability_invocation: Vec<String>,

    /// List of capability delegation verification method references (id strings)
    #[serde(default)]
    pub capability_delegation: Vec<String>,

    /// List of services
    pub service: Vec<Service>,
}

/// Service definition in a DID Document
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Service {
    /// Service ID
    pub id: String,

    /// Service type
    #[serde(rename = "type")]
    pub type_: String,

    /// Service endpoint URL
    pub service_endpoint: String,

    /// Additional properties
    #[serde(flatten)]
    pub properties: HashMap<String, Value>,
}

/// Verification method in a DID Document
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationMethod {
    /// Verification method ID
    pub id: String,

    /// Verification method type
    #[serde(rename = "type")]
    pub type_: VerificationMethodType,

    /// Controller DID
    pub controller: String,

    /// Verification material
    #[serde(flatten)]
    pub verification_material: VerificationMaterial,
}

/// Verification method type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum VerificationMethodType {
    /// Ed25519 Verification Key 2018
    Ed25519VerificationKey2018,

    /// X25519 Key Agreement Key 2019
    X25519KeyAgreementKey2019,

    /// ECDSA Secp256k1 Verification Key 2019
    EcdsaSecp256k1VerificationKey2019,

    /// JSON Web Key 2020
    JsonWebKey2020,
}

/// Verification material
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum VerificationMaterial {
    /// Base58 encoded public key
    Base58 {
        /// Public key encoded in base58
        public_key_base58: String,
    },

    /// Multibase encoded public key
    Multibase {
        /// Public key encoded in multibase
        public_key_multibase: String,
    },

    /// JSON Web Key
    JWK {
        /// Public key in JWK format
        public_key_jwk: Value,
    },
}

/// Key types supported for DID generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    /// Ed25519 key type (EdDSA)
    Ed25519,
    /// P-256 key type (ECDSA secp256r1)
    P256,
    /// Secp256k1 key type (ECDSA secp256k1)
    Secp256k1,
}

/// Generated key information
#[derive(Debug, Clone)]
pub struct GeneratedKey {
    /// The key type
    pub key_type: KeyType,
    /// The generated DID
    pub did: String,
    /// The public key in binary form
    pub public_key: Vec<u8>,
    /// The private key in binary form
    pub private_key: Vec<u8>,
    /// The DID document
    pub did_doc: DIDDoc,
}

/// Options for generating a DID
#[derive(Debug, Clone)]
pub struct DIDGenerationOptions {
    /// Key type to use
    pub key_type: KeyType,
}

impl Default for DIDGenerationOptions {
    fn default() -> Self {
        Self {
            key_type: KeyType::Ed25519,
        }
    }
}

/// A trait for resolving DIDs to DID documents that is Send+Sync.
/// This trait is only available in native builds.
#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub trait SyncDIDResolver: Send + Sync + Debug {
    /// Resolve a DID to a DID document.
    ///
    /// # Parameters
    /// * `did` - The DID to resolve
    ///
    /// # Returns
    /// The DID document as an Option
    async fn resolve(&self, did: &str) -> Result<Option<DIDDoc>>;
}

/// A resolver for a specific DID method.
/// This trait is only available in native builds.
#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
#[cfg(not(target_arch = "wasm32"))]
pub trait DIDMethodResolver: Send + Sync + Debug {
    /// Returns the method name this resolver handles (e.g., "key", "web", "pkh").
    fn method(&self) -> &str;

    /// Resolve a DID to a DID document.
    ///
    /// # Parameters
    /// * `did` - The DID to resolve
    ///
    /// # Returns
    /// The DID document as an Option
    async fn resolve_method(&self, did: &str) -> Result<Option<DIDDoc>>;
}

/// A simplified DID resolver for WebAssembly with no async or Send/Sync requirements.
#[cfg(target_arch = "wasm32")]
pub trait WasmDIDResolver: Debug {
    /// Resolves a DID synchronously, returning the DID document.
    fn resolve(&self, did: &str) -> Result<Option<DIDDoc>>;
}

/// A simplified method-specific DID resolver for WebAssembly.
#[cfg(target_arch = "wasm32")]
pub trait WasmDIDMethodResolver: Debug {
    /// Returns the method name this resolver handles.
    fn method(&self) -> &str;

    /// Resolves a DID synchronously, returning the DID document.
    fn resolve_method(&self, did: &str) -> Result<Option<DIDDoc>>;

    /// Get this resolver as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

/// A resolver for the did:key method.
#[derive(Debug, Default)]
pub struct KeyResolver;

impl KeyResolver {
    /// Create a new KeyResolver
    pub fn new() -> Self {
        Self
    }

    /// Convert an Ed25519 public key to an X25519 public key
    ///
    /// This follows the conversion process described in RFC 7748
    /// https://datatracker.ietf.org/doc/html/rfc7748#section-5
    fn ed25519_to_x25519(ed25519_pubkey: &[u8]) -> Option<[u8; 32]> {
        // The Ed25519 public key should be 32 bytes
        if ed25519_pubkey.len() != 32 {
            return None;
        }

        // Add debugging
        debug!("Ed25519 pubkey: {:?}", ed25519_pubkey);

        // Try to create a CompressedEdwardsY from the bytes
        let edwards_y = match CompressedEdwardsY::try_from(ed25519_pubkey) {
            Ok(point) => point,
            Err(e) => {
                debug!("Error converting to CompressedEdwardsY: {:?}", e);
                return None;
            }
        };

        // Try to decompress to get the Edwards point
        let edwards_point = match edwards_y.decompress() {
            Some(point) => point,
            None => {
                debug!("Failed to decompress Edwards point");
                return None;
            }
        };

        // Convert to Montgomery form
        let montgomery_point = edwards_point.to_montgomery();

        // Get the raw bytes representation of the X25519 key
        Some(montgomery_point.to_bytes())
    }
}

#[cfg(target_arch = "wasm32")]
impl WasmDIDMethodResolver for KeyResolver {
    fn method(&self) -> &str {
        "key"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn resolve_method(&self, did_key: &str) -> Result<Option<DIDDoc>> {
        // Same implementation but without async/await
        // Validate that this is a did:key
        if !did_key.starts_with("did:key:") {
            return Ok(None);
        }

        // Parse the multibase-encoded public key
        let key_id = &did_key[8..]; // Skip the "did:key:" prefix
        let (_, key_bytes) = match decode(key_id) {
            Ok(result) => result,
            Err(_) => return Ok(None),
        };

        // Check the key prefix - for did:key only Ed25519 is supported
        if key_bytes.len() < 2 {
            return Ok(None);
        }

        // Verify the key type - 0xED01 for Ed25519
        if key_bytes[0] != 0xED || key_bytes[1] != 0x01 {
            return Ok(None);
        }

        // Create the DID Document with the Ed25519 public key
        let ed25519_public_key = &key_bytes[2..];

        let ed_vm_id = format!("{}#{}", did_key, key_id);

        // Create the Ed25519 verification method
        let ed_verification_method = VerificationMethod {
            id: ed_vm_id.clone(),
            type_: VerificationMethodType::Ed25519VerificationKey2018,
            controller: did_key.to_string(),
            verification_material: VerificationMaterial::Multibase {
                public_key_multibase: key_id.to_string(),
            },
        };

        // Convert the Ed25519 public key to X25519 for key agreement
        let mut verification_methods = vec![ed_verification_method.clone()];
        let mut key_agreement = Vec::new();

        if let Some(x25519_key) = Self::ed25519_to_x25519(ed25519_public_key) {
            // Encode the X25519 public key in multibase format
            let mut x25519_bytes = vec![0xEC, 0x01]; // Prefix for X25519
            x25519_bytes.extend_from_slice(&x25519_key);
            let x25519_multibase = encode(Base::Base58Btc, x25519_bytes);

            // Create the X25519 verification method ID
            let x25519_vm_id = format!("{}#{}", did_key, x25519_multibase);

            // Create the X25519 verification method
            let x25519_verification_method = VerificationMethod {
                id: x25519_vm_id.clone(),
                type_: VerificationMethodType::X25519KeyAgreementKey2019,
                controller: did_key.to_string(),
                verification_material: VerificationMaterial::Multibase {
                    public_key_multibase: x25519_multibase,
                },
            };

            // Add the X25519 key agreement method
            verification_methods.push(x25519_verification_method);
            key_agreement.push(x25519_vm_id);
        }

        // Create the DID document
        let did_doc = DIDDoc {
            id: did_key.to_string(),
            verification_method: verification_methods,
            authentication: vec![ed_vm_id],
            key_agreement,
            assertion_method: Vec::new(),
            capability_invocation: Vec::new(),
            capability_delegation: Vec::new(),
            service: Vec::new(),
        };

        Ok(Some(did_doc))
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl DIDMethodResolver for KeyResolver {
    fn method(&self) -> &str {
        "key"
    }

    async fn resolve_method(&self, did_key: &str) -> Result<Option<DIDDoc>> {
        // Validate that this is a did:key
        if !did_key.starts_with("did:key:") {
            return Ok(None);
        }

        // Parse the multibase-encoded public key
        let key_id = &did_key[8..]; // Skip the "did:key:" prefix
        let (_, key_bytes) = match decode(key_id) {
            Ok(result) => result,
            Err(_) => return Ok(None),
        };

        // Check the key prefix - for did:key only Ed25519 is supported
        if key_bytes.len() < 2 {
            return Ok(None);
        }

        // Verify the key type - 0xED01 for Ed25519
        if key_bytes[0] != 0xED || key_bytes[1] != 0x01 {
            return Ok(None);
        }

        // Create the DID Document with the Ed25519 public key
        let ed25519_public_key = &key_bytes[2..];

        let ed_vm_id = format!("{}#{}", did_key, key_id);

        // Create the Ed25519 verification method
        let ed_verification_method = VerificationMethod {
            id: ed_vm_id.clone(),
            type_: VerificationMethodType::Ed25519VerificationKey2018,
            controller: did_key.to_string(),
            verification_material: VerificationMaterial::Multibase {
                public_key_multibase: key_id.to_string(),
            },
        };

        // Convert the Ed25519 public key to X25519 for key agreement
        let mut verification_methods = vec![ed_verification_method.clone()];
        let mut key_agreement = Vec::new();

        if let Some(x25519_key) = Self::ed25519_to_x25519(ed25519_public_key) {
            debug!("Successfully converted Ed25519 to X25519!");
            // Encode the X25519 public key in multibase format
            let mut x25519_bytes = vec![0xEC, 0x01]; // Prefix for X25519
            x25519_bytes.extend_from_slice(&x25519_key);
            let x25519_multibase = encode(Base::Base58Btc, x25519_bytes);

            // Create the X25519 verification method ID
            let x25519_vm_id = format!("{}#{}", did_key, x25519_multibase);

            // Create the X25519 verification method
            let x25519_verification_method = VerificationMethod {
                id: x25519_vm_id.clone(),
                type_: VerificationMethodType::X25519KeyAgreementKey2019,
                controller: did_key.to_string(),
                verification_material: VerificationMaterial::Multibase {
                    public_key_multibase: x25519_multibase,
                },
            };

            // Add the X25519 key agreement method
            verification_methods.push(x25519_verification_method);
            key_agreement.push(x25519_vm_id);
        } else {
            debug!("Failed to convert Ed25519 to X25519!");
        }

        // Create the DID document
        let did_doc = DIDDoc {
            id: did_key.to_string(),
            verification_method: verification_methods,
            authentication: vec![ed_vm_id],
            key_agreement,
            assertion_method: Vec::new(),
            capability_invocation: Vec::new(),
            capability_delegation: Vec::new(),
            service: Vec::new(),
        };

        Ok(Some(did_doc))
    }
}

/// A multi-resolver for DID methods. This resolver manages multiple
/// method-specific resolver. New resolvers can be added at runtime.
#[derive(Debug)]
#[cfg(not(target_arch = "wasm32"))]
pub struct MultiResolver {
    resolvers: RwLock<HashMap<String, Arc<dyn DIDMethodResolver>>>,
}

#[cfg(not(target_arch = "wasm32"))]
unsafe impl Send for MultiResolver {}
#[cfg(not(target_arch = "wasm32"))]
unsafe impl Sync for MultiResolver {}

#[cfg(not(target_arch = "wasm32"))]
impl MultiResolver {
    /// Create a new empty MultiResolver
    pub fn new() -> Self {
        Self {
            resolvers: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new MultiResolver with a list of resolvers
    pub fn new_with_resolvers(resolvers: Vec<Arc<dyn DIDMethodResolver>>) -> Self {
        let resolver = Self::new();

        // Add each resolver to the map if we can acquire the write lock
        if let Ok(mut resolver_map) = resolver.resolvers.write() {
            for r in resolvers {
                let method = r.method().to_string();
                resolver_map.insert(method, r);
            }
        }

        resolver
    }

    /// Register a new resolver for a specific DID method
    pub fn register_method<R>(&mut self, method: &str, resolver: R) -> &mut Self
    where
        R: DIDMethodResolver + Send + Sync + 'static,
    {
        if let Ok(mut resolvers) = self.resolvers.write() {
            resolvers.insert(method.to_string(), Arc::new(resolver));
        }
        self
    }
}

/// DID Web Resolver for resolving did:web identifiers
#[derive(Debug, Default)]
pub struct WebResolver;

impl WebResolver {
    /// Create a new WebResolver
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(target_arch = "wasm32")]
impl WasmDIDMethodResolver for WebResolver {
    fn method(&self) -> &str {
        "web"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn resolve_method(&self, did: &str) -> Result<Option<DIDDoc>> {
        // For WASM, return a simple placeholder DID document without actual resolution
        // Because we lack the proper web-fetch capabilities at the moment
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 || parts[0] != "did" || parts[1] != "web" {
            return Err(Error::InvalidDID);
        }

        // Create a minimal DID document for did:web
        let verification_method = VerificationMethod {
            id: format!("{}#keys-1", did),
            type_: VerificationMethodType::Ed25519VerificationKey2018,
            controller: did.to_string(),
            verification_material: VerificationMaterial::Multibase {
                public_key_multibase: "zMockPublicKey".to_string(),
            },
        };

        let did_doc = DIDDoc {
            id: did.to_string(),
            verification_method: vec![verification_method.clone()],
            authentication: vec![verification_method.id.clone()],
            key_agreement: Vec::new(),
            assertion_method: Vec::new(),
            capability_invocation: Vec::new(),
            capability_delegation: Vec::new(),
            service: Vec::new(),
        };

        Ok(Some(did_doc))
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl DIDMethodResolver for WebResolver {
    fn method(&self) -> &str {
        "web"
    }

    async fn resolve_method(&self, did: &str) -> Result<Option<DIDDoc>> {
        // Extract domain from did:web format
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 || parts[0] != "did" || parts[1] != "web" {
            return Err(Error::InvalidDID);
        }

        // Extract the domain (and path if present)
        let domain_path = parts[2..].join(":");
        let domain_path = domain_path.replace("%3A", ":");

        // Construct the URL to fetch the DID document
        // did:web:example.com -> https://example.com/.well-known/did.json
        // did:web:example.com:path:to:resource -> https://example.com/path/to/resource/did.json

        let url = if domain_path.contains(":") {
            // Convert additional colons to slashes for path components
            let path_segments: Vec<&str> = domain_path.split(':').collect();
            let domain = path_segments[0];
            let path = path_segments[1..].join("/");
            format!("https://{}/{}/did.json", domain, path)
        } else {
            // Standard case: did:web:example.com
            format!("https://{}/.well-known/did.json", domain_path)
        };

        // Attempt to fetch and parse the DID document
        #[cfg(feature = "native")]
        {
            use reqwest::Client;

            let client = Client::new();
            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => {
                                // First try normal parsing
                                let parse_result = serde_json::from_str::<DIDDoc>(&text);
                                match parse_result {
                                    Ok(doc) => {
                                        // Validate that the document ID matches the requested DID
                                        if doc.id != did {
                                            return Err(Error::DIDResolution(format!(
                                                "DID Document ID ({}) does not match requested DID ({})",
                                                doc.id, did
                                            )));
                                        }
                                        Ok(Some(doc))
                                    }
                                    Err(parse_error) => {
                                        // If normal parsing fails, try to parse as a generic JSON Value
                                        // and manually construct a DIDDoc with the essential fields
                                        match serde_json::from_str::<serde_json::Value>(&text) {
                                            Ok(json_value) => {
                                                let doc_id = match json_value.get("id") {
                                                    Some(id) => match id.as_str() {
                                                        Some(id_str) => id_str.to_string(),
                                                        None => return Err(Error::DIDResolution(
                                                            "DID Document has invalid 'id' field"
                                                                .to_string(),
                                                        )),
                                                    },
                                                    None => {
                                                        return Err(Error::DIDResolution(
                                                            "DID Document missing 'id' field"
                                                                .to_string(),
                                                        ))
                                                    }
                                                };

                                                // Validate ID
                                                if doc_id != did {
                                                    return Err(Error::DIDResolution(format!(
                                                        "DID Document ID ({}) does not match requested DID ({})",
                                                        doc_id, did
                                                    )));
                                                }

                                                // Try to extract verification methods and other fields
                                                warn!("Using partial DID document parsing due to format issues");
                                                warn!("Original parse error: {}", parse_error);

                                                // Extract verification methods if present
                                                // Create a longer-lived empty vec to handle the None case
                                                let empty_vec = Vec::new();
                                                let vm_array = json_value
                                                    .get("verificationMethod")
                                                    .and_then(|v| v.as_array())
                                                    .unwrap_or(&empty_vec);

                                                // Attempt to parse each verification method
                                                let mut verification_methods = Vec::new();
                                                for vm_value in vm_array {
                                                    if let Ok(vm) = serde_json::from_value::<
                                                        VerificationMethod,
                                                    >(
                                                        vm_value.clone()
                                                    ) {
                                                        verification_methods.push(vm);
                                                    }
                                                }

                                                // Extract authentication references
                                                let authentication = json_value
                                                    .get("authentication")
                                                    .and_then(|v| v.as_array())
                                                    .unwrap_or(&empty_vec)
                                                    .iter()
                                                    .filter_map(|v| {
                                                        v.as_str().map(|s| s.to_string())
                                                    })
                                                    .collect();

                                                // Extract key agreement references
                                                let key_agreement = json_value
                                                    .get("keyAgreement")
                                                    .and_then(|v| v.as_array())
                                                    .unwrap_or(&empty_vec)
                                                    .iter()
                                                    .filter_map(|v| {
                                                        v.as_str().map(|s| s.to_string())
                                                    })
                                                    .collect();

                                                // We'll create an empty services list for the DIDDoc
                                                // But save service information separately for display purposes
                                                let services = Vec::new();

                                                // Extract raw service information for display
                                                if let Some(svc_array) = json_value
                                                    .get("service")
                                                    .and_then(|v| v.as_array())
                                                {
                                                    debug!("\nService endpoints (extracted from JSON):");
                                                    for (i, svc_value) in
                                                        svc_array.iter().enumerate()
                                                    {
                                                        if let (Some(id), Some(endpoint)) = (
                                                            svc_value
                                                                .get("id")
                                                                .and_then(|v| v.as_str()),
                                                            svc_value
                                                                .get("serviceEndpoint")
                                                                .and_then(|v| v.as_str()),
                                                        ) {
                                                            let type_value = svc_value
                                                                .get("type")
                                                                .and_then(|v| v.as_str())
                                                                .unwrap_or("Unknown");

                                                            debug!("  [{}] ID: {}", i + 1, id);
                                                            debug!("      Type: {}", type_value);
                                                            debug!("      Endpoint: {}", endpoint);
                                                        }
                                                    }
                                                }

                                                // Create a simplified DID document with whatever we could extract
                                                let simplified_doc = DIDDoc {
                                                    id: doc_id,
                                                    verification_method: verification_methods,
                                                    authentication,
                                                    key_agreement,
                                                    assertion_method: Vec::new(),
                                                    capability_invocation: Vec::new(),
                                                    capability_delegation: Vec::new(),
                                                    service: services,
                                                };

                                                Ok(Some(simplified_doc))
                                            }
                                            Err(_) => Err(Error::DIDResolution(format!(
                                                "Failed to parse DID document from {}: {}",
                                                url, parse_error
                                            ))),
                                        }
                                    }
                                }
                            }
                            Err(e) => Err(Error::DIDResolution(format!(
                                "Failed to read response body from {}: {}",
                                url, e
                            ))),
                        }
                    } else if response.status().as_u16() == 404 {
                        // Not found is a valid response, just return None
                        Ok(None)
                    } else {
                        Err(Error::DIDResolution(format!(
                            "HTTP error fetching DID document from {}: {}",
                            url,
                            response.status()
                        )))
                    }
                }
                Err(e) => Err(Error::DIDResolution(format!(
                    "Failed to fetch DID document from {}: {}",
                    url, e
                ))),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;
            use wasm_bindgen_futures::JsFuture;
            use web_sys::{Headers, Request, RequestInit, RequestMode, Response};

            // Create request options
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);

            // Create the request
            let request = match Request::new_with_str_and_init(&url, &opts) {
                Ok(req) => req,
                Err(e) => {
                    return Err(Error::DIDResolution(format!(
                        "Failed to create request for {}: {:?}",
                        url, e
                    )));
                }
            };

            // Add Accept header
            let headers = match Headers::new() {
                Ok(h) => h,
                Err(e) => {
                    return Err(Error::DIDResolution(format!(
                        "Failed to create headers: {:?}",
                        e
                    )));
                }
            };

            if let Err(e) = headers.set("Accept", "application/json") {
                return Err(Error::DIDResolution(format!(
                    "Failed to set Accept header: {:?}",
                    e
                )));
            }

            if let Err(e) = request.headers().set("Accept", "application/json") {
                return Err(Error::DIDResolution(format!(
                    "Failed to set Accept header: {:?}",
                    e
                )));
            }

            // Get the window object
            let window = match web_sys::window() {
                Some(w) => w,
                None => {
                    return Err(Error::DIDResolution(
                        "No window object available".to_string(),
                    ));
                }
            };

            // Send the request
            let resp_value = match JsFuture::from(window.fetch_with_request(&request)).await {
                Ok(response) => response,
                Err(e) => {
                    return Err(Error::DIDResolution(format!(
                        "Failed to fetch DID document from {}: {:?}",
                        url, e
                    )));
                }
            };

            // Convert response to Response object
            let resp: Response = match resp_value.dyn_into() {
                Ok(r) => r,
                Err(_) => {
                    return Err(Error::DIDResolution(
                        "Failed to convert response".to_string(),
                    ));
                }
            };

            // Check if successful
            if resp.ok() {
                // Get the text content
                let text_promise = match resp.text() {
                    Ok(t) => t,
                    Err(e) => {
                        return Err(Error::DIDResolution(format!(
                            "Failed to get text from response: {:?}",
                            e
                        )));
                    }
                };

                let text_jsval = match JsFuture::from(text_promise).await {
                    Ok(t) => t,
                    Err(e) => {
                        return Err(Error::DIDResolution(format!(
                            "Failed to await text promise: {:?}",
                            e
                        )));
                    }
                };

                let text = match text_jsval.as_string() {
                    Some(t) => t,
                    None => {
                        return Err(Error::DIDResolution("Response is not a string".to_string()));
                    }
                };

                // Parse the DID document
                match serde_json::from_str::<DIDDoc>(&text) {
                    Ok(doc) => {
                        // Validate that the document ID matches the requested DID
                        if doc.id != did {
                            return Err(Error::DIDResolution(format!(
                                "DID Document ID ({}) does not match requested DID ({})",
                                doc.id, did
                            )));
                        }
                        Ok(Some(doc))
                    }
                    Err(parse_error) => {
                        // If normal parsing fails, try to parse as a generic JSON Value
                        // and manually construct a DIDDoc with the essential fields
                        match serde_json::from_str::<serde_json::Value>(&text) {
                            Ok(json_value) => {
                                let doc_id = match json_value.get("id") {
                                    Some(id) => match id.as_str() {
                                        Some(id_str) => id_str.to_string(),
                                        None => {
                                            return Err(Error::DIDResolution(
                                                "DID Document has invalid 'id' field".to_string(),
                                            ))
                                        }
                                    },
                                    None => {
                                        return Err(Error::DIDResolution(
                                            "DID Document missing 'id' field".to_string(),
                                        ))
                                    }
                                };

                                // Validate ID
                                if doc_id != did {
                                    return Err(Error::DIDResolution(format!(
                                        "DID Document ID ({}) does not match requested DID ({})",
                                        doc_id, did
                                    )));
                                }

                                // Try to extract verification methods and other fields
                                web_sys::console::log_1(&JsValue::from_str(
                                    &format!("WARNING: Using partial DID document parsing due to format issues\nOriginal parse error: {}", parse_error)
                                ));

                                // Extract verification methods if present
                                // Create a longer-lived empty vec to handle the None case
                                let empty_vec = Vec::new();
                                let vm_array = json_value
                                    .get("verificationMethod")
                                    .and_then(|v| v.as_array())
                                    .unwrap_or(&empty_vec);

                                // Attempt to parse each verification method
                                let mut verification_methods = Vec::new();
                                for vm_value in vm_array {
                                    if let Ok(vm) = serde_json::from_value::<VerificationMethod>(
                                        vm_value.clone(),
                                    ) {
                                        verification_methods.push(vm);
                                    }
                                }

                                // Extract authentication references
                                let authentication = json_value
                                    .get("authentication")
                                    .and_then(|v| v.as_array())
                                    .unwrap_or(&empty_vec)
                                    .iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect();

                                // Extract key agreement references
                                let key_agreement = json_value
                                    .get("keyAgreement")
                                    .and_then(|v| v.as_array())
                                    .unwrap_or(&empty_vec)
                                    .iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect();

                                // Create an empty services list for the DIDDoc
                                let services = Vec::new();

                                // Extract raw service information for console logging
                                if let Some(svc_array) =
                                    json_value.get("service").and_then(|v| v.as_array())
                                {
                                    web_sys::console::log_1(&JsValue::from_str(
                                        "Service endpoints (extracted from JSON):",
                                    ));
                                    for (i, svc_value) in svc_array.iter().enumerate() {
                                        if let (Some(id), Some(endpoint)) = (
                                            svc_value.get("id").and_then(|v| v.as_str()),
                                            svc_value
                                                .get("serviceEndpoint")
                                                .and_then(|v| v.as_str()),
                                        ) {
                                            let type_value = svc_value
                                                .get("type")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("Unknown");

                                            web_sys::console::log_1(&JsValue::from_str(&format!(
                                                "[{}] ID: {}\nType: {}\nEndpoint: {}",
                                                i + 1,
                                                id,
                                                type_value,
                                                endpoint
                                            )));
                                        }
                                    }
                                }

                                // Create a simplified DID document with whatever we could extract
                                let simplified_doc = DIDDoc {
                                    id: doc_id,
                                    verification_method: verification_methods,
                                    authentication,
                                    key_agreement,
                                    assertion_method: Vec::new(),
                                    capability_invocation: Vec::new(),
                                    capability_delegation: Vec::new(),
                                    service: services,
                                };

                                Ok(Some(simplified_doc))
                            }
                            Err(_) => Err(Error::DIDResolution(format!(
                                "Failed to parse DID document from {}: {}",
                                url, parse_error
                            ))),
                        }
                    }
                }
            } else if resp.status() == 404 {
                // Not found is a valid response, just return None
                Ok(None)
            } else {
                Err(Error::DIDResolution(format!(
                    "HTTP error fetching DID document from {}: {}",
                    url,
                    resp.status()
                )))
            }
        }

        #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
        {
            Err(Error::DIDResolution(
                "Web DID resolution requires the 'native' feature or WASM".to_string(),
            ))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for MultiResolver {
    fn default() -> Self {
        let mut resolver = Self::new();
        resolver.register_method("key", KeyResolver::new());
        resolver.register_method("web", WebResolver::new());
        resolver
    }
}

/// DID Key Generator for creating DIDs with different key types
#[derive(Debug, Default, Clone)]
pub struct DIDKeyGenerator;

impl DIDKeyGenerator {
    /// Create a new DID key generator
    pub fn new() -> Self {
        Self
    }

    /// Create a Secret from a GeneratedKey for a DID
    pub fn create_secret_from_key(&self, key: &GeneratedKey) -> Secret {
        // Determine the proper key ID based on DID method
        let kid = if key.did.starts_with("did:key:") {
            // For did:key, the key ID is the DID + fragment matching the multibase
            // Get the first verification method's ID which has the proper format
            key.did_doc
                .verification_method
                .first()
                .map(|vm| vm.id.clone())
                .unwrap_or_else(|| {
                    // Fallback: extract the multibase part and construct the ID
                    let multibase = key.did.strip_prefix("did:key:").unwrap_or("");
                    format!("{}#{}", key.did, multibase)
                })
        } else if key.did.starts_with("did:web:") {
            // For did:web, use #keys-1
            format!("{}#keys-1", key.did)
        } else {
            // For other DID methods, use a generic pattern
            format!("{}#key-1", key.did)
        };

        match key.key_type {
            KeyType::Ed25519 => Secret {
                id: key.did.clone(),
                type_: SecretType::JsonWebKey2020,
                secret_material: SecretMaterial::JWK {
                    private_key_jwk: serde_json::json!({
                        "kty": "OKP",
                        "kid": kid,
                        "crv": "Ed25519",
                        "x": base64::engine::general_purpose::STANDARD.encode(&key.public_key),
                        "d": base64::engine::general_purpose::STANDARD.encode(&key.private_key)
                    }),
                },
            },
            KeyType::P256 => Secret {
                id: key.did.clone(),
                type_: SecretType::JsonWebKey2020,
                secret_material: SecretMaterial::JWK {
                    private_key_jwk: serde_json::json!({
                        "kty": "EC",
                        "kid": kid,
                        "crv": "P-256",
                        "x": base64::engine::general_purpose::STANDARD.encode(&key.public_key[0..32]),
                        "y": base64::engine::general_purpose::STANDARD.encode(&key.public_key[32..64]),
                        "d": base64::engine::general_purpose::STANDARD.encode(&key.private_key)
                    }),
                },
            },
            KeyType::Secp256k1 => Secret {
                id: key.did.clone(),
                type_: SecretType::JsonWebKey2020,
                secret_material: SecretMaterial::JWK {
                    private_key_jwk: serde_json::json!({
                        "kty": "EC",
                        "kid": kid,
                        "crv": "secp256k1",
                        "x": base64::engine::general_purpose::STANDARD.encode(&key.public_key[0..32]),
                        "y": base64::engine::general_purpose::STANDARD.encode(&key.public_key[32..64]),
                        "d": base64::engine::general_purpose::STANDARD.encode(&key.private_key)
                    }),
                },
            },
        }
    }

    /// Generate a did:key identifier with the specified key type
    pub fn generate_did(&self, options: DIDGenerationOptions) -> Result<GeneratedKey> {
        match options.key_type {
            KeyType::Ed25519 => self.generate_ed25519_did(),
            KeyType::P256 => self.generate_p256_did(),
            KeyType::Secp256k1 => self.generate_secp256k1_did(),
        }
    }

    /// Generate a did:key identifier with an Ed25519 key
    pub fn generate_ed25519_did(&self) -> Result<GeneratedKey> {
        // Generate a new Ed25519 keypair
        let mut csprng = OsRng;
        let signing_key = Ed25519SigningKey::generate(&mut csprng);
        let verifying_key = Ed25519VerifyingKey::from(&signing_key);

        // Extract public and private keys
        let public_key = verifying_key.to_bytes().to_vec();
        let private_key = signing_key.to_bytes().to_vec();

        // Create did:key identifier
        // Multicodec prefix for Ed25519: 0xed01
        let mut prefixed_key = vec![0xed, 0x01];
        prefixed_key.extend_from_slice(&public_key);

        // Encode the key with multibase (base58btc with 'z' prefix)
        let multibase_encoded = encode(Base::Base58Btc, &prefixed_key);
        let did = format!("did:key:{}", multibase_encoded);

        // Create the DID document
        let doc = self.create_did_doc(&did, &prefixed_key, KeyType::Ed25519)?;

        // Return the generated key information
        Ok(GeneratedKey {
            key_type: KeyType::Ed25519,
            did,
            public_key,
            private_key,
            did_doc: doc,
        })
    }

    /// Generate a did:key identifier with a P-256 key
    pub fn generate_p256_did(&self) -> Result<GeneratedKey> {
        // Generate a new P-256 keypair
        let mut rng = OsRng;
        let signing_key = P256SigningKey::random(&mut rng);

        // Extract public and private keys
        let private_key = signing_key.to_bytes().to_vec();
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

        // Create the DID document
        let doc = self.create_did_doc(&did, &prefixed_key, KeyType::P256)?;

        // Return the generated key information
        Ok(GeneratedKey {
            key_type: KeyType::P256,
            did,
            public_key,
            private_key,
            did_doc: doc,
        })
    }

    /// Generate a did:key identifier with a Secp256k1 key
    pub fn generate_secp256k1_did(&self) -> Result<GeneratedKey> {
        // Generate a new Secp256k1 keypair
        let mut rng = OsRng;
        let signing_key = Secp256k1SigningKey::random(&mut rng);

        // Extract public and private keys
        let private_key = signing_key.to_bytes().to_vec();
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

        // Create the DID document
        let doc = self.create_did_doc(&did, &prefixed_key, KeyType::Secp256k1)?;

        // Return the generated key information
        Ok(GeneratedKey {
            key_type: KeyType::Secp256k1,
            did,
            public_key,
            private_key,
            did_doc: doc,
        })
    }

    /// Generate a did:web identifier with the given domain and key type
    pub fn generate_web_did(
        &self,
        domain: &str,
        options: DIDGenerationOptions,
    ) -> Result<GeneratedKey> {
        // First, generate a key DID of the appropriate type
        let key_did = self.generate_did(options)?;

        // Format the did:web identifier
        let did = format!("did:web:{}", domain);

        // Create a new DID document based on the key DID document but with the web DID
        let verification_methods: Vec<VerificationMethod> = key_did
            .did_doc
            .verification_method
            .iter()
            .map(|vm| {
                let id = format!("{}#keys-1", did);
                VerificationMethod {
                    id: id.clone(),
                    type_: vm.type_.clone(),
                    controller: did.clone(),
                    verification_material: vm.verification_material.clone(),
                }
            })
            .collect();

        let did_doc = DIDDoc {
            id: did.clone(),
            verification_method: verification_methods.clone(),
            authentication: verification_methods
                .iter()
                .map(|vm| vm.id.clone())
                .collect(),
            key_agreement: key_did.did_doc.key_agreement,
            assertion_method: Vec::new(),
            capability_invocation: Vec::new(),
            capability_delegation: Vec::new(),
            service: vec![],
        };

        // Return the generated key information with the web DID
        Ok(GeneratedKey {
            key_type: key_did.key_type,
            did,
            public_key: key_did.public_key,
            private_key: key_did.private_key,
            did_doc,
        })
    }

    /// Create a DID document for a did:key
    fn create_did_doc(
        &self,
        did: &str,
        prefixed_public_key: &[u8],
        key_type: KeyType,
    ) -> Result<DIDDoc> {
        // Determine verification method type based on key type
        let verification_method_type = match key_type {
            KeyType::Ed25519 => VerificationMethodType::Ed25519VerificationKey2018,
            KeyType::P256 => VerificationMethodType::EcdsaSecp256k1VerificationKey2019, // Using Secp256k1 type as P256 isn't available
            KeyType::Secp256k1 => VerificationMethodType::EcdsaSecp256k1VerificationKey2019,
        };

        // Encode the prefixed public key with multibase
        let multibase_encoded = encode(Base::Base58Btc, prefixed_public_key);

        // Create the verification method ID
        let vm_id = format!("{}#{}", did, multibase_encoded);

        // Create the verification method
        let verification_method = VerificationMethod {
            id: vm_id.clone(),
            type_: verification_method_type.clone(),
            controller: did.to_string(),
            verification_material: VerificationMaterial::Multibase {
                public_key_multibase: multibase_encoded.clone(),
            },
        };

        // For Ed25519, also generate an X25519 verification method for key agreement
        let mut verification_methods = vec![verification_method.clone()];
        let mut key_agreement = Vec::new();

        if key_type == KeyType::Ed25519 {
            // Only Ed25519 keys have an X25519 key agreement method
            if let Some(x25519_bytes) = self.ed25519_to_x25519(&prefixed_public_key[2..]) {
                // Prefix for X25519: 0xEC01
                let mut x25519_prefixed = vec![0xEC, 0x01];
                x25519_prefixed.extend_from_slice(&x25519_bytes);

                // Encode the prefixed X25519 key with multibase
                let x25519_multibase = encode(Base::Base58Btc, &x25519_prefixed);

                // Create the X25519 verification method ID
                let x25519_vm_id = format!("{}#{}", did, x25519_multibase);

                // Create the X25519 verification method
                let x25519_verification_method = VerificationMethod {
                    id: x25519_vm_id.clone(),
                    type_: VerificationMethodType::X25519KeyAgreementKey2019,
                    controller: did.to_string(),
                    verification_material: VerificationMaterial::Multibase {
                        public_key_multibase: x25519_multibase,
                    },
                };

                // Add the X25519 verification method and key agreement method
                verification_methods.push(x25519_verification_method);
                key_agreement.push(x25519_vm_id);
            }
        }

        // Create the DID document
        let did_doc = DIDDoc {
            id: did.to_string(),
            verification_method: verification_methods,
            authentication: vec![vm_id.clone()],
            key_agreement,
            assertion_method: Vec::new(),
            capability_invocation: Vec::new(),
            capability_delegation: Vec::new(),
            service: vec![],
        };

        Ok(did_doc)
    }

    /// Convert an Ed25519 public key to an X25519 public key
    ///
    /// This follows the conversion process described in RFC 7748
    /// https://datatracker.ietf.org/doc/html/rfc7748#section-5
    fn ed25519_to_x25519(&self, ed25519_pubkey: &[u8]) -> Option<[u8; 32]> {
        // The Ed25519 public key should be 32 bytes
        if ed25519_pubkey.len() != 32 {
            return None;
        }

        // Try to create a CompressedEdwardsY from the bytes
        let edwards_y = match CompressedEdwardsY::from_slice(ed25519_pubkey) {
            Ok(point) => point,
            Err(_) => return None,
        };

        // Try to decompress to get the Edwards point
        let edwards_point = edwards_y.decompress()?;

        // Convert to Montgomery form
        let montgomery_point = edwards_point.to_montgomery();

        // Get the raw bytes representation of the X25519 key
        Some(montgomery_point.to_bytes())
    }

    // The create_secret_from_key method has been moved up
}

#[derive(Debug)]
#[cfg(target_arch = "wasm32")]
pub struct MultiResolver {
    // WASM-specific implementation with no thread-safety requirements
    resolvers: HashMap<String, Box<dyn WasmDIDMethodResolver>>,
}

#[cfg(target_arch = "wasm32")]
impl MultiResolver {
    pub fn new() -> Self {
        Self {
            resolvers: HashMap::new(),
        }
    }

    pub fn default() -> Self {
        let mut resolver = Self::new();
        // Add default resolvers
        resolver.add_resolver(Box::new(KeyResolver::new()));
        resolver
    }

    pub fn add_resolver(&mut self, resolver: Box<dyn WasmDIDMethodResolver>) {
        self.resolvers
            .insert(resolver.method().to_string(), resolver);
    }
}

#[cfg(target_arch = "wasm32")]
impl WasmDIDResolver for MultiResolver {
    fn resolve(&self, did: &str) -> Result<Option<DIDDoc>> {
        // Extract the DID method
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 {
            return Err(Error::InvalidDID);
        }

        let method = parts[1];

        // Get the resolver from the map
        if let Some(resolver) = self.resolvers.get(method) {
            resolver.resolve_method(did)
        } else {
            Err(Error::UnsupportedDIDMethod(format!(
                "Method {} is not a WasmDIDMethodResolver",
                method
            )))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
#[cfg(not(target_arch = "wasm32"))]
impl SyncDIDResolver for MultiResolver {
    async fn resolve(&self, did: &str) -> Result<Option<DIDDoc>> {
        // Extract the DID method
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 {
            return Err(Error::InvalidDID);
        }

        let method = parts[1];

        // Get the resolver from the map first
        let resolver = {
            let resolver_guard = self
                .resolvers
                .read()
                .map_err(|_| Error::FailedToAcquireResolverReadLock)?;
            if let Some(resolver) = resolver_guard.get(method) {
                resolver.clone()
            } else {
                return Err(Error::UnsupportedDIDMethod(method.to_string()));
            }
            // Lock is dropped here when resolver_guard goes out of scope
        };

        // Now use the resolver without holding the lock
        resolver.resolve_method(did).await
    }
}

// DIDResolver trait from didcomm is no longer needed since we've removed the didcomm dependency

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys::{Function, Promise};

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use wasm_bindgen_futures::JsFuture;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Promise<string>")]
    pub type JsPromiseString;

    #[wasm_bindgen(typescript_type = "Promise<string | null>")]
    pub type JsPromiseStringOrNull;
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct JsDIDResolver {
    method: String,
    resolve_fn: Function,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl JsDIDResolver {
    #[wasm_bindgen(constructor)]
    pub fn new(resolve_fn: Function) -> Self {
        Self {
            method: "".to_string(),
            resolve_fn,
        }
    }

    #[wasm_bindgen]
    pub fn method(&self) -> String {
        // The JS resolver should return its method
        let this = JsValue::null();
        let method = self
            .resolve_fn
            .call1(&this, &JsValue::from_str("method"))
            .unwrap_or_else(|_| JsValue::from_str("unknown"));

        method.as_string().unwrap_or_else(|| "unknown".to_string())
    }
}

// This is a duplicate trait that conflicts with the one defined above
// So we're removing it here to avoid the conflict

/// A wrapper for JavaScript DID resolvers.
#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct JsDIDMethodResolver {
    method: String,
    resolve_fn: Function,
}

#[cfg(target_arch = "wasm32")]
impl JsDIDMethodResolver {
    /// Create a new JavaScript DID method resolver from a function in the global context
    pub fn new(method: &str, resolve_fn: Function) -> Self {
        Self {
            method: method.to_string(),
            resolve_fn,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl WasmDIDMethodResolver for JsDIDMethodResolver {
    fn method(&self) -> &str {
        &self.method
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn resolve_method(&self, did: &str) -> Result<Option<DIDDoc>> {
        // Ensure the DID is for the method that this resolver is for
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 || parts[1] != self.method {
            return Err(Error::InvalidDID);
        }

        // In WASM target mode, we can't use async/await in this interface
        // This implementation is a simplified version that just returns None
        // The proper implementation would be in the JavaScript binding
        Err(Error::NotImplemented(
            "JS resolver not supported in this context".to_string(),
        ))
    }

    #[cfg(not(feature = "wasm"))]
    async fn resolve_method(&self, _did: &str) -> Result<Option<DIDDoc>> {
        Err(Error::NotImplemented(
            "JavaScript DID Method resolver is only available with the 'wasm' feature".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn test_key_resolver() {
        let resolver = KeyResolver::new();

        // Test a valid did:key for Ed25519
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let result = resolver.resolve_method(did).await.unwrap();

        assert!(result.is_some());
        let doc = result.unwrap();

        assert_eq!(doc.id, did);
        assert_eq!(doc.verification_method.len(), 2); // Should have both Ed25519 and X25519 methods

        // Verify Ed25519 verification method is present
        let ed25519_method = doc
            .verification_method
            .iter()
            .find(|vm| matches!(vm.type_, VerificationMethodType::Ed25519VerificationKey2018))
            .expect("Should have an Ed25519 verification method");

        // Verify X25519 verification method
        let x25519_method = doc
            .verification_method
            .iter()
            .find(|vm| matches!(vm.type_, VerificationMethodType::X25519KeyAgreementKey2019))
            .expect("Should have an X25519 key agreement method");

        // Check that authentication uses the Ed25519 key
        assert!(doc.authentication.contains(&ed25519_method.id));

        // Check that key agreement uses the X25519 key
        assert!(doc.key_agreement.contains(&x25519_method.id));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn test_multi_resolver() {
        let resolver = MultiResolver::default();

        // Test resolving a valid did:key
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let result = <MultiResolver as SyncDIDResolver>::resolve(&resolver, did).await;

        assert!(result.is_ok());
        let doc_option = result.unwrap();
        assert!(doc_option.is_some());

        let doc = doc_option.unwrap();
        assert_eq!(doc.id, did);
        assert_eq!(doc.verification_method.len(), 2); // Should have both Ed25519 and X25519 methods

        // Test resolving an unsupported DID method
        let did = "did:unsupported:123";
        let result = <MultiResolver as SyncDIDResolver>::resolve(&resolver, did).await;

        // This should return an error since it's an unsupported method
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unsupported DID method"));
    }

    #[test]
    fn test_did_key_generator_ed25519() {
        let generator = DIDKeyGenerator::new();

        // Generate an Ed25519 DID
        let options = DIDGenerationOptions {
            key_type: KeyType::Ed25519,
        };

        let key_result = generator.generate_did(options);
        assert!(key_result.is_ok());

        let key = key_result.unwrap();

        // Check the DID format
        assert!(key.did.starts_with("did:key:z"));

        // Check that public and private keys have the correct length
        assert_eq!(key.public_key.len(), 32); // Ed25519 public key is 32 bytes
        assert_eq!(key.private_key.len(), 32); // Ed25519 private key is 32 bytes

        // Check the DID document
        assert_eq!(key.did_doc.id, key.did);
        assert_eq!(key.did_doc.verification_method.len(), 2); // Should have both Ed25519 and X25519

        // Verify Ed25519 verification method is present
        let ed25519_method = key
            .did_doc
            .verification_method
            .iter()
            .find(|vm| matches!(vm.type_, VerificationMethodType::Ed25519VerificationKey2018))
            .expect("Should have an Ed25519 verification method");

        // Verify X25519 verification method
        let x25519_method = key
            .did_doc
            .verification_method
            .iter()
            .find(|vm| matches!(vm.type_, VerificationMethodType::X25519KeyAgreementKey2019))
            .expect("Should have an X25519 key agreement method");

        // Check that authentication uses the Ed25519 key
        assert!(key.did_doc.authentication.contains(&ed25519_method.id));

        // Check that key agreement uses the X25519 key
        assert!(key.did_doc.key_agreement.contains(&x25519_method.id));

        // Create a secret from the key
        let secret = generator.create_secret_from_key(&key);
        assert_eq!(secret.id, key.did);
        assert!(matches!(secret.type_, SecretType::JsonWebKey2020));
    }

    #[test]
    fn test_did_key_generator_p256() {
        let generator = DIDKeyGenerator::new();

        // Generate a P-256 DID
        let options = DIDGenerationOptions {
            key_type: KeyType::P256,
        };

        let key_result = generator.generate_did(options);
        assert!(key_result.is_ok());

        let key = key_result.unwrap();

        // Check the DID format
        assert!(key.did.starts_with("did:key:z"));

        // Check the DID document
        assert_eq!(key.did_doc.id, key.did);
        assert_eq!(key.did_doc.verification_method.len(), 1); // P-256 has no key agreement

        // Verify P-256 verification method is present
        let p256_method = key
            .did_doc
            .verification_method
            .iter()
            .find(|vm| {
                matches!(
                    vm.type_,
                    VerificationMethodType::EcdsaSecp256k1VerificationKey2019
                )
            }) // Use available type
            .expect("Should have a P-256 verification method");

        // Check that authentication uses the P-256 key
        assert!(key.did_doc.authentication.contains(&p256_method.id));

        // Create a secret from the key
        let secret = generator.create_secret_from_key(&key);
        assert_eq!(secret.id, key.did);
        assert!(matches!(secret.type_, SecretType::JsonWebKey2020));
    }

    #[test]
    fn test_did_key_generator_secp256k1() {
        let generator = DIDKeyGenerator::new();

        // Generate a Secp256k1 DID
        let options = DIDGenerationOptions {
            key_type: KeyType::Secp256k1,
        };

        let key_result = generator.generate_did(options);
        assert!(key_result.is_ok());

        let key = key_result.unwrap();

        // Check the DID format
        assert!(key.did.starts_with("did:key:z"));

        // Check the DID document
        assert_eq!(key.did_doc.id, key.did);
        assert_eq!(key.did_doc.verification_method.len(), 1); // Secp256k1 has no key agreement

        // Verify Secp256k1 verification method is present
        let secp256k1_method = key
            .did_doc
            .verification_method
            .iter()
            .find(|vm| {
                matches!(
                    vm.type_,
                    VerificationMethodType::EcdsaSecp256k1VerificationKey2019
                )
            })
            .expect("Should have a Secp256k1 verification method");

        // Check that authentication uses the Secp256k1 key
        assert!(key.did_doc.authentication.contains(&secp256k1_method.id));

        // Create a secret from the key
        let secret = generator.create_secret_from_key(&key);
        assert_eq!(secret.id, key.did);
        assert!(matches!(secret.type_, SecretType::JsonWebKey2020));
    }

    #[test]
    fn test_did_web_generator() {
        let generator = DIDKeyGenerator::new();

        // Generate a did:web
        let domain = "example.com";
        let options = DIDGenerationOptions {
            key_type: KeyType::Ed25519,
        };

        let key_result = generator.generate_web_did(domain, options);
        assert!(key_result.is_ok());

        let key = key_result.unwrap();

        // Check the DID format
        assert_eq!(key.did, format!("did:web:{}", domain));

        // Check the DID document
        assert_eq!(key.did_doc.id, key.did);
        assert!(!key.did_doc.verification_method.is_empty());

        // Verify that all verification methods have the correct controller
        for vm in &key.did_doc.verification_method {
            assert_eq!(vm.controller, key.did);
            assert!(vm.id.starts_with(&key.did));
        }

        // Create a secret from the key
        let secret = generator.create_secret_from_key(&key);
        assert_eq!(secret.id, key.did);
        assert!(matches!(secret.type_, SecretType::JsonWebKey2020));
    }
}
