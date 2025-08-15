//! Example demonstrating payment link creation and processing using Out-of-Band messages
//!
//! This example shows how a merchant can:
//! 1. Create a payment request message
//! 2. Generate a payment link URL
//! 3. Customer can parse and process the payment link
//! 4. Customer can authorize and settle the payment
//!
//! Run with: cargo run --example payment_link_flow

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use serde_json::json;
use tap_agent::agent::{Agent, TapAgent};
use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
use tap_agent::config::AgentConfig;
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_agent::payment_link::PaymentLinkConfig;
use tap_caip::AssetId;
use tap_msg::message::payment::InvoiceReference;
use tap_msg::message::{Agent as TapAgent_, Authorize, Party, Payment, Settle};
use tap_msg::settlement_address::SettlementAddress;

async fn create_agent(did: &str, private_key: &str, public_key: &str) -> (TapAgent, String) {
    let secret = Secret {
        id: format!("{}#key-1", did),
        secret_material: SecretMaterial::JWK {
            private_key_jwk: json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "x": public_key,
                "d": private_key
            }),
        },
        type_: SecretType::JsonWebKey2020,
    };

    let key_manager = AgentKeyManagerBuilder::new()
        .add_secret(secret.id.clone(), secret)
        .build()
        .expect("Failed to create key manager");

    let config = AgentConfig::new(did.to_string());

    let agent = TapAgent::new(config, Arc::new(key_manager));
    let agent_did = agent.get_agent_did().to_string();

    (agent, agent_did)
}

