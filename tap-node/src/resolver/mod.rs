//! DID resolution for TAP Node
//!
//! This module provides DID resolution capabilities for the TAP Node.

use std::collections::HashMap;
use std::sync::Arc;

use tap_agent::DidResolver;
use tokio::sync::RwLock;
use serde_json::json;
use sha2::{Sha256, Digest};
use base58::ToBase58;

use crate::error::{Error, Result};

/// Generate a hash of a DID for testing purposes
fn hash_did(did: &str) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(did.as_bytes());
    let result = hasher.finalize();
    Ok(result.to_vec().to_base58())
}

/// Node DID resolver
///
/// This resolver combines multiple DID resolvers for different DID methods
#[derive(Default)]
pub struct NodeResolver {
    /// Resolvers for different DID methods
    resolvers: RwLock<HashMap<String, Arc<dyn DidResolver>>>,
}

impl NodeResolver {
    /// Create a new node resolver
    pub fn new() -> Self {
        let resolvers = RwLock::new(HashMap::new());
        
        // Instantiate and register default resolvers in a real implementation
        // Here's what it might look like:
        // {
        //     use tap_agent::{KeyResolver, MultiResolver, PkhResolver, WebResolver};
        //     
        //     let mut resolvers_map = HashMap::new();
        //     resolvers_map.insert("key".to_string(), Arc::new(KeyResolver::new()) as Arc<dyn DidResolver>);
        //     resolvers_map.insert("web".to_string(), Arc::new(WebResolver::new()) as Arc<dyn DidResolver>);
        //     resolvers_map.insert("pkh".to_string(), Arc::new(PkhResolver::new()) as Arc<dyn DidResolver>);
        //     
        //     resolvers = RwLock::new(resolvers_map);
        // }
        
        Self { resolvers }
    }
    
    /// Add a resolver for a DID method
    pub async fn add_resolver(&self, method: String, resolver: Arc<dyn DidResolver>) {
        let mut resolvers = self.resolvers.write().await;
        resolvers.insert(method, resolver);
    }
    
    /// Get a resolver for a DID method
    pub async fn get_resolver(&self, did: &str) -> Option<Arc<dyn DidResolver>> {
        // Extract the method from the DID
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 || parts[0] != "did" {
            return None;
        }
        
        let method = parts[1].to_string();
        
        // Get the resolver for this method
        let resolvers = self.resolvers.read().await;
        resolvers.get(&method).cloned()
    }
    
    /// Resolve a DID to a DID Document
    pub async fn resolve(&self, did: &str) -> Result<serde_json::Value> {
        // Get the resolver for this DID method
        let resolver = self.get_resolver(did).await
            .ok_or_else(|| Error::Resolver(format!("No resolver found for DID: {}", did)))?;
        
        // Resolve the DID
        let _did_doc = resolver.resolve(did).await
            .map_err(|e| Error::Resolver(format!("Failed to resolve DID {}: {}", did, e)))?;
            
        // Generate hash from DID
        let hash = hash_did(did)?;
        
        // Serialize to JSON
        serde_json::to_value(json!({
            "id": did,
            "publicKey": [
                {
                    "id": format!("{}#keys-1", did),
                    "type": "Ed25519VerificationKey2018",
                    "controller": did,
                    "publicKeyBase58": hash
                }
            ],
            "authentication": [
                format!("{}#keys-1", did)
            ],
            "service": []
        }))
        .map_err(Error::Serialization)
    }
}
