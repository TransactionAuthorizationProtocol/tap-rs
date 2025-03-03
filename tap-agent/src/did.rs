//! DID resolution functionality for the TAP Agent.
//!
//! This module provides a multi-resolver for Decentralized Identifiers (DIDs)
//! that integrates with the didcomm library's DID resolution system. The multi-resolver
//! currently supports the did:key method, with the architecture allowing for additional
//! methods to be added in the future.

use async_trait::async_trait;
use didcomm::did::{
    DIDDoc, DIDResolver, VerificationMethod, VerificationMethodType, VerificationMaterial
};
use didcomm::error::{Error as DidcommError, Result as DidcommResult, ErrorKind as DidcommErrorKind};
use multibase::decode;

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use crate::error::{Error, Result};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// A trait for resolving DIDs to DID documents that is Send+Sync.
///
/// This is a wrapper around didcomm's DIDResolver that adds the
/// Send+Sync bounds required for the TAP Agent.
#[async_trait]
pub trait SyncDIDResolver: Send + Sync + Debug {
    /// Resolve a DID to a DID document.
    ///
    /// # Parameters
    /// * `did` - The DID to resolve
    ///
    /// # Returns
    /// The DID document as an Option
    async fn resolve(&self, did: &str) -> DidcommResult<Option<DIDDoc>>;
}

/// A resolver for a specific DID method.
#[async_trait]
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

/// A resolver for the did:key method.
#[derive(Debug)]
pub struct KeyResolver;

#[async_trait]
impl DIDMethodResolver for KeyResolver {
    fn method(&self) -> &str {
        "key"
    }
    
    async fn resolve_method(&self, did: &str) -> Result<Option<DIDDoc>> {
        // Validate the DID format
        if !did.starts_with("did:key:") {
            return Err(Error::InvalidDID(format!("Not a did:key format: {}", did)));
        }
        
        // Extract the multibase encoded public key
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 {
            return Err(Error::InvalidDID(format!("Invalid did:key format: {}", did)));
        }
        
        let multibase_key = parts[2];
        
        // Decode the multibase key
        let (_encoding, key_bytes) = decode(multibase_key)
            .map_err(|e| Error::DIDResolution(format!("Failed to decode multibase key: {}", e)))?;
        
        // Skip the first byte as it indicates the key type
        if key_bytes.len() < 2 {
            return Err(Error::DIDResolution("Invalid key bytes length".to_string()));
        }
        
        // Determine key type from the first byte
        let key_type = match key_bytes[0] {
            0xed => VerificationMethodType::Ed25519VerificationKey2018,
            0xec => VerificationMethodType::X25519KeyAgreementKey2019,
            0xe7 => VerificationMethodType::EcdsaSecp256k1VerificationKey2019,
            0xe1 => VerificationMethodType::JsonWebKey2020,
            _ => return Err(Error::DIDResolution(format!("Unsupported key type: {:02x}", key_bytes[0]))),
        };
        
        // Create the verification material
        let verification_material = VerificationMaterial::Multibase { 
            public_key_multibase: multibase_key.to_string() 
        };
        
        // Create verification method
        let vm_id = format!("{}#{}", did, multibase_key);
        let verification_method = VerificationMethod {
            id: vm_id.clone(),
            type_: key_type,
            controller: did.to_string(),
            verification_material,
        };
        
        // Create DID document
        let doc = DIDDoc {
            id: did.to_string(),
            verification_method: vec![verification_method],
            authentication: vec![vm_id.clone()],
            key_agreement: vec![vm_id], // Using same key for auth and key agreement
            service: vec![],
        };
        
        Ok(Some(doc))
    }
}

/// A multi-resolver that aggregates multiple DID method resolvers.
///
/// This resolver can handle multiple DID methods by delegating to the appropriate
/// method-specific resolver. New resolvers can be added at runtime.
#[derive(Debug)]
pub struct MultiResolver {
    resolvers: RwLock<HashMap<String, Arc<dyn DIDMethodResolver>>>,
}

unsafe impl Send for MultiResolver {}
unsafe impl Sync for MultiResolver {}

impl Default for MultiResolver {
    fn default() -> Self {
        let mut resolver = Self::new();
        
        // Add default resolver for did:key
        resolver.add_resolver(Arc::new(KeyResolver));
        
        resolver
    }
}

impl MultiResolver {
    /// Creates a new empty multi-resolver.
    pub fn new() -> Self {
        Self {
            resolvers: RwLock::new(HashMap::new()),
        }
    }
    
    /// Creates a new multi-resolver with a list of DID method resolvers.
    pub fn new_with_resolvers(method_resolvers: Vec<Arc<dyn DIDMethodResolver>>) -> Self {
        let mut resolver = Self::new();
        for method_resolver in method_resolvers {
            resolver.add_resolver(method_resolver);
        }
        resolver
    }
    
