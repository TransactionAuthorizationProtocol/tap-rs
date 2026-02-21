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

## External Decision Executable for tap-http
[PRD](prds/external-decision.md)

### Phase 1: Decision Log Storage (tap-node)

- [x] Write tests for decision_log insert, update status, list pending, and expire operations
- [x] Create migration `008_create_decision_log.sql` with table, indexes, and status constraints
- [x] Add `DecisionLogEntry` model to `models.rs`
- [x] Implement `insert_decision()` in `db.rs`
- [x] Implement `update_decision_status()` in `db.rs`
- [x] Implement `list_pending_decisions()` in `db.rs`
- [x] Implement `expire_decisions_for_transaction()` in `db.rs`

### Phase 2: Decision Expiration Handler (tap-node)

- [x] Write tests for automatic expiration when transactions reach terminal states
- [x] Implement `DecisionExpirationHandler` as an `EventSubscriber` that listens for `TransactionStateChanged` to terminal states and expires pending decisions

### Phase 3: Decision MCP Tools (tap-mcp)

- [x] Write tests for `tap_list_pending_decisions` tool
- [x] Write tests for `tap_resolve_decision` tool
- [x] Implement `tap_list_pending_decisions` tool in `tools/decision_tools.rs`
- [x] Implement `tap_resolve_decision` tool (marks resolved + executes action via TapNode)
- [x] Register both tools in `ToolRegistry`

### Phase 4: JSON-RPC Protocol Types (tap-http)

- [x] Write tests for serialization/deserialization of decision protocol messages (tap/decision, tap/event, tap/initialize)
- [x] Define protocol message types for stdin/stdout communication (decision requests, event notifications, initialization handshake)

### Phase 5: External Decision Manager (tap-http)

- [x] Write tests for `ExternalDecisionManager` implementing `DecisionHandler` (writes to decision_log, sends via stdin)
- [x] Write tests for process lifecycle (spawn, detect exit, restart with backoff)
- [x] Write tests for stdout reader (parse JSON-RPC tool calls, route to ToolRegistry)
- [x] Write tests for decision replay on process reconnect
- [x] Implement `ExternalDecisionManager` struct with child process management
- [x] Implement `DecisionHandler` trait — insert into decision_log and send over stdin
- [x] Implement `EventSubscriber` trait — forward events when in "all" mode
- [x] Implement stdout reader task — parse JSON-RPC requests, dispatch to `ToolRegistry`
- [x] Implement stdin writer — send decisions, events, and initialization messages
- [x] Implement process health monitoring and restart with exponential backoff
- [x] Implement decision replay on reconnect (query pending/delivered, send in order)
- [x] Implement graceful shutdown (EOF on stdin, SIGTERM, SIGKILL timeout)

### Phase 6: tap-http Integration

- [x] Write tests for CLI flag parsing (--decision-exec, --decision-exec-args, --decision-subscribe)
- [x] Add CLI flags and environment variables to `main.rs`
- [x] Wire `ExternalDecisionManager` into `NodeConfig.decision_mode` when --decision-exec is set
- [x] Subscribe `ExternalDecisionManager` to event bus
- [x] Disable `auto_act` when external decision executable is configured
- [x] Forward child process stderr to tap-http log output

### Phase 7: Integration Testing

- [x] Create a mock external executable (auto-approve script) for testing
- [x] Write integration test: decision flow end-to-end (receive transfer → decision → authorize)
- [x] Write integration test: process crash and catch-up (kill process, accumulate decisions, restart, verify replay)
- [x] Write integration test: decision expiration (decision pending → transaction rejected → decision expired)
- [x] Write integration test: external process uses tool calls to act (tap_authorize via stdout)

### Phase 8: CI Validation

- [x] Run cargo fmt, clippy, and tests with CI flags
- [x] Fix any warnings or errors
