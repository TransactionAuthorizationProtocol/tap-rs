# TAP-RS Implementation Tasks - Updated TAIP Specifications

This document tracks the implementation of updated TAIP specifications following the submodule update to align with the latest TAP protocol changes.

## Overview

The TAIPs submodule has been updated with significant enhancements including:
- Schema.org Organization attributes for Agents and Parties  
- AuthorizationRequired message moved to TAIP-4
- RFC 8905 PayTo URI support for settlement addresses
- Fallback settlement addresses for Payment messages
- Schema.org Product attributes for invoice line items

## Implementation Tasks (TDD Approach)

### Phase 1: Agent and Party Enhancements

- [x] **Write failing tests for Agent schema.org Organization fields**
  - [x] Test Agent with `name` field serialization/deserialization
  - [x] Test Agent with `url` field serialization/deserialization  
  - [x] Test Agent with `logo` field serialization/deserialization
  - [x] Test Agent with `description` field serialization/deserialization
  - [x] Test Agent with `email` field serialization/deserialization
  - [x] Test Agent with `telephone` field serialization/deserialization
  - [x] Test Agent with `serviceUrl` field (DIDComm endpoint fallback)
  - [x] Test Agent with multiple organization fields combined
  - [x] Test Agent JSON-LD compliance with new fields

- [x] **Implement Agent schema.org Organization fields**
  - [x] Add accessor methods for `name` field to Agent struct
  - [x] Add accessor methods for `url` field to Agent struct
  - [x] Add accessor methods for `logo` field to Agent struct
  - [x] Add accessor methods for `description` field to Agent struct
  - [x] Add accessor methods for `email` field to Agent struct
  - [x] Add accessor methods for `telephone` field to Agent struct
  - [x] Add accessor methods for `serviceUrl` field to Agent struct
  - [x] Add builder methods for new fields
  - [x] Ensure all tests pass

- [x] **Write failing tests for Party schema.org Organization fields**
  - [x] Test Party with `name` field serialization/deserialization
  - [x] Test Party with `url` field serialization/deserialization
  - [x] Test Party with `logo` field serialization/deserialization
  - [x] Test Party with `description` field serialization/deserialization
  - [x] Test Party with `email` field serialization/deserialization
  - [x] Test Party with `telephone` field serialization/deserialization
  - [x] Test Party with multiple organization fields combined
  - [x] Test Party JSON-LD compliance with new fields
  - [x] Test Party with IVMS101 and schema.org fields coexistence

- [x] **Implement Party schema.org Organization fields**
  - [x] Add accessor methods for `name` field to Party struct
  - [x] Add accessor methods for `url` field to Party struct
  - [x] Add accessor methods for `logo` field to Party struct
  - [x] Add accessor methods for `description` field to Party struct
  - [x] Add accessor methods for `email` field to Party struct
  - [x] Add accessor methods for `telephone` field to Party struct
  - [x] Add builder methods for new fields
  - [x] Add `with_metadata_field` builder method
  - [x] Ensure all tests pass

### Phase 2: AuthorizationRequired Message Implementation

- [x] **Write failing tests for AuthorizationRequired message**
  - [x] Test AuthorizationRequired message creation and structure
  - [x] Test field serialization/deserialization (authorizationUrl, expires, from)
  - [x] Test optional `from` field with valid party types
  - [x] Test validation for required fields
  - [x] Test validation for invalid `from` values
  - [x] Test ISO 8601 timestamp format validation
  - [x] Test AuthorizationRequired JSON compliance with TAIP-4
  - [x] Test builder pattern and metadata support

- [x] **Implement AuthorizationRequired message**
  - [x] Update existing `AuthorizationRequired` struct in `connection.rs`
  - [x] Change `url` field to `authorizationUrl` per TAIP-4
  - [x] Make `expires` field required per TAIP-4
  - [x] Add optional `from` field for party type
  - [x] Update validation logic for new requirements
  - [x] Update constructors and builder methods
  - [x] Ensure all tests pass

### Phase 3: Settlement Address Enhancements

- [x] **Write failing tests for PayTo URI support**
  - [x] Test PayTo URI validation and parsing
  - [x] Test settlement address union type (CAIP-10 | PayTo URI)
  - [x] Test PayTo URI examples from RFC 8905 (IBAN, ACH, BIC, UPI)
  - [x] Test invalid PayTo URI rejection

- [x] **Implement PayTo URI support**
  - [x] Create `settlement_address.rs` module in `tap-msg/src/`
  - [x] Define `PayToURI` type with validation
  - [x] Define `SettlementAddress` enum (CAIP10 | PayToURI)
  - [x] Implement serialization/deserialization for SettlementAddress
  - [x] Add PayTo URI validation regex
  - [x] Ensure all tests pass

- [x] **Write failing tests for fallback settlement addresses**
  - [x] Test Payment message with `fallbackSettlementAddresses` array
  - [x] Test mixed CAIP-10 and PayTo URI addresses in fallback array
  - [x] Test optional fallback field serialization

- [x] **Implement fallback settlement addresses in Payment messages**
  - [x] Add optional `fallback_settlement_addresses: Option<Vec<SettlementAddress>>` to Payment
  - [x] Update Payment builder methods
  - [x] Update Payment serialization/deserialization
  - [x] Ensure all tests pass

### Phase 4: Invoice Product Attributes

- [x] **Write failing tests for Product attributes in invoice line items**
  - [x] Test LineItem with `name` field (schema.org/Product)
  - [x] Test LineItem with `image` field (schema.org/Product)  
  - [x] Test LineItem with `url` field (schema.org/Product)
  - [x] Test LineItem with multiple product fields combined

- [x] **Implement Product attributes in invoice line items**
  - [x] Add optional `name: Option<String>` field to LineItem
  - [x] Add optional `image: Option<String>` field to LineItem
  - [x] Add optional `url: Option<String>` field to LineItem
  - [x] Add builder methods for new fields
  - [x] Ensure all tests pass

### Phase 5: Integration and Cleanup

- [x] **Update message exports and integration**
  - [x] Add AuthorizationRequired to TapMessageEnum (already existed)
  - [x] Update message mod.rs exports (settlement_address module added)
  - [ ] Update message factory methods
  - [ ] Update message validation

- [ ] **Update MCP tools integration**
  - [ ] Review MCP tools that may need AuthorizationRequired support
  - [ ] Update transaction tools for new settlement address types
  - [ ] Test MCP integration with new message types

- [x] **Documentation and examples**
  - [x] Update example code for new fields (invoice examples updated)
  - [ ] Add AuthorizationRequired usage examples
  - [x] Add PayTo URI usage examples (in tests)
  - [x] Update CHANGELOG.md

- [x] **Final validation**
  - [x] Run full test suite: `cargo test`
  - [x] Run clippy: `cargo clippy`
  - [x] Run format check: `cargo fmt --check`
  - [ ] Validate against TAIP test vectors
  - [ ] Performance test new serialization paths

## Test-Driven Development Notes

1. **Write tests first** - Each implementation task should start with failing tests
2. **Red-Green-Refactor** - Ensure tests fail, implement minimum code to pass, then refactor
3. **Test edge cases** - Include validation tests for invalid inputs
4. **JSON compliance** - Ensure all new fields serialize correctly for TAIP compliance
5. **Backward compatibility** - All new fields should be optional to maintain compatibility

## Success Criteria

- [x] All tests pass
- [x] No clippy warnings
- [x] Code is properly formatted
- [ ] TAIP test vectors validate successfully
- [ ] MCP integration works with new message types
- [x] Documentation is updated and examples work