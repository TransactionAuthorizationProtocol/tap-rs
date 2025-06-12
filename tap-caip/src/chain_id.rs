use crate::error::Error;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// Regular expression pattern for CAIP-2 chain ID validation
/// The pattern is modified to support longer references for Bitcoin block hashes (64 chars)
static CHAIN_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[-a-z0-9]{3,8}:[-a-zA-Z0-9]{1,64}$").expect("Failed to compile CHAIN_ID_REGEX")
});

/// CAIP-2 Chain ID implementation
///
/// A Chain ID is a string that identifies a blockchain and follows the format:
/// `<namespace>:<reference>`
///
/// - `namespace`: Identifies the blockchain standard (e.g., eip155 for Ethereum)
/// - `reference`: Chain-specific identifier (e.g., 1 for Ethereum mainnet)
///
/// Example: "eip155:1" for Ethereum mainnet
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChainId {
    namespace: String,
    reference: String,
}

impl ChainId {
    /// Create a new ChainId from namespace and reference
    ///
    /// # Arguments
    ///
    /// * `namespace` - The blockchain namespace (e.g., "eip155")
    /// * `reference` - The chain-specific reference (e.g., "1")
    ///
    /// # Returns
    ///
    /// * `Result<ChainId, Error>` - A ChainId or an error if validation fails
    pub fn new(namespace: &str, reference: &str) -> Result<Self, Error> {
        // Validate namespace and reference individually
        Self::validate_namespace(namespace)?;
        Self::validate_reference(reference)?;

        // Construct and validate the full chain ID
        let chain_id = format!("{}:{}", namespace, reference);
        if !CHAIN_ID_REGEX.is_match(&chain_id) {
            return Err(Error::InvalidChainId(chain_id));
        }

        Ok(Self {
            namespace: namespace.to_string(),
            reference: reference.to_string(),
        })
    }

    /// Get the namespace component
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Get the reference component
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// Validate the namespace component
    fn validate_namespace(namespace: &str) -> Result<(), Error> {
        // Namespace must be 3-8 characters, lowercase alphanumeric with possible hyphens
        if !Regex::new(r"^[-a-z0-9]{3,8}$")
            .expect("Failed to compile namespace regex")
            .is_match(namespace)
        {
            return Err(Error::InvalidNamespace(namespace.to_string()));
        }

        // Additional validation could be added here for known namespaces
        // For now, we allow any namespace that matches the format

        Ok(())
    }

    /// Validate the reference component
    fn validate_reference(reference: &str) -> Result<(), Error> {
        // Reference must be 1-64 characters, alphanumeric with possible hyphens
        if !Regex::new(r"^[-a-zA-Z0-9]{1,64}$")
            .expect("Failed to compile reference regex")
            .is_match(reference)
        {
            return Err(Error::InvalidReference(reference.to_string()));
        }

        // Additional validation could be added here for specific references
        // For now, we allow any reference that matches the format

        Ok(())
    }
}

impl FromStr for ChainId {
    type Err = Error;

    /// Parse a string into a ChainId
    ///
    /// # Arguments
    ///
    /// * `s` - A string in the format "namespace:reference"
    ///
    /// # Returns
    ///
    /// * `Result<ChainId, Error>` - A ChainId or an error if parsing fails
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check the overall format first
        if !CHAIN_ID_REGEX.is_match(s) {
            return Err(Error::InvalidChainId(s.to_string()));
        }

        // Split the chain ID into namespace and reference
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(Error::InvalidChainId(s.to_string()));
        }

        ChainId::new(parts[0], parts[1])
    }
}

// Removed the conflicting ToString implementation
// Let the default implementation from Display be used

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.reference)
    }
}

impl Serialize for ChainId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ChainId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ChainId::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_chain_ids() {
        // Test Ethereum mainnet
        let eth_mainnet = ChainId::from_str("eip155:1").unwrap();
        assert_eq!(eth_mainnet.namespace(), "eip155");
        assert_eq!(eth_mainnet.reference(), "1");
        assert_eq!(eth_mainnet.to_string(), "eip155:1");

        // Test Bitcoin mainnet
        let btc_mainnet = ChainId::from_str(
            "bip122:000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
        )
        .unwrap();
        assert_eq!(btc_mainnet.namespace(), "bip122");
        assert_eq!(
            btc_mainnet.reference(),
            "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
        );

        // Test direct creation
        let poly_mainnet = ChainId::new("polkadot", "91b171bb158e2d3848fa23a9f1c25182").unwrap();
        assert_eq!(poly_mainnet.namespace(), "polkadot");
        assert_eq!(poly_mainnet.reference(), "91b171bb158e2d3848fa23a9f1c25182");
    }

    #[test]
    fn test_invalid_chain_ids() {
        // Invalid: empty string
        assert!(ChainId::from_str("").is_err());

        // Invalid: missing separator
        assert!(ChainId::from_str("eip1551").is_err());

        // Invalid: empty namespace
        assert!(ChainId::from_str(":1").is_err());

        // Invalid: empty reference
        assert!(ChainId::from_str("eip155:").is_err());

        // Invalid: namespace too short
        assert!(ChainId::from_str("ei:1").is_err());

        // Invalid: namespace too long
        assert!(ChainId::from_str("eip155toolong:1").is_err());

        // Invalid: invalid namespace characters
        assert!(ChainId::from_str("EIP155:1").is_err()); // uppercase not allowed
        assert!(ChainId::from_str("eip_155:1").is_err()); // underscore not allowed

        // Invalid: reference too long
        let long_reference = "a".repeat(65);
        assert!(ChainId::from_str(&format!("eip155:{}", long_reference)).is_err());
    }

    #[test]
    fn test_serialization() {
        let chain_id = ChainId::from_str("eip155:1").unwrap();
        let serialized = serde_json::to_string(&chain_id).unwrap();
        assert_eq!(serialized, r#""eip155:1""#);

        let deserialized: ChainId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, chain_id);
    }

    #[test]
    fn test_display_formatting() {
        let chain_id = ChainId::from_str("eip155:1").unwrap();
        assert_eq!(format!("{}", chain_id), "eip155:1");
        assert_eq!(chain_id.to_string(), "eip155:1");
    }
}
