# TAP Agent CLI

A command-line tool for working with Decentralized Identifiers (DIDs) in the TAP ecosystem.

## Features

- Generate DIDs with different key types (Ed25519, P256, Secp256k1)
- Support for different DID methods (did:key, did:web)
- Save DID documents and private keys to files
- Simple interface for managing DIDs and keys

## Usage

### Generating a DID

```bash
# Generate a did:key with Ed25519
tap-agent-cli generate --method key --key-type ed25519

# Generate a did:key with P-256
tap-agent-cli generate --method key --key-type p256

# Generate a did:key with Secp256k1
tap-agent-cli generate --method key --key-type secp256k1

# Generate a did:web for a domain
tap-agent-cli generate --method web --domain example.com
```

### Saving Output to Files

```bash
# Save DID document to did.json and key to key.json
tap-agent-cli generate --output did.json --key-output key.json

# Save did:web document (to be placed at /.well-known/did.json on the domain)
tap-agent-cli generate --method web --domain example.com --output did.json
```

## DID Methods

### did:key

The `did:key` method generates a self-contained DID that includes the public key material in the identifier itself. This is useful for situations where simplicity and portability are important, as it doesn't require any external resolution infrastructure.

### did:web

The `did:web` method creates a DID that is associated with a domain name. To use this DID, you need to host the DID document at a specific URL on your domain:

```
https://yourdomain.com/.well-known/did.json
```

This method allows for DID ownership to be linked to domain ownership, and the DID document can be updated by modifying the file on the web server.

## Key Types

The CLI supports three key types:

- **Ed25519**: A fast and secure digital signature algorithm designed for high-speed signature verification
- **P-256**: An elliptic curve digital signature algorithm that follows NIST standards
- **Secp256k1**: The curve used by Bitcoin, Ethereum, and many other blockchain systems