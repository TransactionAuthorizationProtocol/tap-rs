//! DID resolution for TAP Node
//!
//! This module provides DID resolution capabilities for the TAP Node.

use std::collections::HashMap;
use std::sync::Arc;

use base58::ToBase58;
use serde_json::json;
use sha2::{Digest, Sha256};
use tap_agent::did::SyncDIDResolver;
use tokio::sync::RwLock;

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
/// This resolver combines multiple DID resolvers for different DID methods,
/// providing a unified interface for resolving DIDs across various methods.
///
/// The resolver supports:
/// - did:key method, which is used for keys represented as DIDs
/// - Multi-resolver, which combines multiple method-specific resolvers
///
/// # Resolution Process
///
/// When a DID is received for resolution:
///
/// 1. The method is extracted from the DID (e.g., "key" from "did:key:z6Mk...")
/// 2. The appropriate resolver for that method is selected
/// 3. The resolver processes the DID and returns a DID Document
/// 4. The DID Document provides cryptographic material and service endpoints
///
/// # Thread Safety
///
/// The NodeResolver is thread-safe and can be safely shared across threads
/// using `Arc<NodeResolver>`. All mutable state is protected by RwLock.
#[derive(Default)]
pub struct NodeResolver {
    /// Resolvers for different DID methods
    resolvers: RwLock<HashMap<String, Arc<dyn SyncDIDResolver>>>,
}

impl NodeResolver {
    /// Create a new node resolver with default resolvers
    pub fn new() -> Self {
        // Create a HashMap to store our resolvers
        let mut resolvers_map = HashMap::new();

        // Create a MultiResolver that can handle multiple methods
        let multi_resolver = tap_agent::did::MultiResolver::default();
        resolvers_map.insert(
            "multi".to_string(),
            Arc::new(multi_resolver) as Arc<dyn SyncDIDResolver>,
        );

        // Initialize the resolvers RwLock with our map
        let resolvers = RwLock::new(resolvers_map);

        Self { resolvers }
    }

    /// Add a resolver for a DID method
    pub async fn add_resolver(&self, method: String, resolver: Arc<dyn SyncDIDResolver>) {
        let mut resolvers = self.resolvers.write().await;
        resolvers.insert(method, resolver);
    }

    /// Get a resolver for a DID method
    pub async fn get_resolver(&self, did: &str) -> Option<Arc<dyn SyncDIDResolver>> {
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
    ///
    /// This method takes a DID and returns the corresponding DID Document.
    /// The DID Document contains the cryptographic material and service endpoints
    /// associated with the DID.
    ///
    /// # Parameters
    ///
    /// * `did` - The DID to resolve
    ///
    /// # Returns
    ///
    /// The DID Document as a JSON Value
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No resolver is found for the DID method
    /// - The DID resolution fails
    /// - The DID Document cannot be serialized to JSON
    pub async fn resolve(&self, did: &str) -> Result<serde_json::Value> {
        // First try to use a method-specific resolver
        let method_parts: Vec<&str> = did.split(':').collect();
        if method_parts.len() >= 3 && method_parts[0] == "did" {
            let _method = method_parts[1];

            // Get a resolver for this method
            let resolver = self
                .get_resolver(did)
                .await
                .ok_or_else(|| Error::Resolver(format!("No resolver found for DID: {}", did)))?;

            // Resolve the DID
            let did_doc_option = resolver
                .resolve(did)
                .await
                .map_err(|e| Error::Resolver(format!("Failed to resolve DID {}: {}", did, e)))?;

            // Check if we got a DID Document
            if let Some(did_doc) = did_doc_option {
                // Serialize the DID Document to JSON
                return serde_json::to_value(did_doc).map_err(Error::Serialization);
            }
        }

        // If we couldn't resolve with a method-specific resolver or the DID format was invalid,
        // fall back to a simple hash-based approach for testing/development
        let hash = hash_did(did)?;

        // Create a simple DID Document
        serde_json::to_value(json!({
            "id": did,
            "verificationMethod": [
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
