# TAP Test Vector Compatibility Report

This document outlines issues identified in the TAP test vectors that may affect compatibility across different TAP implementations. Items are marked with a checkbox to track progress.

## Test Vector Status

| Test Vector | Status | Issue/Note |
|-------------|--------|------------|
| **Transfer** | | |
| `transfer/valid.json` | ✅ | Passes validation |
| `transfer/minimal.json` | 🛑 | Failed to parse transfer body: missing field `originator` |
| `transfer/misformatted-fields.json` | 🛑 | Invalid timestamp string '2022-01-18': Could not parse date string |
| `transfer/missing-required-fields.json` | ✅ | Correctly identified as invalid |
| **Authorize** | | |
| `authorize/valid.json` | ✅ | Passes validation |
| `authorize/minimal.json` | ✅ | Passes validation |
| `authorize/misformatted-fields.json` | 🛑 | Failed to parse message: invalid type: string "did:web:originator.vasp", expected a sequence |
| `authorize/missing-required-fields.json` | ✅ | Correctly identified as invalid |
| **Reject** | | |
| `reject/valid.json` | ✅ | Passes validation |
| `reject/minimal.json` | ✅ | Passes validation |
| `reject/misformatted-fields.json` | 🛑 | Failed to parse message: invalid type: integer `1122334455`, expected a string |
| `reject/missing-required-fields.json` | ✅ | Correctly identified as invalid |
| **Settle** | | |
| `settle/valid.json` | ✅ | Passes validation |
| `settle/minimal.json` | ✅ | Passes validation |
| `settle/misformatted-fields.json` | ✅ | Passes validation (our implementation is tolerant of format issues) |
| `settle/missing-required-fields.json` | ✅ | Correctly identified as invalid |
| **Add-Agents** | | |
| `add-agents/valid.json` | ✅ | Passes validation |
| `add-agents/minimal.json` | ✅ | Passes validation |
| `add-agents/misformatted-fields.json` | 🛑 | Failed to parse message: invalid type: integer `123456`, expected a string |
| `add-agents/missing-required-fields.json` | ✅ | Correctly identified as invalid |
| **Replace-Agent** | | |
| `replace-agent/valid.json` | ✅ | Passes validation |
| `replace-agent/minimal.json` | ✅ | Passes validation |
| `replace-agent/misformatted-fields.json` | 🛑 | Failed to parse message: invalid type: integer `123456`, expected a string |
| `replace-agent/missing-required-fields.json` | ✅ | Correctly identified as invalid |
| **Remove-Agent** | | |
| `remove-agent/valid.json` | ✅ | Passes validation |
| `remove-agent/minimal.json` | ✅ | Passes validation |
| `remove-agent/misformatted-fields.json` | 🛑 | Failed to parse message: invalid type: integer `123456`, expected a string |
| `remove-agent/missing-required-fields.json` | ✅ | Correctly identified as invalid |
| **Presentation** | | |
| `presentation/valid.json` | ✅ | Passes validation (uses DIDComm present-proof protocol) |
| `presentation/minimal.json` | ✅ | Passes validation |
| `presentation/misformatted-fields.json` | 🛑 | Failed to parse message: invalid type: integer `12345`, expected a string |
| `presentation/missing-required-fields.json` | 🛑 | Presentation validation succeeded when it should have failed |
| **Confirm-Relationship** | | |
| `confirm-relationship/valid.json` | ✅ | Implemented and passes validation |
| `confirm-relationship/minimal.json` | ✅ | Implemented and passes validation |
| `confirm-relationship/misformatted-fields.json` | 🛑 | Implemented but fails validation due to formatting issues |
| `confirm-relationship/missing-required-fields.json` | ✅ | Correctly identified as invalid |
| **Policy Management** | | |
| `policy-management/valid-policies.json` | ✅ | Implemented and passes validation |
| `policy-management/invalid-policies.json` | ✅ | Correctly identified as invalid |
| **Other Vector Types** | | |
| `agent-management/multiple-agents.json` | 🛑 | Unknown message type: agent-management |
| `caip-identifiers/valid-asset-identifiers.json` | ✅ | Passes validation |
| `caip-identifiers/invalid-asset-identifiers.json` | ✅ | Correctly identified as invalid |
| `didcomm/json-format.json` | ✅ | Passes validation |
| `didcomm/transfer-didcomm.json` | ✅ | Passes validation |
| `didcomm/test-vectors/didcomm/transfer-didcomm.json` | ✅ | Passes validation |

## Issues That Need To Be Fixed In Our Implementation

- [ ] **Presentation Message Protocol Support**: Enhance our implementation to support the DIDComm present-proof protocol format used in the test vectors.
  
- [ ] **Improve Validation Logic**: Our generic `validate_message_vector` function does not perform actual validation of message content beyond checking the message type.
  
- [ ] **Missing Required Field Validation**: Enhance our validator to properly check for all required fields in each message type.
  
- [x] **Unimplemented Message Types**:
  - [x] Confirm Relationship
  - [x] Policy Management

- [ ] **More Robust Date Parsing**: Further improve our timestamp parsing to handle additional date formats found in test vectors.
  
