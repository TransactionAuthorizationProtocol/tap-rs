//! Example demonstrating a secure TAIP-3 transfer flow with TAIP-4 authorization
//!
//! This example shows a complete transfer flow with proper security considerations:
//! 1. Proper key management and DID resolution
//! 2. Message validation and error handling
//! 3. Security mode selection based on message type
//! 4. Complete transfer flow with authorization and settlement
//!
//! Run with: cargo run --example secure_transfer_flow

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use tap_agent::agent::{Agent, TapAgent};
use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
use tap_agent::config::AgentConfig;
use tap_agent::error::{Error, Result};
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_caip::AssetId;
use tap_msg::message::{Authorize, Reject, Settle, Transfer};
use tap_msg::Party;

fn main() -> Result<()> {
    tokio_test::block_on(async {
        println!("=== Secure TAIP-3 Transfer Flow with TAIP-4 Authorization ===\n");

        // Step 1: Set up agents with proper key management
        println!("Step 1: Setting up agents with proper key management");

        // Create originator agent
        let (originator_agent, originator_did) = create_agent(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo",
            "nWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Yh",
        )
        .await?;

        // Create beneficiary agent
        let (beneficiary_agent, beneficiary_did) = create_agent(
            "did:key:z6MkhFvVnYxkqLNEiWQmUwhQuVpXiCfNmRUVi5yZ4Cg9w15k",
            "8zYZK2vvsAyVYpNpnYzTnUPjBuWdWpYmPpQmwErV9XQg",
            "8zYZK2vvsAyVYpNpnYzTnUPjBuWdWpYmPpQmwErV9XQg",
        )
        .await?;

        println!("Created agents with DIDs:");
        println!("  Originator: {}", originator_did);
        println!("  Beneficiary: {}\n", beneficiary_did);

        // Step 2: Create and validate a transfer request
        println!("Step 2: Creating and validating a transfer request");

        // Create a settlement address
        let settlement_address = "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb";

        // Create a transfer message
        let transfer =
            match create_transfer_message(&originator_did, &beneficiary_did, settlement_address) {
                Ok(transfer) => transfer,
                Err(e) => {
                    println!("Error creating transfer message: {}", e);
                    return Err(Error::Validation(
                        "Failed to create transfer message".to_string(),
                    ));
                }
            };

        // Generate a unique transfer ID
        let transfer_id = uuid::Uuid::new_v4().to_string();

        // Validate the transfer message
        if let Err(e) = transfer.validate() {
            println!("Transfer validation failed: {}", e);
            return Err(Error::Validation(format!(
                "Transfer validation failed: {}",
                e
            )));
        }

        println!("Transfer message created and validated successfully");
        println!("  Asset: {}", transfer.asset);
        println!("  Amount: {}", transfer.amount);
        println!(
            "  From: {}",
            transfer
                .originator
                .as_ref()
                .map(|o| o.id.as_str())
                .unwrap_or("unknown")
        );
        println!("  To: {}\n", transfer.beneficiary.as_ref().unwrap().id);

        // Step 3: Send the transfer request with proper security
        println!("Step 3: Sending the transfer request with proper security");

        // Pack the transfer message with appropriate security mode
        let (packed_transfer, _delivery_results) = match originator_agent
            .send_message(&transfer, vec![&beneficiary_did], false)
            .await
        {
            Ok((packed, delivery_results)) => (packed, delivery_results),
            Err(e) => {
                println!("Error packing transfer message: {}", e);
                return Err(e);
            }
        };

        println!("Transfer message packed and sent successfully\n");

        // Step 4: Beneficiary receives and validates the transfer request
        println!("Step 4: Beneficiary receives and validates the transfer request");

        // Unpack and validate the transfer message
        let _received_transfer: Transfer = match beneficiary_agent
            .receive_message(&packed_transfer)
            .await
        {
            Ok(plain_message) => serde_json::from_value(plain_message.body)
                .map_err(|e| Error::Validation(format!("Failed to deserialize transfer: {}", e)))?,
            Err(e) => {
                println!("Error unpacking transfer message: {}", e);

                // In case of validation error, send a rejection
                if let Error::Validation(_) = e {
                    let reject = Reject {
                    transaction_id: transfer_id.clone(),
                    reason: Some(format!("validation.failed: Transfer validation failed: {}. Please correct the validation issues and try again", e)),
                };

                    let _ = beneficiary_agent
                        .send_message(&reject, vec![&originator_did], false)
                        .await;
                    println!("Sent rejection due to validation failure");
                }

                return Err(e);
            }
        };

        println!("Transfer message received and validated successfully\n");

        // Step 5: Perform risk assessment (simulated)
        println!("Step 5: Performing risk assessment");

        // Simulate risk assessment
        let risk_score = 20; // Low risk score (0-100)
        let risk_threshold = 70; // Threshold for rejection

        println!("Risk assessment completed");
        println!(
            "  Risk score: {}/100 (threshold: {})\n",
            risk_score, risk_threshold
        );

        // Step 6: Decide whether to authorize or reject based on risk assessment
        println!("Step 6: Processing authorization decision");

        if risk_score >= risk_threshold {
            println!("High risk detected, rejecting transfer");

            let reject = Reject {
            transaction_id: transfer_id.clone(),
            reason: Some(format!("risk.threshold.exceeded: Risk score ({}) exceeds threshold ({}). Please contact support for further assistance", risk_score, risk_threshold)),
        };

            let (packed_reject, _delivery_results) = match beneficiary_agent
                .send_message(&reject, vec![&originator_did], false)
                .await
            {
                Ok((packed, delivery_results)) => (packed, delivery_results),
                Err(e) => {
                    println!("Error sending rejection: {}", e);
                    return Err(e);
                }
            };

            println!("Rejection sent successfully\n");

            // Originator receives the rejection
            let received_reject: Reject = match originator_agent
                .receive_message(&packed_reject)
                .await
            {
                Ok(plain_message) => serde_json::from_value(plain_message.body).map_err(|e| {
                    Error::Validation(format!("Failed to deserialize reject: {}", e))
                })?,
                Err(e) => {
                    println!("Error receiving rejection: {}", e);
                    return Err(e);
                }
            };

            println!("Originator received rejection:");
            println!("  Transfer ID: {}", received_reject.transaction_id);
            println!("  Reason: {:?}", received_reject.reason);
            println!("Transfer flow ended with rejection");

            return Ok(());
        }

        // Low risk, proceed with authorization
        println!("Low risk detected, proceeding with authorization");

        // Beneficiary VASP authorizes the transfer
        let authorize = Authorize {
            transaction_id: transfer_id.clone(),
            settlement_address: None,
            expiry: None,
        };

        let (packed_authorize, _delivery_results) = match beneficiary_agent
            .send_message(&authorize, vec![&originator_did], false)
            .await
        {
            Ok((packed, delivery_results)) => (packed, delivery_results),
            Err(e) => {
                println!("Error sending authorization: {}", e);
                return Err(e);
            }
        };

        println!("Authorization sent successfully with expiry time in note\n");

        // Step 7: Originator receives and validates the authorization
        println!("Step 7: Originator receives and validates the authorization");

        let received_authorize: Authorize =
            match originator_agent.receive_message(&packed_authorize).await {
                Ok(plain_message) => serde_json::from_value(plain_message.body).map_err(|e| {
                    Error::Validation(format!("Failed to deserialize authorize: {}", e))
                })?,
                Err(e) => {
                    println!("Error receiving authorization: {}", e);
                    return Err(e);
                }
            };

        println!("Authorization received and validated successfully");
        println!("  Transfer ID: {}", received_authorize.transaction_id);

        // In a real implementation, we would parse and validate the expiry time from the expiry field

        println!();

        // Step 8: Originator settles the transfer
        println!("Step 8: Originator settles the transfer");

        // In a real scenario, the originator would submit the transaction to the blockchain
        // and get a transaction ID. Here we simulate it with a mock transaction ID.
        let settlement_id =
            "eip155:1:tx/0x3edb98c24d46d148eb926c714f4fbaa117c47b0c0821f38bfce9763604457c33";

        let settle = Settle {
            transaction_id: transfer_id.clone(),
            settlement_id: Some(settlement_id.to_string()),
            amount: Some(transfer.amount.clone()),
        };

        let (packed_settle, _delivery_results) = match originator_agent
            .send_message(&settle, vec![&beneficiary_did], false)
            .await
        {
            Ok((packed, delivery_results)) => (packed, delivery_results),
            Err(e) => {
                println!("Error sending settlement: {}", e);
                return Err(e);
            }
        };

        println!("Settlement sent successfully");
        println!("  Transaction ID: {}\n", settlement_id);

        // Step 9: Beneficiary receives and validates the settlement
        println!("Step 9: Beneficiary receives and validates the settlement");

        let received_settle: Settle = match beneficiary_agent.receive_message(&packed_settle).await
        {
            Ok(plain_message) => serde_json::from_value(plain_message.body)
                .map_err(|e| Error::Validation(format!("Failed to deserialize settle: {}", e)))?,
            Err(e) => {
                println!("Error receiving settlement: {}", e);
                return Err(e);
            }
        };

        println!("Settlement received and validated successfully");
        println!("  Transfer ID: {}", received_settle.transaction_id);
        println!("  Settlement ID: {:?}", received_settle.settlement_id);
        if let Some(amount) = &received_settle.amount {
            println!("  Amount: {}", amount);
        }
        println!();

        println!("=== Secure transfer flow completed successfully ===");

        Ok(())
    })
}

