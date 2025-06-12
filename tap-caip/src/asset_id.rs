use crate::chain_id::ChainId;
use crate::error::Error;
use crate::validation::ValidationRegistry;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// Regular expression pattern for CAIP-19 asset ID validation
static ASSET_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[-a-z0-9]{3,8}:[-a-zA-Z0-9]{1,64}/[-a-z0-9]{3,8}:[-a-zA-Z0-9]{1,64}$")
        .expect("Failed to compile ASSET_ID_REGEX")
});

/// CAIP-19 Asset ID implementation
///
/// An Asset ID is a string that identifies a blockchain asset and follows the format:
/// `<chainId>/<assetNamespace>:<assetReference>`
///
/// - `chainId`: CAIP-2 Chain ID (e.g., "eip155:1" for Ethereum mainnet)
/// - `assetNamespace`: Protocol or standard (e.g., "erc20")
/// - `assetReference`: Asset-specific identifier (e.g., token contract address)
///
/// Example: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48" for USDC on Ethereum
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetId {
    chain_id: ChainId,
    namespace: String,
    reference: String,
}

impl AssetId {
    /// Create a new AssetId from ChainId, asset namespace, and reference
    ///
    /// # Arguments
    ///
    /// * `chain_id` - The CAIP-2 Chain ID
    /// * `namespace` - The asset namespace (e.g., "erc20")
    /// * `reference` - The asset reference (e.g., token contract address)
    ///
    /// # Returns
    ///
    /// * `Result<AssetId, Error>` - An AssetId or an error if validation fails
    pub fn new(chain_id: ChainId, namespace: &str, reference: &str) -> Result<Self, Error> {
        // Validate namespace format
        Self::validate_namespace(namespace)?;

        // Validate reference format
        Self::validate_reference(namespace, reference)?;

        // Validate full asset ID
        let asset_id_str = format!("{}/{namespace}:{reference}", chain_id);
        if !ASSET_ID_REGEX.is_match(&asset_id_str) {
            return Err(Error::InvalidAssetId(asset_id_str));
        }

        Ok(Self {
            chain_id,
            namespace: namespace.to_string(),
            reference: reference.to_string(),
        })
    }

    /// Get the chain ID component
    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    /// Get the asset namespace component
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Get the asset reference component
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// Validate the asset namespace
    fn validate_namespace(namespace: &str) -> Result<(), Error> {
        // Namespace must be 3-8 characters, lowercase alphanumeric with possible hyphens
        if !Regex::new(r"^[-a-z0-9]{3,8}$")
            .expect("Failed to compile namespace regex")
            .is_match(namespace)
        {
            return Err(Error::InvalidAssetNamespace(namespace.to_string()));
        }

        Ok(())
    }

    /// Validate the asset reference with respect to the namespace
    fn validate_reference(namespace: &str, reference: &str) -> Result<(), Error> {
        // Reference must be 1-64 characters, alphanumeric with possible hyphens
        if !Regex::new(r"^[-a-zA-Z0-9]{1,64}$")
            .expect("Failed to compile reference regex")
            .is_match(reference)
        {
            return Err(Error::InvalidAssetReference(reference.to_string()));
        }

        // Get the global validation registry
        let registry = ValidationRegistry::global();
        let registry_guard = registry.lock().unwrap();

        // Apply namespace-specific validation rules
        if let Some(validator) = registry_guard.get_asset_validator(namespace) {
            validator(reference)
                .map_err(|err| Error::InvalidAssetReference(format!("{}: {}", reference, err)))?;
        }

        Ok(())
    }
}

impl FromStr for AssetId {
    type Err = Error;

    /// Parse a string into an AssetId
    ///
    /// # Arguments
    ///
    /// * `s` - A string in the format "namespace:reference/assetNamespace:assetReference"
    ///
    /// # Returns
    ///
    /// * `Result<AssetId, Error>` - An AssetId or an error if parsing fails
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check the overall format first
        if !ASSET_ID_REGEX.is_match(s) {
            return Err(Error::InvalidAssetId(s.to_string()));
        }

