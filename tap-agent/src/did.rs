//! DID resolution functionality for the TAP Agent

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::error::{Error, Result};

/// Represents a DID Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDoc {
    /// The DID of the document
    pub id: String,
    /// The verification methods in the DID Document
    pub verification_method: Vec<VerificationMethod>,
    /// The service endpoints in the DID Document
    pub service: Option<Vec<Service>>,
}

/// Represents a verification method in a DID Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    /// The ID of the verification method
    pub id: String,
    /// The controller DID of the verification method
    pub controller: String,
    /// The type of the verification method
    #[serde(rename = "type")]
    pub type_: String,
    /// The public key JWK
    pub public_key_jwk: Option<serde_json::Value>,
    /// The public key base58
    pub public_key_base58: Option<String>,
}

/// Represents a service endpoint in a DID Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// The ID of the service
    pub id: String,
    /// The type of service
    #[serde(rename = "type")]
    pub type_: String,
    /// The service endpoint URI
    pub service_endpoint: String,
}

/// A trait for resolving DIDs to DID documents
#[async_trait]
pub trait DidResolver: Send + Sync + Debug {
    /// Resolve a DID to a DID document
    async fn resolve(&self, did: &str) -> Result<String>;
}

/// A basic resolver that resolves DIDs using a static map
#[derive(Debug)]
pub struct BasicResolver {
    /// Map of DIDs to DID documents
    map: HashMap<String, DidDoc>,
}

impl Default for BasicResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicResolver {
    /// Creates a new empty BasicResolver
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

#[async_trait]
impl DidResolver for BasicResolver {
    async fn resolve(&self, did: &str) -> Result<String> {
        if let Some(doc) = self.map.get(did) {
            return Ok(serde_json::to_string(doc).unwrap());
        }

        Err(Error::DidResolution(format!(
            "No resolver found for DID: {}",
            did
        )))
    }
}

/// A resolver for did:key method
#[derive(Debug, Default)]
pub struct KeyResolver;

#[async_trait]
impl DidResolver for KeyResolver {
    async fn resolve(&self, did: &str) -> Result<String> {
        if !did.starts_with("did:key:") {
            return Err(Error::DidResolution(format!(
                "Invalid did:key format: {}",
                did
            )));
        }

        // A real implementation would handle proper did:key parsing and key extraction
        // For now, just return a placeholder DID Doc
        let doc = DidDoc {
            id: did.to_string(),
            verification_method: vec![],
            service: None,
        };
        Ok(serde_json::to_string(&doc).unwrap())
    }
}

/// A resolver for did:web method
#[derive(Debug, Default)]
pub struct WebResolver;

#[async_trait]
impl DidResolver for WebResolver {
    async fn resolve(&self, did: &str) -> Result<String> {
        if !did.starts_with("did:web:") {
            return Err(Error::DidResolution(format!(
                "Invalid did:web format: {}",
                did
            )));
        }

        // A real implementation would fetch the DID document from the web
        // For now, just return a placeholder DID Doc
        let doc = DidDoc {
            id: did.to_string(),
            verification_method: vec![],
            service: None,
        };
        Ok(serde_json::to_string(&doc).unwrap())
    }
}

/// A resolver for did:pkh method
#[derive(Debug, Default)]
pub struct PkhResolver;

#[async_trait]
impl DidResolver for PkhResolver {
    async fn resolve(&self, did: &str) -> Result<String> {
        if !did.starts_with("did:pkh:") {
            return Err(Error::DidResolution(format!(
                "Invalid did:pkh format: {}",
                did
            )));
        }

        // A real implementation would handle proper did:pkh parsing and key derivation
        // For now, just return a placeholder DID Doc
        let doc = DidDoc {
            id: did.to_string(),
            verification_method: vec![],
            service: None,
        };
        Ok(serde_json::to_string(&doc).unwrap())
    }
}

/// Multi resolver that tries multiple resolvers
#[derive(Debug, Default)]
pub struct MultiResolver {
    resolvers: Vec<Arc<dyn DidResolver>>,
}

impl MultiResolver {
    /// Creates a new empty MultiResolver
    pub fn new() -> Self {
        Self {
            resolvers: Vec::new(),
        }
    }

    /// Adds a resolver to the list of resolvers
    pub fn add_resolver<R: DidResolver + 'static>(&mut self, resolver: R) {
        self.resolvers.push(Arc::new(resolver));
    }
}

#[async_trait]
impl DidResolver for MultiResolver {
    async fn resolve(&self, did: &str) -> Result<String> {
        for resolver in &self.resolvers {
            match resolver.resolve(did).await {
                Ok(doc) => return Ok(doc),
                Err(_) => continue,
            }
        }

        Err(Error::DidResolution(format!(
            "No resolver found for DID: {}",
            did
        )))
    }
}

/// Default implementation of [DidResolver]
#[derive(Debug)]
pub struct DefaultDIDResolver {
    /// Cache of resolved DID documents
    cache: Arc<std::sync::RwLock<HashMap<String, String>>>,
}

impl Default for DefaultDIDResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultDIDResolver {
    /// Creates a new [DefaultDIDResolver]
    pub fn new() -> Self {
        Self {
            cache: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl DidResolver for DefaultDIDResolver {
    /// Resolves a DID to a DID document
    async fn resolve(&self, did: &str) -> Result<String> {
        // Check cache first
        if let Some(doc) = self.cache.read().unwrap().get(did) {
            return Ok(doc.clone());
        }

        // Simple mock resolver for testing
        // In a real implementation, this would delegate to specific resolvers based on the DID method
        let doc = match did {
            did if did.starts_with("did:key:") => {
                format!("{{ \"id\": \"{}\", \"authentication\": [{{ \"id\": \"#{}\", \"type\": \"Ed25519VerificationKey2018\" }}] }}", did, did)
            }
            did if did.starts_with("did:web:") => {
                format!("{{ \"id\": \"{}\", \"authentication\": [{{ \"id\": \"#{}\", \"type\": \"Ed25519VerificationKey2018\" }}] }}", did, did)
            }
            did if did.starts_with("did:pkh:") => {
                format!("{{ \"id\": \"{}\", \"authentication\": [{{ \"id\": \"#{}\", \"type\": \"EcdsaSecp256k1VerificationKey2019\" }}] }}", did, did)
            }
            did if did.starts_with("did:example:") => {
                format!("{{ \"id\": \"{}\", \"authentication\": [{{ \"id\": \"#{}\", \"type\": \"Ed25519VerificationKey2018\" }}] }}", did, did)
            }
            _ => {
                return Err(Error::DidResolution(format!(
                    "Unsupported DID method for {}",
                    did
                )))
            }
        };

        // Update cache
        self.cache
            .write()
            .unwrap()
            .insert(did.to_string(), doc.clone());

        Ok(doc)
    }
}
