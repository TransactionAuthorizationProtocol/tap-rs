# PRD: TAP WASM Agent v2 - Simplified Browser-First Implementation

## Executive Summary

This PRD defines a complete rewrite of the TAP WASM implementation to create a minimal, browser-optimized agent focused on message packing/unpacking with flexible key management. The new implementation prioritizes simplicity, bundle size optimization, and compatibility with existing JavaScript cryptographic libraries.

### Key Objectives
- **Simplicity**: Single `TapAgent` class focused on core message operations
- **Browser-First**: Direct JavaScript access to private keys for flexible key storage
- **Type Safety**: Leverage `@taprsvp/types` for consistent message structures
- **Compatibility**: Ensure interoperability with `@veramo/did-comm` and `did-resolver` ecosystems
- **Performance**: Minimize bundle size and maximize runtime efficiency

## 1. Goals & Non-Goals

### Goals
✅ **Core Message Operations**
- Pack TAP messages to JWE/JWS format
- Unpack and verify received messages
- Support all TAP message types defined in TAIPs

✅ **Browser Key Management**
- Export private keys to JavaScript
- Allow browser-based key storage (IndexedDB, localStorage, etc.)
- Support multiple cryptographic key types (Ed25519, P-256, secp256k1)

✅ **TypeScript Integration**
- Use `@taprsvp/types` as the canonical message type definitions
- Provide seamless type mapping between TypeScript and Rust structures
- Export as `@taprsvp/agent` npm package

✅ **Ecosystem Compatibility**
- Message format compatible with `@veramo/did-comm` for testing interoperability
- Support `did-resolver` package interface for pluggable DID resolution
- Interoperable with existing DIDComm v2 implementations

✅ **Developer Experience**
- Minimal API surface area
- Clear error messages and debugging support
- Comprehensive TypeScript type definitions

### Non-Goals
❌ **Node Management** - Remove `TapNode` multi-agent coordination
❌ **Built-in Networking** - No transport layer implementation
❌ **Complex State Management** - No transaction state machines
❌ **Server-Side Features** - Optimize purely for browser environments
❌ **Legacy API Compatibility** - Clean break from current implementation

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    @taprsvp/agent                           │
├─────────────────────────────────────────────────────────────┤
│  TypeScript Wrapper Layer                                  │
│  ├─ TapAgent class                                         │
│  ├─ Key management utilities                               │
│  ├─ Type mapping (@taprsvp/types ↔ Rust)                  │
│  └─ DID resolver integration                               │
├─────────────────────────────────────────────────────────────┤
│                WASM Bindings (tap-wasm)                    │
│  ├─ Thin wrapper around existing TapAgent                 │
│  ├─ JsValue ↔ Rust type conversions                       │
│  ├─ Private key export/import methods                     │
│  └─ WASM-specific optimizations                           │
├─────────────────────────────────────────────────────────────┤
│            Existing tap-agent Core (Reused)                │
│  ├─ TapAgent struct (tap-agent/src/agent.rs)             │
│  ├─ AgentKeyManager (existing key management)             │
│  ├─ Message packing/unpacking (existing)                  │
│  ├─ Cryptographic operations (existing)                   │
│  └─ DID generation & resolution (existing)                │
├─────────────────────────────────────────────────────────────┤
│                    Dependencies                            │
│  ├─ tap-msg (message structures)                          │
│  ├─ tap-agent (existing implementation)                   │
│  └─ @taprsvp/types (TypeScript definitions)               │
└─────────────────────────────────────────────────────────────┘
```

## 3. API Design

### 3.1 Core WASM Interface

**IMPORTANT**: The WASM implementation will reuse the existing `tap-agent/src/agent.rs` codebase and its `TapAgent` struct. We will NOT recreate functionality that already exists. Instead, we'll provide WASM bindings that wrap the existing implementation.

```rust
// WASM wrapper for the existing TapAgent from tap-agent/src/agent.rs
// This is a thin wrapper, NOT a reimplementation
#[wasm_bindgen]
pub struct WasmTapAgent {
    // Internally uses the existing tap_agent::TapAgent
    inner: tap_agent::TapAgent,
}

