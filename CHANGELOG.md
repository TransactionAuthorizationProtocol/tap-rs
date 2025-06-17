# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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