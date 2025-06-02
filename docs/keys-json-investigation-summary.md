# Investigation Summary: TAP Key Creation and Storage in ~/.tap/keys.json

## Executive Summary

The TAP ecosystem implements a robust cryptographic key management system where keys are generated using secure random number generators, stored in a standardized JSON format at `~/.tap/keys.json`, and shared across multiple TAP tools. This investigation revealed the complete lifecycle of key generation, storage, and usage within the TAP framework.

## Key Findings

### 1. Storage Location and Format

**Location**: `~/.tap/keys.json`
- Created automatically by TAP tools
- Shared between `tap-agent-cli`, `tap-http`, and other TAP utilities
- Parent directory `~/.tap/` is created if it doesn't exist

**File Structure**:
```json
{
  "keys": {
    "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK": {
      "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "key_type": "Ed25519",
      "private_key": "base64-encoded-private-key",
      "public_key": "base64-encoded-public-key",
      "metadata": {}
    }
  },
  "default_did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### 2. Key Generation Process

**Step 1: Cryptographic Key Generation**
- Uses `OsRng` (Operating System Random Number Generator) for secure randomness
- Supports three key types:
  - Ed25519 (default) - 32-byte keys
  - P256 (NIST P-256) - Variable length
  - Secp256k1 - Bitcoin/Ethereum compatible

**Step 2: DID Creation**
- Generates `did:key` format DIDs
- Public key is prefixed with multicodec bytes:
  - Ed25519: `0xed01`
  - P256: `0x1200`
  - Secp256k1: `0xe701`
- Encoded using multibase (base58btc with 'z' prefix)

**Step 3: Storage**
- Keys are base64-encoded for storage
- First key added becomes the default automatically
- Timestamps track creation and updates

### 3. Implementation Architecture

**Core Modules**:
- `storage.rs`: Handles file I/O and key persistence
- `did.rs`: Manages DID generation and key creation
- `key_manager.rs`: Integrates keys with the agent's cryptographic operations
- `cli.rs`: Provides user interface for key management

**Key Classes**:
- `KeyStorage`: Main storage container
- `StoredKey`: Individual key representation
- `DIDKeyGenerator`: Key and DID generation logic
- `GeneratedKey`: Temporary key representation before storage

### 4. Usage Patterns

**Generation Methods**:
```bash
# CLI generation
tap-agent-cli generate --save --default
tap-agent-cli generate --key-type ed25519 --save

# Programmatic generation
let generator = DIDKeyGenerator::new();
let key = generator.generate_ed25519_did()?;
```

**Loading Keys**:
```rust
// Automatic loading
let storage = KeyStorage::load_default()?;

// Access default key
if let Some(did) = storage.default_did {
    let key = &storage.keys[&did];
}
```

### 5. Security Considerations

- Private keys are stored in plaintext (base64-encoded)
- File permissions should be set to 600 (owner read/write only)
- Keys should never be committed to version control
- Consider additional encryption for production environments

## Technical Details

### Key Generation Algorithm (Ed25519 Example)

```rust
pub fn generate_ed25519_did(&self) -> Result<GeneratedKey> {
    // 1. Generate keypair
    let mut csprng = OsRng;
    let signing_key = Ed25519SigningKey::generate(&mut csprng);
    let verifying_key = Ed25519VerifyingKey::from(&signing_key);
    
    // 2. Extract keys
    let public_key = verifying_key.to_bytes().to_vec();
    let private_key = signing_key.to_bytes().to_vec();
    
    // 3. Create DID
    let mut prefixed_key = vec![0xed, 0x01];
    prefixed_key.extend_from_slice(&public_key);
    let multibase_encoded = encode(Base::Base58Btc, &prefixed_key);
    let did = format!("did:key:{}", multibase_encoded);
    
    // 4. Return complete key
    Ok(GeneratedKey {
        key_type: KeyType::Ed25519,
        did,
        public_key,
        private_key,
        did_doc: self.create_did_doc(&did, &prefixed_key, KeyType::Ed25519)?,
    })
}
```

### Storage Operations

```rust
// Save operation
pub fn save_default(&self) -> Result<()> {
    let path = Self::default_key_path()?;
    let contents = serde_json::to_string_pretty(self)?;
    fs::write(path, contents)?;
    Ok(())
}

// Load operation
pub fn load_default() -> Result<Self> {
    let path = Self::default_key_path()?;
    if !path.exists() {
        return Ok(Self::new());
    }
    let contents = fs::read_to_string(path)?;
    let storage: KeyStorage = serde_json::from_str(&contents)?;
    Ok(storage)
}
```

## Integration Points

### 1. TAP Agent
- Uses keys for DIDComm message signing and encryption
- Converts stored keys to JWK format for cryptographic operations
- Supports multiple keys with default selection

### 2. TAP HTTP Server
- `--use-stored-key` flag loads keys from storage
- Can specify specific DID with `--agent-did`
- Falls back to ephemeral keys if no storage exists

### 3. CLI Tools
- `tap-agent-cli generate`: Creates new keys
- `tap-agent-cli keys list`: Shows all stored keys
- `tap-agent-cli keys set-default`: Changes default key
- `tap-agent-cli import`: Imports external keys

## Best Practices Identified

1. **Key Lifecycle Management**
   - Generate keys with appropriate metadata
   - Rotate keys periodically
   - Maintain old keys for historical data decryption

2. **Storage Security**
   - Set restrictive file permissions
   - Consider encryption at rest
   - Implement secure backup procedures

3. **Multi-Environment Support**
   - Use different keys for dev/staging/prod
   - Document DID usage per environment
   - Implement key isolation strategies

## Recommendations for Improvement

1. **Enhanced Security**
   - Add optional password protection for keys.json
   - Implement key encryption at rest
   - Add hardware security module (HSM) support

2. **Better Key Management**
   - Add key expiration dates
   - Implement key usage auditing
   - Create key backup/restore utilities

3. **Developer Experience**
   - Add key visualization tools
   - Implement key migration utilities
   - Create comprehensive key management documentation

## Conclusion

The TAP key management system provides a solid foundation for cryptographic operations with a well-structured storage format and comprehensive generation capabilities. The `~/.tap/keys.json` file serves as a central repository for agent identities, enabling seamless integration across the TAP ecosystem while maintaining security through proper key isolation and management practices.