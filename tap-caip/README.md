# TAP CAIP

Implementation of Chain Agnostic Identifier Standards for the Transaction Authorization Protocol (TAP).

## Features

- **CAIP-2**: Support for Chain IDs in namespace:reference format
- **CAIP-10**: Support for Account IDs 
- **CAIP-19**: Support for Asset IDs
- **Validation**: Proper validation of identifiers against the CAIP specifications
- **Parsing and Serialization**: Easy conversion between strings and structured types

## Usage

### Chain ID (CAIP-2)

```rust
use tap_caip::ChainId;
use std::str::FromStr;

// Create from string
let chain_id = ChainId::from_str("eip155:1")?;

// Access components
assert_eq!(chain_id.namespace, "eip155");
assert_eq!(chain_id.reference, "1");

// Convert back to string
assert_eq!(chain_id.to_string(), "eip155:1");
```

### Account ID (CAIP-10)

```rust
use tap_caip::{AccountId, ChainId};
use std::str::FromStr;

// Create from string
let account_id = AccountId::from_str("eip155:1:0x7c47c2532f745a59e405711020c59d8d2d650136")?;

// Create from components
let chain_id = ChainId::from_str("eip155:1")?;
let account_id = AccountId::new(chain_id, "0x7c47c2532f745a59e405711020c59d8d2d650136");

// Access components
assert_eq!(account_id.chain_id.namespace, "eip155");
assert_eq!(account_id.chain_id.reference, "1");
assert_eq!(account_id.address, "0x7c47c2532f745a59e405711020c59d8d2d650136");

// Convert back to string
assert_eq!(account_id.to_string(), "eip155:1:0x7c47c2532f745a59e405711020c59d8d2d650136");
```

### Asset ID (CAIP-19)

```rust
use tap_caip::{AssetId, ChainId};
use std::str::FromStr;

// Create from string
let asset_id = AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7")?;

// Create from components
let chain_id = ChainId::from_str("eip155:1")?;
let asset_id = AssetId::new(
    chain_id,
    "erc20",
    "0xdac17f958d2ee523a2206206994597c13d831ec7"
);

// Access components
assert_eq!(asset_id.chain_id.namespace, "eip155");
assert_eq!(asset_id.chain_id.reference, "1");
assert_eq!(asset_id.asset_namespace, "erc20");
assert_eq!(asset_id.asset_reference, "0xdac17f958d2ee523a2206206994597c13d831ec7");

// Convert back to string
assert_eq!(asset_id.to_string(), "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7");
```

## Validation

Each identifier type includes validation to ensure it conforms to the CAIP standards:

```rust
use tap_caip::{ChainId, AccountId, AssetId};
use std::str::FromStr;

// Valid chain ID
let valid_chain = ChainId::from_str("eip155:1");
assert!(valid_chain.is_ok());

// Invalid chain ID (missing reference)
let invalid_chain = ChainId::from_str("eip155:");
assert!(invalid_chain.is_err());

// Valid account ID
let valid_account = AccountId::from_str("eip155:1:0x1234567890abcdef");
assert!(valid_account.is_ok());

// Invalid account ID (missing address)
let invalid_account = AccountId::from_str("eip155:1:");
assert!(invalid_account.is_err());

// Valid asset ID
let valid_asset = AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7");
assert!(valid_asset.is_ok());

// Invalid asset ID (missing asset reference)
let invalid_asset = AssetId::from_str("eip155:1/erc20:");
assert!(invalid_asset.is_err());
```

## Integration with TAP Messages

The CAIP types are designed to integrate seamlessly with TAP messages:

```rust
use tap_msg::message::types::{Transfer, Participant};
use tap_caip::AssetId;
use std::str::FromStr;
use std::collections::HashMap;

// Create a TAP Transfer message with a CAIP-19 Asset ID
let asset = AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7")?;

let transfer = Transfer {
    asset,
    originator: Participant {
        id: "did:example:123".to_string(),
        role: Some("originator".to_string()),
    },
    beneficiary: Some(Participant {
        id: "did:example:456".to_string(),
        role: Some("beneficiary".to_string()),
    }),
    amount: "100.0".to_string(),
    agents: vec![],
    settlement_id: None,
    memo: Some("Test transfer".to_string()),
    metadata: HashMap::new(),
};
```

## Supported CAIP Standards

- [CAIP-2](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md): Chain ID Specification
- [CAIP-10](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-10.md): Account ID Specification
- [CAIP-19](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-19.md): Asset ID Specification

## Examples

See the [examples directory](./examples) for more detailed examples of using CAIP identifiers.
