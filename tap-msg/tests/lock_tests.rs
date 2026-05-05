//! Integration tests for Lock and Capture messages (TAIP-17)
//!
//! TAIP-17 was renamed from "Escrow" to "Lock" while keeping the Capture
//! message and `EscrowAgent` role unchanged. These tests cover the new
//! primary name as well as backward compatibility for peers still sending
//! the legacy `https://tap.rsvp/schema/1.0#Escrow` URI.

use tap_msg::didcomm::PlainMessage;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{Agent, Capture, Lock, Party, TapMessage};

#[test]
fn test_lock_payment_guarantee_flow() {
    let customer = Party::new("did:eg:customer");
    let merchant = Party::new("did:web:merchant.example");

    let merchant_agent = Agent::new(
        "did:web:merchant.example",
        "MerchantAgent",
        "did:web:merchant.example",
    );
    let payment_processor = Agent::new(
        "did:web:paymentprocessor.example",
        "EscrowAgent",
        "did:web:paymentprocessor.example",
    );
    let customer_wallet = Agent::new(
        "did:web:customer.wallet",
        "CustomerAgent",
        "did:eg:customer",
    );

    let lock = Lock::new_with_asset(
        "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
        "100.00".to_string(),
        customer,
        merchant,
        "2025-06-25T00:00:00Z".to_string(),
        vec![merchant_agent, payment_processor, customer_wallet],
    )
    .with_agreement("https://merchant.example/order/12345/terms".to_string());

    assert!(lock.validate().is_ok());

    let escrow_agent = lock.escrow_agent().unwrap();
    assert_eq!(escrow_agent.role, Some("EscrowAgent".to_string()));
    assert_eq!(escrow_agent.id, "did:web:paymentprocessor.example");

    let capture = Capture::with_amount("95.00".to_string())
        .with_settlement_address("eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f1234".to_string());

    assert!(capture.validate().is_ok());
    assert_eq!(capture.amount, Some("95.00".to_string()));
}

#[test]
fn test_lock_asset_swap_flow() {
    let alice = Party::new("did:eg:alice");
    let bob = Party::new("did:eg:bob");

    let alice_wallet = Agent::new("did:web:alice.wallet", "AliceAgent", "did:eg:alice");
    let bob_wallet = Agent::new("did:web:bob.wallet", "BobAgent", "did:eg:bob");
    let swap_service = Agent::new(
        "did:web:swap.service",
        "EscrowAgent",
        "did:web:swap.service",
    );

    let alice_lock = Lock::new_with_asset(
        "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
        "100.00".to_string(),
        alice.clone(),
        bob.clone(),
        "2025-06-25T00:00:00Z".to_string(),
        vec![
            alice_wallet.clone(),
            bob_wallet.clone(),
            swap_service.clone(),
        ],
    )
    .with_agreement("https://swap.service/trades/abc123".to_string());

    assert!(alice_lock.validate().is_ok());

    let bob_lock = Lock::new_with_asset(
        "eip155:1/slip44:60".to_string(),
        "0.05".to_string(),
        bob,
        alice,
        "2025-06-25T00:00:00Z".to_string(),
        vec![alice_wallet, bob_wallet, swap_service],
    )
    .with_agreement("https://swap.service/trades/abc123".to_string());

    assert!(bob_lock.validate().is_ok());

    assert_eq!(
        alice_lock.escrow_agent().unwrap().id,
        bob_lock.escrow_agent().unwrap().id
    );
}

#[test]
fn test_lock_fiat_currency() {
    let buyer = Party::new("did:eg:buyer");
    let seller = Party::new("did:eg:seller");

    let marketplace = Agent::new(
        "did:web:marketplace.example",
        "MarketplaceAgent",
        "did:eg:seller",
    );
    let buyer_bank = Agent::new("did:web:buyer.bank", "BuyerAgent", "did:eg:buyer");
    let escrow_bank = Agent::new("did:web:escrow.bank", "EscrowAgent", "did:web:escrow.bank");

    let lock = Lock::new_with_currency(
        "USD".to_string(),
        "500.00".to_string(),
        buyer,
        seller,
        "2025-07-01T00:00:00Z".to_string(),
        vec![marketplace, buyer_bank, escrow_bank],
    )
    .with_agreement("https://marketplace.example/purchase/98765".to_string());

    assert!(lock.validate().is_ok());
    assert_eq!(lock.currency, Some("USD".to_string()));
    assert!(lock.asset.is_none());
}

#[test]
fn test_lock_serialization() {
    let originator = Party::new("did:example:alice");
    let beneficiary = Party::new("did:example:bob");
    let escrow_agent = Agent::new("did:example:escrow", "EscrowAgent", "did:example:escrow");

    let lock = Lock::new_with_asset(
        "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
        "100.00".to_string(),
        originator,
        beneficiary,
        "2025-06-25T00:00:00Z".to_string(),
        vec![escrow_agent],
    );

    let json = serde_json::to_value(&lock).unwrap();

    assert_eq!(
        json["asset"],
        "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
    );
    assert_eq!(json["amount"], "100.00");
    assert_eq!(json["originator"]["@id"], "did:example:alice");
    assert_eq!(json["beneficiary"]["@id"], "did:example:bob");
    assert_eq!(json["expiry"], "2025-06-25T00:00:00Z");
    assert_eq!(json["agents"][0]["@id"], "did:example:escrow");
    assert_eq!(json["agents"][0]["role"], "EscrowAgent");

    let deserialized: Lock = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.amount, lock.amount);
    assert_eq!(deserialized.asset, lock.asset);
}

