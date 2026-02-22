# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-02-22

### Added

#### New Crate: tap-cli
- Full-featured command-line interface for all TAP agent operations
- Agent management (`agent create`, `agent list`)
- Transaction creation for all message types (`transfer`, `payment`, `connect`, `escrow`, `capture`)
- Transaction lifecycle actions (`authorize`, `reject`, `cancel`, `settle`, `revert`)
- Customer management with IVMS101 data generation (`customer create`, `customer ivms101`)
- Message delivery tracking and received message inspection
- DID operations (`did generate`, `did lookup`, `did keys`)
- Communication commands (`comm ping`, `comm message`)
- JSON and text output formats with `--format` flag

#### Transaction State Machine (tap-node)
- Formal finite state machine (`TransactionFsm`) for TAP transaction lifecycle
- 8 states: Received, PolicyRequired, PartiallyAuthorized, ReadyToSettle, Settled, Rejected, Cancelled, Reverted
- 3 explicit decision points: AuthorizationRequired, PolicySatisfactionRequired, SettlementRequired
- Configurable decision modes: AutoApprove, EventBus, Custom handler

#### External Decision Support (tap-http, tap-node, tap-mcp)
- `--decision-mode poll`: decisions logged to `decision_log` SQLite table for external polling
- `--decision-exec`: spawn a long-running child process communicating via JSON-RPC 2.0 over stdin/stdout
- `DecisionLogHandler` and `DecisionStateHandler` for durable decision management
- Process lifecycle management with health monitoring, restart with backoff, and graceful shutdown
- Decision replay on process reconnect for crash recovery
- Auto-resolve: action tools (`tap_authorize`, `tap_reject`, `tap_settle`, `tap_cancel`, `tap_revert`) automatically resolve matching pending decisions
- New MCP tools: `tap_list_pending_decisions`, `tap_resolve_decision`

#### did:web Hosting (tap-http)
- Optional `/.well-known/did.json` endpoint for serving did:web DID documents
- Derives DID from HTTP Host header with RFC 1035 domain validation
- Auto-creates agents with Ed25519 keys and DIDCommMessaging service endpoint
- Enabled via `--enable-web-did` flag or `TAP_ENABLE_WEB_DID` env var

#### WASM Agent v2 (tap-wasm, tap-ts)
- Complete rewrite of WASM bindings with browser-first design
- Real Ed25519 cryptographic key generation replacing UUID-based DID generation
- End-to-end message signing and verification working in browser
- TypeScript SDK (`@taprsvp/agent`) with full DIDComm v2 support
- Pluggable DID resolver interface for JavaScript delegation
- Multiple key types: Ed25519, P-256, secp256k1
- Optimized bundle: 272KB WASM gzipped, 3.72KB TypeScript gzipped
- Verified Veramo interoperability with 15+ integration tests

#### Asset Exchange & Quote Messages (TAIP-18)
- New `Exchange` message type for initiating asset exchanges between parties
- New `Quote` message type for responding to exchange requests with pricing
- Full validation, builders, CLI subcommands, and MCP tools for both message types

#### Transfer & Payment Enhancements (TAIP-3, TAIP-14)
- `transactionValue` and `expiry` fields on Transfer messages (TAIP-3)
- Flexible asset pricing in Payment via `SupportedAsset` enum with Simple and Priced variants (TAIP-14)
- `expiry`, `invoice`, and `fallbackSettlementAddresses` exposed in Payment MCP tools and CLI

#### Connect Message Restructure (TAIP-15)
- `requester`, `agents`, `agreement`, and `expiry` fields on Connect messages
- Expanded `TransactionLimits` with per-day/week/month/year limits
- Expanded `ConnectionConstraints` with `allowedBeneficiaries`, `allowedSettlementAddresses`, `allowedAssets`

#### Agent Management CLI Commands (TAIP-5, TAIP-7)
- `agent-mgmt add-agents` subcommand for adding agents to transactions
- `agent-mgmt remove-agent` and `agent-mgmt replace-agent` subcommands
- `agent-mgmt update-policies` subcommand (TAIP-7)

#### Decision Management CLI Commands (tap-cli)
- `decision list` and `decision resolve` subcommands matching tap-mcp's decision tools
- Auto-resolve on all action commands (`authorize`, `reject`, `cancel`, `settle`, `revert`)
- Detailed `--help` text with decision type references and auto-resolve mapping

#### Docker Support (tap-http)
- Multi-stage Dockerfile for containerized deployment
- docker-compose.yml with persistent volume at `/data/tap`
- Single volume for keys, databases, and logs

### Changed
- `tap-agent` default features now include all three crypto backends (`crypto-ed25519`, `crypto-p256`, `crypto-secp256k1`)
- `Complete` message type removed per updated TAIP specifications
- Improved installation documentation across all README files with explicit `cargo install` and `cargo add` instructions

### Security
- Replace insecure XOR-based key wrapping with AES-KW (RFC 3394)
- Implement Concat KDF (NIST SP 800-56A) for ECDH key derivation
- Fix `encrypt_to_jwk` to use real ECDH-ES+A256KW encryption
- Fix `verify_jws` to perform actual cryptographic signature verification
- Add bounds checking to prevent panics on malformed DID and PayTo URI input
- JWE messages encrypted with old XOR method are no longer decryptable (intentional — the old method provided no security)

