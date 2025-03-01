use caip::{AccountId, AssetId, ChainId};
use proptest::prelude::*;
use std::str::FromStr;

// Strategy for generating valid namespace strings
fn namespace_strategy() -> impl Strategy<Value = String> {
    "[a-z0-9]{3,8}".prop_map(|s| s)
}

// Strategy for generating valid reference strings
fn reference_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{1,32}".prop_map(|s| s)
}

// Strategy for generating valid address strings
fn address_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{1,64}".prop_map(|s| s)
}

// Strategy for generating valid chain IDs
fn chain_id_strategy() -> impl Strategy<Value = String> {
    (namespace_strategy(), reference_strategy()).prop_map(|(ns, ref_str)| {
        format!("{}:{}", ns, ref_str)
    })
}

// Strategy for generating valid account IDs
fn account_id_strategy() -> impl Strategy<Value = String> {
    (chain_id_strategy(), address_strategy()).prop_map(|(chain_id, addr)| {
        format!("{}:{}", chain_id, addr)
    })
}

// Strategy for generating valid asset IDs
fn asset_id_strategy() -> impl Strategy<Value = String> {
    (chain_id_strategy(), namespace_strategy(), reference_strategy()).prop_map(|(chain_id, ns, ref_str)| {
        format!("{}/{}:{}", chain_id, ns, ref_str)
    })
}

// Strategies for generating invalid strings
fn invalid_namespace_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec("[^a-z0-9]|[A-Z]", 1..10).prop_map(|v| v.into_iter().collect())
}

fn invalid_reference_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(".", 0..=0).prop_map(|_| String::new()) // Empty string
}

proptest! {
    // Test that valid chain IDs parse correctly
    #[test]
    fn test_valid_chain_id_parsing(chain_id in chain_id_strategy()) {
        let parsed = ChainId::from_str(&chain_id);
        prop_assert!(parsed.is_ok());
        
        // If parsing succeeded, ensure the string representation matches
        if let Ok(parsed_id) = parsed {
            prop_assert_eq!(parsed_id.to_string(), chain_id);
        }
    }

    // Test that valid account IDs parse correctly
    #[test]
    fn test_valid_account_id_parsing(account_id in account_id_strategy()) {
        let parsed = AccountId::from_str(&account_id);
        prop_assert!(parsed.is_ok());
        
        // If parsing succeeded, ensure the string representation matches
        if let Ok(parsed_id) = parsed {
            prop_assert_eq!(parsed_id.to_string(), account_id);
        }
    }

    // Test that valid asset IDs parse correctly
    #[test]
    fn test_valid_asset_id_parsing(asset_id in asset_id_strategy()) {
        let parsed = AssetId::from_str(&asset_id);
        prop_assert!(parsed.is_ok());
        
        // If parsing succeeded, ensure the string representation matches
        if let Ok(parsed_id) = parsed {
            prop_assert_eq!(parsed_id.to_string(), asset_id);
        }
    }

    // Test that invalid chain IDs are properly rejected
    #[test]
    fn test_invalid_chain_id_rejection(
        ns in invalid_namespace_strategy(),
        ref_str in invalid_reference_strategy()
    ) {
        let chain_id = format!("{}:{}", ns, ref_str);
        let parsed = ChainId::from_str(&chain_id);
        prop_assert!(parsed.is_err());
    }

    // Test that invalid account IDs are properly rejected
    #[test]
    fn test_invalid_account_id_rejection(
        ns in invalid_namespace_strategy(),
        ref_str in reference_strategy(),
        addr in address_strategy()
    ) {
        let chain_id = format!("{}:{}", ns, ref_str);
        let account_id = format!("{}:{}", chain_id, addr);
        let parsed = AccountId::from_str(&account_id);
        prop_assert!(parsed.is_err());
    }

    // Test that invalid asset IDs are properly rejected
    #[test]
    fn test_invalid_asset_id_rejection(
        chain_id in chain_id_strategy(),
        ns in invalid_namespace_strategy(),
        ref_str in reference_strategy()
    ) {
        let asset_id = format!("{}/{}:{}", chain_id, ns, ref_str);
        let parsed = AssetId::from_str(&asset_id);
        prop_assert!(parsed.is_err());
    }

    // Test CaipId enum parsing with various CAIP identifiers
    #[test]
    fn test_caip_id_parsing(
        chain_id in chain_id_strategy(),
        account_id in account_id_strategy(),
        asset_id in asset_id_strategy()
    ) {
        let parsed_chain = caip::parse(&chain_id);
        prop_assert!(parsed_chain.is_ok());
        
        let parsed_account = caip::parse(&account_id);
        prop_assert!(parsed_account.is_ok());
        
        let parsed_asset = caip::parse(&asset_id);
        prop_assert!(parsed_asset.is_ok());
    }

    // Test robustness against entirely random strings
    #[test]
    fn test_robustness_against_random_strings(s in ".*") {
        // These shouldn't panic, just return errors for invalid inputs
        let _ = ChainId::from_str(&s);
        let _ = AccountId::from_str(&s);
        let _ = AssetId::from_str(&s);
        let _ = caip::parse(&s);
    }
}

// Additional stress tests with unusual but potentially valid inputs
#[test]
fn test_edge_case_chain_ids() {
    // Test minimum length namespace and reference
    let min_chain_id = "abc:1";
    assert!(ChainId::from_str(min_chain_id).is_ok());

    // Test maximum length namespace and reference
    let max_chain_id = "abcdefgh:".to_string() + &"A".repeat(64);
    assert!(ChainId::from_str(&max_chain_id).is_ok());
}

#[test]
fn test_edge_case_account_ids() {
    // Test with minimum length components
    let min_account_id = "abc:1:1";
    assert!(AccountId::from_str(min_account_id).is_ok());

    // Test with maximum length components
    let max_namespace = "abcdefgh";
    let max_reference = "A".repeat(32);
    let max_address = "B".repeat(64);
    let max_account_id = format!("{}:{}:{}", max_namespace, max_reference, max_address);
    assert!(AccountId::from_str(&max_account_id).is_ok());
}

#[test]
fn test_edge_case_asset_ids() {
    // Test with minimum length components
    let min_asset_id = "abc:1/def:1";
    assert!(AssetId::from_str(min_asset_id).is_ok());

    // Test with maximum length components
    let max_namespace1 = "abcdefgh";
    let max_reference1 = "A".repeat(64);
    let max_namespace2 = "abcdefgh";
    let max_reference2 = "B".repeat(64);
    let max_asset_id = format!("{}:{}/{}:{}", max_namespace1, max_reference1, max_namespace2, max_reference2);
    assert!(AssetId::from_str(&max_asset_id).is_ok());
}
