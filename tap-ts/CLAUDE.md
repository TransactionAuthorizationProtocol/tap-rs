# CLAUDE.md - TAP TypeScript Agent

This file provides guidance to Claude Code when working with the TAP TypeScript Agent library.

## Overview

The TAP TypeScript Agent (`@taprsvp/agent`) is a browser-first TypeScript wrapper around the TAP WASM implementation. It provides a clean, type-safe API for working with TAP (Transaction Authorization Protocol) messages in JavaScript/TypeScript environments.

## Build Commands

```bash
# Install dependencies
npm install

# Run tests
npm test -- --no-watch

# Run tests in watch mode
npm run test:watch

# Build the library
npm run build

# Type check
npm run typecheck

# Lint
npm run lint

# Format code
npm run format
```

## Project Structure

```
tap-ts/
├── src/              # Source code
│   ├── index.ts      # Main exports
│   ├── tap-agent.ts  # Core TapAgent class
│   ├── types.ts      # TypeScript type definitions
│   ├── utils.ts      # Utility functions
│   └── type-mapping.ts # Type conversion layer
├── tests/            # Test files
├── dist/             # Built output
└── package.json      # Package configuration
```

## Key Features

- **Browser-First**: Optimized for browser environments with WASM
- **Type Safety**: Full TypeScript support with comprehensive types
- **DID Support**: Built-in DID:key resolution with pluggable resolver interface
- **TAP Compliant**: Supports all TAP message types (Transfer, Payment, Authorize, etc.)
- **Flexible Key Management**: Export/import private keys for custom storage

## API Usage Examples

### Creating an Agent

```typescript
import { TapAgent } from '@taprsvp/agent';

// Create a new agent with auto-generated keys
const agent = await TapAgent.create();

// Create from existing private key
const agent = await TapAgent.fromPrivateKey(privateKeyHex, 'Ed25519');

// Get agent's DID
const did = agent.getDid();
```

### Message Operations

```typescript
// Pack a message for sending
const packedResult = await agent.packMessage({
  id: 'msg-123',
  type: 'https://tap.rsvp/schema/1.0#Transfer',
  from: agent.getDid(),
  to: ['did:key:recipient'],
  body: {
    amount: '100.00',
    asset: 'eip155:1/erc20:0x...',
    originator: { '@id': agent.getDid() },
    beneficiary: { '@id': 'did:key:beneficiary' }
  }
});

// Unpack a received message
const message = await agent.unpackMessage(packedResult.message);
```

### Key Management

```typescript
// Export private key for storage
const privateKey = agent.exportPrivateKey();

// Export public key
const publicKey = agent.exportPublicKey();

// Generate utilities
import { generatePrivateKey, generateUuid } from '@taprsvp/agent';

const key = generatePrivateKey('Ed25519');
const uuid = generateUuid();
```

## Testing

Tests are written using Vitest and follow TDD principles:

- `tap-agent.test.ts` - Core agent functionality tests
- `type-mapping.test.ts` - Type conversion tests
- `utils.test.ts` - Utility function tests
- `integration.test.ts` - End-to-end integration tests

Run tests with coverage:
```bash
npm run test -- --coverage
```

## Type System

The library uses TypeScript strict mode with comprehensive type definitions:

- `DIDCommMessage` - Main message type
- `MessageBody` - Generic message body interface
- `TransferBody`, `PaymentBody`, etc. - Specific message body types
- `PackedMessageResult` - Result of packing operation
- `SecurityMode` - Message security options

## WASM Integration

The TypeScript library wraps the WASM implementation from `tap-wasm`:

- WASM module is imported from `../tap-wasm/pkg/`
- All cryptographic operations are performed in WASM
- TypeScript provides the API layer and type safety

## Bundle Size Targets

- WASM: < 500KB
- TypeScript: < 50KB gzipped
- Current sizes well under targets

## Development Guidelines

1. **Type Safety**: Always use TypeScript strict mode
2. **Browser Compatibility**: Test in browser environments
3. **Error Handling**: Provide clear error messages
4. **Documentation**: Keep JSDoc comments updated
5. **Testing**: Maintain >80% test coverage
6. **Performance**: Profile critical paths

## Common Tasks

### Adding a New Message Type

1. Add type definition in `types.ts`
2. Update type mapping in `type-mapping.ts`
3. Add tests in `type-mapping.test.ts`
4. Update documentation

### Debugging WASM Issues

1. Check console for WASM loading errors
2. Verify `tap-wasm/pkg/` is built
3. Check browser DevTools Network tab
4. Use `agent.debug = true` for verbose logging

## Release Process

1. Update version in `package.json`
2. Run tests: `npm test`
3. Build: `npm run build`
4. Update CHANGELOG
5. Publish: `npm publish`
