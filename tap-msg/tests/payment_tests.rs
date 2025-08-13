//! Tests for Payment messages with fallback settlement addresses

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use tap_caip::AssetId;
    use tap_msg::message::Party;
    use tap_msg::settlement_address::SettlementAddress;
    use tap_msg::Payment;

    #[test]
    fn test_payment_with_fallback_settlement_addresses() {
        let payment = Payment::builder()
            .currency_code("USD".to_string())
            .amount("100.00".to_string())
            .merchant(Party::new("did:web:merchant.example"))
            .add_fallback_settlement_address(
                SettlementAddress::from_string("payto://iban/DE75512108001245126199".to_string())
                    .unwrap(),
            )
            .add_fallback_settlement_address(
                SettlementAddress::from_string(
                    "eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
                )
                .unwrap(),
            )
            .build();

        assert!(payment.validate().is_ok());
        assert!(payment.fallback_settlement_addresses.is_some());

        let addresses = payment.fallback_settlement_addresses.as_ref().unwrap();
        assert_eq!(addresses.len(), 2);
        assert!(addresses[0].is_traditional());
        assert!(addresses[1].is_blockchain());
    }

    #[test]
    fn test_payment_serialization_with_fallback_addresses() {
        let payment = Payment::builder()
            .currency_code("EUR".to_string())
            .amount("500.00".to_string())
            .merchant(Party::new("did:web:merchant.example"))
            .fallback_settlement_addresses(vec![
                SettlementAddress::from_string("payto://iban/GB33BUKB20201555555555".to_string())
                    .unwrap(),
                SettlementAddress::from_string("payto://ach/122000247/111000025".to_string())
                    .unwrap(),
            ])
            .build();

        let json = serde_json::to_value(&payment).unwrap();

        // Check that fallback addresses are serialized correctly with camelCase
        assert!(json["fallbackSettlementAddresses"].is_array());
        let fallback_addrs = json["fallbackSettlementAddresses"].as_array().unwrap();
        assert_eq!(fallback_addrs.len(), 2);
        assert_eq!(fallback_addrs[0], "payto://iban/GB33BUKB20201555555555");
        assert_eq!(fallback_addrs[1], "payto://ach/122000247/111000025");

        // Deserialize and verify
        let deserialized: Payment = serde_json::from_value(json).unwrap();
        assert!(deserialized.fallback_settlement_addresses.is_some());
        assert_eq!(deserialized.fallback_settlement_addresses.unwrap().len(), 2);
    }

    #[test]
    fn test_payment_with_mixed_settlement_types() {
        let payment = Payment::builder()
            .asset(
                AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
                    .unwrap(),
            )
            .amount("1000.00".to_string())
            .merchant(Party::new("did:web:merchant.example"))
            .fallback_settlement_addresses(vec![
                SettlementAddress::from_string(
                    "eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
                )
                .unwrap(),
                SettlementAddress::from_string("payto://upi/9999999999@paytm".to_string()).unwrap(),
                SettlementAddress::from_string("payto://bic/SOGEDEFFXXX".to_string()).unwrap(),
            ])
            .build();

        assert!(payment.validate().is_ok());

        let addresses = payment.fallback_settlement_addresses.as_ref().unwrap();
        assert_eq!(addresses.len(), 3);
        assert!(addresses[0].is_blockchain());
        assert!(addresses[1].is_traditional());
        assert!(addresses[2].is_traditional());

        // Test that the PayTo URIs have correct methods
        if let SettlementAddress::PayTo(uri) = &addresses[1] {
            assert_eq!(uri.method(), "upi");
        }
        if let SettlementAddress::PayTo(uri) = &addresses[2] {
            assert_eq!(uri.method(), "bic");
        }
    }

    #[test]
    fn test_payment_without_fallback_addresses() {
        let payment = Payment::builder()
            .currency_code("USD".to_string())
            .amount("50.00".to_string())
            .merchant(Party::new("did:web:merchant.example"))
            .build();

        assert!(payment.validate().is_ok());
        assert!(payment.fallback_settlement_addresses.is_none());

        // Verify it's not serialized when None
        let json = serde_json::to_value(&payment).unwrap();
        assert!(!json
            .as_object()
            .unwrap()
            .contains_key("fallbackSettlementAddresses"));
    }
}