### Breaking Changes
- `Complete` message type removed
- JWE encryption format changed (AES-KW replaces XOR key wrapping)
- `AuthorizationRequired` field `url` renamed to `authorizationUrl` (from 0.5.0)
- Connect message restructured with new `requester`, `agents`, `agreement`, `expiry` fields (TAIP-15)
- Transfer message gains `transactionValue` and `expiry` fields (TAIP-3)

## [0.5.0] - 2025-08-14

### Added

#### Composable Escrow Support (TAIP-17)
- New `Escrow` message for holding assets on behalf of parties
- New `Capture` message for releasing escrowed funds
- Support for both cryptocurrency assets and fiat currencies in escrows
- Automatic expiry handling for escrows
- Support for payment guarantees, asset swaps, and conditional payments
- Multiple agent roles including dedicated EscrowAgent role
- Full validation ensuring exactly one EscrowAgent per escrow

#### Settlement Address Enhancements
- PayTo URI support (RFC 8905) for traditional payment systems (IBAN, ACH, BIC, UPI)
- `SettlementAddress` enum supporting both CAIP-10 blockchain addresses and PayTo URIs
- `fallbackSettlementAddresses` field in Payment messages for flexible payment options
- Full validation and serialization for PayTo URIs

#### Invoice Product Attributes
- Schema.org/Product attributes to LineItem (name, image, url)
- LineItem builder pattern for easier construction
- Support for product metadata in invoice line items

#### Agent and Party Enhancements  
- Schema.org Organization fields for Agent and Party structures
- Added fields: name, url, logo, description, email, telephone, serviceUrl
- Builder methods and accessor functions for all new fields
- Backward compatible with existing IVMS101 data

### Changed
- AuthorizationRequired message updated to match TAIP-4 specification
  - Field `url` renamed to `authorizationUrl`
  - Field `expires` now required
  - Added optional `from` field

## [0.4.0] - 2025-06-17

### Added

#### Travel Rule Compliance & IVMS101 Support
- New `tap-ivms101` crate implementing IVMS 101.2023 (interVASP Messaging Standard)
- Complete Natural Person and Legal Person data structures with validation
- Automatic IVMS101 attachment to Transfer messages based on configurable policies
- Amount threshold checking for Travel Rule compliance (e.g., $1000 USD/EUR)
- Policy-based compliance requests via DIDComm Presentation messages
- Builder pattern for easy IVMS101 construction
- Full FATF Recommendation 16 (Travel Rule) implementation

#### Customer Management System
- Automatic extraction of customer data from TAP messages
- Schema.org JSON-LD profile storage for customer data
- IVMS101 data generation from customer profiles
- Per-agent isolated storage for data privacy
- Customer relationship tracking for TAIP-9 compliance
- New database tables: `customers`, `customer_identifiers`, `customer_relationships`

#### Privacy-Preserving Features
- PII (Personally Identifiable Information) hashing functionality
- Privacy-preserving data exchange by default for MCP transfers
- Automatic PII hashing for natural persons (name hash instead of raw data)
- Legal Entity Identifier (LEI) support for organizations
- Selective data disclosure based on compliance requirements

#### Enhanced MCP Tools
- `tap_create_customer` - Create new customer profiles
- `tap_list_customers` - List customers managed by an agent
- `tap_get_customer_details` - Retrieve customer profiles and IVMS101 data
- `tap_generate_ivms101` - Generate compliant IVMS101 data for customers
- `tap_update_customer_profile` - Update customer Schema.org profiles
- `tap_update_customer_from_ivms101` - Import customer data from IVMS101

#### Database & Storage Improvements
- SQLite storage support with full database schema
- Enhanced storage API with customer management
- Improved transaction and message handling
- Better error handling and logging

#### Documentation
- `tap-node/TRAVEL-RULE.md` - Complete Travel Rule implementation guide
- `tap-node/CUSTOMER-MANAGEMENT.md` - Customer data management documentation
- `tap-ivms101/README.md` - IVMS101 crate documentation
- Updated examples and test vectors

### Changed

#### Core Improvements
- Refactored DIDComm implementation (removed external didcomm crate dependency)
- Improved key management with new `AgentKeyManager`
- Better handling of keys from CLI
- Enhanced WASM and TypeScript support
- Improved message packing/unpacking with native crypto implementation

#### Transaction Handling
- Fixed customer record handling in tool calls
- Fixed handling of `transaction_id` in initiator messages
- Improved transaction ID serialization for Connect, Transfer, and Payment messages
- Better support for message threading and correlation

#### Testing & CI
- Added GitHub Actions CI workflow
- Comprehensive test coverage for Travel Rule workflows
- Customer extraction tests
- MCP tool tests
- Updated test vectors

### Fixed

- Fixed issues with agent tools
- Corrected transaction ID handling in various message types
- Resolved customer data extraction edge cases
- Fixed WASM compatibility issues
- Removed deprecated `wee_alloc` dependency

### Security

- Customer data is stored per-agent in isolated databases
- No cross-agent data leakage
- Compliance data only shared when required by regulation
- Full audit trail of data sharing
- Privacy-first design with PII hashing by default

## [0.3.0] - Previous Release

- Initial public release of TAP implementation
- Core TAP message types (Transfer, Authorize, Settle, etc.)
- Basic agent functionality
- DIDComm messaging support
- Initial MCP server implementation