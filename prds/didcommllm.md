# DIDComm Messaging Specification v2.1 - Condensed Reference

## Core Concepts

DIDComm Messaging is a secure, private communication methodology built on Decentralized Identifiers (DIDs) that enables:
- Decentralized, transport-agnostic communication
- End-to-end encryption with forward secrecy
- Message-based, asynchronous interactions
- Sender authentication and non-repudiation
- Consistent experiences across devices and communication channels

## Message Types

### 1. Plaintext Messages
- Base format for all DIDComm messages (before encryption)
- JSON structure with required headers:
  ```json
  {
    "id": "unique-message-id",
    "type": "protocol-type/version/message-type",
    "from": "did:example:sender",
    "to": ["did:example:recipient1", "did:example:recipient2"],
    "body": { /* message content */ }
  }
  ```
- Media type: `application/didcomm-plain+json`

### 2. Signed Messages
- Provides non-repudiation by adding digital signatures
- JWS structure with the payload containing the plaintext message
- Media type: `application/didcomm-signed+json`
- Process: 
  1. Take a plaintext message
  2. Convert it to canonical form
  3. Create a JWS using the sender's signing key
  4. Include the signature in the JWS Protected Header

#### Signed Message Structure
```json
{
  "payload": "base64url_encoded_plaintext_message",
  "signatures": [
    {
      "protected": "base64url_encoded_protected_header",
      "signature": "base64url_encoded_signature",
      "header": {
        "kid": "did:example:sender#key-1"
      }
    }
  ]
}
```

#### Example Code (Pseudocode)
```javascript
// Create a plaintext message
const plaintextMessage = {
  "id": "1234567890",
  "type": "https://example.org/protocols/example/1.0",
  "from": "did:example:alice",
  "to": ["did:example:bob"],
  "created_time": 1609459200,
  "body": { "message": "Hello, Bob!" }
};

// Sign the message
const signingKey = await resolver.resolveSigningKey("did:example:alice#key-1");
const signedMessage = await DIDComm.sign({
  message: plaintextMessage,
  sign: {
    from: "did:example:alice#key-1",
    signingKey: signingKey
  }
});

// signedMessage is now in application/didcomm-signed+json format
```

### 3. Encrypted Messages
- Provides confidentiality and integrity
- Two modes:
  - **Authcrypt**: Authenticated encryption (sender identity known to recipient)
  - **Anoncrypt**: Anonymous encryption (sender identity hidden)
- JWE structure with nested encryption for multiple recipients
- Media type: `application/didcomm-encrypted+json`
- Process:
  1. Start with plaintext or signed message
  2. Encrypt the message content with a Content Encryption Key (CEK)
  3. Encrypt the CEK for each recipient using their public key
  4. Package as a JWE with appropriate headers

#### Encrypted Message Structure
```json
{
  "protected": "base64url_encoded_protected_header",
  "recipients": [
    {
      "header": {
        "kid": "did:example:bob#key-x25519-1",
        "alg": "ECDH-ES+A256KW",
        "epk": {
          "kty": "OKP",
          "crv": "X25519",
          "x": "base64url_encoded_ephemeral_public_key"
        },
        "apu": "base64url_encoded_sender_did_reference",
        "apv": "base64url_encoded_recipient_did_reference"
      },
      "encrypted_key": "base64url_encoded_encrypted_cek"
    }
  ],
  "iv": "base64url_encoded_initialization_vector",
  "ciphertext": "base64url_encoded_ciphertext",
  "tag": "base64url_encoded_authentication_tag"
}
```

#### Example Code for Authcrypt (Pseudocode)
```javascript
// Start with a plaintext or signed message
const message = signedMessage || plaintextMessage;

// Encrypt the message (Authenticated - sender is known)
const encryptedMessage = await DIDComm.encrypt({
  message: message,
  to: "did:example:bob#key-x25519-1",
  from: "did:example:alice#key-x25519-1",
  encAlgAuth: "A256GCM"
});

// encryptedMessage is now in application/didcomm-encrypted+json format
```

#### Example Code for Anoncrypt (Pseudocode)
```javascript
// Encrypt the message (Anonymous - sender is hidden)
const encryptedMessage = await DIDComm.encrypt({
  message: message,
  to: "did:example:bob#key-x25519-1",
  encAlgAnon: "A256GCM"
});
```

#### Multi-Recipient Encryption
```javascript
// Encrypt for multiple recipients
const encryptedMessage = await DIDComm.encrypt({
  message: message,
  to: [
    "did:example:bob#key-x25519-1",
    "did:example:charlie#key-x25519-1",
    "did:example:diana#key-x25519-1"
  ],
  from: "did:example:alice#key-x25519-1"
});
```

## Message Processing

1. **Creation**: Construct plaintext message → Sign (optional) → Encrypt
2. **Reception**: Decrypt → Verify signature (if signed) → Process plaintext

## Security Considerations

- **Encryption Algorithms**:
  - Content encryption: A256GCM (AES-GCM with 256-bit key)
  - Key agreement: ECDH-ES with X25519, P-384, or P-256 curves
  - Key wrapping: A256KW (AES-KW with 256-bit key)

- **Signing Algorithms**:
  - EdDSA, ES256, ES256K

- **Best Practices**:
  - Use fresh DID keys for each relationship
  - Implement regular key rotation
  - Verify DID document permissions
  - Validate message structure and content

## Routing

- **Forward Routing**: Messages pass through intermediaries to reach recipient
- **Return Routing**: Response channels using the same path as incoming messages
- **Routing Headers**:
  ```json
  {
    "next": "did:example:nextrecipient",
    "forward_headers": ["trace_headers"],
    "return_route": "all"
  }
  ```

## Protocol Support

DIDComm supports layering of higher-level protocols including:
- Connection establishment
- Trust establishment
- Credential exchange
- Payments and transactions
- Secure data sharing

## Implementation Guidance

1. Use DID Document service endpoints for delivery
2. Implement robust error handling with problem reports
3. Support message threading for conversation context
4. Prioritize privacy by minimizing metadata exposure
5. Maintain transport independence (HTTP, WebSockets, Bluetooth, etc.)

## Extensions

The specification is designed for extensibility through:
- Custom message types
- Protocol-specific headers
- Specialized attachments
- Feature discovery mechanisms

## Reference Resources

- Full specification: https://identity.foundation/didcomm-messaging/spec/v2.1/
- Test vectors and examples: https://github.com/decentralized-identity/didcomm-messaging