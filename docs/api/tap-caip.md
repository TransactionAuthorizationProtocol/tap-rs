# tap-caip API Reference

The `tap-caip` crate provides implementations of the Chain Agnostic Improvement Proposals (CAIP) standards for the TAP protocol. This includes CAIP-2 (Chain ID), CAIP-10 (Account ID), and CAIP-19 (Asset ID) implementations.

## Core Types

### `ChainId`

Represents a CAIP-2 Chain ID, which identifies a blockchain in a namespace.

```rust
pub struct ChainId {
    /// Namespace of the chain (e.g., "eip155" for Ethereum)
    pub namespace: String,
    
    /// Reference of the chain within the namespace (e.g., "1" for Ethereum Mainnet)
    pub reference: String,
}
```

#### Methods

```rust
/// Create a new ChainId from namespace and reference
pub fn new<S: Into<String>>(namespace: S, reference: S) -> Self;

/// Parse a ChainId from a string in the format "namespace:reference"
pub fn from_str(s: &str) -> Result<Self, Error>;

/// Convert the ChainId to a string in the format "namespace:reference"
pub fn to_string(&self) -> String;

/// Validate that the ChainId follows CAIP-2 requirements
pub fn validate(&self) -> Result<(), Error>;
```

#### Example

```rust
use tap_caip::ChainId;
use std::str::FromStr;

// Create a ChainId from components
let chain_id = ChainId::new("eip155", "1");
assert_eq!(chain_id.namespace, "eip155");
assert_eq!(chain_id.reference, "1");

// Parse a ChainId from a string
let chain_id = ChainId::from_str("eip155:1").unwrap();
assert_eq!(chain_id.to_string(), "eip155:1");

// Validate a ChainId
chain_id.validate().unwrap();
```

### `AccountId`

Represents a CAIP-10 Account ID, which identifies an account on a specific chain.

```rust
pub struct AccountId {
    /// The chain ID component
    pub chain_id: ChainId,
    
    /// The account address on the chain
    pub address: String,
}
```

#### Methods

```rust
/// Create a new AccountId from chain ID and address
pub fn new(chain_id: ChainId, address: String) -> Self;

/// Parse an AccountId from a string in the format "namespace:reference:address"
pub fn from_str(s: &str) -> Result<Self, Error>;

/// Convert the AccountId to a string in the format "namespace:reference:address"
pub fn to_string(&self) -> String;

/// Validate that the AccountId follows CAIP-10 requirements
pub fn validate(&self) -> Result<(), Error>;
```

#### Example

```rust
use tap_caip::{AccountId, ChainId};
use std::str::FromStr;

// Create an AccountId from components
let chain_id = ChainId::new("eip155", "1");
let account_id = AccountId::new(chain_id, "0x0000000000000000000000000000000000000000".to_string());

// Parse an AccountId from a string
let account_id = AccountId::from_str("eip155:1:0x0000000000000000000000000000000000000000").unwrap();
assert_eq!(account_id.to_string(), "eip155:1:0x0000000000000000000000000000000000000000");

// Validate an AccountId
account_id.validate().unwrap();
```

### `AssetId`

Represents a CAIP-19 Asset ID, which identifies a specific asset on a specific chain.

```rust
pub struct AssetId {
    /// The chain ID component
    pub chain_id: ChainId,
    
    /// The asset namespace (e.g., "erc20")
    pub asset_namespace: String,
    
    /// The asset reference within the namespace (e.g., token contract address)
    pub asset_reference: String,
}
```

#### Methods

```rust
/// Create a new AssetId from components
pub fn new(chain_id: ChainId, asset_namespace: String, asset_reference: String) -> Self;

/// Parse an AssetId from a string in the format "namespace:reference/assetNamespace:assetReference"
pub fn from_str(s: &str) -> Result<Self, Error>;

/// Convert the AssetId to a string in the format "namespace:reference/assetNamespace:assetReference"
pub fn to_string(&self) -> String;

/// Validate that the AssetId follows CAIP-19 requirements
pub fn validate(&self) -> Result<(), Error>;
```

#### Example

