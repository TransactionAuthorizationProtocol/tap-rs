//! Integration tests for RFQ and Quote messages (TAIP-18)
//!
//! TAIP-18 was renamed from "Exchange" to "RFQ" while keeping the Quote
//! message and the body shape unchanged. These tests cover the new
//! primary name as well as backward compatibility for peers still
//! sending the legacy `https://tap.rsvp/schema/1.0#Exchange` URI.

use tap_msg::didcomm::PlainMessage;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{Agent, Party, Quote, Rfq, TapMessage};

#[test]
fn test_rfq_creation_and_validation() {
    let rfq = Rfq::new_from(
        vec!["eip155:1/erc20:0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()],
        vec!["eip155:1/erc20:0xB00b00b00b00b00b00b00b00b00b00b00b00b00b".to_string()],
        "1000.00".to_string(),
        Party::new("did:web:business.example"),
        vec![Agent::new_without_role(
            "did:web:wallet.example",
            "did:web:business.example",
        )],
    )
    .with_provider(Party::new("did:web:liquidity.provider"));

    assert_eq!(rfq.from_assets.len(), 1);
    assert_eq!(rfq.to_assets.len(), 1);
    assert_eq!(rfq.from_amount, Some("1000.00".to_string()));
    assert!(rfq.to_amount.is_none());
    assert!(rfq.provider.is_some());
    assert!(rfq.validate().is_ok());
}

#[test]
fn test_rfq_to_amount_variant() {
    let rfq = Rfq::new_to(
        vec!["USD".to_string(), "EUR".to_string()],
        vec!["eip155:1/erc20:0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()],
        "500.00".to_string(),
        Party::new("did:web:user.entity"),
        vec![Agent::new_without_role(
            "did:web:user.wallet",
            "did:web:user.entity",
        )],
    );

    assert!(rfq.from_amount.is_none());
    assert_eq!(rfq.to_amount, Some("500.00".to_string()));
    assert_eq!(rfq.from_assets.len(), 2);
    assert!(rfq.validate().is_ok());
}

#[test]
fn test_rfq_message_type_returns_new_uri() {
    assert_eq!(Rfq::message_type(), "https://tap.rsvp/schema/1.0#RFQ");
}

#[test]
fn test_rfq_to_plain_message_uses_new_uri() {
    let rfq = Rfq::new_from(
        vec!["eip155:1/erc20:0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()],
        vec!["eip155:1/erc20:0xB00b00b00b00b00b00b00b00b00b00b00b00b00b".to_string()],
        "1000.00".to_string(),
        Party::new("did:web:business.example"),
        vec![Agent::new_without_role(
            "did:web:wallet.example",
            "did:web:business.example",
        )],
    );

    let plain_msg = rfq.to_didcomm("did:example:sender").unwrap();
    assert_eq!(plain_msg.type_, "https://tap.rsvp/schema/1.0#RFQ");
}

#[test]
fn test_quote_creation_and_validation() {
    let quote = Quote::new(
        "eip155:1/erc20:0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
        "eip155:1/erc20:0xB00b00b00b00b00b00b00b00b00b00b00b00b00b".to_string(),
        "1000.00".to_string(),
        "950.00".to_string(),
        Party::new("did:web:liquidity.provider"),
        vec![],
        "2026-01-01T00:00:00Z".to_string(),
    );

    assert_eq!(quote.from_amount, "1000.00");
    assert_eq!(quote.to_amount, "950.00");
    assert!(quote.validate().is_ok());
}

#[test]
fn test_quote_message_type_unchanged() {
    assert_eq!(Quote::message_type(), "https://tap.rsvp/schema/1.0#Quote");
}

/// Backward compatibility: a wire message bearing the legacy `#Exchange`
/// type URI MUST still parse into a `TapMessage::Rfq`.
#[test]
fn test_legacy_exchange_uri_dispatches_to_rfq() {
    let body = serde_json::json!({
        "@context": "https://tap.rsvp/schema/1.0",
        "@type": "https://tap.rsvp/schema/1.0#Exchange",
        "fromAssets": ["eip155:1/erc20:0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"],
        "toAssets": ["eip155:1/erc20:0xB00b00b00b00b00b00b00b00b00b00b00b00b00b"],
        "fromAmount": "1000.00",
        "requester": {
            "@type": "Party",
            "@id": "did:web:business.example"
        },
        "agents": [
            {
                "@type": "Agent",
                "@id": "did:web:wallet.example",
                "for": "did:web:business.example"
            }
        ]
    });

    let plain_msg = PlainMessage {
        id: "rfq-legacy-1".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0#Exchange".to_string(),
        body,
        from: "did:example:sender".to_string(),
        to: vec!["did:web:liquidity.provider".to_string()],
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
        TapMessage::Rfq(rfq) => {
            assert_eq!(rfq.from_amount, Some("1000.00".to_string()));
            assert_eq!(rfq.requester.id, "did:web:business.example");
        }
        other => panic!("expected TapMessage::Rfq, got {:?}", other),
    }
}

/// New canonical URI also dispatches to `TapMessage::Rfq`.
#[test]
fn test_new_rfq_uri_dispatches_to_rfq() {
    let body = serde_json::json!({
        "@context": "https://tap.rsvp/schema/1.0",
        "@type": "https://tap.rsvp/schema/1.0#RFQ",
        "fromAssets": ["eip155:1/erc20:0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"],
        "toAssets": ["eip155:1/erc20:0xB00b00b00b00b00b00b00b00b00b00b00b00b00b"],
        "fromAmount": "1000.00",
        "requester": {
            "@type": "Party",
            "@id": "did:web:business.example"
        },
        "agents": [
            {
                "@type": "Agent",
                "@id": "did:web:wallet.example",
                "for": "did:web:business.example"
            }
        ]
    });

    let plain_msg = PlainMessage {
        id: "rfq-new-1".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0#RFQ".to_string(),
        body,
        from: "did:example:sender".to_string(),
        to: vec!["did:web:liquidity.provider".to_string()],
        thid: None,
        pthid: None,
        extra_headers: Default::default(),
        created_time: None,
        expires_time: None,
        from_prior: None,
        attachments: None,
    };

    let parsed = TapMessage::from_plain_message(&plain_msg).expect("new URI must parse");
    assert!(matches!(parsed, TapMessage::Rfq(_)));
}

/// `Exchange` is preserved as a type alias for `Rfq`.
#[test]
fn test_exchange_alias_compiles_and_emits_new_uri() {
    use tap_msg::message::Exchange;

    assert_eq!(Exchange::message_type(), "https://tap.rsvp/schema/1.0#RFQ");

    let exchange = Exchange::new_from(
        vec!["USD".to_string()],
        vec!["eip155:1/erc20:0xA0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()],
        "1000.00".to_string(),
        Party::new("did:web:user.entity"),
        vec![Agent::new_without_role(
            "did:web:user.wallet",
            "did:web:user.entity",
        )],
    );

    let plain_msg = exchange.to_didcomm("did:example:sender").unwrap();
    assert_eq!(plain_msg.type_, "https://tap.rsvp/schema/1.0#RFQ");
}
