# TAP Agent Interoperability Guide

## Overview

The TAP TypeScript Agent (`@taprsvp/agent`) is designed to be compatible with DIDComm v2 implementations like Veramo while maintaining full support for TAP-specific message types. This document outlines compatibility considerations, supported features, and known limitations.

## Supported Standards

### DIDComm v2 Compliance

The TAP Agent supports core DIDComm v2 features:

- **Message Structure**: Full DIDComm v2 envelope format
- **Encryption**: JWE format with standard algorithms
- **Message Headers**: All standard DIDComm headers (id, type, from, to, created_time, expires_time, thid, pthid)
- **Attachments**: DIDComm attachment format

### Supported Message Types

#### TAP Message Types
- `https://tap.rsvp/schema/1.0#Transfer`
- `https://tap.rsvp/schema/1.0#Payment`
- `https://tap.rsvp/schema/1.0#Authorize`
- `https://tap.rsvp/schema/1.0#Reject`
- `https://tap.rsvp/schema/1.0#Settle`
- `https://tap.rsvp/schema/1.0#Cancel`
- `https://tap.rsvp/schema/1.0#Revert`
- `https://tap.rsvp/schema/1.0#Connect`
- `https://tap.rsvp/schema/1.0#Escrow`
- `https://tap.rsvp/schema/1.0#Capture`
- `https://tap.rsvp/schema/1.0#AddAgents`
- `https://tap.rsvp/schema/1.0#ReplaceAgent`
- `https://tap.rsvp/schema/1.0#RemoveAgent`
- `https://tap.rsvp/schema/1.0#UpdatePolicies`
- `https://tap.rsvp/schema/1.0#UpdateParty`
- `https://tap.rsvp/schema/1.0#ConfirmRelationship`
- `https://tap.rsvp/schema/1.0#AuthorizationRequired`
- `https://tap.rsvp/schema/1.0#Presentation`
- `https://tap.rsvp/schema/1.0#TrustPing`
- `https://tap.rsvp/schema/1.0#BasicMessage`

#### Standard DIDComm Message Types
The agent can pack/unpack standard DIDComm messages:
- `https://didcomm.org/basicmessage/2.0/message`
- `https://didcomm.org/trust-ping/2.0/ping`
- `https://didcomm.org/trust-ping/2.0/ping-response`
- `https://didcomm.org/discover-features/2.0/queries`
- `https://didcomm.org/discover-features/2.0/disclose`
- `https://didcomm.org/issue-credential/3.0/*`
- Other DIDComm protocol messages

## Encryption Compatibility

### Supported Algorithms

#### Key Agreement
- `ECDH-ES` - Ephemeral-Static key agreement
- `ECDH-ES+A256KW` - ES with AES Key Wrap
- `ECDH-1PU+A256KW` - One-Pass Unified Model (authenticated encryption)

#### Content Encryption
- `A256GCM` - AES-GCM with 256-bit key
- `A256CBC-HS512` - AES-CBC with HMAC-SHA512
- `XC20P` - XChaCha20-Poly1305

### Key Types
- Ed25519 - Default, widely supported
- P256 (secp256r1) - NIST curve
- secp256k1 - Bitcoin/Ethereum curve

## DID Resolution

The TAP Agent supports pluggable DID resolution:

```typescript
import { TapAgent } from '@taprsvp/agent';
import { Resolver } from 'did-resolver';
import { getResolver as getWebResolver } from 'web-did-resolver';
import { getResolver as getKeyResolver } from 'key-did-resolver';

const didResolver = new Resolver({
  ...getWebResolver(),
  ...getKeyResolver(),
});

const agent = await TapAgent.create({ didResolver });
```

Supported DID methods (with appropriate resolver):
- `did:key` - For ephemeral keys
- `did:web` - For domain-linked DIDs
- `did:ethr` - For Ethereum-based DIDs
- `did:ion` - For ION (Sidetree on Bitcoin)
- Any other DID method with a resolver

## Veramo Integration

### Message Exchange with Veramo