#[wasm_bindgen]
impl WasmTapAgent {
    // Create agent with generated keys (wraps TapAgent::from_ephemeral_key)
    #[wasm_bindgen(constructor)]
    pub async fn new(key_type: KeyType) -> Result<WasmTapAgent, JsValue>;

    // Create agent from existing private key (wraps TapAgent::from_private_key)
    #[wasm_bindgen]
    pub async fn from_private_key(private_key: &str, key_type: KeyType) -> Result<WasmTapAgent, JsValue>;

    // Core operations (delegates to existing TapAgent methods)
    pub fn get_did(&self) -> String; // Uses inner.get_agent_did()
    pub fn export_private_key(&self) -> Result<String, JsValue>; // Extract from key_manager
    pub fn export_public_key(&self) -> Result<String, JsValue>; // Extract from key_manager

    // Message operations (delegates to existing TapAgent pack/unpack)
    pub async fn pack_message(&self, message: JsValue) -> Result<String, JsValue>;
    pub async fn unpack_message(&self, packed: &str) -> Result<JsValue, JsValue>;

    // DID resolution (can use existing resolver infrastructure)
    pub async fn resolve_did(&self, did: &str) -> Result<JsValue, JsValue>;
}

// Utility functions (can reuse from existing codebase)
#[wasm_bindgen]
pub fn generate_private_key(key_type: KeyType) -> Result<String, JsValue>;

#[wasm_bindgen]
pub fn generate_uuid() -> String; // Can use existing UUID generation
```

### 3.2 TypeScript Wrapper API

```typescript
import {
  DIDCommMessage,
  Transfer,
  Payment,
  Authorize,
  // ... other message types
} from '@taprsvp/types';

export interface TapAgentConfig {
  keyType?: 'Ed25519' | 'P256' | 'secp256k1';
  privateKey?: string; // Optional: use existing key
  didResolver?: DIDResolver; // Optional: custom resolver
}

export interface PackedMessage {
  message: string; // JWE/JWS formatted message
  metadata: {
    type: 'encrypted' | 'signed' | 'plain';
    recipients?: string[];
    sender?: string;
  };
}

export class TapAgent {
  // Static factory methods
  static async create(config?: TapAgentConfig): Promise<TapAgent>;
  static async fromPrivateKey(privateKey: string, keyType?: string): Promise<TapAgent>;

  // Identity management
  get did(): string;
  get publicKey(): string;
  exportPrivateKey(): string;

  // Message operations
  async pack<T>(message: DIDCommMessage<T>): Promise<PackedMessage>;
  async unpack<T>(packed: string): Promise<DIDCommMessage<T>>;

  // DID resolution
  async resolveDID(did: string): Promise<DIDDocument>;

  // Utility methods
  createMessage<T>(type: string, body: T): DIDCommMessage<T>;
}

// Utility exports
export function generatePrivateKey(keyType?: string): string;
export function generateUUID(): string;
```

## 4. Key Management Strategy

### 4.1 Browser-Accessible Keys

The new implementation allows JavaScript access to private keys, enabling flexible storage options:

```typescript
// Export keys for browser storage
const agent = await TapAgent.create();
const privateKey = agent.exportPrivateKey();

// Store in IndexedDB
await indexedDB.setItem('tapAgent.privateKey', privateKey);

// Restore from storage
const restoredAgent = await TapAgent.fromPrivateKey(privateKey);
```

### 4.2 Key Storage Patterns

**Recommended Storage Options:**
1. **IndexedDB**: Persistent, large storage capacity
2. **Session Storage**: Temporary, session-based storage
3. **Encrypted Local Storage**: With user-provided encryption key
4. **Hardware Security Modules**: Via WebAuthn/FIDO2 integration

### 4.3 Key Rotation Support

```typescript
// Generate new key and migrate
const newAgent = await TapAgent.create();
const oldAgent = await TapAgent.fromPrivateKey(oldPrivateKey);

