//! Example demonstrating a complete TAIP-3 transfer flow with TAIP-4 authorization
//!
//! This example shows how two agents can participate in a transfer flow:
//! 1. Originator agent initiates a transfer request
//! 2. Beneficiary agent authorizes the transfer
//! 3. Originator agent settles the transfer
//!
//! Run with: cargo run --example transfer_flow

use std::collections::HashMap;
use std::str::FromStr;

use tap_agent::agent::{Agent, TapAgent};
use tap_caip::AssetId;
use tap_msg::message::{Authorize, Settle, Transfer};
use tap_msg::Party;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio_test::block_on(async {
        println!("=== TAIP-3 Transfer Flow with TAIP-4 Authorization ===\n");

        // Create originator agent with an ephemeral key
        let (originator_agent, originator_did) = TapAgent::from_ephemeral_key().await?;

        // Create beneficiary agent with an ephemeral key
        let (beneficiary_agent, beneficiary_did) = TapAgent::from_ephemeral_key().await?;

        println!("Created originator agent with DID: {}", originator_did);
        println!("Created beneficiary agent with DID: {}\n", beneficiary_did);

        // Create a settlement address (in a real scenario, this would be a blockchain address)
        let settlement_address = "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb";

        // Step 1: Originator creates and sends a transfer request
        println!("Step 1: Originator creates a transfer request");

        let transfer =
            create_transfer_message(&originator_did, &beneficiary_did, settlement_address);
        println!("Transfer details:");
        println!("  Asset: {}", transfer.asset);
        println!("  Amount: {}", transfer.amount);
        println!("  From: {}", transfer.originator.id);
        println!("  To: {}\n", transfer.beneficiary.as_ref().unwrap().id);

        // Pack the transfer message
        let (packed_transfer, _delivery_results) = originator_agent
            .send_message(&transfer, vec![&beneficiary_did], false)
            .await?;
        println!("Originator sends the transfer request to the beneficiary\n");

        // Step 2: Beneficiary receives and processes the transfer request
        println!("Step 2: Beneficiary receives and processes the transfer request");

        let plain_message = beneficiary_agent.receive_message(&packed_transfer).await?;
        let received_transfer: Transfer = serde_json::from_value(plain_message.body)?;
        println!("Beneficiary received transfer request:");
        println!("  Asset: {}", received_transfer.asset);
        println!("  Amount: {}", received_transfer.amount);
        println!("  From: {}", received_transfer.originator.id);
        println!(
            "  To: {}\n",
            received_transfer.beneficiary.as_ref().unwrap().id
        );

        // Step 3: Beneficiary authorizes the transfer
        println!("Step 3: Beneficiary authorizes the transfer");

        // Generate a unique transfer ID (in a real scenario, this would be from the original transfer)
        let transfer_id = uuid::Uuid::new_v4().to_string();

        let authorize = Authorize {
            transaction_id: transfer_id.clone(),
            settlement_address: None,
            expiry: None,
        };

        let (packed_authorize, _delivery_results) = beneficiary_agent
            .send_message(&authorize, vec![&originator_did], false)
            .await?;
        println!("Beneficiary sends authorization to the originator\n");

        // Step 4: Originator receives the authorization
        println!("Step 4: Originator receives the authorization");

        let plain_message = originator_agent.receive_message(&packed_authorize).await?;
        let received_authorize: Authorize = serde_json::from_value(plain_message.body)?;
        println!("Originator received authorization:");
        println!("  Transfer ID: {}", received_authorize.transaction_id);

        // Step 5: Originator settles the transfer
        println!("Step 5: Originator settles the transfer");

        // In a real scenario, the originator would submit the transaction to the blockchain
        // and get a transaction ID. Here we simulate it with a mock transaction ID.
        let settlement_id =
            "eip155:1:tx/0x3edb98c24d46d148eb926c714f4fbaa117c47b0c0821f38bfce9763604457c33";

        let settle = Settle {
            transaction_id: transfer_id.clone(),
            settlement_id: settlement_id.to_string(),
            amount: Some(transfer.amount.clone()),
        };

        let (packed_settle, _delivery_results) = originator_agent
            .send_message(&settle, vec![&beneficiary_did], false)
            .await?;
        println!("Originator sends settlement confirmation to the beneficiary");
        println!("  Settlement ID: {}\n", settlement_id);

        // Step 6: Beneficiary receives the settlement confirmation
        println!("Step 6: Beneficiary receives the settlement confirmation");

        let plain_message = beneficiary_agent.receive_message(&packed_settle).await?;
        let received_settle: Settle = serde_json::from_value(plain_message.body)?;
        println!("Beneficiary received settlement confirmation:");
        println!("  Transfer ID: {}", received_settle.transaction_id);
        println!("  Settlement ID: {}", received_settle.settlement_id);
        if let Some(amount) = &received_settle.amount {
            println!("  Amount: {}\n", amount);
        }

        println!("=== Transfer flow completed successfully ===");

        Ok(())
    })
}

/// Create a transfer message
fn create_transfer_message(
    originator_did: &str,
    beneficiary_did: &str,
    settlement_address: &str,
) -> Transfer {
    // Create originator and beneficiary parties
    let originator = Party::new(originator_did);
    let beneficiary = Party::new(beneficiary_did);

    // Create settlement agent
    let settlement_agent =
        tap_msg::Agent::new(settlement_address, "SettlementAddress", originator_did);

    // Create a transfer message
    Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset: AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f")
            .unwrap(),
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents: vec![settlement_agent],
        settlement_id: None,
        memo: Some("Example transfer".to_string()),
        connection_id: None,
        metadata: HashMap::new(),
    }
}
