//! Settlement address types supporting both blockchain (CAIP-10) and traditional payment systems (RFC 8905).
//!
//! This module provides types for handling settlement addresses that can be either
//! blockchain addresses (CAIP-10 format) or traditional payment system identifiers
//! (PayTo URI format per RFC 8905).

use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Errors that can occur when parsing settlement addresses.
#[derive(Debug, Error)]
pub enum SettlementAddressError {
    /// Invalid PayTo URI format.
    #[error("Invalid PayTo URI format: {0}")]
    InvalidPayToUri(String),

    /// Invalid CAIP-10 format.
    #[error("Invalid CAIP-10 format: {0}")]
    InvalidCaip10(String),

    /// Unknown settlement address format.
    #[error("Unknown settlement address format")]
    UnknownFormat,
}

/// A PayTo URI per RFC 8905 for traditional payment systems.
///
/// Format: `payto://METHOD/ACCOUNT[?parameters]`
///
/// Examples:
/// - `payto://iban/DE75512108001245126199`
/// - `payto://ach/122000247/111000025`
/// - `payto://bic/SOGEDEFFXXX`
/// - `payto://upi/9999999999@paytm`
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct PayToUri(String);

impl<'de> Deserialize<'de> for PayToUri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PayToUri::new(s).map_err(de::Error::custom)
    }
}

impl PayToUri {
    /// Create a new PayTo URI, validating the format.
    pub fn new(uri: String) -> Result<Self, SettlementAddressError> {
        if !uri.starts_with("payto://") {
            return Err(SettlementAddressError::InvalidPayToUri(
                "PayTo URI must start with 'payto://'".to_string(),
            ));
        }

        // Basic validation: must have method and account parts
        let after_scheme = &uri[8..]; // Skip "payto://"
        if !after_scheme.contains('/') {
            return Err(SettlementAddressError::InvalidPayToUri(
                "PayTo URI must have method and account parts".to_string(),
            ));
        }

        let parts: Vec<&str> = after_scheme.splitn(2, '/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(SettlementAddressError::InvalidPayToUri(
                "PayTo URI must have non-empty method and account".to_string(),
            ));
        }

        Ok(PayToUri(uri))
    }

    /// Get the payment method (e.g., "iban", "ach", "bic", "upi").
    pub fn method(&self) -> &str {
        let after_scheme = &self.0[8..];
        after_scheme.split('/').next().unwrap_or("")
    }

    /// Get the full URI as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PayToUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PayToUri {
    type Err = SettlementAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PayToUri::new(s.to_string())
    }
}

/// A settlement address that can be either a blockchain address (CAIP-10) or
/// a traditional payment system identifier (PayTo URI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettlementAddress {
    /// A blockchain address in CAIP-10 format.
    Caip10(String),

    /// A traditional payment system identifier as a PayTo URI.
    PayTo(PayToUri),
}

impl SettlementAddress {
    /// Create a settlement address from a string, auto-detecting the format.
    pub fn from_string(s: String) -> Result<Self, SettlementAddressError> {
        if s.starts_with("payto://") {
            Ok(SettlementAddress::PayTo(PayToUri::new(s)?))
        } else if s.contains(':') {
            // Basic CAIP-10 validation - should have at least chain_id:address format
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
                Ok(SettlementAddress::Caip10(s))
            } else {
                Err(SettlementAddressError::InvalidCaip10(
                    "CAIP-10 must have chain_id and address parts".to_string(),
                ))
            }
        } else {
            Err(SettlementAddressError::UnknownFormat)
        }
    }

    /// Check if this is a blockchain address.
    pub fn is_blockchain(&self) -> bool {
        matches!(self, SettlementAddress::Caip10(_))
    }

    /// Check if this is a traditional payment address.
    pub fn is_traditional(&self) -> bool {
        matches!(self, SettlementAddress::PayTo(_))
    }

    /// Get the address as a string.
    pub fn as_str(&self) -> &str {
        match self {
            SettlementAddress::Caip10(s) => s,
            SettlementAddress::PayTo(uri) => uri.as_str(),
        }
    }
}