fn create_payment_message(
    merchant_did: &str,
    customer_did: &str,
    settlement_address: &str,
    transaction_id: &str,
) -> Payment {
    Payment {
        asset: Some(
            AssetId::from_str("eip155:1/erc20:0xA0b86991c53D94fa4C0bCBf0C1C4DF2F15F1b7A8").unwrap(),
        ), // USDC
        currency_code: None,
        amount: "250.00".to_string(),
        invoice: Some(InvoiceReference::Url(
            "https://merchant.example/invoice/12345".to_string(),
        )),
        expiry: Some("2025-12-31T23:59:59Z".to_string()),
        memo: Some("Payment for premium service subscription".to_string()),

        customer: Some(Party::new(customer_did)),

        merchant: Party::new(merchant_did),

        agents: vec![TapAgent_::new(
            merchant_did,
            "paymentProcessor",
            merchant_did,
        )],

        supported_assets: Some(vec![
            AssetId::from_str("eip155:1/erc20:0xA0b86991c53D94fa4C0bCBf0C1C4DF2F15F1b7A8").unwrap(), // USDC Ethereum
            AssetId::from_str("eip155:137/erc20:0x2791Bca1f2de4661ED88A30C2A8A6b5E7C54fD3A")
                .unwrap(), // USDC Polygon
        ]),

        fallback_settlement_addresses: Some(vec![
            SettlementAddress::from_string(format!("eip155:1:{}", settlement_address)).unwrap(),
            SettlementAddress::from_string(format!("eip155:137:{}", settlement_address)).unwrap(),
        ]),

        metadata: {
            let mut meta = HashMap::new();
            meta.insert("order_id".to_string(), json!("ORD-2024-12345"));
            meta.insert("subscription_type".to_string(), json!("premium"));
            meta
        },
        transaction_id: Some(transaction_id.to_string()),
        connection_id: None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Payment Link Flow Example ===\n");

    // Create merchant agent (Payment Service Provider)
    let (merchant_agent, merchant_did) = create_agent(
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo",
        "nWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Yh",
    )
    .await;

    // Create customer agent (Wallet)
    let (customer_agent, customer_did) = create_agent(
        "did:key:z6MkhFvVnYxkqLNEiWQmUwhQuVpXiCfNmRUVi5yZ4Cg9w15k",
        "8zYZK2vvsAyVYpNpnYzTnUPjBuWdWpYmPpQmwErV9XQg",
        "8zYZK2vvsAyVYpNpnYzTnUPjBuWdWpYmPpQmwErV9XQg",
    )
    .await;

    println!("Created merchant agent with DID: {}", merchant_did);
    println!("Created customer agent with DID: {}\n", customer_did);

    // Settlement address (in a real scenario, this would be a blockchain address controlled by the merchant)
    let settlement_address = "0x742d35Cc6651C0532925a3b8D563d1a6f4e7F5BB";
    let transaction_id = uuid::Uuid::new_v4().to_string();

    // Step 1: Merchant creates a payment request
    println!("Step 1: Merchant creates a payment request");
    let payment = create_payment_message(
        &merchant_did,
        &customer_did,
        settlement_address,
        &transaction_id,
    );

    println!("Payment details:");
    println!("  Asset: {}", payment.asset.as_ref().unwrap());
    println!("  Amount: {} USDC", payment.amount);
    println!(
        "  Merchant: {}",
        payment.merchant.name().unwrap_or("Unknown")
    );
    println!(
        "  Customer: {}",
        payment
            .customer
            .as_ref()
            .unwrap()
            .name()
            .unwrap_or("Unknown")
    );
    if let Some(invoice) = &payment.invoice {
        println!("  Invoice: {:?}", invoice);
    }
    println!();

    // Step 2: Merchant creates a payment link
    println!("Step 2: Merchant creates a payment link");

    let config = PaymentLinkConfig::new()
        .with_service_url("https://pay.example.com/checkout")
        .with_metadata("theme", json!("dark"))
        .with_metadata("return_url", json!("https://merchant.example/success"))
        .with_goal("Complete your purchase for premium subscription");

    let payment_link_url = merchant_agent
        .create_payment_link(&payment, Some(config))
        .await?;

    println!("Generated payment link:");
    println!("  URL: {}", payment_link_url);
    println!("  QR Code data: {}", payment_link_url);
    println!();

    // Step 3: Customer receives and processes the payment link
    println!("Step 3: Customer processes the payment link");

    // Parse the OOB invitation from the URL
    let oob_invitation = customer_agent.parse_oob_invitation(&payment_link_url)?;

    println!("Parsed Out-of-Band invitation:");
    println!("  From: {}", oob_invitation.from);
    println!("  Goal: {}", oob_invitation.body.goal);
    println!("  Goal Code: {}", oob_invitation.body.goal_code);
    println!();

    // Process the invitation to extract the payment message
    let plain_message = customer_agent
        .process_oob_invitation(&oob_invitation)
        .await?;

    // Parse the payment from the message body
    let received_payment: Payment = serde_json::from_value(plain_message.body)?;

    println!("Customer received payment request:");
    println!("  Asset: {}", received_payment.asset.as_ref().unwrap());
    println!("  Amount: {} USDC", received_payment.amount);
    println!(
        "  Merchant: {}",
        received_payment.merchant.name().unwrap_or("Unknown")
    );
    if let Some(memo) = &received_payment.memo {
        println!("  Memo: {}", memo);
    }
    println!();

    // Step 4: Customer authorizes the payment
    println!("Step 4: Customer authorizes the payment");

    let authorize = Authorize {
        transaction_id: plain_message
            .thid
            .unwrap_or_else(|| plain_message.id.clone()),
        settlement_address: Some(format!("eip155:1:{}", settlement_address)),
        expiry: None,
    };

    let (packed_authorize, _delivery_results) = customer_agent
        .send_message(&authorize, vec![&merchant_did], false)
        .await?;

    println!("Customer sent authorization message");

    // Merchant receives and processes the authorization
    let auth_message = merchant_agent.receive_message(&packed_authorize).await?;
    let received_auth: Authorize = serde_json::from_value(auth_message.body)?;

    println!("Merchant received authorization:");
    println!("  Transaction ID: {}", received_auth.transaction_id);
    if let Some(addr) = &received_auth.settlement_address {
        println!("  Settlement Address: {}", addr);
    }
    println!();

    // Step 5: Customer settles the payment (simulates on-chain transaction)
    println!("Step 5: Customer settles the payment");

    let settle = Settle {
        transaction_id: received_auth.transaction_id.clone(),
        amount: Some(received_payment.amount.clone()),
        settlement_id: Some(
            "0x1234567890abcdef1234567890abcdef12345678901234567890abcdef123456".to_string(),
        ),
    };

    let (packed_settle, _delivery_results) = customer_agent
        .send_message(&settle, vec![&merchant_did], false)
        .await?;

    println!("Customer sent settlement message");

    // Merchant receives settlement confirmation
    let settle_message = merchant_agent.receive_message(&packed_settle).await?;
    let received_settle: Settle = serde_json::from_value(settle_message.body)?;

    println!("Merchant received settlement confirmation:");
    println!("  Transaction ID: {}", received_settle.transaction_id);
    if let Some(amount) = &received_settle.amount {
        println!("  Amount: {} USDC", amount);
    }
    if let Some(settlement_id) = &received_settle.settlement_id {
        println!("  Settlement ID: {}", settlement_id);
    }
    println!();

    println!("🎉 Payment flow completed successfully!");
    println!(
        "The customer has successfully paid {} USDC to the merchant.",
        received_payment.amount
    );

    // Step 6: Demonstrate short link creation
    println!("\nStep 6: Creating short payment link");

    // For demonstration, we'll show how to create a short link using the invitation ID
    let short_url = format!("https://pay.example.com/p?id={}", oob_invitation.id);
    println!("Short payment link: {}", short_url);
    println!("(This would require the payment service to store the OOB invitation by ID)");

    Ok(())
}