    /// Adds a DID method resolver to this multi-resolver.
    pub fn add_resolver(&mut self, resolver: Arc<dyn DIDMethodResolver>) {
        if let Ok(mut resolvers) = self.resolvers.write() {
            resolvers.insert(resolver.method().to_string(), resolver);
        }
    }
    
    /// Gets all supported DID methods.
    pub fn supported_methods(&self) -> Vec<String> {
        if let Ok(resolvers) = self.resolvers.read() {
            resolvers.keys().cloned().collect()
        } else {
            vec![]
        }
    }
    
    /// Checks if a DID method is supported.
    pub fn supports_method(&self, method: &str) -> bool {
        if let Ok(resolvers) = self.resolvers.read() {
            resolvers.contains_key(method)
        } else {
            false
        }
    }
    
    /// Resolves a DID to a DID document internally.
    async fn resolve_internal(&self, did: &str) -> Result<Option<DIDDoc>> {
        // Parse the DID to get the method
        if !did.starts_with("did:") {
            return Err(Error::InvalidDID(format!("Invalid DID: {}", did)));
        }
        
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 {
            return Err(Error::InvalidDID(format!("Invalid DID format: {}", did)));
        }
        
        let method = parts[1];
        
        // Get the appropriate resolver for this method
        let resolver = if let Ok(resolvers) = self.resolvers.read() {
            resolvers.get(method).cloned()
        } else {
            None
        };
        
        // Use the method-specific resolver to resolve the DID
        match resolver {
            Some(resolver) => resolver.resolve_method(did).await,
            None => Err(Error::InvalidDID(format!("Unsupported DID method: {}", method))),
        }
    }
}

#[async_trait]
impl SyncDIDResolver for MultiResolver {
    async fn resolve(&self, did: &str) -> DidcommResult<Option<DIDDoc>> {
        self.resolve_internal(did)
            .await
            .map_err(|e| DidcommError::new(DidcommErrorKind::InvalidState, e))
    }
}

#[async_trait(?Send)]
impl DIDResolver for MultiResolver {
    async fn resolve(&self, did: &str) -> DidcommResult<Option<DIDDoc>> {
        self.resolve_internal(did)
            .await
            .map_err(|e| DidcommError::new(DidcommErrorKind::InvalidState, e))
    }
}

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
    resolve_fn: js_sys::Function,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl JsDIDResolver {
    #[wasm_bindgen(constructor)]
    pub fn new(resolve_fn: js_sys::Function) -> Self {
        Self { resolve_fn }
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

/// A wrapper for JavaScript DID resolvers.
#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct JsDIDMethodResolver {
    js_resolver: JsDIDResolver,
    method_name: String,
}

#[cfg(target_arch = "wasm32")]
impl JsDIDMethodResolver {
    pub fn new(js_resolver: JsDIDResolver) -> Self {
        let method_name = js_resolver.method();
        Self {
            js_resolver,
            method_name,
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait]
impl DIDMethodResolver for JsDIDMethodResolver {
    fn method(&self) -> &str {
        &self.method_name
    }
    
    async fn resolve_method(&self, did: &str) -> Result<Option<DIDDoc>> {
        let this = JsValue::null();
        let js_did = JsValue::from_str(did);
        
        let promise = self.js_resolver.resolve_fn.call1(&this, &js_did)
            .map_err(|e| Error::JsError(format!("Error calling JS resolver: {:?}", e)))?;
        
        let promise = js_sys::Promise::from(promise);
        let doc_json = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|e| Error::JsError(format!("Error from JS promise: {:?}", e)))?;
        
        if doc_json.is_null() || doc_json.is_undefined() {
            return Ok(None);
        }
        
        let doc_str = doc_json
            .as_string()
            .ok_or_else(|| Error::JsError("JS resolver did not return a string".to_string()))?;
        
        let doc: DIDDoc = serde_json::from_str(&doc_str)
            .map_err(|e| Error::Serialization(format!("Invalid DID document: {}", e)))?;
        
        Ok(Some(doc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_key_resolver() {
        let resolver = KeyResolver;
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        
        let result = resolver.resolve_method(did).await;
        assert!(result.is_ok());
        
        let doc = result.unwrap();
        assert!(doc.is_some());
        
        let doc = doc.unwrap();
        assert_eq!(doc.id, did);
        assert!(!doc.verification_method.is_empty());
        assert!(!doc.authentication.is_empty());
        assert!(!doc.key_agreement.is_empty());
    }
    
    #[tokio::test]
    async fn test_multi_resolver() {
        let multi_resolver = MultiResolver::default();
        
        // Test with did:key
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let result = <MultiResolver as SyncDIDResolver>::resolve(&multi_resolver, did).await;
        assert!(result.is_ok());
        
        // Test with unsupported method
        let did = "did:unsupported:123";
        let result = <MultiResolver as SyncDIDResolver>::resolve(&multi_resolver, did).await;
        assert!(result.is_err());
    }
}