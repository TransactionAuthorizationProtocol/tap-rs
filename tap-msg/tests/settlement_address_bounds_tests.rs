//! Tests for settlement address bounds checking
//!
//! These tests verify that malformed settlement addresses do not cause panics.

use tap_msg::settlement_address::{PayToUri, SettlementAddress};

/// Test that PayToUri::new rejects short strings without panic
#[test]
fn test_payto_uri_short_string_no_panic() {
    let short_inputs = vec!["", "p", "pa", "pay", "payt", "payto", "payto:", "payto:/"];

    for input in short_inputs {
        let result = PayToUri::new(input.to_string());
        assert!(
            result.is_err(),
            "Input '{}' should return Err, not panic",
            input
        );
    }
}

/// Test that PayToUri::new validates minimum format
#[test]
fn test_payto_uri_missing_parts_no_panic() {
    let invalid_inputs = vec![
        "payto://",         // Missing method and account
        "payto://iban",     // Missing account (no slash)
        "payto://iban/",    // Empty account
        "payto:///account", // Empty method
    ];

    for input in invalid_inputs {
        let result = PayToUri::new(input.to_string());
        assert!(
            result.is_err(),
            "Input '{}' should return Err, not panic",
            input
        );
    }
}

/// Test that deserialized PayToUri with invalid data doesn't panic on method()
/// This is the critical test - serde can bypass new() validation
#[test]
fn test_payto_uri_deserialize_invalid_method_no_panic() {
    // Attempt to deserialize invalid URIs that would bypass new()
    // serde(transparent) means it deserializes directly to the inner String

    let invalid_json_values = vec![
        r#""""#,           // Empty string
        r#""short""#,      // Too short
        r#""payto""#,      // No scheme separator
        r#""payto:/""#,    // Incomplete scheme
        r#""http://x/y""#, // Wrong scheme
    ];

    for json in invalid_json_values {
        let result: Result<PayToUri, _> = serde_json::from_str(json);
        if let Ok(uri) = result {
            // If deserialization succeeds, method() must not panic
            let _ = uri.method(); // This should not panic
            let _ = uri.as_str(); // This should not panic either
        }
        // If deserialization fails, that's also acceptable
    }
}

/// Test SettlementAddress::new doesn't panic on malformed input
#[test]
fn test_settlement_address_malformed_no_panic() {
    let malformed = vec![
        "",
        "x",
        "payto",
        "payto://",
        "eip155",
        "eip155:",
        "eip155:1",
        "eip155:1:",
    ];

    for input in malformed {
        // from_string() should return Err or Ok, not panic
        let result = SettlementAddress::from_string(input.to_string());
        // We don't care if it's Ok or Err, just that it doesn't panic
        let _ = result;
    }
}

/// Test that valid PayToUri works correctly (sanity check)
#[test]
fn test_payto_uri_valid_works() {
    let valid = PayToUri::new("payto://iban/DE75512108001245126199".to_string());
    assert!(valid.is_ok());

    let uri = valid.unwrap();
    assert_eq!(uri.method(), "iban");
    assert_eq!(uri.as_str(), "payto://iban/DE75512108001245126199");
}