// Application-specific: Update DID mappings in your system
// This is NOT part of the WASM agent API - developers must implement:
// - Update local databases with new DID
// - Notify contacts of DID change
// - Update service registrations
// - Migrate pending transactions
await myApp.updateDIDMapping(oldAgent.did, newAgent.did);
```

## 5. Type System & Message Mapping

### 5.1 TypeScript to Rust Mapping

The implementation provides automatic conversion between `@taprsvp/types` and internal Rust structures:

```typescript
// TypeScript side - using @taprsvp/types
import { Transfer, TransferMessage } from '@taprsvp/types';

const transfer: Transfer = {
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "Transfer",
  asset: "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7",
  amount: "100.0",
  originator: { "@id": "did:key:z6Mk..." },
  beneficiary: { "@id": "did:key:z6Mr..." },
  agents: []
};

const message: TransferMessage = {
  id: generateUUID(),
  type: "https://tap.rsvp/schema/1.0#Transfer",
  from: agent.did,
  to: ["did:key:z6Mr..."],
  created_time: Date.now(),
  body: transfer
};

// Automatic conversion to Rust tap-msg types
const packed = await agent.pack(message);
```

### 5.2 Supported Message Types

All TAP message types from `@taprsvp/types`:
- `Transfer` / `TransferMessage`
- `Payment` / `PaymentMessage`
- `Authorize` / `AuthorizeMessage`
- `Reject` / `RejectMessage`
- `Settle` / `SettleMessage`
- `Cancel` / `CancelMessage`
- `Revert` / `RevertMessage`
- `Connect` / `ConnectMessage`
- `Escrow` / `EscrowMessage`
- `Capture` / `CaptureMessage`

## 6. DID Resolution Interface

### 6.1 Pluggable Resolver Architecture

```typescript
import { Resolver, DIDDocument } from 'did-resolver';

// Built-in DID:key support
const agent = await TapAgent.create();
const didDoc = await agent.resolveDID('did:key:z6Mk...');

// Custom resolver integration
const customResolver = new Resolver({
  web: webDIDResolver,
  ethr: ethrDIDResolver,
  // ... other method resolvers
});

const agentWithResolver = await TapAgent.create({
  didResolver: customResolver
});
```

### 6.2 DID:key Built-in Support

The WASM core includes optimized DID:key resolution for common key types:

```rust
// Built into WASM for performance
impl TapAgent {
    pub async fn resolve_did_key(&self, did: &str) -> Result<JsValue, JsValue> {
        // Fast path for did:key resolution
        if did.starts_with("did:key:") {
            return self.resolve_key_did(did);
        }
        // Delegate to JavaScript resolver
        self.delegate_resolution(did)
    }
}
```

## 7. Veramo Interoperability Testing

### 7.1 Message Format Compatibility Testing

The TAP agent's message format should be testable for compatibility with `@veramo/did-comm`, but Veramo integration is **only for testing purposes**, not runtime integration:

```typescript
// Test: TAP Agent packing should be readable by Veramo
const tapAgent = await TapAgent.create();
const packed = await tapAgent.pack(message);

// Veramo should be able to unpack TAP-generated messages (TEST ONLY)
const veramoAgent = new Agent({
  plugins: [new DIDComm()]
});

const unpacked = await veramoAgent.unpackDIDCommMessage({
  message: packed.message
});
```

### 7.2 Interoperability Test Suite

```typescript
// Test compatibility in both directions - FOR TESTING ONLY
describe('Veramo Interoperability Tests', () => {
  it('should allow Veramo to unpack TAP-packed messages', async () => {
    const tapPacked = await tapAgent.pack(testMessage);
    const veramoUnpacked = await veramoAgent.unpackDIDCommMessage(tapPacked);
    expect(veramoUnpacked.message).toEqual(testMessage);
  });

  it('should allow TAP to unpack Veramo-packed messages', async () => {
    const veramoPacked = await veramoAgent.packDIDCommMessage(testMessage);
    const tapUnpacked = await tapAgent.unpack(veramoPacked.message);
    expect(tapUnpacked).toEqual(testMessage);
  });

  // NOTE: These tests verify message format compatibility
  // Veramo is NOT integrated into the TAP runtime - only for testing
});
```

## 8. Bundle Size Optimization

### 8.1 Size Targets

- **WASM Binary**: < 500KB (current: ~800KB)
- **TypeScript Bundle**: < 50KB gzipped
- **Total Package Size**: < 1MB

### 8.2 Optimization Strategies

**WASM Optimizations:**
- Remove unused message types at compile time
- Minimize debug symbols in release builds
- Use `wee_alloc` for smaller memory allocator
- Strip unnecessary dependencies

**TypeScript Optimizations:**
- Tree-shakable exports
- Minimal dependency graph
- Lazy loading of optional features

```typescript
// Tree-shakable exports
export { TapAgent } from './agent';
export { generatePrivateKey, generateUUID } from './utils';
export type { TapAgentConfig, PackedMessage } from './types';

