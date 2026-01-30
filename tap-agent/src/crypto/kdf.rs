//! ECDH-ES Key Derivation Function (Concat KDF)
//!
//! Implements the Concat KDF per NIST SP 800-56A and RFC 7518 Section 4.6.
//! This is used to derive key encryption keys (KEK) from ECDH shared secrets
//! for use with AES Key Wrap.

use crate::error::{Error, Result};
use sha2::{Digest, Sha256};

/// Derive a key using Concat KDF (NIST SP 800-56A)
///
/// This implements the single-step key derivation function specified in
/// NIST SP 800-56A Section 5.8.1 using SHA-256 as the hash function.
///
/// # Arguments
/// * `shared_secret` - The raw ECDH shared secret (Z value)
/// * `apu` - Agreement PartyU Info (sender identifier, can be empty)
/// * `apv` - Agreement PartyV Info (recipient identifier, can be empty)
/// * `key_data_len` - Desired output length in bits (must be multiple of 8)
///
/// # Returns
/// The derived key material of length `key_data_len / 8` bytes
///
/// # Algorithm
/// The OtherInfo structure per RFC 7518 Section 4.6.2:
/// - AlgorithmID: length (4 bytes) || "ECDH-ES+A256KW"
/// - PartyUInfo: length (4 bytes) || apu
/// - PartyVInfo: length (4 bytes) || apv
/// - SuppPubInfo: keydatalen in bits (4 bytes, big-endian)
///
/// DerivedKey = Hash(counter || Z || OtherInfo) for each round
pub fn derive_key_ecdh_es(
    shared_secret: &[u8],
    apu: &[u8],
    apv: &[u8],
    key_data_len: usize,
) -> Result<Vec<u8>> {
    if key_data_len == 0 || key_data_len % 8 != 0 {
        return Err(Error::Cryptography(
            "key_data_len must be a positive multiple of 8".to_string(),
        ));
    }

    // Algorithm identifier for ECDH-ES+A256KW
    let algorithm_id = b"ECDH-ES+A256KW";

    // Build OtherInfo per RFC 7518 Section 4.6.2
    let mut other_info = Vec::new();

    // AlgorithmID: length (4 bytes BE) || algorithm
    other_info.extend_from_slice(&(algorithm_id.len() as u32).to_be_bytes());
    other_info.extend_from_slice(algorithm_id);

    // PartyUInfo: length (4 bytes BE) || apu
    other_info.extend_from_slice(&(apu.len() as u32).to_be_bytes());
    other_info.extend_from_slice(apu);

    // PartyVInfo: length (4 bytes BE) || apv
    other_info.extend_from_slice(&(apv.len() as u32).to_be_bytes());
    other_info.extend_from_slice(apv);

    // SuppPubInfo: keydatalen in bits as big-endian u32
    other_info.extend_from_slice(&(key_data_len as u32).to_be_bytes());

    // Concat KDF with SHA-256 (produces 32 bytes per round)
    let key_data_len_bytes = key_data_len / 8;
    let hash_len = 32; // SHA-256 output size
    let reps = (key_data_len_bytes + hash_len - 1) / hash_len;

    let mut derived = Vec::with_capacity(key_data_len_bytes);

    for counter in 1..=reps {
        let mut hasher = Sha256::new();
        // counter as 4-byte big-endian
        hasher.update((counter as u32).to_be_bytes());
        // Z (shared secret)
        hasher.update(shared_secret);
        // OtherInfo
        hasher.update(&other_info);

        derived.extend_from_slice(&hasher.finalize());
    }

    // Truncate to exact requested length
    derived.truncate(key_data_len_bytes);
    Ok(derived)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdf_basic() {
        let secret = [0x42u8; 32];
        let result = derive_key_ecdh_es(&secret, b"", b"", 256);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn test_kdf_with_apu_apv() {
        let secret = [0x42u8; 32];
        let result = derive_key_ecdh_es(&secret, b"sender", b"recipient", 256);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn test_kdf_invalid_length() {
        let secret = [0x42u8; 32];
        // 0 bits is invalid
        assert!(derive_key_ecdh_es(&secret, b"", b"", 0).is_err());
        // Non-multiple of 8 is invalid
        assert!(derive_key_ecdh_es(&secret, b"", b"", 100).is_err());
    }

    #[test]
    fn test_kdf_deterministic() {
        let secret = [0x42u8; 32];
        let k1 = derive_key_ecdh_es(&secret, b"a", b"b", 256).unwrap();
        let k2 = derive_key_ecdh_es(&secret, b"a", b"b", 256).unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_kdf_different_inputs() {
        let secret = [0x42u8; 32];
        let k1 = derive_key_ecdh_es(&secret, b"a", b"b", 256).unwrap();
        let k2 = derive_key_ecdh_es(&secret, b"a", b"c", 256).unwrap();
        assert_ne!(k1, k2);
    }
}
