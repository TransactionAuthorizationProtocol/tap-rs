extern crate tap_msg;

use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::types::{
    Authorizable, Participant, Payment, PaymentBuilder, Transfer, UpdateParty,
};

// Helper function to create a simple participant
fn create_participant(did: &str) -> Participant {
    Participant {
        id: did.to_string(),
        role: None,
        policies: None,
        leiCode: None,
    }
}

#[test]
fn test_create_message() {
    // Create a Transfer message
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    let originator = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let beneficiary = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    let body = Transfer {
        asset,
        originator: originator.clone(),
        beneficiary: Some(beneficiary.clone()),
        amount: "100000000".to_string(),
        agents: vec![originator, beneficiary],
        settlement_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Convert to DIDComm message
    let message = body
        .to_didcomm_with_route(
            Some("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"),
            ["did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6"]
                .iter()
                .copied(),
        )
        .unwrap();

    // Verify the message was created correctly
    assert!(!message.id.is_empty());
    assert_eq!(message.type_, "https://tap.rsvp/schema/1.0#transfer");
    assert!(message.created_time.is_some());
    assert_eq!(
        message.from.as_deref(),
        Some("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
    );
    assert_eq!(
        message.to,
        Some(vec![
            "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string()
        ])
    );
}

// --- Payment Tests Module ---
#[cfg(test)]
mod payment_tests {
    use super::*;
    use tap_msg::error::Error;

    fn create_valid_payment() -> Payment {
        let merchant_did = "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH";
        let customer_did = "did:key:z6MkhTBLxt9a7sWX77zn1GnzYam743kc9HvzA9qnKXqpVmXC";
        let asset_id_str = "eip155:1/slip44:60";

        PaymentBuilder::new()
            .payment_id("pay_123".to_string())
            .merchant(create_participant(merchant_did))
            .customer(create_participant(customer_did))
            .asset(AssetId::from_str(asset_id_str).unwrap())
            .amount("100.50".to_string())
            .build()
            .expect("Failed to build valid payment")
    }

    #[test]
    fn test_build_valid_payment() {
        let payment = create_valid_payment();
        assert_eq!(payment.payment_id, "pay_123");
        assert_eq!(payment.amount.parse::<f64>().unwrap(), 100.50);
    }

    #[test]
    fn test_payment_validation_failures() {
        let merchant_did = "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH";
        let customer_did = "did:key:z6MkhTBLxt9a7sWX77zn1GnzYam743kc9HvzA9qnKXqpVmXC";
        let asset_id_str = "eip155:1/slip44:60";

        // Missing payment_id
        let res = PaymentBuilder::new()
            .merchant(create_participant(merchant_did))
            .customer(create_participant(customer_did))
            .asset(AssetId::from_str(asset_id_str).unwrap())
            .amount("100".to_string())
            .build();
        assert!(matches!(res, Err(Error::Validation(_))));

        // Invalid amount (zero)
        let res = PaymentBuilder::new()
            .payment_id("pay_000".to_string())
            .merchant(create_participant(merchant_did))
            .customer(create_participant(customer_did))
            .asset(AssetId::from_str(asset_id_str).unwrap())
            .amount("0.00".to_string())
            .build();
        assert!(
            matches!(res.err().unwrap(), Error::Validation(msg) if msg == "Amount must be positive")
        );

        // Missing merchant
        let res = PaymentBuilder::new()
            .payment_id("pay_111".to_string())
            .customer(create_participant(customer_did))
            .asset(AssetId::from_str(asset_id_str).unwrap())
            .amount("50".to_string())
            .build();
        assert!(matches!(res, Err(Error::Validation(_))));
    }

    #[test]
    fn test_payment_to_didcomm() {
        let payment = create_valid_payment();
        let merchant_did = &payment.merchant.id;
        let customer_did = &payment.customer.id;

        let message_from_merchant = payment.to_didcomm(Some(merchant_did)).unwrap();

        assert_eq!(message_from_merchant.type_, Payment::message_type());
        assert_eq!(
            message_from_merchant.from.as_deref(),
            Some(merchant_did.as_str())
        );
        assert!(message_from_merchant.to.is_some());
        let recipients = message_from_merchant.to.unwrap();
        assert_eq!(recipients.len(), 1); // Only customer should be recipient
        assert!(recipients.contains(customer_did));
        assert!(!recipients.contains(merchant_did));

        let body: Payment = serde_json::from_value(message_from_merchant.body).unwrap();
        assert_eq!(body, payment);
    }

    #[test]
    fn test_payment_authorizable_trait() {
        let payment = create_valid_payment();
        let payment_id = payment.payment_id.clone();

        // Test authorize
        let authorize =
            payment.authorize(Some("Authorized via manual struct creation".to_string()));
        // Don't assert on transfer_id as it's generated from message_id()
        assert_eq!(
            authorize.note,
            Some("Authorized via manual struct creation".to_string())
        );

        // Test reject
        let reject = payment.reject("E001".to_string(), "Insufficient funds".to_string());
        // Don't assert on transfer_id as it's generated from message_id()
        assert_eq!(reject.reason, "E001: Insufficient funds");

        // Test settle
        let settle = payment.settle("tx-abc".to_string(), Some("100.0".to_string()));
        // Don't assert on transfer_id as it's now generated from message_id()
        assert_eq!(settle.settlement_id, "tx-abc".to_string());
        assert_eq!(settle.amount, Some("100.0".to_string()));

        // Test update party
        let updated_participant =
            Participant::new("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH");

        let update_party = UpdateParty {
            transfer_id: payment_id.clone(),
            party_type: "beneficiary".to_string(),
            party: updated_participant.clone(),
            note: Some("Updated via manual struct creation".to_string()),
            context: None,
        };
        assert_eq!(update_party.transfer_id, payment_id);
    }
}
