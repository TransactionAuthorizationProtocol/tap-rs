# Changelog

All notable changes to @taprsvp/agent will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2024-12-19

### Added
- Initial release of @taprsvp/agent TypeScript SDK
- Full DIDComm v2 support with JWS message format
- Support for all TAP message types (Transfer, Payment, Authorize, Reject, Settle, etc.)
- Multiple key type support (Ed25519, P-256, secp256k1)
- Browser and Node.js compatibility
- WASM-powered cryptography for high performance
- Pluggable DID resolver interface
- Message threading support
- Comprehensive TypeScript type definitions
- Zero runtime dependencies (only @taprsvp/types for types)
- Bundled WASM module for easy npm installation
- Private key import/export for flexible key management
- Full Veramo interoperability verified with 15+ integration tests

### Performance
- TypeScript bundle: 3.72KB gzipped (93% under target)
- WASM module: 272KB gzipped (46% under target)
- Message operations: < 10ms typical latency
- Key generation: < 5ms typical latency

### Documentation
- Comprehensive README with API reference
- Getting started guide with examples
- Browser and Node.js example applications
- Full API documentation
- TypeScript type documentation

### Testing
- 117 passing tests
- Real WASM interoperability tests (no mocks)
- Veramo compatibility tests
- Performance benchmarks
- Cross-key-type testing

### Security
- Private keys never leave WASM module
- Cryptographically secure random generation
- Standard algorithms (Ed25519, P-256, secp256k1)
- Minimal attack surface

## [0.1.0-alpha] - 2024-12-01

### Added
- Initial alpha release for testing
- Basic message packing/unpacking
- Ed25519 key support only
- Limited browser support

---

For more information, see the [README](README.md) and [documentation](docs/).
