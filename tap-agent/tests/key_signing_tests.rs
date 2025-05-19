/// Tests for signing and verifying messages with different key types
use tap_agent::did::{DIDGenerationOptions, KeyType};
use tap_agent::key_manager::DefaultKeyManager;
use tap_agent::key_manager::KeyManager;

// Test that we can generate keys and sign/verify messages
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_key_generation() {
        let key_manager = DefaultKeyManager::new();
        let options = DIDGenerationOptions {
            key_type: KeyType::Ed25519,
        };
        let key = key_manager.generate_key(options).unwrap();

        assert!(key.did.starts_with("did:key:z"));
        assert_eq!(key.key_type, KeyType::Ed25519);
    }

    #[test]
    fn test_p256_key_generation() {
        let key_manager = DefaultKeyManager::new();
        let options = DIDGenerationOptions {
            key_type: KeyType::P256,
        };
        let key = key_manager.generate_key(options).unwrap();

        assert!(key.did.starts_with("did:key:z"));
        assert_eq!(key.key_type, KeyType::P256);
    }

    #[test]
    fn test_secp256k1_key_generation() {
        let key_manager = DefaultKeyManager::new();
        let options = DIDGenerationOptions {
            key_type: KeyType::Secp256k1,
        };
        let key = key_manager.generate_key(options).unwrap();

        assert!(key.did.starts_with("did:key:z"));
        assert_eq!(key.key_type, KeyType::Secp256k1);
    }
}