```typescript
// TAP Agent sending to Veramo
const tapAgent = await TapAgent.create();
const message = tapAgent.createMessage('BasicMessage', {
  content: 'Hello Veramo!'
});
const packed = await tapAgent.pack(message);
// Send packed.message to Veramo agent

// TAP Agent receiving from Veramo
const veramoMessage = '...'; // JWE from Veramo
const unpacked = await tapAgent.unpack(veramoMessage);
console.log(unpacked.body);
```

### Credential Exchange

The TAP Agent can participate in credential exchange protocols:

```typescript
// Handle credential offers from Veramo
const credentialOffer = await tapAgent.unpack(veramoJWE);
if (credentialOffer.type.includes('issue-credential')) {
  // Process credential offer
  const preview = credentialOffer.body.credential_preview;
  // ...
}
```

## Known Limitations

### Features Not Yet Supported

1. **Forward Messages**: Routing/forwarding not implemented
2. **Message Receipts**: Return receipts not handled
3. **Live Mode**: No WebSocket/real-time transport
4. **Mediation**: No mediator/relay support

### Compatibility Considerations

1. **Message Validation**: TAP Agent validates message structure but allows any message type. Unsupported types may fail during processing.

2. **Attachment Formats**: Supports base64 and JSON attachments. Link attachments require external fetching.

3. **Signature Formats**: JWS signatures are validated by WASM layer. Custom signature formats may not be supported.

4. **Transport**: Library handles message packing/unpacking only. Transport layer (HTTP, WebSocket, etc.) must be implemented separately.

## Testing Interoperability

### Running Compatibility Tests

```bash
# Run full test suite including interop tests
npm test

# Run only interoperability tests
npm test interoperability cross-implementation
```

### Test Coverage

- Message format compatibility ✅
- Encryption/decryption with standard algorithms ✅
- TAP ↔ DIDComm message conversion ✅
- Thread ID preservation ✅
- Attachment handling ✅
- Error recovery ✅

### Integration Testing

For testing with real Veramo instances:

```typescript
import { TapAgent } from '@taprsvp/agent';
import { createAgent } from '@veramo/core';

// Setup both agents
const tapAgent = await TapAgent.create();
const veramoAgent = createAgent({...});

// Exchange messages
const message = tapAgent.createMessage('BasicMessage', {
  content: 'Test message'
});
const packed = await tapAgent.pack(message);

// Veramo unpacks TAP message
const veramoUnpacked = await veramoAgent.unpackDIDCommMessage({
  message: packed.message
});
```

## Migration Guide

### From Veramo to TAP Agent

```typescript
// Before (Veramo)
const message = {
  type: 'https://didcomm.org/basicmessage/2.0/message',
  from: agent.identifier.did,
  to: [recipientDid],
  id: uuidv4(),
  body: { content: 'Hello' }
};
const packed = await agent.packDIDCommMessage({ message });

// After (TAP Agent)
const tapAgent = await TapAgent.create();
const message = tapAgent.createMessage('BasicMessage', {
  content: 'Hello'
});
message.to = [recipientDid];
const packed = await tapAgent.pack(message);
```

### From TAP Agent to Veramo

Both libraries use similar DIDComm v2 structures, making migration straightforward. Main differences:
- TAP Agent uses typed message helpers
- Veramo requires manual message construction
- TAP Agent has built-in TAP protocol support

## Best Practices

1. **Use Standard Types**: When interoperating, prefer standard DIDComm message types
2. **Validate DIDs**: Ensure DIDs are resolvable by both agents
3. **Test Encryption**: Verify encryption algorithms are supported by recipients
4. **Handle Errors**: Implement graceful fallbacks for unsupported features
5. **Document Protocols**: Clearly specify which message types your application uses

## Support

For issues or questions about interoperability:
- GitHub Issues: [TAP-RS Repository](https://github.com/yourusername/tap-rs)
- Documentation: This guide and test files
- Examples: See `tests/interoperability.test.ts` and `tests/cross-implementation.test.ts`