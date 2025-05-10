//! Example demonstrating a TAIP-14 payment flow with cancellation
//!
//! This example shows how a merchant and customer can participate in a payment flow
//! where the customer decides to cancel the payment:
//! 1. Merchant agent initiates a payment request
//! 2. Customer agent receives the payment request
//! 3. Customer agent cancels the payment
//! 4. Merchant agent acknowledges the cancellation
//!
//! Run with: cargo run --example payment_cancel_flow

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use didcomm::secrets::{Secret, SecretMaterial, SecretType};
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{BasicSecretResolver, DefaultMessagePacker};
use tap_agent::did::{KeyResolver, MultiResolver};
use tap_caip::AssetId;
use tap_msg::message::types::Cancel;
use tap_msg::{Participant, PaymentRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio_test::block_on(async {
        println!("=== TAIP-14 Payment Flow with Cancellation ===\n");

        // Create merchant agent (Payment Service Provider - PSP)
        let (merchant_agent, merchant_did) = create_agent(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo",
            "nWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Yh",
        )
        .await;

        // Create customer agent (Customer Wallet)
        let (customer_agent, customer_did) = create_agent(
            "did:key:z6MkhFvVnYxkqLNEiWQmUwhQuVpXiCfNmRUVi5yZ4Cg9w15k",
            "8zYZK2vvsAyVYpNpnYzTnUPjBuWdWpYmPpQmwErV9XQg",
            "8zYZK2vvsAyVYpNpnYzTnUPjBuWdWpYmPpQmwErV9XQg",
        )
        .await;

        println!("Created merchant agent with DID: {}", merchant_did);
        println!("Created customer agent with DID: {}\n", customer_did);

        // Create a settlement address (in a real scenario, this would be a blockchain address)
        let settlement_address = "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb";

        // Step 1: Merchant creates and sends a payment request
        println!("Step 1: Merchant creates a payment request");

        // Generate a unique payment ID
        let payment_id = uuid::Uuid::new_v4().to_string();

        let payment = create_payment_message(&merchant_did, &customer_did, settlement_address);
        println!("Payment details:");
        println!("  Asset: {}", payment.asset.as_ref().unwrap());
        println!("  Amount: {}", payment.amount);
        println!("  Merchant: {}", payment.merchant.id);
        if let Some(customer) = &payment.customer {
            println!("  Customer: {}", customer.id);
        }
        println!();

        // Pack the payment message
        let packed_payment = merchant_agent.send_message(&payment, &customer_did).await?;
        println!("Merchant sends the payment request to the customer\n");

        // Step 2: Customer receives and processes the payment request
        println!("Step 2: Customer receives and processes the payment request");

        let received_payment: PaymentRequest =
            customer_agent.receive_message(&packed_payment).await?;
        println!("Customer received payment request:");
        println!("  Asset: {}", received_payment.asset.as_ref().unwrap());
        println!("  Amount: {}", received_payment.amount);
        println!("  Merchant: {}", received_payment.merchant.id);
        if let Some(customer) = &received_payment.customer {
            println!("  Customer: {}", customer.id);
        }
        println!();

        // Step 3: Customer decides to cancel the payment
        println!("Step 3: Customer decides to cancel the payment");

        let cancel = Cancel {
            transaction_id: payment_id.clone(),
            reason: Some("Changed my mind".to_string()),
            note: Some("Will consider purchasing at a later date".to_string()),
        };

        let packed_cancel = customer_agent.send_message(&cancel, &merchant_did).await?;
        println!("Customer sends cancellation to the merchant\n");

        // Step 4: Merchant receives the cancellation
        println!("Step 4: Merchant receives the cancellation");

        let received_cancel: Cancel = merchant_agent.receive_message(&packed_cancel).await?;
        println!("Merchant received cancellation:");
        println!("  Payment ID: {}", received_cancel.transaction_id);
        if let Some(reason) = &received_cancel.reason {
            println!("  Reason: {}", reason);
        }
        if let Some(note) = &received_cancel.note {
            println!("  Note: {}", note);
        }
        println!();

        // Step 5: Merchant acknowledges the cancellation (out of band)
        println!("Step 5: Merchant acknowledges the cancellation (out of band)");
        println!("  Merchant marks the payment as canceled in their system.\n");

        println!("=== Payment cancellation flow completed successfully ===");

        Ok(())
    })
}

/// Create an agent with the given DID and key material
async fn create_agent(
    did: &str,
    public_key: &str,
    private_key: &str,
) -> (Arc<DefaultAgent>, String) {
    // Create agent configuration
    let agent_config = AgentConfig::new(did.to_string());

    // Create DID resolver
    let mut did_resolver = MultiResolver::new();
    did_resolver.register_method("key", KeyResolver::new());
    let did_resolver = Arc::new(did_resolver);

    // Create secret resolver with the agent's key
    let mut secret_resolver = BasicSecretResolver::new();
    let secret = Secret {
        id: format!("{}#keys-1", did),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "x": public_key,
                "d": private_key
            }),
        },
    };

    secret_resolver.add_secret(did, secret);
    let secret_resolver = Arc::new(secret_resolver);

    // Create message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(did_resolver, secret_resolver));

    // Create agent
    let agent = Arc::new(DefaultAgent::new(agent_config, message_packer));

    (agent, did.to_string())
}

/// Create a payment message
fn create_payment_message(
    merchant_did: &str,
    customer_did: &str,
    settlement_address: &str,
) -> PaymentRequest {
    // Create merchant and customer participants
    let merchant = Participant {
        id: merchant_did.to_string(),
        role: Some("merchant".to_string()),
        policies: None,
        leiCode: None,
    };

    let customer = Participant {
        id: customer_did.to_string(),
        role: Some("customer".to_string()),
        policies: None,
        leiCode: None,
    };

    // Create settlement agent
    let settlement_agent = Participant {
        id: settlement_address.to_string(),
        role: Some("settlementAddress".to_string()),
        policies: None,
        leiCode: None,
    };

    // Create a payment message
    PaymentRequest {
        asset: Some(
            AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        ),
        currency: None,
        amount: "100.0".to_string(),
        supported_assets: None,
        invoice: None,
        expiry: None,
        merchant,
        customer: Some(customer),
        agents: vec![settlement_agent],
        metadata: HashMap::new(),
    }
}
