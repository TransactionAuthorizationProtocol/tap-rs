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

use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{BasicSecretResolver, DefaultMessagePacker};
use tap_agent::did::{KeyResolver, MultiResolver};
use tap_agent::error::Result;
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_caip::AssetId;
use tap_msg::message::{AddAgents, Authorize, Reject, Settle, Transfer};
use tap_msg::Participant;

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
            transaction_id: uuid::Uuid::new_v4().to_string(),
            asset,
            originator: Participant {
                id: originator_party.to_string(),
                role: Some("originator".to_string()),
                policies: None,
                leiCode: None,
                name: None,
            },
            beneficiary: Some(Participant {
                id: beneficiary_party.to_string(),
                role: Some("beneficiary".to_string()),
                policies: None,
                leiCode: None,
                name: None,
            }),
            amount: "100.0".to_string(),
            agents: vec![
                // Originator agents
                Participant {
                    id: originator_vasp_did.clone(),
                    role: Some("originatorVASP".to_string()),
                    policies: None,
                    leiCode: None,
                    name: None,
                },
                Participant {
                    id: originator_wallet_did.clone(),
                    role: Some("originatorWallet".to_string()),
                    policies: None,
                    leiCode: None,
                    name: None,
                },
                Participant {
                    id: originator_wallet_api_did.clone(),
                    role: Some("originatorWalletAPI".to_string()),
                    policies: None,
                    leiCode: None,
                    name: None,
                },
                // Beneficiary agent
                Participant {
                    id: beneficiary_vasp_did.clone(),
                    role: Some("beneficiaryVASP".to_string()),
                    policies: None,
                    leiCode: None,
                    name: None,
                },
            ],
            settlement_id: None,
            memo: Some("Multi-agent transfer example with dynamic agent addition".to_string()),
            metadata: HashMap::new(),
        };

        println!("Transfer details:");
        println!("  Asset: {}", transfer.asset);
        println!("  Amount: {}", transfer.amount);
        println!("  From: {} (party)", transfer.originator.id);
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
        let received_transfer_vasp: Transfer = beneficiary_vasp
            .receive_message(&packed_transfer_vasp)
            .await?;

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
                Participant {
                    id: beneficiary_wallet_did.clone(),
                    role: Some("beneficiaryWallet".to_string()),
                    policies: None,
                    leiCode: None,
                    name: None,
                },
                Participant {
                    id: beneficiary_wallet_api_did.clone(),
                    role: Some("beneficiaryWalletAPI".to_string()),
                    policies: None,
                    leiCode: None,
                    name: None,
                },
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
        let received_add_agents: AddAgents =
            originator_vasp.receive_message(&packed_add_agents).await?;

        println!("Originator VASP received AddAgents message:");
        println!("  Transfer ID: {}", received_add_agents.transaction_id);
        println!("  Added agents count: {}", received_add_agents.agents.len());

        // In a real implementation, the originator would update their local state with the new agents
        println!(
            "  Added agents: {} ({}), {} ({})\n",
            received_add_agents.agents[0].id,
            received_add_agents.agents[0]
                .role
                .as_ref()
                .unwrap_or(&"unknown".to_string()),
            received_add_agents.agents[1].id,
            received_add_agents.agents[1]
                .role
                .as_ref()
                .unwrap_or(&"unknown".to_string())
        );

        // Step 7: Initial rejection by beneficiary VASP for compliance
        println!(
            "Step 6: Beneficiary VASP initially rejects the transfer due to compliance concerns"
        );

        let reject = Reject {
        transaction_id: transfer_id.to_string(),
        reason: "compliance.policy: Additional beneficiary information required. Please provide additional beneficiary information to comply with regulations".to_string(),
    };

        let (packed_reject, _delivery_results) = beneficiary_vasp
            .send_message(&reject, vec![&originator_vasp_did], false)
            .await?;

        // Originator receives the rejection
        let received_reject: Reject = originator_vasp.receive_message(&packed_reject).await?;
        println!("Originator VASP received rejection:");
        println!("  Transfer ID: {}", received_reject.transaction_id);
        println!("  Reason: {}", received_reject.reason);
        println!();

        // Step 8: After resolving the compliance concerns, the beneficiary VASP authorizes
        println!(
            "Step 7: After resolving compliance concerns, beneficiary VASP authorizes the transfer"
        );

        let _settlement_address = "eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
        let authorize = Authorize {
            transaction_id: transfer_id.to_string(),
            note: Some("Transfer authorized after compliance review".to_string()),
        };

        let (packed_authorize_vasp, _delivery_results) = beneficiary_vasp
            .send_message(&authorize, vec![&originator_vasp_did], false)
            .await?;

        // Originator receives the authorization
        let received_authorize: Authorize = originator_vasp
            .receive_message(&packed_authorize_vasp)
            .await?;
        println!("Originator VASP received authorization:");
        println!("  Transfer ID: {}", received_authorize.transaction_id);
        if let Some(note) = &received_authorize.note {
            println!("  Note: {}\n", note);
        }

        // Step 9: Beneficiary wallet also authorizes the transfer
        println!("Step 8: Beneficiary wallet also authorizes the transfer");

        let authorize_wallet = Authorize {
            transaction_id: transfer_id.to_string(),
            note: Some("Wallet ready to receive funds".to_string()),
        };

        let (packed_authorize_wallet, _delivery_results) = beneficiary_wallet
            .send_message(&authorize_wallet, vec![&originator_wallet_did], false)
            .await?;

        // Originator wallet receives the authorization
        let received_authorize_wallet: Authorize = originator_wallet
            .receive_message(&packed_authorize_wallet)
            .await?;
        println!("Originator wallet received authorization:");
        println!(
            "  Transfer ID: {}",
            received_authorize_wallet.transaction_id
        );
        if let Some(note) = &received_authorize_wallet.note {
            println!("  Note: {}\n", note);
        }

        // Step 10: Wallet APIs exchange information
        println!("Step 9: Wallet APIs exchange technical information for settlement");

        // Simulate wallet API communication (in a real scenario, this would be more complex)
        let api_note = format!(
            "API technical details: callback_url=https://api.wallet.example/callbacks/{}, nonce={}",
            transfer_id,
            uuid::Uuid::new_v4()
        );

        let api_authorize = Authorize {
            transaction_id: transfer_id.to_string(),
            note: Some(api_note.clone()),
        };

        let (packed_api_authorize, _delivery_results) = originator_wallet_api
            .send_message(&api_authorize, vec![&beneficiary_wallet_api_did], false)
            .await?;

        // Beneficiary wallet API receives the technical details
        let received_api_authorize: Authorize = beneficiary_wallet_api
            .receive_message(&packed_api_authorize)
            .await?;
        println!("Beneficiary wallet API received technical details:");
        println!("  Transfer ID: {}", received_api_authorize.transaction_id);
        if let Some(note) = &received_api_authorize.note {
            println!("  Technical details: {}\n", note);
        }

        // Step 11: Originator wallet initiates settlement
        println!("Step 10: Originator wallet initiates settlement");

        // In a real scenario, the wallet would submit the transaction to the blockchain
        // and get a transaction ID. Here we simulate it with a mock transaction ID.
        let settlement_id =
            "eip155:1:tx/0x3edb98c24d46d148eb926c714f4fbaa117c47b0c0821f38bfce9763604457c33";

        let settle = Settle {
            transaction_id: transfer_id.to_string(),
            settlement_id: settlement_id.to_string(),
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
            settlement_id: api_settlement_id.to_string(),
            amount: Some(transfer.amount.clone()),
        };

        let (packed_api_settle, _delivery_results) = originator_wallet_api
            .send_message(&api_settle, vec![&beneficiary_wallet_api_did], false)
            .await?;

        // Beneficiary wallet API receives settlement confirmation
        let received_api_settle: Settle = beneficiary_wallet_api
            .receive_message(&packed_api_settle)
            .await?;
        println!("Beneficiary wallet API received settlement confirmation:");
        println!("  Transfer ID: {}", received_api_settle.transaction_id);
        println!("  Settlement ID: {}", received_api_settle.settlement_id);
        if let Some(amount) = &received_api_settle.amount {
            println!("  Amount: {}\n", amount);
        }

        // Step 13: Beneficiaries receive settlement confirmation
        println!("Step 12: Beneficiaries receive settlement confirmation");

        // Receive at VASP and wallet
        let received_settle_vasp: Settle = beneficiary_vasp
            .receive_message(&packed_settle_vasp)
            .await?;
        let _received_settle_wallet: Settle = beneficiary_wallet
            .receive_message(&packed_settle_wallet)
            .await?;

        println!("Settlement received by beneficiary VASP and wallet:");
        println!("  Transfer ID: {}", received_settle_vasp.transaction_id);
        println!("  Settlement ID: {}", received_settle_vasp.settlement_id);
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
) -> Result<(Arc<DefaultAgent>, String)> {
    // Create agent configuration
    let agent_config = AgentConfig::new(did.to_string());

    // Create DID resolver with proper error handling
    let mut did_resolver = MultiResolver::new();
    did_resolver.register_method("key", KeyResolver::new());
    let did_resolver = Arc::new(did_resolver);

    // Create secret resolver with the agent's key
    let mut secret_resolver = BasicSecretResolver::new();

    // Create a proper Ed25519 key
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

    // Add the secret to the resolver
    secret_resolver.add_secret(did, secret);
    let secret_resolver = Arc::new(secret_resolver);

    // Create message packer with proper DID and secret resolvers
    let message_packer = Arc::new(DefaultMessagePacker::new(
        did_resolver,
        secret_resolver,
        true,
    ));

    // Create agent
    let agent = Arc::new(DefaultAgent::new(agent_config, message_packer));

    Ok((agent, did.to_string()))
}
