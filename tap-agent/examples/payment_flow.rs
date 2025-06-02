//! Example demonstrating a complete TAIP-14 payment flow with TAIP-4 authorization
//!
//! This example shows how a merchant and customer can participate in a payment flow:
//! 1. Merchant agent initiates a payment request
//! 2. Customer agent authorizes the payment
//! 3. Customer agent settles the payment
//! 4. Merchant agent completes the payment
//!
//! Run with: cargo run --example payment_flow

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use tap_agent::agent::{Agent, TapAgent};
use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
use tap_agent::config::AgentConfig;
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_caip::AssetId;
use tap_msg::message::{Authorize, Settle};
use tap_msg::{Party, Payment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio_test::block_on(async {
        println!("=== TAIP-14 Payment Flow with TAIP-4 Authorization ===\n");

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

        // Generate a unique transaction ID
        let transaction_id = uuid::Uuid::new_v4().to_string();

        let payment = create_payment_message(
            &merchant_did,
            &customer_did,
            settlement_address,
            &transaction_id,
        );
        println!("Payment details:");
        println!("  Asset: {}", payment.asset.as_ref().unwrap());
        println!("  Amount: {}", payment.amount);
        println!("  Merchant: {}", payment.merchant.id);
        if let Some(customer) = &payment.customer {
            println!("  Customer: {}", customer.id);
        }
        println!();

        // Pack the payment message
        let (packed_payment, _delivery_results) = merchant_agent
            .send_message(&payment, vec![&customer_did], false)
            .await?;
        println!("Merchant sends the payment request to the customer\n");

        // Step 2: Customer receives and processes the payment request
        println!("Step 2: Customer receives and processes the payment request");

        let plain_message = customer_agent.receive_message(&packed_payment).await?;
        let received_payment: Payment = serde_json::from_value(plain_message.body)?;
        println!("Customer received payment request:");
        println!("  Asset: {}", received_payment.asset.as_ref().unwrap());
        println!("  Amount: {}", received_payment.amount);
        println!("  Merchant: {}", received_payment.merchant.id);
        if let Some(customer) = &received_payment.customer {
            println!("  Customer: {}", customer.id);
        }
        println!();

        // Step 3: Customer authorizes the payment
        println!("Step 3: Customer authorizes the payment");

        let authorize = Authorize {
            transaction_id: transaction_id.clone(),
            settlement_address: None,
            expiry: None,
        };

        let (packed_authorize, _delivery_results) = customer_agent
            .send_message(&authorize, vec![&merchant_did], false)
            .await?;
        println!("Customer sends authorization to the merchant\n");

        // Step 4: Merchant receives the authorization
        println!("Step 4: Merchant receives the authorization");

        let plain_message = merchant_agent.receive_message(&packed_authorize).await?;
        let received_authorize: Authorize = serde_json::from_value(plain_message.body)?;
        println!("Merchant received authorization:");
        println!("  Payment ID: {}", received_authorize.transaction_id);

        // Step 5: Customer settles the payment
        println!("Step 5: Customer settles the payment");

        // In a real scenario, the customer would submit the transaction to the blockchain
        // and get a transaction ID. Here we simulate it with a mock transaction ID.
        let settlement_id =
            "eip155:1:tx/0x3edb98c24d46d148eb926c714f4fbaa117c47b0c0821f38bfce9763604457c33";

        let settle = Settle {
            transaction_id: transaction_id.clone(),
            settlement_id: settlement_id.to_string(),
            amount: Some(payment.amount.clone()),
        };

        let (packed_settle, _delivery_results) = customer_agent
            .send_message(&settle, vec![&merchant_did], false)
            .await?;
        println!("Customer sends settlement confirmation to the merchant");
        println!("  Settlement ID: {}\n", settlement_id);

        // Step 6: Merchant receives the settlement confirmation
        println!("Step 6: Merchant receives the settlement confirmation");

        let plain_message = merchant_agent.receive_message(&packed_settle).await?;
        let received_settle: Settle = serde_json::from_value(plain_message.body)?;
        println!("Merchant received settlement confirmation:");
        println!("  Payment ID: {}", received_settle.transaction_id);
        println!("  Settlement ID: {}", received_settle.settlement_id);
        if let Some(amount) = &received_settle.amount {
            println!("  Amount: {}\n", amount);
        }

        // Step 7: Merchant fulfills the order (out of band)
        println!("Step 7: Merchant fulfills the order (out of band)");
        println!("  Merchant has received the payment and will now fulfill the order.\n");

        println!("=== Payment flow completed successfully ===");

        Ok(())
    })
}

/// Create an agent with the given DID and key material
async fn create_agent(did: &str, public_key: &str, private_key: &str) -> (Arc<TapAgent>, String) {
    // Create agent configuration
    let agent_config = AgentConfig::new(did.to_string());

    // Create key manager builder
    let mut builder = AgentKeyManagerBuilder::new();

    // Add the agent's key
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

    // Add the secret to the key manager
    builder = builder.add_secret(did.to_string(), secret);

    // Build the key manager
    let key_manager = builder.build().expect("Failed to build key manager");

    // Create the agent
    let agent = TapAgent::new(agent_config, Arc::new(key_manager));

    (Arc::new(agent), did.to_string())
}

/// Create a payment message
fn create_payment_message(
    merchant_did: &str,
    customer_did: &str,
    settlement_address: &str,
    transaction_id: &str,
) -> Payment {
    // Create merchant and customer parties
    let merchant = Party::new(merchant_did);
    let customer = Party::new(customer_did);

    // Create settlement agent
    let settlement_agent =
        tap_msg::Agent::new(settlement_address, "SettlementAddress", merchant_did);

    // Create a payment message
    Payment {
        asset: Some(
            AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        ),
        currency_code: None,
        amount: "100.0".to_string(),
        supported_assets: None,
        invoice: None,
        expiry: None,
        merchant,
        customer: Some(customer),
        agents: vec![settlement_agent],
        connection_id: None,
        metadata: HashMap::new(),
        transaction_id: transaction_id.to_string(),
        memo: Some("Payment for goods or services".to_string()),
    }
}
