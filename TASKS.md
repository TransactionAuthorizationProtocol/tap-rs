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

- [ ] **Write failing tests for AuthorizationRequired message**
  - [ ] Test AuthorizationRequired message creation and structure
  - [ ] Test requirements array serialization/deserialization
  - [ ] Test optional presentationDefinition field
  - [ ] Test optional challengeNonce field
  - [ ] Test optional validUntil field
  - [ ] Test optional fallbackInstructions field
  - [ ] Test AuthorizationRequired DIDComm message type constant
  - [ ] Test AuthorizationRequired JSON schema compliance

- [ ] **Implement AuthorizationRequired message**
  - [ ] Create `authorization_required.rs` module in `tap-msg/src/message/`
  - [ ] Define `AuthorizationRequired` struct with required fields
  - [ ] Define `Requirement` struct for requirements array
  - [ ] Implement TapMessage trait for AuthorizationRequired
  - [ ] Add message type constant for DIDComm
  - [ ] Add to message enum and mod.rs exports
  - [ ] Implement validation logic
  - [ ] Ensure all tests pass

### Phase 3: Settlement Address Enhancements

- [ ] **Write failing tests for PayTo URI support**
  - [ ] Test PayTo URI validation and parsing
  - [ ] Test settlement address union type (CAIP-10 | PayTo URI)
  - [ ] Test PayTo URI examples from RFC 8905 (IBAN, ACH, BIC, UPI)
  - [ ] Test invalid PayTo URI rejection

- [ ] **Implement PayTo URI support**
  - [ ] Create `settlement_address.rs` module in `tap-msg/src/`
  - [ ] Define `PayToURI` type with validation
  - [ ] Define `SettlementAddress` enum (CAIP10 | PayToURI)
  - [ ] Implement serialization/deserialization for SettlementAddress
  - [ ] Add PayTo URI validation regex
  - [ ] Ensure all tests pass

- [ ] **Write failing tests for fallback settlement addresses**
  - [ ] Test Payment message with `fallbackSettlementAddresses` array
  - [ ] Test mixed CAIP-10 and PayTo URI addresses in fallback array
  - [ ] Test optional fallback field serialization

- [ ] **Implement fallback settlement addresses in Payment messages**
  - [ ] Add optional `fallback_settlement_addresses: Option<Vec<SettlementAddress>>` to Payment
  - [ ] Update Payment builder methods
  - [ ] Update Payment serialization/deserialization
  - [ ] Ensure all tests pass

### Phase 4: Invoice Product Attributes

- [ ] **Write failing tests for Product attributes in invoice line items**
  - [ ] Test LineItem with `name` field (schema.org/Product)
  - [ ] Test LineItem with `image` field (schema.org/Product)  
  - [ ] Test LineItem with `url` field (schema.org/Product)
  - [ ] Test LineItem with multiple product fields combined

- [ ] **Implement Product attributes in invoice line items**
  - [ ] Add optional `name: Option<String>` field to LineItem
  - [ ] Add optional `image: Option<String>` field to LineItem
  - [ ] Add optional `url: Option<String>` field to LineItem
  - [ ] Add builder methods for new fields
  - [ ] Ensure all tests pass

### Phase 5: Integration and Cleanup

- [ ] **Update message exports and integration**
  - [ ] Add AuthorizationRequired to TapMessageEnum
  - [ ] Update message mod.rs exports
  - [ ] Update message factory methods
  - [ ] Update message validation

- [ ] **Update MCP tools integration**
  - [ ] Review MCP tools that may need AuthorizationRequired support
  - [ ] Update transaction tools for new settlement address types
  - [ ] Test MCP integration with new message types

- [ ] **Documentation and examples**
  - [ ] Update example code for new fields
  - [ ] Add AuthorizationRequired usage examples
  - [ ] Add PayTo URI usage examples
  - [ ] Update CHANGELOG.md

- [ ] **Final validation**
  - [ ] Run full test suite: `cargo test`
  - [ ] Run clippy: `cargo clippy`
  - [ ] Run format check: `cargo fmt --check`
  - [ ] Validate against TAIP test vectors
  - [ ] Performance test new serialization paths

## Test-Driven Development Notes

1. **Write tests first** - Each implementation task should start with failing tests
2. **Red-Green-Refactor** - Ensure tests fail, implement minimum code to pass, then refactor
3. **Test edge cases** - Include validation tests for invalid inputs
4. **JSON compliance** - Ensure all new fields serialize correctly for TAIP compliance
5. **Backward compatibility** - All new fields should be optional to maintain compatibility

## Success Criteria

- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code is properly formatted
- [ ] TAIP test vectors validate successfully
- [ ] MCP integration works with new message types
- [ ] Documentation is updated and examples work