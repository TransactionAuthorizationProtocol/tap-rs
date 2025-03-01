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

/// DID Resolution trait
#[async_trait]
pub trait DidResolver: Send + Sync + Debug {
    /// Resolves a DID to a DID Document
    async fn resolve(&self, did: &str) -> Result<DidDoc>;
}

/// A basic resolver that resolves DIDs using a static map
#[derive(Debug)]
pub struct BasicResolver {
    /// Map of DIDs to DID documents
    map: HashMap<String, DidDoc>,
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
    async fn resolve(&self, did: &str) -> Result<DidDoc> {
        if let Some(doc) = self.map.get(did) {
            return Ok(doc.clone());
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
    async fn resolve(&self, did: &str) -> Result<DidDoc> {
        if !did.starts_with("did:key:") {
            return Err(Error::DidResolution(format!(
                "Invalid did:key format: {}",
                did
            )));
        }

        // A real implementation would handle proper did:key parsing and key extraction
        // For now, just return a placeholder DID Doc
        Ok(DidDoc {
            id: did.to_string(),
            verification_method: vec![],
            service: None,
        })
    }
}

/// A resolver for did:web method
#[derive(Debug, Default)]
pub struct WebResolver;

#[async_trait]
impl DidResolver for WebResolver {
    async fn resolve(&self, did: &str) -> Result<DidDoc> {
        if !did.starts_with("did:web:") {
            return Err(Error::DidResolution(format!(
                "Invalid did:web format: {}",
                did
            )));
        }

        // A real implementation would fetch the DID document from the web
        // For now, just return a placeholder DID Doc
        Ok(DidDoc {
            id: did.to_string(),
            verification_method: vec![],
            service: None,
        })
    }
}

/// A resolver for did:pkh method
#[derive(Debug, Default)]
pub struct PkhResolver;

#[async_trait]
impl DidResolver for PkhResolver {
    async fn resolve(&self, did: &str) -> Result<DidDoc> {
        if !did.starts_with("did:pkh:") {
            return Err(Error::DidResolution(format!(
                "Invalid did:pkh format: {}",
                did
            )));
        }

        // A real implementation would handle proper did:pkh parsing and key derivation
        // For now, just return a placeholder DID Doc
        Ok(DidDoc {
            id: did.to_string(),
            verification_method: vec![],
            service: None,
        })
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
    async fn resolve(&self, did: &str) -> Result<DidDoc> {
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
