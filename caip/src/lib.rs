/// CAIP: Chain Agnostic Identifier Standard Implementation
///
/// This library provides a Rust implementation of the Chain Agnostic Identifier Standards,
/// including CAIP-2 (Chain ID), CAIP-10 (Account ID), and CAIP-19 (Asset ID).
///
/// See <https://github.com/ChainAgnostic/CAIPs> for the full specifications.
// Re-export main structs and functions
mod account_id;
mod asset_id;
mod chain_id;
pub mod error;
mod validation;

pub use account_id::AccountId;
pub use asset_id::AssetId;
pub use chain_id::ChainId;
pub use error::Error;
pub use validation::{ValidationRegistry, ValidatorFn};

use std::str::FromStr;

/// Type representing any CAIP identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaipId {
    /// CAIP-2 Chain ID
    ChainId(ChainId),
    /// CAIP-10 Account ID
    AccountId(AccountId),
    /// CAIP-19 Asset ID
    AssetId(AssetId),
}

/// Attempt to parse a string as any recognized CAIP identifier
///
/// This function will try to parse the string as a ChainId, AccountId, or AssetId
/// in that order, and return the first successful parse.
///
/// # Arguments
///
/// * `s` - The string to parse
///
/// # Returns
///
/// * `Result<CaipId, Error>` - The parsed CAIP identifier or an error
pub fn parse(s: &str) -> Result<CaipId, error::Error> {
    if let Ok(chain_id) = ChainId::from_str(s) {
        return Ok(CaipId::ChainId(chain_id));
    }

    if let Ok(account_id) = AccountId::from_str(s) {
        return Ok(CaipId::AccountId(account_id));
    }

    if let Ok(asset_id) = AssetId::from_str(s) {
        return Ok(CaipId::AssetId(asset_id));
    }

    Err(error::Error::UnrecognizedFormat(s.to_string()))
}

impl std::fmt::Display for CaipId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaipId::ChainId(id) => write!(f, "{}", id),
            CaipId::AccountId(id) => write!(f, "{}", id),
            CaipId::AssetId(id) => write!(f, "{}", id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chain_id() {
        let id = parse("eip155:1").unwrap();
        match id {
            CaipId::ChainId(chain_id) => {
                assert_eq!(chain_id.namespace(), "eip155");
                assert_eq!(chain_id.reference(), "1");
            }
            _ => panic!("Expected ChainId"),
        }
    }

    #[test]
    fn test_parse_account_id() {
        let id = parse("eip155:1:0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db").unwrap();
        match id {
            CaipId::AccountId(account_id) => {
                assert_eq!(account_id.chain_id().namespace(), "eip155");
                assert_eq!(account_id.chain_id().reference(), "1");
                assert_eq!(account_id.address(), "0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db");
            }
            _ => panic!("Expected AccountId"),
        }
    }

    #[test]
    fn test_parse_asset_id() {
        let id = parse("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
        match id {
            CaipId::AssetId(asset_id) => {
                assert_eq!(asset_id.chain_id().namespace(), "eip155");
                assert_eq!(asset_id.chain_id().reference(), "1");
                assert_eq!(asset_id.namespace(), "erc20");
                assert_eq!(
                    asset_id.reference(),
                    "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                );
            }
            _ => panic!("Expected AssetId"),
        }
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse("").is_err());
        assert!(parse("invalid").is_err());
        assert!(parse("foo:bar:baz:qux").is_err());
    }

    #[test]
    fn test_caip_id_to_string() {
        let chain_id = ChainId::from_str("eip155:1").unwrap();
        let account_id = AccountId::from_str("eip155:1:0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db").unwrap();
        let asset_id = AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();

        let chain_caip = CaipId::ChainId(chain_id);
        let account_caip = CaipId::AccountId(account_id);
        let asset_caip = CaipId::AssetId(asset_id);

        assert_eq!(chain_caip.to_string(), "eip155:1");
        assert_eq!(
            account_caip.to_string(),
            "eip155:1:0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db"
        );
        assert_eq!(
            asset_caip.to_string(),
            "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        );
    }
}
