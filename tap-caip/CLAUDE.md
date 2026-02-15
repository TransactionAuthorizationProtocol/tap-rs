# tap-caip Crate

Chain Agnostic Identifier Protocol (CAIP) implementation providing standardized identifiers for blockchain assets, accounts, and chains.

## Purpose

The `tap-caip` crate provides:
- CAIP standard identifier parsing and validation
- Chain ID, Account ID, and Asset ID implementations
- Serialization support for CAIP identifiers
- Validation utilities for blockchain identifiers
- WASM compatibility for browser usage

## Key Components

- `chain_id.rs` - CAIP-2 Chain ID implementation
- `account_id.rs` - CAIP-10 Account ID implementation  
- `asset_id.rs` - CAIP-19 Asset ID implementation
- `validation.rs` - Validation utilities and patterns
- `error.rs` - CAIP-specific error types

## Build Commands

```bash
# Build the crate
cargo build -p tap-caip

# Run tests
cargo test -p tap-caip

# Run specific test
cargo test -p tap-caip test_name

# Run benchmarks
cargo bench -p tap-caip

# Run CAIP benchmark
cargo bench --bench caip_benchmark

# Build with WASM support
cargo build -p tap-caip --features wasm

# Run property-based tests
cargo test -p tap-caip --features arbitrary
```

## Development Guidelines

### CAIP Compliance
- Follow official CAIP specifications exactly
- Validate all identifiers according to CAIP standards
- Support all standard blockchain namespaces
- Include comprehensive error messaging
- Maintain backward compatibility with CAIP updates

### Validation Patterns
- Use regex patterns for identifier structure validation
- Cache compiled regex patterns for performance
- Provide detailed error messages for invalid identifiers
- Support both strict and lenient validation modes
- Include format normalization utilities

### Serialization
- Support both string and structured serialization
- Maintain roundtrip serialization consistency
- Include proper error handling for invalid formats
- Support serde integration for JSON/YAML/etc.
- Provide custom serialization for specific needs

### Testing
- Include comprehensive test vectors
- Test all supported blockchain namespaces
- Use property-based testing for edge cases
- Test serialization roundtrips
- Include fuzzing for security validation

## CAIP Standards Supported

### CAIP-2: Chain ID
```rust
use tap_caip::ChainId;

let chain_id = ChainId::from_str("eip155:1")?; // Ethereum mainnet
let namespace = chain_id.namespace(); // "eip155"
let reference = chain_id.reference(); // "1"
```

### CAIP-10: Account ID  
```rust
use tap_caip::AccountId;

let account = AccountId::from_str("eip155:1:0x1234...5678")?;
let chain_id = account.chain_id(); // ChainId("eip155:1")
let address = account.address(); // "0x1234...5678"
```

### CAIP-19: Asset ID
```rust
use tap_caip::AssetId;

let asset = AssetId::from_str("eip155:1/erc20:0x1234...5678")?;
let chain_id = asset.chain_id(); // ChainId("eip155:1")
let asset_namespace = asset.asset_namespace(); // "erc20"
let asset_reference = asset.asset_reference(); // "0x1234...5678"
```

## Supported Blockchain Namespaces

### Ethereum Ecosystem
- `eip155` - Ethereum and EVM chains
- ERC-20, ERC-721, ERC-1155 token support
- Layer 2 solutions (Polygon, Arbitrum, etc.)

### Bitcoin Ecosystem  
- `bip122` - Bitcoin and Bitcoin-based chains
- UTXO-based chain support
- Address format validation

### Cosmos Ecosystem
- `cosmos` - Cosmos Hub and IBC chains
- Bech32 address validation
- IBC token support

### Other Chains
- `polkadot` - Polkadot and Substrate chains
- `solana` - Solana blockchain
- `near` - NEAR Protocol
- `tezos` - Tezos blockchain

## Features

- `wasm` - WebAssembly support with JavaScript compatibility
- Property-based testing with `proptest`
- Fuzzing support with `arbitrary`

## Validation Features

### Identifier Structure
- Namespace validation against known standards
- Reference format validation per namespace
- Character set validation
- Length limit enforcement

### Checksum Validation
- Address checksum validation where applicable
- Chain-specific validation rules
- Format normalization

### Error Reporting
- Detailed error messages with context
- Position information for parsing errors
- Suggestions for common mistakes
- Chain-specific error types

## Examples

### Basic Usage
```rust
use tap_caip::{ChainId, AccountId, AssetId};

// Parse identifiers
let chain = ChainId::from_str("eip155:1")?;
let account = AccountId::from_str("eip155:1:0x742d35Cc6634C0532925a3b8D45D0c36F4A7C5Bc")?;
let asset = AssetId::from_str("eip155:1/erc20:0xA0b86a33E6441E4c5e25A8C7E8A95C9e6a5c59d1")?;

// Convert to strings
let chain_str = chain.to_string();
let account_str = account.to_string();
let asset_str = asset.to_string();
```

### Validation
```rust
use tap_caip::validation::validate_ethereum_address;

let is_valid = validate_ethereum_address("0x742d35Cc6634C0532925a3b8D45D0c36F4A7C5Bc");
assert!(is_valid);
```

### Serialization
```rust
use serde::{Serialize, Deserialize};
use tap_caip::AssetId;

#[derive(Serialize, Deserialize)]
struct Transaction {
    asset: AssetId,
    amount: String,
}

let tx = Transaction {
    asset: AssetId::from_str("eip155:1/erc20:0x...")?,
    amount: "100.0".to_string(),
};

let json = serde_json::to_string(&tx)?;
```

## Performance

The crate is optimized for performance:
- Compiled regex patterns cached with `once_cell`
- Zero-copy parsing where possible
- Efficient string operations
- Minimal allocations for validation

Benchmark performance:
```bash
cargo bench -p tap-caip --bench caip_benchmark
```

## WASM Compatibility

When built with the `wasm` feature:
- JavaScript-compatible APIs
- Browser-compatible random number generation
- WebAssembly-optimized builds
- TypeScript definition support

## Error Handling

Comprehensive error types for different validation failures:
- `InvalidFormat` - Malformed identifier structure
- `InvalidNamespace` - Unsupported or invalid namespace
- `InvalidReference` - Invalid reference for namespace
- `InvalidChecksum` - Checksum validation failure
- `UnsupportedChain` - Chain not supported

## Testing

The crate includes extensive testing:
- Unit tests for all identifier types
- Property-based testing with `proptest`
- Fuzzing tests with `cargo fuzz`
- Integration tests with real blockchain data
- Performance benchmarks

Run comprehensive tests:
```bash
cargo test -p tap-caip --all-features
```

## Standards Compliance

The implementation follows these CAIP standards:
- [CAIP-2](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md) - Chain ID Specification
- [CAIP-10](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-10.md) - Account ID Specification  
- [CAIP-19](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-19.md) - Asset ID Specification

All implementations are validated against official test vectors and maintained for compatibility with specification updates.