        // Split by "/" to separate chain ID from asset identifier
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(Error::InvalidAssetId(s.to_string()));
        }

        // Parse the chain ID
        let chain_id = ChainId::from_str(parts[0])?;

        // Split asset identifier by ":" to get namespace and reference
        let asset_parts: Vec<&str> = parts[1].split(':').collect();
        if asset_parts.len() != 2 {
            return Err(Error::InvalidAssetId(s.to_string()));
        }

        // Create the asset ID
        AssetId::new(chain_id, asset_parts[0], asset_parts[1])
    }
}

// Removed the conflicting ToString implementation
// Let the default implementation from Display be used

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}:{}", self.chain_id, self.namespace, self.reference)
    }
}

impl Serialize for AssetId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for AssetId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        AssetId::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization_format() {
        let asset_str = "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        let asset_id = AssetId::from_str(asset_str).unwrap();
        
        // Check current serialization
        let json = serde_json::to_string(&asset_id).unwrap();
        assert_eq!(json, format!("\"{}\"", asset_str));
        
        // Try to deserialize from string
        let json_string = format!("\"{}\"", asset_str);
        let result = serde_json::from_str::<AssetId>(&json_string);
        assert!(result.is_ok());
        
        // Test array serialization (like in test vectors)
        let assets = vec![
            AssetId::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
            AssetId::from_str("eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap(),
        ];
        let json_array = serde_json::to_string(&assets).unwrap();
        assert_eq!(json_array, r#"["eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","eip155:1/erc20:0x6B175474E89094C44Da98b954EedeAC495271d0F"]"#);
        
        // Test deserializing from array
        let test_vector_json = r#"["eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"]"#;
        let result: Result<Vec<AssetId>, _> = serde_json::from_str(test_vector_json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_asset_ids() {
        // Test Ethereum ERC-20 token (USDC)
        let usdc =
            AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
        assert_eq!(usdc.chain_id().to_string(), "eip155:1");
        assert_eq!(usdc.namespace(), "erc20");
        assert_eq!(
            usdc.reference(),
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        );
        assert_eq!(
            usdc.to_string(),
            "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        );

        // Test direct creation
        let chain_id = ChainId::from_str("eip155:1").unwrap();
        let dai = AssetId::new(
            chain_id,
            "erc20",
            "0x6b175474e89094c44da98b954eedeac495271d0f",
        )
        .unwrap();
        assert_eq!(
            dai.to_string(),
            "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f"
        );
    }

    #[test]
    fn test_invalid_asset_ids() {
        // Invalid: empty string
        assert!(AssetId::from_str("").is_err());

        // Invalid: missing separators
        assert!(AssetId::from_str("eip1551erc20address").is_err());

        // Invalid: missing slash
        assert!(
            AssetId::from_str("eip155:1erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").is_err()
        );

        // Invalid: empty namespace
        assert!(AssetId::from_str(":1/:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").is_err());

        // Invalid: empty reference
        assert!(AssetId::from_str("eip155:1/erc20:").is_err());

        // Invalid: namespace too short
        assert!(
            AssetId::from_str("eip155:1/er:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").is_err()
        );

        // Invalid: reference too long
        let long_reference = "a".repeat(65);
        assert!(AssetId::from_str(&format!("eip155:1/erc20:{}", long_reference)).is_err());
    }

    #[test]
    fn test_serialization() {
        let asset_id =
            AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
        let serialized = serde_json::to_string(&asset_id).unwrap();

        // The JSON representation should be a string
        assert_eq!(serialized, r#""eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48""#);

        let deserialized: AssetId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, asset_id);
    }

    #[test]
    fn test_display_formatting() {
        let asset_id =
            AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
        assert_eq!(
            format!("{}", asset_id),
            "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        );
        assert_eq!(
            asset_id.to_string(),
            "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        );
    }
}
