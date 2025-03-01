# caip: Chain Agnostic Identifier Standard Implementation

The `caip` crate provides a Rust implementation of the Chain Agnostic Identifier Standards, including CAIP-2 (Chain ID), CAIP-10 (Account ID), and CAIP-19 (Asset ID). These standards enable consistent, chain-agnostic representation of blockchain identifiers across different protocols.

## Features

- CAIP-2 Chain ID parsing and validation (`namespace:reference`)
- CAIP-10 Account ID parsing and validation (`chainID:accountAddress`)
- CAIP-19 Asset ID parsing and validation (`chainID/assetNamespace:assetReference`)
- Serialization/deserialization via Serde
- Comprehensive validation rules for major blockchain networks
- WASM compatibility

## Usage

Add `caip` to your `Cargo.toml`:

```toml
[dependencies]
caip = "0.1.0"
```

### CAIP-2 Chain ID

```rust
use caip::chain_id::ChainId;

fn main() {
    // Create from string
    let ethereum_mainnet = ChainId::from_str("eip155:1").unwrap();
    println!("Chain ID: {}", ethereum_mainnet);  // "eip155:1"
    
    // Create using constructor
    let bitcoin_mainnet = ChainId::new("bip122", "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f").unwrap();
    println!("Namespace: {}", bitcoin_mainnet.namespace());  // "bip122"
    println!("Reference: {}", bitcoin_mainnet.reference());  // "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
    
    // Validation
    assert!(ChainId::from_str("eip155:1").is_ok());  // Valid Ethereum mainnet
    assert!(ChainId::from_str("eip155:").is_err());  // Invalid: missing reference
    assert!(ChainId::from_str(":1").is_err());       // Invalid: missing namespace
}
```

### CAIP-10 Account ID

```rust
use caip::{chain_id::ChainId, account_id::AccountId};

fn main() {
    // Create from string
    let eth_account = AccountId::from_str("eip155:1:0xab16a96d359ec26a11e2c2b3d8f8b8942d5bfcdb").unwrap();
    
    // Access chain ID and address components
    println!("Chain ID: {}", eth_account.chain_id());  // "eip155:1"
    println!("Address: {}", eth_account.address());    // "0xab16a96d359ec26a11e2c2b3d8f8b8942d5bfcdb"
    
    // Create from chain ID and address
    let chain_id = ChainId::from_str("eip155:1").unwrap();
    let account = AccountId::new(chain_id, "0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db").unwrap();
    
    // Validation with blockchain-specific rules
    assert!(AccountId::from_str("eip155:1:0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db").is_ok());  // Valid ETH address
    assert!(AccountId::from_str("eip155:1:0xinvalid").is_err());  // Invalid ETH address format
}
```

### CAIP-19 Asset ID

```rust
use caip::{chain_id::ChainId, asset_id::AssetId};

fn main() {
    // Create from string (Ethereum ERC-20 USDC token)
    let usdc = AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
    
    // Access components
    println!("Chain ID: {}", usdc.chain_id());           // "eip155:1"
    println!("Asset Namespace: {}", usdc.namespace());   // "erc20"
    println!("Asset Reference: {}", usdc.reference());   // "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
    
    // Create using constructor
    let chain_id = ChainId::from_str("eip155:1").unwrap();
    let dai = AssetId::new(chain_id, "erc20", "0x6b175474e89094c44da98b954eedeac495271d0f").unwrap();
    
    // Validation
    assert!(AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f").is_ok());  // Valid ERC-20
    assert!(AssetId::from_str("eip155:1/erc20:").is_err());  // Invalid: missing reference
}
```

## Supported Chain Namespaces

The library supports validation for the following blockchain namespaces:

- `eip155`: Ethereum and EVM-compatible chains
- `bip122`: Bitcoin
- `cosmos`: Cosmos Hub and related chains
- `polkadot`: Polkadot and Substrate-based chains
- `solana`: Solana
- `tezos`: Tezos
- `fil`: Filecoin
- `near`: NEAR Protocol

## Advanced Usage

### Serialization with Serde

```rust
use caip::chain_id::ChainId;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct BlockchainIdentifier {
    chain: ChainId,
    name: String,
}

fn main() {
    let identifier = BlockchainIdentifier {
        chain: ChainId::from_str("eip155:1").unwrap(),
        name: "Ethereum Mainnet".to_string(),
    };
    
    let json = serde_json::to_string(&identifier).unwrap();
    println!("JSON: {}", json);  // {"chain":"eip155:1","name":"Ethereum Mainnet"}
    
    let parsed: BlockchainIdentifier = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.chain.to_string(), "eip155:1");
}
```

### Custom Validation Rules

```rust
use caip::{chain_id::ChainId, account_id::AccountId, ValidationRegistry};

// Register a custom validator for a new blockchain namespace
fn register_custom_validator() {
    let mut registry = ValidationRegistry::global();
    
    // Register a custom chain validator for "mychain" namespace
    registry.register_chain_validator("mychain", |reference| {
        // Validate that reference is a valid 64-character hex string
        if reference.len() == 64 && reference.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(())
        } else {
            Err("Reference must be a 64-character hex string".into())
        }
    });
    
    // Create and validate a ChainId with the custom namespace
    let custom_chain = ChainId::new("mychain", "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
    assert_eq!(custom_chain.namespace(), "mychain");
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
