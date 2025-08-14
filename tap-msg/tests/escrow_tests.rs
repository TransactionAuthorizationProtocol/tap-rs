//! Integration tests for Escrow and Capture messages (TAIP-17)

use serde_json::json;
use tap_msg::didcomm::PlainMessage;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{Agent, Capture, Escrow, Party};

#[test]
fn test_escrow_payment_guarantee_flow() {
    // Create parties
    let customer = Party::new("did:eg:customer");
    let merchant = Party::new("did:web:merchant.example");

    // Create agents
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

    // Create escrow request for payment guarantee
    let escrow = Escrow::new_with_asset(
        "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(),
        "100.00".to_string(),
        customer,
        merchant,
        "2025-06-25T00:00:00Z".to_string(),
        vec![merchant_agent, payment_processor, customer_wallet],
    )
    .with_agreement("https://merchant.example/order/12345/terms".to_string());

    // Validate escrow message
    assert!(escrow.validate().is_ok());

    // Check escrow agent
    let escrow_agent = escrow.escrow_agent().unwrap();
    assert_eq!(escrow_agent.role, Some("EscrowAgent".to_string()));
    assert_eq!(escrow_agent.id, "did:web:paymentprocessor.example");

    // Create capture message
    let capture = Capture::with_amount("95.00".to_string())
        .with_settlement_address("eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f1234".to_string());

    assert!(capture.validate().is_ok());
    assert_eq!(capture.amount, Some("95.00".to_string()));
}

#[test]
fn test_escrow_asset_swap_flow() {
    // Alice and Bob want to swap assets
    let alice = Party::new("did:eg:alice");
    let bob = Party::new("did:eg:bob");

    // Create agents
    let alice_wallet = Agent::new("did:web:alice.wallet", "AliceAgent", "did:eg:alice");
    let bob_wallet = Agent::new("did:web:bob.wallet", "BobAgent", "did:eg:bob");
    let swap_service = Agent::new(
        "did:web:swap.service",
        "EscrowAgent",
        "did:web:swap.service",
    );

    // Alice creates escrow with Bob as beneficiary (100 USDC)
    let alice_escrow = Escrow::new_with_asset(
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

    assert!(alice_escrow.validate().is_ok());

    // Bob creates escrow with Alice as beneficiary (0.05 ETH)
    let bob_escrow = Escrow::new_with_asset(
        "eip155:1/slip44:60".to_string(),
        "0.05".to_string(),
        bob,
        alice,
        "2025-06-25T00:00:00Z".to_string(),
        vec![alice_wallet, bob_wallet, swap_service],
    )
    .with_agreement("https://swap.service/trades/abc123".to_string());

    assert!(bob_escrow.validate().is_ok());

    // Both escrows have the same escrow agent
    assert_eq!(
        alice_escrow.escrow_agent().unwrap().id,
        bob_escrow.escrow_agent().unwrap().id
    );
}

#[test]
fn test_escrow_fiat_currency() {
    let buyer = Party::new("did:eg:buyer");
    let seller = Party::new("did:eg:seller");

    let marketplace = Agent::new(
        "did:web:marketplace.example",
        "MarketplaceAgent",
        "did:eg:seller",
    );
    let buyer_bank = Agent::new("did:web:buyer.bank", "BuyerAgent", "did:eg:buyer");
    let escrow_bank = Agent::new("did:web:escrow.bank", "EscrowAgent", "did:web:escrow.bank");

    // Create fiat currency escrow
    let escrow = Escrow::new_with_currency(
        "USD".to_string(),
        "500.00".to_string(),
        buyer,
        seller,
        "2025-07-01T00:00:00Z".to_string(),
        vec![marketplace, buyer_bank, escrow_bank],
    )
    .with_agreement("https://marketplace.example/purchase/98765".to_string());

    assert!(escrow.validate().is_ok());
    assert_eq!(escrow.currency, Some("USD".to_string()));
    assert!(escrow.asset.is_none());
}

#[test]
fn test_escrow_serialization() {
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

    // Serialize to JSON
    let json = serde_json::to_value(&escrow).unwrap();

    // Check required fields
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

    // Deserialize back
    let deserialized: Escrow = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.amount, escrow.amount);
    assert_eq!(deserialized.asset, escrow.asset);
}

#[test]
fn test_capture_serialization() {
    let capture = Capture::with_amount("95.00".to_string())
        .with_settlement_address("eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f1234".to_string());

    // Serialize to JSON
    let json = serde_json::to_value(&capture).unwrap();

    assert_eq!(json["amount"], "95.00");
    assert_eq!(
        json["settlementAddress"],
        "eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f1234"
    );

    // Deserialize back
    let deserialized: Capture = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.amount, capture.amount);
    assert_eq!(deserialized.settlement_address, capture.settlement_address);
}

#[test]
fn test_escrow_to_plain_message() {
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

    // Convert to DIDComm message
    let plain_msg = escrow.to_didcomm("did:example:sender").unwrap();

    assert_eq!(plain_msg.type_, "https://tap.rsvp/schema/1.0#Escrow");
    assert_eq!(plain_msg.from, "did:example:sender");

    // Check body contains @type field
    let body = plain_msg.body.as_object().unwrap();
    assert_eq!(body["@type"], "https://tap.rsvp/schema/1.0#Escrow");
}
