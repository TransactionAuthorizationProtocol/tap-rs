//! Validation utilities for IVMS 101 data
//!
//! This module provides validation functions for various IVMS data types
//! including country codes, currency codes, LEI codes, and BIC codes.

use crate::error::{Error, Result};
use iso_currency::Currency;
use regex::Regex;
use std::sync::OnceLock;

/// Regex for LEI validation (20 alphanumeric characters)
static LEI_REGEX: OnceLock<Regex> = OnceLock::new();

/// Regex for BIC validation (8 or 11 alphanumeric characters)
static BIC_REGEX: OnceLock<Regex> = OnceLock::new();

/// Get or create the LEI validation regex
fn lei_regex() -> &'static Regex {
    LEI_REGEX.get_or_init(|| Regex::new(r"^[A-Z0-9]{20}$").unwrap())
}

/// Get or create the BIC validation regex
fn bic_regex() -> &'static Regex {
    BIC_REGEX.get_or_init(|| Regex::new(r"^[A-Z]{4}[A-Z]{2}[A-Z0-9]{2}([A-Z0-9]{3})?$").unwrap())
}

/// Validate an ISO 3166-1 alpha-2 country code
pub fn validate_country_code(code: &str) -> Result<()> {
    if code.len() != 2 {
        return Err(Error::InvalidCountryCode(format!(
            "Country code must be 2 characters: {}",
            code
        )));
    }

    // Check if it's uppercase letters
    if !code.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(Error::InvalidCountryCode(format!(
            "Country code must be uppercase letters: {}",
            code
        )));
    }

    // TODO: Could add actual ISO 3166-1 validation here
    Ok(())
}

/// Validate an ISO 4217 currency code
pub fn validate_currency_code(code: &str) -> Result<()> {
    if code.len() != 3 {
        return Err(Error::InvalidCurrencyCode(format!(
            "Currency code must be 3 characters: {}",
            code
        )));
    }

    // Try to parse as ISO currency
    if Currency::from_code(code).is_none() {
        return Err(Error::InvalidCurrencyCode(format!(
            "Invalid ISO 4217 currency code: {}",
            code
        )));
    }

    Ok(())
}

/// Validate a Legal Entity Identifier (LEI)
pub fn validate_lei(lei: &str) -> Result<()> {
    if !lei_regex().is_match(lei) {
        return Err(Error::InvalidLei(format!(
            "LEI must be 20 alphanumeric characters: {}",
            lei
        )));
    }

    // TODO: Could add checksum validation here
    Ok(())
}

/// Validate a Business Identifier Code (BIC)
pub fn validate_bic(bic: &str) -> Result<()> {
    if !bic_regex().is_match(bic) {
        return Err(Error::InvalidBic(format!(
            "Invalid BIC format: {}",
            bic
        )));
    }

    Ok(())
}

/// Validate an amount string
pub fn validate_amount(amount: &str) -> Result<()> {
    if amount.is_empty() {
        return Err(Error::MissingRequiredField("Amount cannot be empty".to_string()));
    }

    // Check if it's a valid number
    if amount.parse::<f64>().is_err() {
        return Err(Error::ValidationFailed {
            issues: vec![format!("Invalid amount format: {}", amount)],
        });
    }

    Ok(())
}

/// Validate a datetime string (ISO 8601 format)
pub fn validate_datetime(datetime: &str) -> Result<()> {
    if chrono::DateTime::parse_from_rfc3339(datetime).is_err() {
        return Err(Error::InvalidDate(format!(
            "Invalid ISO 8601 datetime format: {}",
            datetime
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_country_code() {
        assert!(validate_country_code("US").is_ok());
        assert!(validate_country_code("GB").is_ok());
        assert!(validate_country_code("JP").is_ok());
        
        assert!(validate_country_code("usa").is_err());
        assert!(validate_country_code("U").is_err());
        assert!(validate_country_code("12").is_err());
    }

    #[test]
    fn test_validate_currency_code() {
        assert!(validate_currency_code("USD").is_ok());
        assert!(validate_currency_code("EUR").is_ok());
        assert!(validate_currency_code("JPY").is_ok());
        
        assert!(validate_currency_code("US").is_err());
        assert!(validate_currency_code("USDT").is_err());
        assert!(validate_currency_code("XXX").is_err()); // Not a real currency
    }

    #[test]
    fn test_validate_lei() {
        assert!(validate_lei("529900HNOAA1KXQJUQ27").is_ok());
        assert!(validate_lei("ABCDEFGHIJ1234567890").is_ok());
        
        assert!(validate_lei("529900HNOAA1KXQJUQ2").is_err()); // Too short
        assert!(validate_lei("529900HNOAA1KXQJUQ277").is_err()); // Too long
        assert!(validate_lei("529900hnoaa1kxqjuq27").is_err()); // Lowercase
    }

    #[test]
    fn test_validate_bic() {
        assert!(validate_bic("DEUTDEFF").is_ok()); // 8 chars
        assert!(validate_bic("DEUTDEFFXXX").is_ok()); // 11 chars
        
        assert!(validate_bic("DEUT").is_err()); // Too short
        assert!(validate_bic("DEUTDEFFXX").is_err()); // Wrong length
        assert!(validate_bic("deutdeff").is_err()); // Lowercase
    }

    #[test]
    fn test_validate_amount() {
        assert!(validate_amount("100").is_ok());
        assert!(validate_amount("100.50").is_ok());
        assert!(validate_amount("0.000001").is_ok());
        
        assert!(validate_amount("").is_err());
        assert!(validate_amount("abc").is_err());
        assert!(validate_amount("100.50.25").is_err());
    }

    #[test]
    fn test_validate_datetime() {
        assert!(validate_datetime("2024-01-15T10:30:00Z").is_ok());
        assert!(validate_datetime("2024-01-15T10:30:00+02:00").is_ok());
        
        assert!(validate_datetime("2024-01-15").is_err());
        assert!(validate_datetime("2024-01-15 10:30:00").is_err());
        assert!(validate_datetime("invalid").is_err());
    }
}