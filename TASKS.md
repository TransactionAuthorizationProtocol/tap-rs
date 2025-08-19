# TAP-RS Implementation Tasks

## TAP WASM Agent v2 - Simplified Browser-First Implementation
[PRD](prds/wasmagent.md)

### Phase 1: Core WASM Bindings & Tests

- [x] Write tests for WasmTapAgent wrapper around existing TapAgent
- [x] Write tests for private key export functionality
- [x] Write tests for private key import functionality
- [x] Write tests for JsValue to Rust type conversions
- [x] Create WasmTapAgent wrapper struct in tap-wasm
- [x] Implement private key export from AgentKeyManager
- [x] Implement private key import to create TapAgent
- [x] Implement JsValue conversion layer

### Phase 2: Message Operations & Tests

- [x] Write tests for delegating pack_message to existing TapAgent
- [x] Write tests for delegating unpack_message to existing TapAgent
- [x] Write tests for all TAP message types via WASM
- [x] Write tests for WASM-specific error handling
- [x] Implement pack_message delegation to TapAgent
- [x] Implement unpack_message delegation to TapAgent
- [x] Ensure TAP message type compatibility
- [x] Add WASM error conversion layer

### Phase 3: WASM Bindings & JavaScript Interface

- [x] Write tests for WASM bindings (new, from_private_key, get_did)
- [x] Write tests for async pack_message and unpack_message
- [x] Write tests for utility functions (generate_private_key, generate_uuid)
- [x] Write tests for JavaScript type conversions (JsValue <-> Rust types)
- [x] Implement WASM bindings for TapAgent
- [x] Implement async message operations
- [x] Implement utility function exports
- [x] Implement type conversion layer
- [x] Update tap-wasm/CLAUDE.md

### Phase 4: TypeScript Wrapper & npm Package

- [x] Write tests for TypeScript TapAgent class
- [x] Write tests for type mapping with @taprsvp/types
- [x] Write tests for static factory methods (create, fromPrivateKey)
- [x] Write tests for message creation helpers
- [x] Create TypeScript wrapper in tap-ts
- [x] Implement type mapping with @taprsvp/types
- [x] Implement factory methods and utilities
- [x] Setup npm package structure (@taprsvp/agent)
- [x] Create tap-ts/CLAUDE.md documentation

### Phase 5: DID Resolution & Integration

- [x] Write tests for pluggable DID resolver interface
- [x] Write tests for resolution error handling
- [x] Implement pluggable resolver interface
- [x] Implement JavaScript resolver delegation

### Phase 6: Interoperability Testing

- [x] Write tests for Veramo message format compatibility
- [x] Write tests for TAP -> Veramo message unpacking
- [x] Write tests for Veramo -> TAP message unpacking
- [x] Write cross-implementation test suite
- [x] Verify message format compatibility
- [x] Test with real Veramo instances (via real WASM tests)
- [x] Document any compatibility issues
- [x] Create interoperability test fixtures

### Phase 7: Bundle Optimization

- [x] Implement wee_alloc for smaller WASM
- [x] Remove unused dependencies
- [x] Optimize TypeScript bundle
- [x] Verify < 500KB WASM target (272KB gzipped ✅)
- [x] Verify < 50KB gzipped TypeScript (3.72KB gzipped ✅)

### Phase 8: Documentation & Release

- [ ] Write API documentation
- [ ] Write getting started guide
- [ ] Write example applications
- [ ] Create API reference docs
- [ ] Publish to npm as @taprsvp/agent
- [ ] Update main README
- [ ] Create release notes
