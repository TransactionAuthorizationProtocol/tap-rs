//! Example demonstrating a multi-agent TAIP-3 transfer flow with TAIP-4 authorization
//!
//! This example shows a more complex scenario with multiple agents:
//! 1. Originator VASP and Wallet
//! 2. Beneficiary VASP and Wallet
//! 3. Wallet API agents that join the flow dynamically
//! 4. Rejection handling and recovery
//! 5. Agent addition using TAIP-5 AddAgents message
//!
//! Run with: cargo run --example multi_agent_flow

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use tap_agent::agent::{Agent, TapAgent};
use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
use tap_agent::config::AgentConfig;
use tap_agent::error::Result;
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_caip::AssetId;
use tap_msg::message::{AddAgents, Authorize, Reject, Settle, Transfer};
use tap_msg::{Agent as TapAgent_, Party};

fn main() -> Result<()> {
    tokio_test::block_on(async {
        println!("=== Multi-Agent TAIP-3 Transfer Flow with TAIP-4 Authorization ===\n");

        // Step 1: Set up agents
        // In a real scenario, these would be different entities

        // Create originator VASP agent
        let (originator_vasp, originator_vasp_did) = create_agent(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo",
            "nWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Yh",
        )
        .await?;

        // Create originator wallet agent
        let (originator_wallet, originator_wallet_did) = create_agent(
            "did:key:z6MkhFvVnYxkqLNEiWQmUwhQuVpXiCfNmRUVi5yZ4Cg9w15k",
            "8zYZK2vvsAyVYpNpnYzTnUPjBuWdWpYmPpQmwErV9XQg",
            "8zYZK2vvsAyVYpNpnYzTnUPjBuWdWpYmPpQmwErV9XQg",
        )
        .await?;

        // Create originator wallet API agent
        let (originator_wallet_api, originator_wallet_api_did) = create_agent(
            "did:key:z6MkgYAFirTGpAaHxfQrJxSUVNBsrGZEXEqnawEUCPnVKVXJ",
            "CnEDU0Jxr6Jx9XH+61JrGK8Bz1xm0xwLOqVDjd+5FVM",
            "CnEDU0Jxr6Jx9XH+61JrGK8Bz1xm0xwLOqVDjd+5FVM",
        )
        .await?;

        // Create beneficiary VASP agent
        let (beneficiary_vasp, beneficiary_vasp_did) = create_agent(
            "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
            "B12NYF8RrR3h41TDCTJojY59usg3mbtbjnFs7Eud1Y6u",
            "B12NYF8RrR3h41TDCTJojY59usg3mbtbjnFs7Eud1Y6u",
        )
        .await?;

        // Create beneficiary wallet agent
        let (beneficiary_wallet, beneficiary_wallet_did) = create_agent(
            "did:key:z6MkrJVkLHBdQQS5y2CnXAHJcgBWMVv7V5aukAtQyBx4qJA4",
            "5TVS4YKJxmqVVQUM7xbVsYiFrCbdwgLLdu6QB98q3a4",
            "5TVS4YKJxmqVVQUM7xbVsYiFrCbdwgLLdu6QB98q3a4",
        )
        .await?;

        // Create beneficiary wallet API agent
        let (beneficiary_wallet_api, beneficiary_wallet_api_did) = create_agent(
            "did:key:z6MkqyYXcBQH7dJyXWrTrEKJNEjXnkajQ2xjGPEgBsqVRmVS",
            "D9WbJ5H9sTNXLVLYARpVSXhwrLGJUHNn6vJUUXFqYj4",
            "D9WbJ5H9sTNXLVLYARpVSXhwrLGJUHNn6vJUUXFqYj4",
        )
        .await?;

        println!("Created agents with DIDs:");
        println!("  Originator VASP: {}", originator_vasp_did);
        println!("  Originator Wallet: {}", originator_wallet_did);
        println!("  Originator Wallet API: {}", originator_wallet_api_did);
        println!("  Beneficiary VASP: {}", beneficiary_vasp_did);
        println!("  Beneficiary Wallet: {}", beneficiary_wallet_did);
        println!("  Beneficiary Wallet API: {}\n", beneficiary_wallet_api_did);

        // Step 2: Create a transfer request
        println!("Step 1: Originator VASP creates a transfer request");

        // Create a transfer message
        let asset =
            match AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f") {
                Ok(asset) => asset,
                Err(e) => {
                    println!("Error parsing asset ID: {}", e);
                    return Err(tap_agent::error::Error::Validation(format!(
                        "Invalid asset ID: {}",
                        e
                    )));
                }
            };
        let transfer_id = uuid::Uuid::new_v4();

        // Create originator and beneficiary parties (different from the agents)
        let originator_party = "did:pkh:eip155:1:0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"; // Example customer DID
        let beneficiary_party = "did:pkh:eip155:1:0x71C7656EC7ab88b098defB751B7401B5f6d8976F"; // Example recipient DID

        // Include both originator and beneficiary agents in the initial transfer
        let transfer = Transfer {
            transaction_id: Some(uuid::Uuid::new_v4().to_string()),
            asset,
            originator: Some(Party::new(originator_party)),
            beneficiary: Some(Party::new(beneficiary_party)),
            amount: "100.0".to_string(),
            agents: vec![
                // Originator agents
                TapAgent_::new(&originator_vasp_did, "originatorVASP", originator_party),
                TapAgent_::new(&originator_wallet_did, "originatorWallet", originator_party),
                TapAgent_::new(
                    &originator_wallet_api_did,
                    "originatorWalletAPI",
                    originator_party,
                ),
                // Beneficiary agent
                TapAgent_::new(&beneficiary_vasp_did, "beneficiaryVASP", beneficiary_party),
            ],
            settlement_id: None,
            memo: Some("Multi-agent transfer example with dynamic agent addition".to_string()),
            connection_id: None,
            metadata: HashMap::new(),
        };

        println!("Transfer details:");
        println!("  Asset: {}", transfer.asset);
        println!("  Amount: {}", transfer.amount);
        println!(
            "  From: {} (party)",
            transfer
                .originator
                .as_ref()
                .map(|o| o.id.as_str())
                .unwrap_or("unknown")
        );
        println!(
            "  To: {} (party)",
            transfer.beneficiary.as_ref().unwrap().id
        );
        println!("  Initial agents:");
        println!("    - {} (originator VASP)", originator_vasp_did);
        println!("    - {} (originator wallet)", originator_wallet_did);
        println!(
            "    - {} (originator wallet API)",
            originator_wallet_api_did
        );
        println!("    - {} (beneficiary VASP)", beneficiary_vasp_did);
        println!();

        // Step 3: Send the transfer to the beneficiary VASP
        println!("Step 2: Sending transfer to beneficiary VASP");

        // Pack and send to VASP
        let (packed_transfer_vasp, _delivery_results) = originator_vasp
            .send_message(&transfer, vec![&beneficiary_vasp_did], false)
            .await?;

        println!("Transfer sent to beneficiary VASP\n");

        // Step 4: Beneficiary VASP receives the transfer
        println!("Step 3: Beneficiary VASP receives the transfer");

        // Receive at VASP
        let plain_message = beneficiary_vasp
            .receive_message(&packed_transfer_vasp)
            .await?;
        let received_transfer_vasp: Transfer = serde_json::from_value(plain_message.body)?;

        println!("Transfer received by beneficiary VASP");
        println!("  Transfer ID: {}", transfer_id);
        println!(
            "  Initial agents count: {}\n",
            received_transfer_vasp.agents.len()
        );

        // Step 5: Beneficiary VASP adds their wallet and wallet API as agents
        println!("Step 4: Beneficiary VASP adds their wallet and wallet API as agents");

        // Create AddAgents message to add the beneficiary wallet and wallet API
        let add_agents = AddAgents {
            transaction_id: transfer_id.to_string(),
            agents: vec![
                TapAgent_::new(
                    &beneficiary_wallet_did,
                    "beneficiaryWallet",
                    beneficiary_party,
                ),
                TapAgent_::new(
                    &beneficiary_wallet_api_did,
                    "beneficiaryWalletAPI",
                    beneficiary_party,
                ),
            ],
        };

        // Send AddAgents message to originator VASP
        let (packed_add_agents, _delivery_results) = beneficiary_vasp
            .send_message(&add_agents, vec![&originator_vasp_did], false)
            .await?;

        println!("Beneficiary VASP sends AddAgents message to add wallet and wallet API");
        println!(
            "  Added agents: {} (beneficiary wallet), {} (beneficiary wallet API)\n",
            beneficiary_wallet_did, beneficiary_wallet_api_did
        );

        // Step 6: Originator VASP receives the AddAgents message
        println!("Step 5: Originator VASP receives the AddAgents message");

        // Receive AddAgents message
        let plain_message = originator_vasp.receive_message(&packed_add_agents).await?;
        let received_add_agents: AddAgents = serde_json::from_value(plain_message.body)?;

        println!("Originator VASP received AddAgents message:");
        println!("  Transfer ID: {}", received_add_agents.transaction_id);
        println!("  Added agents count: {}", received_add_agents.agents.len());

        // In a real implementation, the originator would update their local state with the new agents
        println!(
            "  Added agents: {} ({}), {} ({})\n",
            received_add_agents.agents[0].id,
            received_add_agents.agents[0]
                .role
                .as_deref()
                .unwrap_or("unknown"),
            received_add_agents.agents[1].id,
            received_add_agents.agents[1]
                .role
                .as_deref()
                .unwrap_or("unknown")
        );

        // Step 7: Initial rejection by beneficiary VASP for compliance
        println!(
            "Step 6: Beneficiary VASP initially rejects the transfer due to compliance concerns"
        );

        let reject = Reject {
        transaction_id: transfer_id.to_string(),
        reason: Some("compliance.policy: Additional beneficiary information required. Please provide additional beneficiary information to comply with regulations".to_string()),
    };

        let (packed_reject, _delivery_results) = beneficiary_vasp
            .send_message(&reject, vec![&originator_vasp_did], false)
            .await?;

        // Originator receives the rejection
        let plain_message = originator_vasp.receive_message(&packed_reject).await?;
        let received_reject: Reject = serde_json::from_value(plain_message.body)?;
        println!("Originator VASP received rejection:");
        println!("  Transfer ID: {}", received_reject.transaction_id);
        println!("  Reason: {:?}", received_reject.reason);
        println!();

        // Step 8: After resolving the compliance concerns, the beneficiary VASP authorizes
        println!(
            "Step 7: After resolving compliance concerns, beneficiary VASP authorizes the transfer"
        );

        let _settlement_address = "eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
        let authorize = Authorize {
            transaction_id: transfer_id.to_string(),
            settlement_address: None,
            expiry: None,
        };

        let (packed_authorize_vasp, _delivery_results) = beneficiary_vasp
            .send_message(&authorize, vec![&originator_vasp_did], false)
            .await?;

        // Originator receives the authorization
        let plain_message = originator_vasp
            .receive_message(&packed_authorize_vasp)
            .await?;
        let received_authorize: Authorize = serde_json::from_value(plain_message.body)?;
        println!("Originator VASP received authorization:");
        println!("  Transfer ID: {}", received_authorize.transaction_id);

        // Step 9: Beneficiary wallet also authorizes the transfer
        println!("Step 8: Beneficiary wallet also authorizes the transfer");

        let authorize_wallet = Authorize {
            transaction_id: transfer_id.to_string(),
            settlement_address: None,
            expiry: None,
        };

        let (packed_authorize_wallet, _delivery_results) = beneficiary_wallet
            .send_message(&authorize_wallet, vec![&originator_wallet_did], false)
            .await?;

        // Originator wallet receives the authorization
        let plain_message = originator_wallet
            .receive_message(&packed_authorize_wallet)
            .await?;
        let received_authorize_wallet: Authorize = serde_json::from_value(plain_message.body)?;
        println!("Originator wallet received authorization:");
        println!(
            "  Transfer ID: {}",
            received_authorize_wallet.transaction_id
        );

        // Step 10: Wallet APIs exchange information
        println!("Step 9: Wallet APIs exchange technical information for settlement");

        // Simulate wallet API communication (in a real scenario, this would be more complex)
        let api_note = format!(
            "API technical details: callback_url=https://api.wallet.example/callbacks/{}, nonce={}",
            transfer_id,
            uuid::Uuid::new_v4()
        );
        println!("  - {}", api_note);

        let api_authorize = Authorize {
            transaction_id: transfer_id.to_string(),
            settlement_address: None,
            expiry: None,
        };

        let (packed_api_authorize, _delivery_results) = originator_wallet_api
            .send_message(&api_authorize, vec![&beneficiary_wallet_api_did], false)
            .await?;

        // Beneficiary wallet API receives the technical details
        let plain_message = beneficiary_wallet_api
            .receive_message(&packed_api_authorize)
            .await?;
        let received_api_authorize: Authorize = serde_json::from_value(plain_message.body)?;
        println!("Beneficiary wallet API received technical details:");
        println!("  Transfer ID: {}", received_api_authorize.transaction_id);

        // Step 11: Originator wallet initiates settlement
        println!("Step 10: Originator wallet initiates settlement");

        // In a real scenario, the wallet would submit the transaction to the blockchain
        // and get a transaction ID. Here we simulate it with a mock transaction ID.
        let settlement_id =
            "eip155:1:tx/0x3edb98c24d46d148eb926c714f4fbaa117c47b0c0821f38bfce9763604457c33";

        let settle = Settle {
            transaction_id: transfer_id.to_string(),
            settlement_id: Some(settlement_id.to_string()),
            amount: Some(transfer.amount.clone()),
        };

        // Send settlement to all relevant parties
        let (packed_settle_vasp, _delivery_results1) = originator_wallet
            .send_message(&settle, vec![&beneficiary_vasp_did], false)
            .await?;
        let (packed_settle_wallet, _delivery_results2) = originator_wallet
            .send_message(&settle, vec![&beneficiary_wallet_did], false)
            .await?;

        println!("Settlement sent to beneficiary VASP and wallet");
        println!("  Transaction ID: {}\n", settlement_id);

        // Step 12: Wallet API confirms settlement details
        println!("Step 11: Wallet APIs confirm settlement details");

        // Originator wallet API settles the transfer
        let api_settlement_id =
            "eip155:1:tx/0x3edb98c24d46d148eb926c714f4fbaa117c47b0c0821f38bfce9763604457c33";
        let api_settle = Settle {
            transaction_id: transfer_id.to_string(),
            settlement_id: Some(api_settlement_id.to_string()),
            amount: Some(transfer.amount.clone()),
        };

        let (packed_api_settle, _delivery_results) = originator_wallet_api
            .send_message(&api_settle, vec![&beneficiary_wallet_api_did], false)
            .await?;

        // Beneficiary wallet API receives settlement confirmation
        let plain_message = beneficiary_wallet_api
            .receive_message(&packed_api_settle)
            .await?;
        let received_api_settle: Settle = serde_json::from_value(plain_message.body)?;
        println!("Beneficiary wallet API received settlement confirmation:");
        println!("  Transfer ID: {}", received_api_settle.transaction_id);
        println!("  Settlement ID: {:?}", received_api_settle.settlement_id);
        if let Some(amount) = &received_api_settle.amount {
            println!("  Amount: {}\n", amount);
        }

        // Step 13: Beneficiaries receive settlement confirmation
        println!("Step 12: Beneficiaries receive settlement confirmation");

        // Receive at VASP and wallet
        let plain_message = beneficiary_vasp
            .receive_message(&packed_settle_vasp)
            .await?;
        let received_settle_vasp: Settle = serde_json::from_value(plain_message.body)?;
        let plain_message_wallet = beneficiary_wallet
            .receive_message(&packed_settle_wallet)
            .await?;
        let _received_settle_wallet: Settle = serde_json::from_value(plain_message_wallet.body)?;

        println!("Settlement received by beneficiary VASP and wallet:");
        println!("  Transfer ID: {}", received_settle_vasp.transaction_id);
        println!("  Settlement ID: {:?}", received_settle_vasp.settlement_id);
        if let Some(amount) = &received_settle_vasp.amount {
            println!("  Amount: {}", amount);
        }

        println!("\n=== Multi-agent transfer flow with dynamic agent addition completed successfully ===");

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
