//! AES Key Wrap per RFC 3394
//!
//! Implements the AES Key Wrap algorithm for securely wrapping
//! content encryption keys (CEK) using a key encryption key (KEK).
//!
//! This replaces the insecure XOR-based key wrapping that was
//! previously used in the codebase.

use crate::error::{Error, Result};
use aes::Aes256;
use aes_kw::Kek;

/// Wrap a key using AES-256-KW (RFC 3394)
///
/// # Arguments
/// * `kek` - The 256-bit Key Encryption Key
/// * `plaintext_key` - The key to wrap (must be multiple of 8 bytes, minimum 16 bytes)
///
/// # Returns
/// The wrapped key (input length + 8 bytes for integrity check value)
///
/// # Security
/// AES-KW provides both confidentiality and integrity protection for the wrapped key.
/// It uses the AES cipher in a chained mode with an integrity check value (ICV)
/// that detects any tampering with the wrapped key.
pub fn wrap_key_aes_kw(kek: &[u8; 32], plaintext_key: &[u8]) -> Result<Vec<u8>> {
    if plaintext_key.len() < 16 {
        return Err(Error::Cryptography(
            "Key to wrap must be at least 16 bytes".to_string(),
        ));
    }
    if plaintext_key.len() % 8 != 0 {
        return Err(Error::Cryptography(
            "Key to wrap must be multiple of 8 bytes".to_string(),
        ));
    }

    let kek = Kek::<Aes256>::from(*kek);

    let mut output = vec![0u8; plaintext_key.len() + 8];
    kek.wrap(plaintext_key, &mut output)
        .map_err(|e| Error::Cryptography(format!("Key wrap failed: {:?}", e)))?;

    Ok(output)
}

/// Unwrap a key using AES-256-KW (RFC 3394)
///
/// # Arguments
/// * `kek` - The 256-bit Key Encryption Key
/// * `wrapped_key` - The wrapped key (must be input length + 8 bytes)
///
/// # Returns
/// The unwrapped plaintext key
///
/// # Security
/// The unwrap operation verifies the integrity check value (ICV) and will
/// return an error if:
/// - The KEK is incorrect
/// - The wrapped key has been tampered with
/// - The wrapped key is malformed
pub fn unwrap_key_aes_kw(kek: &[u8; 32], wrapped_key: &[u8]) -> Result<Vec<u8>> {
    if wrapped_key.len() < 24 {
        return Err(Error::Cryptography(
            "Wrapped key must be at least 24 bytes".to_string(),
        ));
    }
    if wrapped_key.len() % 8 != 0 {
        return Err(Error::Cryptography(
            "Wrapped key must be multiple of 8 bytes".to_string(),
        ));
    }

    let kek = Kek::<Aes256>::from(*kek);

    let mut output = vec![0u8; wrapped_key.len() - 8];
    kek.unwrap(wrapped_key, &mut output)
        .map_err(|e| Error::Cryptography(format!("Key unwrap failed: {:?}", e)))?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_unwrap_roundtrip() {
        let kek = [0x42u8; 32];
        let plaintext = [0xABu8; 32];

        let wrapped = wrap_key_aes_kw(&kek, &plaintext).unwrap();
        let unwrapped = unwrap_key_aes_kw(&kek, &wrapped).unwrap();

        assert_eq!(&unwrapped[..], &plaintext[..]);
    }

    #[test]
    fn test_wrap_produces_longer_output() {
        let kek = [0x42u8; 32];
        let plaintext = [0xABu8; 32];

        let wrapped = wrap_key_aes_kw(&kek, &plaintext).unwrap();
        assert_eq!(wrapped.len(), plaintext.len() + 8);
    }

    #[test]
    fn test_wrong_kek_fails() {
        let kek1 = [0x42u8; 32];
        let kek2 = [0x43u8; 32];
        let plaintext = [0xABu8; 32];

        let wrapped = wrap_key_aes_kw(&kek1, &plaintext).unwrap();
        assert!(unwrap_key_aes_kw(&kek2, &wrapped).is_err());
    }

    #[test]
    fn test_tampering_detected() {
        let kek = [0x42u8; 32];
        let plaintext = [0xABu8; 32];

        let mut wrapped = wrap_key_aes_kw(&kek, &plaintext).unwrap();
        wrapped[0] ^= 0xFF;

        assert!(unwrap_key_aes_kw(&kek, &wrapped).is_err());
    }

    #[test]
    fn test_short_key_rejected() {
        let kek = [0x42u8; 32];
        let plaintext = [0xABu8; 8]; // Too short

        assert!(wrap_key_aes_kw(&kek, &plaintext).is_err());
    }

    #[test]
    fn test_non_aligned_key_rejected() {
        let kek = [0x42u8; 32];
        let plaintext = [0xABu8; 17]; // Not multiple of 8

        assert!(wrap_key_aes_kw(&kek, &plaintext).is_err());
    }
}