impl fmt::Display for SettlementAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for SettlementAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SettlementAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SettlementAddress::from_string(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payto_uri_creation() {
        let uri = PayToUri::new("payto://iban/DE75512108001245126199".to_string()).unwrap();
        assert_eq!(uri.method(), "iban");
        assert_eq!(uri.as_str(), "payto://iban/DE75512108001245126199");
    }

    #[test]
    fn test_payto_uri_with_parameters() {
        let uri = PayToUri::new(
            "payto://iban/GB33BUKB20201555555555?receiver-name=UK%20Receiver%20Ltd".to_string(),
        )
        .unwrap();
        assert_eq!(uri.method(), "iban");
        assert!(uri.as_str().contains("receiver-name"));
    }

    #[test]
    fn test_payto_uri_various_methods() {
        let test_cases = vec![
            "payto://iban/DE75512108001245126199",
            "payto://ach/122000247/111000025",
            "payto://bic/SOGEDEFFXXX",
            "payto://upi/9999999999@paytm",
        ];

        for case in test_cases {
            let uri = PayToUri::new(case.to_string()).unwrap();
            assert!(uri.as_str().starts_with("payto://"));
        }
    }

    #[test]
    fn test_payto_uri_invalid_format() {
        let invalid_cases = vec![
            "http://example.com",          // Wrong scheme
            "payto://",                    // Missing method and account
            "payto://iban",                // Missing account
            "payto://iban/",               // Empty account
            "iban/DE75512108001245126199", // Missing scheme
        ];

        for case in invalid_cases {
            assert!(PayToUri::new(case.to_string()).is_err());
        }
    }

    #[test]
    fn test_settlement_address_from_payto() {
        let addr =
            SettlementAddress::from_string("payto://iban/DE75512108001245126199".to_string())
                .unwrap();

        assert!(addr.is_traditional());
        assert!(!addr.is_blockchain());
        assert_eq!(addr.as_str(), "payto://iban/DE75512108001245126199");
    }

    #[test]
    fn test_settlement_address_from_caip10() {
        let addr = SettlementAddress::from_string(
            "eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
        )
        .unwrap();

        assert!(addr.is_blockchain());
        assert!(!addr.is_traditional());
        assert_eq!(
            addr.as_str(),
            "eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
        );
    }

    #[test]
    fn test_settlement_address_simple_caip10() {
        // Simple chain_id:address format
        let addr = SettlementAddress::from_string(
            "ethereum:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
        )
        .unwrap();

        assert!(addr.is_blockchain());
        assert_eq!(
            addr.as_str(),
            "ethereum:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
        );
    }

    #[test]
    fn test_settlement_address_invalid() {
        let invalid_cases = vec![
            "just-some-text", // No clear format
            "",               // Empty string
            ":",              // Just separator
            "payto://",       // Invalid PayTo
        ];

        for case in invalid_cases {
            assert!(SettlementAddress::from_string(case.to_string()).is_err());
        }
    }

    #[test]
    fn test_settlement_address_serialization() {
        let payto_addr =
            SettlementAddress::from_string("payto://iban/DE75512108001245126199".to_string())
                .unwrap();

        let json = serde_json::to_string(&payto_addr).unwrap();
        assert_eq!(json, "\"payto://iban/DE75512108001245126199\"");

        let deserialized: SettlementAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, payto_addr);
    }

    #[test]
    fn test_settlement_address_caip10_serialization() {
        let caip_addr = SettlementAddress::from_string(
            "eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
        )
        .unwrap();

        let json = serde_json::to_string(&caip_addr).unwrap();
        assert_eq!(
            json,
            "\"eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb\""
        );

        let deserialized: SettlementAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, caip_addr);
    }

    #[test]
    fn test_settlement_address_array_serialization() {
        let addresses = vec![
            SettlementAddress::from_string("payto://iban/DE75512108001245126199".to_string())
                .unwrap(),
            SettlementAddress::from_string(
                "eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
            )
            .unwrap(),
        ];

        let json = serde_json::to_string(&addresses).unwrap();
        assert!(json.contains("payto://iban"));
        assert!(json.contains("eip155:1"));

        let deserialized: Vec<SettlementAddress> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 2);
        assert!(deserialized[0].is_traditional());
        assert!(deserialized[1].is_blockchain());
    }
}
