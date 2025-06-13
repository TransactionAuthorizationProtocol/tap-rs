//! Name hashing utilities for TAIP-12 compliance
//!
//! This module provides functionality for hashing participant names according to TAIP-12,
//! which enables privacy-preserving Travel Rule compliance by sharing hashed names instead
//! of plaintext names.

use sha2::{Digest, Sha256};

/// Trait for types that can generate a hashed name according to TAIP-12
pub trait NameHashable {
    /// Generate a SHA-256 hash of the name according to TAIP-12 normalization rules
    ///
    /// The normalization process:
    /// 1. Remove all whitespace characters
    /// 2. Convert to uppercase
    /// 3. Encode as UTF-8
    /// 4. Hash with SHA-256
    /// 5. Return as lowercase hex string
    ///
    /// # Arguments
    ///
    /// * `name` - The name to hash (can be a person's full name or organization name)
    ///
    /// # Returns
    ///
    /// A 64-character lowercase hex string representing the SHA-256 hash
    ///
    /// # Example
    ///
    /// ```
    /// use tap_msg::utils::name_hash::NameHashable;
    ///
    /// struct Person;
    /// impl NameHashable for Person {}
    ///
    /// let hash = Person::hash_name("Alice Lee");
    /// assert_eq!(hash, "b117f44426c9670da91b563db728cd0bc8bafa7d1a6bb5e764d1aad2ca25032e");
    /// ```
    fn hash_name(name: &str) -> String {
        // Normalize: remove whitespace and convert to uppercase
        let normalized = name
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_uppercase();

        // Hash with SHA-256
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        let result = hasher.finalize();

        // Convert to lowercase hex string
        hex::encode(result)
    }
}

/// Generate a TAIP-12 compliant name hash
///
/// This is a standalone function that implements the same algorithm as the trait method.
///
/// # Arguments
///
/// * `name` - The name to hash
///
/// # Returns
///
/// A 64-character lowercase hex string representing the SHA-256 hash
pub fn hash_name(name: &str) -> String {
    struct Hasher;
    impl NameHashable for Hasher {}
    Hasher::hash_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_name_basic() {
        // Test case from TAIP-12 specification
        let hash = hash_name("Alice Lee");
        assert_eq!(
            hash,
            "b117f44426c9670da91b563db728cd0bc8bafa7d1a6bb5e764d1aad2ca25032e"
        );
    }

    #[test]
    fn test_hash_name_bob_smith() {
        // Test case from TAIP-12 specification
        let hash = hash_name("Bob Smith");
        assert_eq!(
            hash,
            "5432e86b4d4a3a2b4be57b713b12c5c576c88459fe1cfdd760fd6c99a0e06686"
        );
    }

    #[test]
    fn test_hash_name_normalization() {
        // All these should produce the same hash
        let expected = hash_name("ALICELEE");
        assert_eq!(hash_name("Alice Lee"), expected);
        assert_eq!(hash_name("alice lee"), expected);
        assert_eq!(hash_name("ALICE LEE"), expected);
        assert_eq!(hash_name("Alice  Lee"), expected);
        assert_eq!(hash_name(" Alice Lee "), expected);
        assert_eq!(hash_name("Alice\tLee"), expected);
        assert_eq!(hash_name("Alice\nLee"), expected);
    }

    #[test]
    fn test_hash_name_with_middle_name() {
        let hash = hash_name("Alice Marie Lee");
        // Should normalize to "ALICEMARIELEE"
        assert_eq!(hash.len(), 64); // SHA-256 produces 32 bytes = 64 hex chars
    }

    #[test]
    fn test_hash_name_organization() {
        let hash = hash_name("Example VASP Ltd.");
        // Should normalize to "EXAMPLEVASPLTD"
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_name_special_characters() {
        // Note: TAIP-12 only removes whitespace, not punctuation
        let hash1 = hash_name("O'Brien");
        let hash2 = hash_name("OBrien");
        assert_ne!(hash1, hash2); // These should be different
    }

    #[test]
    fn test_hash_name_unicode() {
        let hash = hash_name("José García");
        // Should normalize to "JOSÉBGARCÍA" (preserving accented characters)
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_trait_implementation() {
        struct TestHasher;
        impl NameHashable for TestHasher {}

        let hash = TestHasher::hash_name("Alice Lee");
        assert_eq!(
            hash,
            "b117f44426c9670da91b563db728cd0bc8bafa7d1a6bb5e764d1aad2ca25032e"
        );
    }
}