#[test]
fn test_capture_serialization() {
    let capture = Capture::with_amount("95.00".to_string())
        .with_settlement_address("eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f1234".to_string());

    let json = serde_json::to_value(&capture).unwrap();

    assert_eq!(json["amount"], "95.00");
    assert_eq!(
        json["settlementAddress"],
        "eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f1234"
    );

    let deserialized: Capture = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.amount, capture.amount);
    assert_eq!(deserialized.settlement_address, capture.settlement_address);
}

#[test]
fn test_lock_to_plain_message_uses_new_uri() {
    let originator = Party::new("did:example:alice");
    let beneficiary = Party::new("did:example:bob");
    let escrow_agent = Agent::new("did:example:escrow", "EscrowAgent", "did:example:escrow");

    let lock = Lock::new_with_asset(
        "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
        "100.00".to_string(),
        originator,
        beneficiary,
        "2025-06-25T00:00:00Z".to_string(),
        vec![escrow_agent],
    );

    let plain_msg = lock.to_didcomm("did:example:sender").unwrap();

    assert_eq!(plain_msg.type_, "https://tap.rsvp/schema/1.0#Lock");
    assert_eq!(plain_msg.from, "did:example:sender");

    let body = plain_msg.body.as_object().unwrap();
    assert_eq!(body["@type"], "https://tap.rsvp/schema/1.0#Lock");
}

#[test]
fn test_message_type_returns_lock_uri() {
    assert_eq!(Lock::message_type(), "https://tap.rsvp/schema/1.0#Lock");
}

/// Backward compatibility: a wire message bearing the legacy `#Escrow` type
/// URI (sent by older peers) MUST still parse into a `TapMessage::Lock`.
#[test]
fn test_legacy_escrow_uri_dispatches_to_lock() {
    let body = serde_json::json!({
        "@context": "https://tap.rsvp/schema/1.0",
        "@type": "https://tap.rsvp/schema/1.0#Escrow",
        "asset": "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "amount": "100.00",
        "originator": {
            "@type": "Party",
            "@id": "did:example:alice"
        },
        "beneficiary": {
            "@type": "Party",
            "@id": "did:example:bob"
        },
        "expiry": "2025-06-25T00:00:00Z",
        "agents": [
            {
                "@type": "Agent",
                "@id": "did:example:escrow",
                "role": "EscrowAgent"
            }
        ]
    });

    let plain_msg = PlainMessage {
        id: "msg-1".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0#Escrow".to_string(),
        body,
        from: "did:example:sender".to_string(),
        to: vec!["did:example:recipient".to_string()],
        thid: None,
        pthid: None,
        extra_headers: Default::default(),
        created_time: None,
        expires_time: None,
        from_prior: None,
        attachments: None,
    };

    let parsed = TapMessage::from_plain_message(&plain_msg).expect("legacy URI must parse");
    match parsed {
        TapMessage::Lock(lock) => {
            assert_eq!(lock.amount, "100.00");
            assert_eq!(lock.originator.id, "did:example:alice");
        }
        other => panic!("expected TapMessage::Lock, got {:?}", other),
    }
}

/// New canonical URI also dispatches to `TapMessage::Lock`.
#[test]
fn test_new_lock_uri_dispatches_to_lock() {
    let body = serde_json::json!({
        "@context": "https://tap.rsvp/schema/1.0",
        "@type": "https://tap.rsvp/schema/1.0#Lock",
        "asset": "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "amount": "100.00",
        "originator": {
            "@type": "Party",
            "@id": "did:example:alice"
        },
        "beneficiary": {
            "@type": "Party",
            "@id": "did:example:bob"
        },
        "expiry": "2025-06-25T00:00:00Z",
        "agents": [
            {
                "@type": "Agent",
                "@id": "did:example:escrow",
                "role": "EscrowAgent"
            }
        ]
    });

    let plain_msg = PlainMessage {
        id: "msg-2".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0#Lock".to_string(),
        body,
        from: "did:example:sender".to_string(),
        to: vec!["did:example:recipient".to_string()],
        thid: None,
        pthid: None,
        extra_headers: Default::default(),
        created_time: None,
        expires_time: None,
        from_prior: None,
        attachments: None,
    };

    let parsed = TapMessage::from_plain_message(&plain_msg).expect("new URI must parse");
    assert!(matches!(parsed, TapMessage::Lock(_)));
}

/// `Escrow` is preserved as a type alias for `Lock` so existing
/// downstream code keeps compiling.
#[test]
fn test_escrow_alias_compiles_and_emits_new_uri() {
    use tap_msg::message::Escrow;

    assert_eq!(Escrow::message_type(), "https://tap.rsvp/schema/1.0#Lock");

    let originator = Party::new("did:example:alice");
    let beneficiary = Party::new("did:example:bob");
    let escrow_agent = Agent::new("did:example:escrow", "EscrowAgent", "did:example:escrow");

    let escrow = Escrow::new_with_asset(
        "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
        "100.00".to_string(),
        originator,
        beneficiary,
        "2025-06-25T00:00:00Z".to_string(),
        vec![escrow_agent],
    );

    let plain_msg = escrow.to_didcomm("did:example:sender").unwrap();
    assert_eq!(plain_msg.type_, "https://tap.rsvp/schema/1.0#Lock");
}
