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

- [ ] Write tests for WASM bindings (new, from_private_key, get_did)
- [ ] Write tests for async pack_message and unpack_message
- [ ] Write tests for utility functions (generate_private_key, generate_uuid)
- [ ] Write tests for JavaScript type conversions (JsValue <-> Rust types)
- [ ] Implement WASM bindings for TapAgent
- [ ] Implement async message operations
- [ ] Implement utility function exports
- [ ] Implement type conversion layer
- [ ] Update tap-wasm/CLAUDE.md

### Phase 4: TypeScript Wrapper & npm Package

- [ ] Write tests for TypeScript TapAgent class
- [ ] Write tests for type mapping with @taprsvp/types
- [ ] Write tests for static factory methods (create, fromPrivateKey)
- [ ] Write tests for message creation helpers
- [ ] Create TypeScript wrapper in tap-wasm-js
- [ ] Implement type mapping with @taprsvp/types
- [ ] Implement factory methods and utilities
- [ ] Setup npm package structure (@taprsvp/agent)
- [ ] Update tap-wasm/pkg/CLAUDE.md

### Phase 5: DID Resolution & Integration

- [ ] Write tests for built-in DID:key resolution
- [ ] Write tests for pluggable DID resolver interface
- [ ] Write tests for custom resolver integration
- [ ] Write tests for resolution error handling
- [ ] Implement built-in DID:key resolver
- [ ] Implement pluggable resolver interface
- [ ] Implement JavaScript resolver delegation
- [ ] Add resolver configuration options

### Phase 6: Interoperability Testing

- [ ] Write tests for Veramo message format compatibility
- [ ] Write tests for TAP -> Veramo message unpacking
- [ ] Write tests for Veramo -> TAP message unpacking
- [ ] Write cross-implementation test suite
- [ ] Verify message format compatibility
- [ ] Test with real Veramo instances
- [ ] Document any compatibility issues
- [ ] Create interoperability test fixtures

### Phase 7: Bundle Optimization

- [ ] Write tests for bundle size verification
- [ ] Write performance benchmarks for pack/unpack
- [ ] Write tests for tree-shaking effectiveness
- [ ] Implement wee_alloc for smaller WASM
- [ ] Remove unused dependencies
- [ ] Optimize TypeScript bundle
- [ ] Verify < 500KB WASM target
- [ ] Verify < 50KB gzipped TypeScript

### Phase 8: Documentation & Release

- [ ] Write API documentation
- [ ] Write getting started guide
- [ ] Write migration guide from v1
- [ ] Write example applications
- [ ] Create API reference docs
- [ ] Publish to npm as @taprsvp/agent
- [ ] Update main README
- [ ] Create release notes