// Optional features as separate imports
export { AdvancedKeyManagement } from './advanced'; // Optional
export { BatchOperations } from './batch'; // Optional
```

## 9. Testing Requirements

### 9.1 Unit Tests

- Key generation and management
- Message packing/unpacking for all types
- Type conversion between TypeScript and Rust
- DID resolution functionality
- Error handling and edge cases
- Use vitest for typescript tests

### 9.2 Integration Tests

- Veramo interoperability testing (format compatibility only)
- Cross-browser compatibility (Chrome, Firefox, Safari, Edge)
- Bundle size verification
- Performance benchmarks

### 9.3 End-to-End Tests

```typescript
describe('E2E Message Flow', () => {
  it('should complete full message lifecycle', async () => {
    // 1. Create agents
    const sender = await TapAgent.create();
    const receiver = await TapAgent.create();

    // 2. Create and pack message
    const message = sender.createMessage('Transfer', transferBody);
    const packed = await sender.pack(message);

    // 3. Transmit (simulated)
    const transmitted = simulateTransmission(packed);

    // 4. Unpack and verify
    const unpacked = await receiver.unpack(transmitted);
    expect(unpacked.body).toEqual(transferBody);
  });
});
```

## 10. Implementation Phases

### Phase 1: Core WASM Bindings (2 weeks)
- [ ] Create `WasmTapAgent` wrapper around existing `TapAgent`
- [ ] Implement private key export/import methods
- [ ] Add JsValue conversions for message operations
- [ ] Basic WASM-specific error handling

### Phase 2: TypeScript Wrapper (2 weeks)
- [ ] TypeScript wrapper with `@taprsvp/types` integration
- [ ] Type mapping between TypeScript and Rust
- [ ] DID resolver integration
- [ ] npm package setup

### Phase 3: Testing & Compatibility (2 weeks)
- [ ] Comprehensive test suite
- [ ] Veramo interoperability testing (format compatibility only)
- [ ] Performance benchmarks

### Phase 4: Optimization & Documentation (1 week)
- [ ] Bundle size optimization
- [ ] Performance tuning
- [ ] API documentation
- [ ] Usage examples and guides

### Phase 5: Release & Documentation (1 week)
- [ ] Final testing and QA
- [ ] Release preparation
- [ ] Documentation finalization
- [ ] Community communication

**Note**: Total timeline reduced from 13 weeks to 8 weeks by reusing existing codebase

## 11. Success Metrics

- **Bundle Size**: < 500KB total package size
- **Performance**: < 50ms for pack/unpack operations
- **Compatibility**: 100% interoperability test coverage with Veramo message formats
- **Developer Experience**: < 5 minutes from install to first packed message
- **Browser Support**: Chrome 80+, Firefox 75+, Safari 13+, Edge 80+

## 12. Risk Assessment

### Technical Risks
- **WASM-JS Boundary Performance**: Mitigation through minimal data crossing
- **Bundle Size**: Mitigation through aggressive tree-shaking and optimization
- **Browser Compatibility**: Mitigation through comprehensive testing matrix

### Product Risks
- **Ecosystem Fragmentation**: Mitigation through standards compliance
- **Security Considerations**: Mitigation through security audit and best practices
- **Developer Adoption**: Mitigation through clear documentation and examples

---

**Document Version**: 1.0
**Last Updated**: 2025-01-19
**Owner**: TAP Development Team
**Stakeholders**: Frontend Teams, DIDComm Working Group, TAP Community
