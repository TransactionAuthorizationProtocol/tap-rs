use crate::chain_id::ChainId;
use crate::error::Error;
use crate::validation::ValidationRegistry;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Regular expression pattern for CAIP-10 account ID validation
static ACCOUNT_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[-a-z0-9]{3,8}:[-a-zA-Z0-9]{1,32}:[-a-zA-Z0-9]{1,64}$")
        .expect("Failed to compile ACCOUNT_ID_REGEX")
});

/// CAIP-10 Account ID implementation
///
/// An Account ID is a string that identifies a blockchain account and follows the format:
/// `<chainId>:<accountAddress>`
///
/// - `chainId`: CAIP-2 Chain ID (e.g., "eip155:1" for Ethereum mainnet)
/// - `accountAddress`: Chain-specific account address (e.g., "0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db")
///
/// Example: "eip155:1:0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db" for an Ethereum mainnet account
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId {
    chain_id: ChainId,
    address: String,
}

impl AccountId {
    /// Create a new AccountId from a ChainId and address
    ///
    /// # Arguments
    ///
    /// * `chain_id` - The CAIP-2 Chain ID
    /// * `address` - The account address on the specified chain
    ///
    /// # Returns
    ///
    /// * `Result<AccountId, Error>` - An AccountId or an error if validation fails
    pub fn new(chain_id: ChainId, address: &str) -> Result<Self, Error> {
        // Validate the address format according to the chain
        Self::validate_address(&chain_id, address)?;

        Ok(Self {
            chain_id,
            address: address.to_string(),
        })
    }

    /// Get the chain ID component
    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    /// Get the address component
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Validate that the address is valid for the given chain
    fn validate_address(chain_id: &ChainId, address: &str) -> Result<(), Error> {
        // Validate basic address format (1-64 characters, alphanumeric with possible hyphens)
        if !Regex::new(r"^[-a-zA-Z0-9]{1,64}$")
            .expect("Failed to compile address regex")
            .is_match(address)
        {
            return Err(Error::InvalidAddressFormat(
                chain_id.to_string(),
                address.to_string(),
            ));
        }

        // Get the global validation registry
        let registry = ValidationRegistry::global();
        let registry_guard = registry.lock().unwrap();

        // Apply chain-specific validation rules
        if let Some(validator) = registry_guard.get_account_validator(chain_id.namespace()) {
            validator(address).map_err(|err| {
                Error::InvalidAddressFormat(chain_id.to_string(), err.to_string())
            })?;
        }

        Ok(())
    }
}

impl FromStr for AccountId {
    type Err = Error;

    /// Parse a string into an AccountId
    ///
    /// # Arguments
    ///
    /// * `s` - A string in the format "namespace:reference:address"
    ///
    /// # Returns
    ///
    /// * `Result<AccountId, Error>` - An AccountId or an error if parsing fails
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check the overall format first
        if !ACCOUNT_ID_REGEX.is_match(s) {
            return Err(Error::InvalidAccountId(s.to_string()));
        }

        // Split the account ID into its components
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(Error::InvalidAccountId(s.to_string()));
        }

        // Parse the chain ID (namespace:reference)
        let chain_id_str = format!("{}:{}", parts[0], parts[1]);
        let chain_id = ChainId::from_str(&chain_id_str)?;

        // Validate and create the account ID
        AccountId::new(chain_id, parts[2])
    }
}

// Removed the conflicting ToString implementation
// Let the default implementation from Display be used

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.chain_id, self.address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_account_ids() {
        // Test Ethereum account
        let eth_account =
            AccountId::from_str("eip155:1:0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db").unwrap();
        assert_eq!(eth_account.chain_id().to_string(), "eip155:1");
        assert_eq!(
            eth_account.address(),
            "0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db"
        );
        assert_eq!(
            eth_account.to_string(),
            "eip155:1:0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db"
        );

        // Test direct creation
        let chain_id = ChainId::from_str("eip155:1").unwrap();
        let account =
            AccountId::new(chain_id, "0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db").unwrap();
        assert_eq!(
            account.to_string(),
            "eip155:1:0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db"
        );
    }

    #[test]
    fn test_invalid_account_ids() {
        // Invalid: empty string
        assert!(AccountId::from_str("").is_err());

        // Invalid: missing separators
        assert!(AccountId::from_str("eip1551address").is_err());

        // Invalid: empty namespace
        assert!(AccountId::from_str(":1:address").is_err());

        // Invalid: empty reference
        assert!(AccountId::from_str("eip155::address").is_err());

        // Invalid: empty address
        assert!(AccountId::from_str("eip155:1:").is_err());

        // Invalid: address too long
        let long_address = "a".repeat(65);
        assert!(AccountId::from_str(&format!("eip155:1:{}", long_address)).is_err());
    }

    #[test]
    fn test_serialization() {
        let account_id =
            AccountId::from_str("eip155:1:0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db").unwrap();
        let serialized = serde_json::to_string(&account_id).unwrap();

        // The JSON representation should include the chain_id and address fields
        assert!(serialized.contains("chain_id"));
        assert!(serialized.contains("address"));

        let deserialized: AccountId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, account_id);
    }

    #[test]
    fn test_display_formatting() {
        let account_id =
            AccountId::from_str("eip155:1:0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db").unwrap();
        assert_eq!(
            format!("{}", account_id),
            "eip155:1:0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db"
        );
        assert_eq!(
            account_id.to_string(),
            "eip155:1:0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db"
        );
    }
}
