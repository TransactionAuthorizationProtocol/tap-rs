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
use multibase::{decode, encode, Base};
use curve25519_dalek::edwards::CompressedEdwardsY;

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use crate::error::{Error, Result};

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
    async fn resolve(&self, did: &str) -> Result<Option<DIDDoc>>;
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
        println!("Ed25519 pubkey: {:?}", ed25519_pubkey);
        
        // Try to create a CompressedEdwardsY from the bytes
        let edwards_y = match CompressedEdwardsY::try_from(ed25519_pubkey) {
            Ok(point) => point,
            Err(e) => {
                println!("Error converting to CompressedEdwardsY: {:?}", e);
                return None;
            },
        };
        
        // Try to decompress to get the Edwards point
        let edwards_point = match edwards_y.decompress() {
            Some(point) => point,
            None => {
                println!("Failed to decompress Edwards point");
                return None;
            },
        };
        
        // Convert to Montgomery form
        let montgomery_point = edwards_point.to_montgomery();
        
        // Get the raw bytes representation of the X25519 key
        Some(montgomery_point.to_bytes())
    }
}

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
            println!("Successfully converted Ed25519 to X25519!");
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
            println!("Failed to convert Ed25519 to X25519!");
        }
        
        // Create the DID document
        let did_doc = DIDDoc {
            id: did_key.to_string(),
            verification_method: verification_methods,
            authentication: vec![ed_vm_id],
            key_agreement,
            service: Vec::new(),
        };
        
        Ok(Some(did_doc))
    }
}

/// A multi-resolver for DID methods. This resolver manages multiple 
/// method-specific resolver. New resolvers can be added at runtime.
#[derive(Debug)]
pub struct MultiResolver {
    resolvers: RwLock<HashMap<String, Arc<dyn DIDMethodResolver>>>,
}

unsafe impl Send for MultiResolver {}
unsafe impl Sync for MultiResolver {}

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

impl Default for MultiResolver {
    fn default() -> Self {
        let mut resolver = Self::new();
        resolver.register_method("key", KeyResolver::new());
        resolver
    }
}

#[async_trait]
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
            let resolver_guard = self.resolvers.read().map_err(|_| Error::FailedToAcquireResolverReadLock)?;
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

#[async_trait(?Send)]
impl DIDResolver for MultiResolver {
    async fn resolve(&self, did: &str) -> DidcommResult<Option<DIDDoc>> {
        match SyncDIDResolver::resolve(self, did).await {
            Ok(did_doc) => Ok(did_doc),
            Err(e) => Err(DidcommError::new(DidcommErrorKind::InvalidState, e))
        }
    }
}

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

/// A resolver for a specific DID method without Send+Sync requirements (for WASM usage)
#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
pub trait WasmDIDMethodResolver: Debug {
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
#[async_trait(?Send)]
impl WasmDIDMethodResolver for JsDIDMethodResolver {
    fn method(&self) -> &str {
        &self.method
    }
    
    #[cfg(feature = "wasm")]
    async fn resolve_method(&self, did: &str) -> Result<Option<DIDDoc>> {
        // Ensure the DID is for the method that this resolver is for
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 || parts[1] != self.method {
            return Err(Error::InvalidDID);
        }
        
        let this = JsValue::null();
        let js_did = JsValue::from_str(did);
        
        let promise = self.resolve_fn.call1(&this, &js_did)
            .map_err(|e| Error::JsResolverError(format!("Error calling JS resolver: {:?}", e)))?;
        
        let promise = Promise::from(promise);
        let doc_json = JsFuture::from(promise)
            .await
            .map_err(|e| Error::JsResolverError(format!("Error from JS promise: {:?}", e)))?;
        
        if doc_json.is_null() || doc_json.is_undefined() {
            return Ok(None);
        }
        
        let doc_str = doc_json
            .as_string()
            .ok_or_else(|| Error::JsResolverError("JS resolver did not return a string".to_string()))?;
            
        // Parse the JSON string into a DIDDoc
        serde_json::from_str(&doc_str)
            .map(Some)
            .map_err(|e| Error::SerdeError(e))
    }
    
    #[cfg(not(feature = "wasm"))]
    async fn resolve_method(&self, _did: &str) -> Result<Option<DIDDoc>> {
        Err(Error::NotImplemented("JavaScript DID Method resolver is only available with the 'wasm' feature".to_string()))
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
        let ed25519_method = doc.verification_method.iter()
            .find(|vm| matches!(vm.type_, VerificationMethodType::Ed25519VerificationKey2018))
            .expect("Should have an Ed25519 verification method");
        
        // Verify X25519 verification method
        let x25519_method = doc.verification_method.iter()
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
}