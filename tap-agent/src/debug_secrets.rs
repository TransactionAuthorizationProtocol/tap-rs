//! Debug utilities for working with crypto secrets.
//!
//! This module provides traits and utilities for handling cryptographic
//! secrets in debug contexts, primarily useful for testing and debugging.

use crate::key_manager::Secret;
use std::fmt::Debug;

/// A trait to extend types with an as_any method for downcasting.
pub trait AsAny: 'static {
    /// Return a reference to self as Any
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A trait for resolving secrets for cryptographic operations.
///
/// This trait provides access to cryptographic secrets needed by the TAP Agent
/// for signing, encryption, and other security operations.
pub trait DebugSecretsResolver: Debug + Send + Sync + AsAny {
    /// Get a reference to the secrets map for debugging purposes
    fn get_secrets_map(&self) -> &std::collections::HashMap<String, Secret>;

    /// Get a secret by ID
    fn get_secret_by_id(&self, id: &str) -> Option<Secret>;
}

/// A basic implementation of DebugSecretsResolver.
///
/// This implementation provides a simple in-memory store for cryptographic secrets
/// used by the TAP Agent for DIDComm operations.
#[derive(Debug, Default, Clone)]
pub struct BasicSecretResolver {
    /// Maps DIDs to their associated secrets
    secrets: std::collections::HashMap<String, Secret>,
}

impl BasicSecretResolver {
    /// Create a new empty BasicSecretResolver
    pub fn new() -> Self {
        Self {
            secrets: std::collections::HashMap::new(),
        }
    }

    /// Add a secret for a DID
    ///
    /// # Parameters
    /// * `did` - The DID to associate with the secret
    /// * `secret` - The secret to add
    pub fn add_secret(&mut self, did: &str, secret: Secret) {
        self.secrets.insert(did.to_string(), secret);
    }
}

impl DebugSecretsResolver for BasicSecretResolver {
    fn get_secrets_map(&self) -> &std::collections::HashMap<String, Secret> {
        &self.secrets
    }

    fn get_secret_by_id(&self, id: &str) -> Option<Secret> {
        self.secrets.get(id).cloned()
    }
}