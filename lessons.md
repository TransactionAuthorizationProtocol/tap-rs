# TAP-RS Implementation: Lessons Learned

This document tracks key learnings, decisions, and insights discovered during the development of the TAP-RS implementation.

## Project Setup
- Following implementation order: tap-msg, tap-agent, caip, tap-node, tap-server, tap-ts
- Using test-driven development approach
- Using didcomm v2 library from crates.io
- Supporting did:key and did:pkh methods by default with did:web only available in non-WASM environments

## Technical Decisions
- WASM compatibility is ensured throughout all core libraries with appropriate feature flags
- Libraries are structured to avoid dependencies that don't work in WASM (e.g., tokio's full feature set)
- Cryptographic libraries are aligned with those used by the didcomm v2 library
- Message structures include flexible metadata fields to support extensions and custom data

## Implementation Insights
- TAP message validation is critical to ensure all required fields are present and properly formatted
- Implementing the `Validate` trait for message structures provides a clean interface for validation
- DIDComm integration requires careful handling of message types and formats
- Fuzzing is essential for identifying edge cases in message parsing and validation
- WebAssembly compatibility requires careful management of dependencies and feature flags

## Message Structure Design
- Base message structure with common fields shared across all TAP message types
- Specific message body types for different TAP operations (transactions, identity, travel rule)
- Use of enums for type-safe message handling
- Flexible attachments system for including additional data

## WASM Compatibility Strategy
- Feature flags are used to conditionally include or exclude functionality based on compilation target
- Network-dependent features like did:web resolution are only enabled in non-WASM environments
- Core cryptographic operations use libraries that have WASM support
- Dependencies with WASM issues (like full Tokio runtime) are properly managed with conditional compilation

## Testing Approach
- Unit tests for individual message types and validation logic
- Integration tests for round-trip message conversion (TAP → DIDComm → TAP)
- Fuzz testing for robustness against malformed inputs
- Test-driven development ensures functionality works as expected

## Performance Considerations
- Target performance: thousands of messages per second (long-term goal)
- Message serialization/deserialization is a potential bottleneck
- DIDComm encryption/decryption operations will likely impact throughput

## Challenges and Solutions
- Balancing strict validation with extensibility
- Implementing proper error handling with descriptive messages
- Supporting multiple DID methods with different resolution requirements
- Ensuring WASM compatibility while maintaining full functionality
- Managing dependencies that aren't WASM-friendly (e.g., Tokio's full feature set, networking libraries)