/// Create an agent with the given DID and key material
async fn create_agent(
    did: &str,
    public_key: &str,
    private_key: &str,
) -> Result<(Arc<TapAgent>, String)> {
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
    let key_manager = builder.build()?;

    // Create the agent
    let agent = TapAgent::new(agent_config, Arc::new(key_manager));

    Ok((Arc::new(agent), did.to_string()))
}

/// Create a transfer message with validation
fn create_transfer_message(
    originator_did: &str,
    beneficiary_did: &str,
    settlement_address: &str,
) -> Result<Transfer> {
    // Validate DIDs
    if originator_did.is_empty() || beneficiary_did.is_empty() {
        return Err(Error::Validation("Invalid DIDs provided".to_string()));
    }

    // Create originator and beneficiary parties
    let originator = Party::new(originator_did);
    let beneficiary = Party::new(beneficiary_did);

    // Create settlement agent
    let settlement_agent =
        tap_msg::Agent::new(settlement_address, "SettlementAddress", originator_did);

    // Validate asset ID
    let asset = match AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f")
    {
        Ok(asset) => asset,
        Err(_) => return Err(Error::Validation("Invalid asset ID".to_string())),
    };

    // Create a transfer message
    let transfer = Transfer {
        transaction_id: Some(uuid::Uuid::new_v4().to_string()),
        asset,
        originator: Some(originator),
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents: vec![settlement_agent],
        settlement_id: None,
        memo: Some("Secure example transfer".to_string()),
        connection_id: None,
        metadata: HashMap::new(),
    };

    Ok(transfer)
}
