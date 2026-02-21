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
- [x] Write cross-implementation test suite with real Veramo DIDComm library
- [x] Verify message format compatibility between TAP and Veramo agents
- [x] Test with real Veramo instances (15 comprehensive integration tests)
- [x] Document compatibility - Full DIDComm v2 compatibility confirmed
- [x] Create interoperability test fixtures and real integration tests

### Phase 7: Bundle Optimization

- [x] Implement wee_alloc for smaller WASM
- [x] Remove unused dependencies
- [x] Optimize TypeScript bundle
- [x] Verify < 500KB WASM target (272KB gzipped ✅)
- [x] Verify < 50KB gzipped TypeScript (3.72KB gzipped ✅)

### Phase 8: Documentation & Release

- [x] Write API documentation
- [x] Write getting started guide
- [x] Write example applications
- [x] Create API reference docs
- [ ] Publish to npm as @taprsvp/agent
- [x] Update main README
- [x] Create release notes

## TAP CLI - Command-Line Interface for TAP Agent Operations
[PRD](prds/cli.md)

### Phase 1: Project Scaffold & Core Integration

- [x] Write tests for CLI argument parsing and global flags
- [x] Write tests for TapIntegration initialization from CLI args
- [x] Create tap-cli crate with Cargo.toml, add to workspace
- [x] Implement main.rs with global flags (--agent-did, --tap-root, --debug, --format)
- [x] Implement TapIntegration setup (reuse pattern from tap-mcp)
- [x] Implement output formatting layer (JSON and text modes)

### Phase 2: Agent Management Commands

- [x] Write tests for `agent create` and `agent list` commands
- [x] Implement `agent create` subcommand
- [x] Implement `agent list` subcommand

### Phase 3: Transaction Creation Commands

- [x] Write tests for `transfer` command
- [x] Write tests for `payment` command
- [x] Write tests for `connect` command
- [x] Write tests for `escrow` and `capture` commands
- [x] Implement `transfer` subcommand
- [x] Implement `payment` subcommand
- [x] Implement `connect` subcommand
- [x] Implement `escrow` subcommand
- [x] Implement `capture` subcommand

### Phase 4: Transaction Action Commands

- [x] Write tests for `authorize`, `reject`, `cancel` commands
- [x] Write tests for `settle` and `revert` commands
- [x] Implement `authorize` subcommand
- [x] Implement `reject` subcommand
- [x] Implement `cancel` subcommand
- [x] Implement `settle` subcommand
- [x] Implement `revert` subcommand

### Phase 5: Query Commands

- [x] Write tests for `transaction list` command
- [x] Write tests for `delivery list` command
- [x] Write tests for `received list/pending/view` commands
- [x] Implement `transaction list` subcommand
- [x] Implement `delivery list` subcommand
- [x] Implement `received list`, `received pending`, `received view` subcommands

### Phase 6: Customer Management Commands

- [x] Write tests for customer CRUD commands
- [x] Implement `customer list` subcommand
- [x] Implement `customer create` subcommand
- [x] Implement `customer details` subcommand
- [x] Implement `customer update` subcommand
- [x] Implement `customer ivms101` subcommand

### Phase 7: Communication & DID Commands

- [x] Write tests for `ping` and `message` commands
- [x] Implement `ping` subcommand
- [x] Implement `message` subcommand
- [x] Integrate existing DID commands from tap-agent-cli (`did generate`, `did lookup`, `did keys`)

### Phase 8: CI Validation & Polish

- [x] Run cargo fmt, clippy, and tests with CI flags
- [x] Fix any warnings or errors
