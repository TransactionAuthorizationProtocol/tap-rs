# TAP Agent CLI

A command-line tool for managing Decentralized Identifiers (DIDs) and cryptographic keys in the TAP ecosystem.

## Features

- Generate DIDs with different key types (Ed25519, P-256, Secp256k1)
- Support for different DID methods (did:key, did:web)
- Save and manage DIDs and keys in a local key store
- Import and export keys for backup or transfer
- Resolve DIDs to display their DID documents
- Integration with the TAP Agent library

## Installation

From the TAP repository:
```bash
cargo install --path tap-agent
```

From crates.io:
```bash
cargo install tap-agent
```

## Commands

### Generate

Creates a new DID with the specified method and key type:

```bash
# Basic usage
tap-agent-cli generate

# Specify method and key type
tap-agent-cli generate --method key --key-type ed25519
tap-agent-cli generate --method key --key-type p256
tap-agent-cli generate --method key --key-type secp256k1
tap-agent-cli generate --method web --domain example.com

# Save outputs
tap-agent-cli generate --output did.json --key-output key.json
tap-agent-cli generate --save --default
```

Options:
- `--method, -m`: DID method to use (`key` or `web`, default: `key`)
- `--key-type, -t`: Key type to generate (`ed25519`, `p256`, or `secp256k1`, default: `ed25519`)
- `--domain, -d`: Domain for did:web (required when method is `web`)
- `--output, -o`: Output file path for the DID document
- `--key-output, -k`: Output file path for the private key
- `--save, -s`: Save key to default location (~/.tap/keys.json)
- `--default`: Set as default key when saving
- `--label, -l`: Label for the key (defaults to agent-{n})

### Lookup

Resolves a DID to its DID document:

```bash
# Basic lookup
tap-agent-cli lookup did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

# Save to file
tap-agent-cli lookup did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK --output doc.json

# Look up WebDIDs
tap-agent-cli lookup did:web:example.com
tap-agent-cli lookup did:web:example.com:path:to:resource
```

Options:
- `--output, -o`: Output file path for the resolved DID document

### Keys

Manages stored keys in the local key store:

```bash
# List all keys (shows labels, DIDs, and key types)
tap-agent-cli keys list

# View a specific key (by DID or label)
tap-agent-cli keys view did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
tap-agent-cli keys view "my-signing-key"
tap-agent-cli keys view "agent-1"

# Set a key as default (by DID or label)
tap-agent-cli keys set-default did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
tap-agent-cli keys set-default "production-key"

# Delete a key (by DID or label)
tap-agent-cli keys delete did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
tap-agent-cli keys delete "test-key" --force

# Relabel an existing key
tap-agent-cli keys relabel "agent-1" "development-key"
tap-agent-cli keys relabel did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK "new-label"
```

### Import

Imports an existing key into the key store:

```bash
# Basic import
tap-agent-cli import key.json

# Import and set as default
tap-agent-cli import key.json --default

# Import with custom label
tap-agent-cli import key.json --label "imported-key"
tap-agent-cli import key.json --label "backup-key" --default
```

Options:
- `--default`: Set the imported key as the default key

## DID Methods

### did:key

The `did:key` method generates self-contained DIDs that include the public key material directly in the identifier. These DIDs are portable and don't require external infrastructure for resolution.

Prefix encodings:
- Ed25519: `0xed01`
- P-256: `0x1200`
- Secp256k1: `0xe701`

### did:web

The `did:web` method creates DIDs associated with domain names. To use this method:

1. Generate a did:web DID with your domain
2. Host the DID document at the appropriate location:
   - `did:web:example.com` → `https://example.com/.well-known/did.json`
   - `did:web:example.com:path:to:resource` → `https://example.com/path/to/resource/did.json`

## Key Storage

Keys are stored locally in `~/.tap/keys.json`. This file contains:
- A collection of all your DIDs and their associated key material
- Information about the default DID (if set)
- Metadata about each key

The storage format is JSON-based and can be backed up or transferred between systems.

## Key Types

The CLI supports three key types:

- **Ed25519**: A fast and secure digital signature algorithm with small signatures
- **P-256**: An NIST standardized elliptic curve algorithm (also known as secp256r1)
- **Secp256k1**: The elliptic curve used by Bitcoin, Ethereum, and many other blockchain systems