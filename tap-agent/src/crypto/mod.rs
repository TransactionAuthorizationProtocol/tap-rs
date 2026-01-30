//! Cryptographic primitives for TAP Agent
//!
//! This module provides secure implementations of:
//! - ECDH-ES key derivation (Concat KDF per NIST SP 800-56A)
//! - AES Key Wrap per RFC 3394
//!
//! These primitives are used for JWE encryption and decryption in the
//! DIDComm messaging layer.

mod kdf;
mod key_wrap;

pub use kdf::derive_key_ecdh_es;
pub use key_wrap::{unwrap_key_aes_kw, wrap_key_aes_kw};
