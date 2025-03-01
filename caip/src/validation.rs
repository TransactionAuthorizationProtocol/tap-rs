use crate::error::Error;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Type for validator functions that validate addresses, references, etc.
pub type ValidatorFn = fn(&str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Registry for CAIP-specific validation functions
pub struct ValidationRegistry {
    chain_validators: HashMap<String, ValidatorFn>,
    account_validators: HashMap<String, ValidatorFn>,
    asset_validators: HashMap<String, ValidatorFn>,
}

impl Default for ValidationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationRegistry {
    /// Create a new empty validation registry
    pub fn new() -> Self {
        Self {
            chain_validators: HashMap::new(),
            account_validators: HashMap::new(),
            asset_validators: HashMap::new(),
        }
    }

    /// Get access to the global ValidationRegistry singleton
    pub fn global() -> Arc<Mutex<Self>> {
        static REGISTRY: Lazy<Arc<Mutex<ValidationRegistry>>> = Lazy::new(|| {
            let registry = ValidationRegistry::new_with_defaults();
            Arc::new(Mutex::new(registry))
        });
        REGISTRY.clone()
    }

    /// Create a new validation registry with default validators
    pub fn new_with_defaults() -> Self {
        let mut registry = Self::new();

        // Register Ethereum validators
        registry.register_account_validator("eip155", ethereum_address_validator);
        registry.register_asset_validator("erc20", ethereum_address_validator);
        registry.register_asset_validator("erc721", ethereum_address_validator);

        // Register Bitcoin validators
        registry.register_account_validator("bip122", bitcoin_address_validator);

        registry
    }

    /// Register a validator for chain IDs with the specified namespace
    pub fn register_chain_validator(&mut self, namespace: &str, validator: ValidatorFn) {
        self.chain_validators
            .insert(namespace.to_string(), validator);
    }

    /// Register a validator for account addresses with the specified chain namespace
    pub fn register_account_validator(&mut self, namespace: &str, validator: ValidatorFn) {
        self.account_validators
            .insert(namespace.to_string(), validator);
    }

    /// Register a validator for asset references with the specified asset namespace
    pub fn register_asset_validator(&mut self, namespace: &str, validator: ValidatorFn) {
        self.asset_validators
            .insert(namespace.to_string(), validator);
    }

    /// Get a validator for chain IDs with the specified namespace
    pub fn get_chain_validator(&self, namespace: &str) -> Option<ValidatorFn> {
        self.chain_validators.get(namespace).copied()
    }

    /// Get a validator for account addresses with the specified chain namespace
    pub fn get_account_validator(&self, namespace: &str) -> Option<ValidatorFn> {
        self.account_validators.get(namespace).copied()
    }

    /// Get a validator for asset references with the specified asset namespace
    pub fn get_asset_validator(&self, namespace: &str) -> Option<ValidatorFn> {
        self.asset_validators.get(namespace).copied()
    }
}

impl fmt::Debug for ValidationRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidationRegistry")
            .field(
                "chain_validators",
                &format!("{} entries", self.chain_validators.len()),
            )
            .field(
                "account_validators",
                &format!("{} entries", self.account_validators.len()),
            )
            .field(
                "asset_validators",
                &format!("{} entries", self.asset_validators.len()),
            )
            .finish()
    }
}

/// Validator for Ethereum addresses
fn ethereum_address_validator(
    address: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Ethereum addresses are 42 characters long (0x + 40 hex chars)
    if !address.starts_with("0x") || address.len() != 42 {
        return Err(Error::InvalidEthereumAddress(address.to_string()).into());
    }

    // Check if the address (after 0x) is valid hexadecimal
    if hex::decode(&address[2..]).is_err() {
        return Err(Error::InvalidEthereumAddress(address.to_string()).into());
    }

    // Additional validation could be added here, like checksum validation
    Ok(())
}

/// Validator for Bitcoin addresses
fn bitcoin_address_validator(
    address: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Basic validation for now - implement more complete validation as needed
    if address.len() < 26 || address.len() > 35 {
        return Err(Error::InvalidBitcoinAddress(address.to_string()).into());
    }

    // Additional validation criteria could be added here
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethereum_validator() {
        let valid_eth_address = "0x4b20993Bc481177ec7E8f571ceCaE8A9e22C02db";
        let too_short_address = "0x4b20";
        let not_hex_address = "0xZZZZ993Bc481177ec7E8f571ceCaE8A9e22C02db";
        let no_prefix_address = "4b20993Bc481177ec7E8f571ceCaE8A9e22C02db";

        assert!(ethereum_address_validator(valid_eth_address).is_ok());
        assert!(ethereum_address_validator(too_short_address).is_err());
        assert!(ethereum_address_validator(not_hex_address).is_err());
        assert!(ethereum_address_validator(no_prefix_address).is_err());
    }

    #[test]
    fn test_bitcoin_validator() {
        let valid_btc_address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"; // Genesis block address
        let too_short_address = "1A1zP";
        let too_long_address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa12345678901234";

        assert!(bitcoin_address_validator(valid_btc_address).is_ok());
        assert!(bitcoin_address_validator(too_short_address).is_err());
        assert!(bitcoin_address_validator(too_long_address).is_err());
    }

    #[test]
    fn test_registry() {
        let registry = ValidationRegistry::new_with_defaults();

        // Ethereum validators should be registered
        assert!(registry.get_account_validator("eip155").is_some());
        assert!(registry.get_asset_validator("erc20").is_some());
        assert!(registry.get_asset_validator("erc721").is_some());

        // Bitcoin validators should be registered
        assert!(registry.get_account_validator("bip122").is_some());

        // Non-registered validators should not be present
        assert!(registry.get_account_validator("polkadot").is_none());
        assert!(registry.get_asset_validator("unknown").is_none());
    }

    #[test]
    fn test_global_registry() {
        let registry = ValidationRegistry::global();
        let registry_guard = registry.lock().unwrap();

        // Global registry should have the default validators
        assert!(registry_guard.get_account_validator("eip155").is_some());
        assert!(registry_guard.get_asset_validator("erc20").is_some());
    }
}
