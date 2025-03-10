# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Implementation of `SecretsResolver` in tap-wasm for WASM bindings
- Key management functionality in the Agent struct for storing and retrieving keys
- Message signing and verification using the DIDComm v2 library
- Support for Ed25519 keys in JsonWebKey2020 format
- New utility functions for conversion between Uint8Array and Vec<u8>
- README.md for tap-wasm describing WebAssembly bindings and usage

### Changed
- Updated Agent struct in tap-wasm to use the DIDComm SecretsResolver trait
- Improved Message handling to better integrate with DIDComm messaging
- Updated code to follow Rust best practices (using clippy and rustfmt)
- Enhanced README files with more documentation and examples
- Fixed type implementations for MessageType enum
- Added proper Display trait implementation for MessageType instead of ToString

### Security
- Implemented proper key management with SecretMaterial::JWK format
- Added methods for secure message signing and verification

## [0.1.0] - 2025-03-01

### Added
- Initial release of TAP-RS
- Core message processing with DIDComm integration
- Agent functionality and identity management
- Chain Agnostic Identifier Standards implementation
- Multiple DID method support (did:key, did:web, did:pkh)
- WASM bindings and TypeScript wrappers