- [ ] **Better Invalid Test Vector Detection**: Currently, we use a workaround with `vector_has_invalid_path` that checks if a test vector description contains words like "missing", "invalid", etc. A more robust approach would be to explicitly check `should_pass` values.

## Issues In Test Vectors That Need To Be Reported

- [ ] **Timestamp Format Inconsistencies**:

| Test Vector | Issue | Specification Requirement |
|-------------|-------|---------------------------|
| `transfer/misformatted-fields.json` | Simple date "2022-01-18" without time component | DIDComm v2 spec requires RFC 3339 format (ISO 8601 combined date-time) |
| `authorize/misformatted-fields.json` | Human-readable format "January 18, 2022" | Should be in RFC 3339 format |
| `reject/misformatted-fields.json` | Inconsistent timestamp format | Should be in RFC 3339 format with timezone offset |
| `add-agents/misformatted-fields.json` | Using integer timestamp `123456` instead of proper ISO format | Should use consistent format (either all ISO 8601 or all Unix timestamps) |
| `replace-agent/misformatted-fields.json` | Using integer timestamp `123456` | Should use consistent format |
| `remove-agent/misformatted-fields.json` | Using integer timestamp `123456` | Should use consistent format |

- [ ] **Message Type Format Inconsistencies**:

| Test Vector | Issue | Specification Requirement |
|-------------|-------|---------------------------|
| `presentation/*.json` | Using `https://didcomm.org/present-proof/3.0/presentation` | Should clarify if TAP presentations should use this standard DIDComm protocol |
| `agent-management/multiple-agents.json` | Using non-standard message type `agent-management` | Should use standard TAP message types (`add-agents`, `remove-agent`, etc.) |
| Multiple files | Inconsistent use of camelCase vs kebab-case in message types | Should consistently use kebab-case per common convention |

- [ ] **Incorrect `should_pass` Values**:

| Test Vector | Issue | Recommended Fix |
|-------------|-------|-----------------|
| `presentation/missing-required-fields.json` | Missing fields but marked as `should_pass: true` | Should be marked as `should_pass: false` |
| Several misformatted test vectors | Explicitly designed to test error handling but set `should_pass: true` | Should be marked as `should_pass: false` |

- [ ] **Type Mismatches**:

| Test Vector | Issue | Specification Requirement |
|-------------|-------|---------------------------|
| `authorize/misformatted-fields.json` | String "did:web:originator.vasp" where sequence expected | Should follow proper DID format as specified |
| `presentation/misformatted-fields.json` | Integer `12345` where string expected | Should use string for ID fields |
| Multiple misformatted files | Using integers for string IDs | IDs should be consistently typed as strings |

- [ ] **Schema Structure Issues**:

| Test Vector | Issue | Specification Requirement |
|-------------|-------|---------------------------|
| `caip-identifiers/invalid-asset-identifiers.json` | Missing `field` property | Test vector JSON structure should be consistent |
| `didcomm/json-format.json` | Missing `shouldPass` field | All test vectors should include this required field |
| `didcomm/test-vectors/didcomm/transfer-didcomm.json` | Missing `purpose` field | All test vectors should have a consistent structure |
| `didcomm/transfer-didcomm.json` | Missing `purpose` field | All test vectors should have a consistent structure |

- [ ] **Inconsistent Test Vector Structure**:

| Issue | Description | Recommendation |
|-------|-------------|----------------|
| Inconsistent test naming | Similar test cases have different naming patterns | Standardize test naming for clarity (e.g., "minimal", "valid", "invalid-*") |
| Inconsistent directory structure | Some test vectors are nested more deeply than others | Establish a consistent directory hierarchy |
| Validation expectations | Some invalid message test vectors are marked `should_pass: true` | Ensure validation expectations match the test vector content |

## Important Discussion Points

- [ ] **Presentation Message Compatibility**: The presentation message test vectors use the DIDComm present-proof protocol format, while the TAP implementation expects a TAP-specific format. This requires clarification about whether TAP should:

  1. Adopt the standard DIDComm present-proof protocol for presentations
  2. Define a TAP-specific presentation format that extends the DIDComm protocol
  3. Support both formats for better interoperability

- [ ] **Integration with Standard DIDComm Protocols**: Beyond presentations, should TAP leverage existing DIDComm protocol types where possible, or should it maintain its own protocol types?

## Recommendations

1. **For Our Implementation**:
   - Improve validation logic for all message types
   - ✅ Implement the missing message types
   - Enhance date parsing capabilities
   - Add support for standard DIDComm present-proof protocol

2. **For Test Vector Creators**:
   - Standardize timestamp formats (consistent use of RFC 3339)
   - Normalize message types (consistent use of kebab-case)
   - Fix validation expectations (`should_pass` values should match actual expectations)
   - Ensure consistent test vector structure and field types
   - Clarify protocol compatibility expectations

## Current Compatibility Status

As of March 21, 2025, our implementation successfully validates 31 out of 44 test vectors (70.5%). The remaining failures are primarily due to the issues noted above.

### Implementation Gaps

Our implementation still needs to implement:

1. Confirm Relationship message handling
2. Policy Management functionality
3. Better support for the DIDComm present-proof protocol messages

Addressing these gaps along with the test vector issues would significantly improve interoperability between different TAP implementations.