```rust
use tap_caip::{AssetId, ChainId};
use std::str::FromStr;

// Create an AssetId from components
let chain_id = ChainId::new("eip155", "1");
let asset_id = AssetId::new(
    chain_id, 
    "erc20".to_string(), 
    "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string()
);

// Parse an AssetId from a string
let asset_id = AssetId::from_str("eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap();
assert_eq!(asset_id.to_string(), "eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F");

// Validate an AssetId
asset_id.validate().unwrap();
```

## Validation

### `Validator`

A trait for validating CAIP identifiers.

```rust
pub trait Validator {
    /// Validate the identifier according to its CAIP standard
    fn validate(&self) -> Result<(), Error>;
}
```

### Validation Functions

```rust
/// Validate a ChainId according to CAIP-2
pub fn validate_chain_id(chain_id: &ChainId) -> Result<(), Error>;

/// Validate an AccountId according to CAIP-10
pub fn validate_account_id(account_id: &AccountId) -> Result<(), Error>;

/// Validate an AssetId according to CAIP-19
pub fn validate_asset_id(asset_id: &AssetId) -> Result<(), Error>;
```

## Error Handling

```rust
pub enum Error {
    /// Error when parsing an identifier
    ParseError(String),
    
    /// Error when validating an identifier
    ValidationError(String),
    
    /// General error
    General(String),
}
```

## Usage Patterns

### Working with Multiple Asset Types

```rust
use tap_caip::{AssetId, ChainId};
use std::str::FromStr;

fn process_assets(asset_ids: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    for asset_str in asset_ids {
        let asset_id = AssetId::from_str(asset_str)?;
        
        match (asset_id.chain_id.namespace.as_str(), asset_id.asset_namespace.as_str()) {
            // Ethereum ERC-20 tokens
            ("eip155", "erc20") => {
                println!(
                    "Found ERC-20 token on Ethereum chain {}: Contract {}",
                    asset_id.chain_id.reference,
                    asset_id.asset_reference
                );
            },
            // Ethereum ERC-721 NFTs
            ("eip155", "erc721") => {
                println!(
                    "Found ERC-721 NFT on Ethereum chain {}: Contract {}",
                    asset_id.chain_id.reference,
                    asset_id.asset_reference
                );
            },
            // Solana SPL tokens
            ("solana", "spl") => {
                println!(
                    "Found SPL token on Solana chain {}: {}",
                    asset_id.chain_id.reference,
                    asset_id.asset_reference
                );
            },
            // Handle other asset types
            _ => {
                println!(
                    "Unknown asset type: {} on chain {}:{}",
                    asset_id.asset_namespace,
                    asset_id.chain_id.namespace,
                    asset_id.chain_id.reference
                );
            }
        }
    }
    
    Ok(())
}
```

### Creating Assets from Different Chain Types

```rust
use tap_caip::{AssetId, ChainId};

fn create_common_assets() -> Vec<AssetId> {
    let mut assets = Vec::new();
    
    // Ethereum Mainnet DAI ERC-20 token
    let eth_chain = ChainId::new("eip155", "1");
    let dai = AssetId::new(
        eth_chain.clone(),
        "erc20".to_string(),
        "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string()
    );
    assets.push(dai);
    
    // Ethereum Mainnet CryptoKitties ERC-721 token
    let crypto_kitties = AssetId::new(
        eth_chain.clone(),
        "erc721".to_string(),
        "0x06012c8cf97BEaD5deAe237070F9587f8E7A266d".to_string()
    );
    assets.push(crypto_kitties);
    
    // Polygon (Matic) USDC ERC-20 token
    let polygon_chain = ChainId::new("eip155", "137");
    let polygon_usdc = AssetId::new(
        polygon_chain,
        "erc20".to_string(),
        "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string()
    );
    assets.push(polygon_usdc);
    
    // Solana USDC SPL token
    let solana_chain = ChainId::new("solana", "mainnet");
    let solana_usdc = AssetId::new(
        solana_chain,
        "spl".to_string(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()
    );
    assets.push(solana_usdc);
    
    assets
}
```
