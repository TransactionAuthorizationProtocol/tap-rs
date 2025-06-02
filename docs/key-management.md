# TAP Key Management Documentation

## Overview

The TAP ecosystem uses cryptographic keys for agent identity and secure communication. Keys are stored in a standardized format at `~/.tap/keys.json`, which is shared across TAP tools like `tap-agent-cli` and `tap-http`.

## Key Storage Location

Keys are stored in:
- **Default location**: `~/.tap/keys.json`
- **Directory**: `~/.tap/` (created automatically if it doesn't exist)
- **File format**: JSON

## Key Generation Process

### 1. Cryptographic Key Generation

TAP supports three key types:
- **Ed25519** (default) - Fast, secure, and widely supported
- **P256** - NIST P-256 elliptic curve
- **Secp256k1** - Bitcoin/Ethereum compatible curve

Keys are generated using cryptographically secure random number generators:

```rust
// Ed25519 example
let mut csprng = OsRng;
let signing_key = Ed25519SigningKey::generate(&mut csprng);
let verifying_key = Ed25519VerifyingKey::from(&signing_key);
```

### 2. DID Creation

Each key pair is associated with a Decentralized Identifier (DID):
- DIDs use the `did:key` method by default
- Format: `did:key:z6Mk...` (multibase-encoded public key)
- The DID is derived deterministically from the public key

### 3. Storage Format

The `~/.tap/keys.json` file structure:

```json
{
  "keys": {
    "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK": {
      "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "label": "agent-1",
      "key_type": "Ed25519",
      "private_key": "base64-encoded-private-key",
      "public_key": "base64-encoded-public-key",
      "metadata": {}
    }
  },
  "labels": {
    "agent-1": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
  },
  "default_did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

## Key Generation Methods

### Using tap-agent-cli

Generate a new key and save it to `~/.tap/keys.json`:
```bash
# Generate and save as default (with auto-generated label agent-1, agent-2, etc.)
tap-agent-cli generate --save --default

# Generate with custom label
tap-agent-cli generate --save --label "production-key" --default
tap-agent-cli generate --save --label "signing-key"

# Generate specific key type with label
tap-agent-cli generate --key-type ed25519 --save --label "my-ed25519-key"

# Generate and save to custom files
tap-agent-cli generate --output did.json --key-output key.json
```

### Programmatic Key Generation

```rust
use tap_agent::{
    did::{DIDKeyGenerator, DIDGenerationOptions, KeyType},
    storage::{KeyStorage, StoredKey},
};

// Generate a new key
let generator = DIDKeyGenerator::new();
let options = DIDGenerationOptions {
    key_type: KeyType::Ed25519,
};
let generated_key = generator.generate_did(options)?;

// Convert to storage format
let stored_key = StoredKey {
    did: generated_key.did.clone(),
    label: "my-custom-label".to_string(), // Or leave empty for auto-generated
    key_type: generated_key.key_type,
    private_key: base64::encode(&generated_key.private_key),
    public_key: base64::encode(&generated_key.public_key),
    metadata: HashMap::new(),
};

// Save to storage
let mut storage = KeyStorage::load_default()
    .unwrap_or_else(|_| KeyStorage::new());
storage.add_key(stored_key);
storage.save_default()?;
```

## Key Loading Process

### Automatic Loading

TAP tools automatically load keys from `~/.tap/keys.json`:

```rust
// Load storage
let storage = KeyStorage::load_default()?;

// Get default key
if let Some(default_did) = storage.default_did {
    let key = &storage.keys[&default_did];
    // Use key...
}

// Get key by label
if let Some(key) = storage.find_by_label("production-key") {
    // Use key...
}
```

### Using Stored Keys

With `tap-http`:
```bash
# Use default key from storage
tap-http --use-stored-key

# Use specific key from storage (by DID or label)
tap-http --use-stored-key --agent-did "did:key:z6Mk..."
tap-http --use-stored-key --agent-did "production-key"
```

With `tap-agent-cli`:
```bash
# List all stored keys (shows labels, DIDs, and key types)
tap-agent-cli keys list

# Show specific key details (by DID or label)
tap-agent-cli keys view "did:key:z6Mk..."
tap-agent-cli keys view "production-key"
tap-agent-cli keys view "agent-1"

# Set a different default key (by DID or label)
tap-agent-cli keys set-default "did:key:z6Mk..."
tap-agent-cli keys set-default "production-key"

# Relabel an existing key
tap-agent-cli keys relabel "agent-1" "development-key"
tap-agent-cli keys relabel "old-label" "new-label"

# Delete a key (by DID or label)
tap-agent-cli keys delete "test-key"
tap-agent-cli keys delete "did:key:z6Mk..." --force
```

## Key Management Best Practices

### 1. Security
- The `~/.tap/keys.json` file contains private keys and should be protected
- Set appropriate file permissions: `chmod 600 ~/.tap/keys.json`
- Never commit `keys.json` to version control
- Consider encrypting the file at rest

### 2. Backup
- Regularly backup your `~/.tap/keys.json` file
- Store backups securely (encrypted)
- Test restore procedures

### 3. Key Rotation
- Generate new keys periodically
- Update services to use new DIDs
- Keep old keys for decrypting historical data

### 4. Multi-Environment Setup
- Use different keys for development, staging, and production
- Consider using environment-specific key files
- Document which DIDs are used in each environment

## Internal Implementation Details

### Key Storage Module (`storage.rs`)

The `KeyStorage` struct manages the keys:
- Handles loading from and saving to disk
- Manages default key selection
- Tracks creation and update timestamps
- Converts between storage format and runtime format
- Maintains label-to-DID mapping for quick lookup
- Ensures label uniqueness (auto-generates agent-1, agent-2, etc.)

### Key Generation Module (`did.rs`)

The `DIDKeyGenerator` handles:
- Cryptographic key generation for all supported curves
- DID creation from public keys
- DID Document generation
- Multicodec encoding for did:key method

### Key Manager Integration

Keys are integrated with the TAP Agent's key manager:
- Converts stored keys to JWK format for DIDComm
- Handles key resolution for message encryption/decryption
- Supports multiple key types and algorithms

## Troubleshooting

### Missing Keys File
If `~/.tap/keys.json` doesn't exist:
1. Generate a new key: `tap-agent-cli generate --save --default`
2. Or import existing key: `tap-agent-cli import <key-file>`

### Permission Errors
If you get permission errors:
```bash
# Fix permissions
chmod 600 ~/.tap/keys.json
# Fix ownership
chown $USER:$USER ~/.tap/keys.json
```

### Corrupted Keys File
If the keys file is corrupted:
1. Backup the corrupted file
2. Try to manually fix JSON syntax
3. Or restore from backup
4. As last resort, generate new keys

## API Reference

### KeyStorage Methods
- `load_default()` - Load from `~/.tap/keys.json`
- `load_from_path(path)` - Load from custom path
- `save_default()` - Save to `~/.tap/keys.json`
- `save_to_path(path)` - Save to custom path
- `add_key(key)` - Add a new key to storage (auto-assigns label if needed)
- `find_by_label(label)` - Find a key by its label
- `update_label(did, new_label)` - Change a key's label
- `from_generated_key(key)` - Convert generated key to stored format
- `from_generated_key_with_label(key, label)` - Convert with specific label

### DIDKeyGenerator Methods
- `generate_did(options)` - Generate DID with specified key type
- `generate_ed25519_did()` - Generate Ed25519 DID
- `generate_p256_did()` - Generate P256 DID
- `generate_secp256k1_did()` - Generate Secp256k1 DID