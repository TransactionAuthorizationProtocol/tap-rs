//! Example demonstrating a multi-agent TAIP-3 transfer flow with TAIP-4 authorization
//! 
//! This example shows a more complex scenario with multiple agents:
//! 1. Originator VASP and Wallet
//! 2. Beneficiary VASP and Wallet
//! 3. Rejection handling and recovery
//! 
//! Run with: cargo run --example multi_agent_flow

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use didcomm::secrets::{Secret, SecretMaterial, SecretType};
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{BasicSecretResolver, DefaultMessagePacker};
use tap_agent::did::{KeyResolver, MultiResolver};
use tap_agent::error::Result;
use tap_caip::AssetId;
use tap_msg::message::{Authorize, Reject, Settle, Transfer};
use tap_msg::Participant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Multi-Agent TAIP-3 Transfer Flow with TAIP-4 Authorization ===\n");
    
    // Step 1: Set up agents
    // In a real scenario, these would be different entities
    
    // Create originator VASP agent
    let (originator_vasp, originator_vasp_did) = create_agent(
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo",
        "nWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Yh",
    ).await?;
    
    // Create originator wallet agent
    let (originator_wallet, originator_wallet_did) = create_agent(
        "did:key:z6MkhFvVnYxkqLNEiWQmUwhQuVpXiCfNmRUVi5yZ4Cg9w15k",
        "8zYZK2vvsAyVYpNpnYzTnUPjBuWdWpYmPpQmwErV9XQg",
        "8zYZK2vvsAyVYpNpnYzTnUPjBuWdWpYmPpQmwErV9XQg",
    ).await?;
    
    // Create beneficiary VASP agent
    let (beneficiary_vasp, beneficiary_vasp_did) = create_agent(
        "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
        "B12NYF8RrR3h41TDCTJojY59usg3mbtbjnFs7Eud1Y6u",
        "B12NYF8RrR3h41TDCTJojY59usg3mbtbjnFs7Eud1Y6u",
    ).await?;
    
    // Create beneficiary wallet agent
    let (beneficiary_wallet, beneficiary_wallet_did) = create_agent(
        "did:key:z6MkrJVkLHBdQQS5y2CnXAHJcgBWMVv7V5aukAtQyBx4qJA4",
        "5TVS4YKJxmqVVQUM7xbVsYiFrCbdwgLLdu6QB98q3a4",
        "5TVS4YKJxmqVVQUM7xbVsYiFrCbdwgLLdu6QB98q3a4",
    ).await?;
    
    println!("Created agents with DIDs:");
    println!("  Originator VASP: {}", originator_vasp_did);
    println!("  Originator Wallet: {}", originator_wallet_did);
    println!("  Beneficiary VASP: {}", beneficiary_vasp_did);
    println!("  Beneficiary Wallet: {}\n", beneficiary_wallet_did);
    
    // Step 2: Create a transfer request
    println!("Step 1: Originator VASP creates a transfer request");
    
    // Create a transfer message
    let asset = match AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f") {
        Ok(asset) => asset,
        Err(e) => {
            println!("Error parsing asset ID: {}", e);
            return Err(tap_agent::error::Error::Validation(format!("Invalid asset ID: {}", e)));
        }
    };
    let transfer_id = uuid::Uuid::new_v4().to_string();
    
    let transfer = Transfer {
        asset,
        originator: Participant {
            id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
            role: Some("originator".to_string()),
            policies: None,
            leiCode: None,
        },
        beneficiary: Some(Participant {
            id: "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH".to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
        }),
        amount: "100.0".to_string(),
        agents: vec![
            Participant {
                id: "did:key:z6MkhFvVnYxkqLNEiWQmUwhQuVpXiCfNmRUVi5yZ4Cg9w15k".to_string(),
                role: Some("originatorWallet".to_string()),
                policies: None,
                leiCode: None,
            },
            Participant {
                id: "did:key:z6MkrJVkLHBdQQS5y2CnXAHJcgBWMVv7V5aukAtQyBx4qJA4".to_string(),
                role: Some("beneficiaryWallet".to_string()),
                policies: None,
                leiCode: None,
            },
        ],
        settlement_id: None,
        memo: Some("Multi-agent transfer example".to_string()),
        metadata: HashMap::new(),
    };
    
    println!("Transfer details:");
    println!("  Asset: {}", transfer.asset);
    println!("  Amount: {}", transfer.amount);
    println!("  From: {}", transfer.originator.id);
    println!("  To: {}\n", transfer.beneficiary.as_ref().unwrap().id);
    
    // Step 3: Send the transfer to both the beneficiary VASP and wallet
    println!("Step 2: Sending transfer to beneficiary VASP and wallet");
    
    // Pack and send to VASP
    let packed_transfer_vasp = originator_vasp.send_message(&transfer, &beneficiary_vasp_did).await?;
    
    // Pack and send to wallet
    let packed_transfer_wallet = originator_vasp.send_message(&transfer, &beneficiary_wallet_did).await?;
    
    println!("Transfer sent to both beneficiary VASP and wallet\n");
    
    // Step 4: Beneficiaries receive the transfer
    println!("Step 3: Beneficiaries receive the transfer");
    
    // Receive at VASP and wallet
    let _received_transfer_vasp: Transfer = beneficiary_vasp.receive_message(&packed_transfer_vasp).await?;
    let _received_transfer_wallet: Transfer = beneficiary_wallet.receive_message(&packed_transfer_wallet).await?;
    
    println!("Transfer received by both beneficiary VASP and wallet\n");
    
    // Step 5: Initial rejection by beneficiary VASP
    println!("Let's assume the beneficiary VASP initially rejects the transfer due to compliance concerns");
    
    let reject = Reject {
        transfer_id: transfer_id.clone(),
        reason: "compliance.policy: Additional beneficiary information required".to_string(),
    };
    
    let packed_reject = beneficiary_vasp.send_message(&reject, &originator_vasp_did).await?;
    
    // Originator receives the rejection
    let received_reject: Reject = originator_vasp.receive_message(&packed_reject).await?;
    println!("Originator VASP received rejection:");
    println!("  Transfer ID: {}", received_reject.transfer_id);
    println!("  Reason: {}\n", received_reject.reason);
    
    // Step 4: After resolving the compliance concerns (in a real scenario), 
    // the beneficiary VASP now authorizes the transfer
    println!("Step 4: After resolving compliance concerns, beneficiary VASP authorizes the transfer");
    
    let authorize = Authorize {
        transfer_id: transfer_id.clone(),
        note: Some("Compliance requirements satisfied, transfer authorized".to_string()),
    };
    
    let packed_authorize_vasp = beneficiary_vasp.send_message(&authorize, &originator_vasp_did).await?;
    
    // Originator receives the authorization
    let received_authorize: Authorize = originator_vasp.receive_message(&packed_authorize_vasp).await?;
    println!("Originator VASP received authorization:");
    println!("  Transfer ID: {}", received_authorize.transfer_id);
    if let Some(note) = received_authorize.note {
        println!("  Note: {}\n", note);
    }
    
    // Step 5: Beneficiary wallet also authorizes the transfer
    println!("Step 5: Beneficiary wallet also authorizes the transfer");
    
    let authorize_wallet = Authorize {
        transfer_id: transfer_id.clone(),
        note: Some("Wallet ready to receive funds".to_string()),
    };
    
    let packed_authorize_wallet = beneficiary_wallet.send_message(&authorize_wallet, &originator_wallet_did).await?;
    
    // Originator wallet receives the authorization
    let received_authorize_wallet: Authorize = originator_wallet.receive_message(&packed_authorize_wallet).await?;
    println!("Originator wallet received authorization:");
    println!("  Transfer ID: {}", received_authorize_wallet.transfer_id);
    if let Some(note) = received_authorize_wallet.note {
        println!("  Note: {}\n", note);
    }
    
    // Step 6: Originator wallet initiates settlement
    println!("Step 6: Originator wallet initiates settlement");
    
    // In a real scenario, the wallet would submit the transaction to the blockchain
    // and get a transaction ID. Here we simulate it with a mock transaction ID.
    let settlement_id = "eip155:1:tx/0x3edb98c24d46d148eb926c714f4fbaa117c47b0c0821f38bfce9763604457c33";
    
    let settle = Settle {
        transfer_id: transfer_id.clone(),
        settlement_id: settlement_id.to_string(),
        amount: Some(transfer.amount.clone()),
    };
    
    // Send settlement to both VASP and wallet
    let packed_settle_vasp = originator_wallet.send_message(&settle, &beneficiary_vasp_did).await?;
    let packed_settle_wallet = originator_wallet.send_message(&settle, &beneficiary_wallet_did).await?;
    
    println!("Settlement sent to both beneficiary VASP and wallet");
    println!("  Settlement ID: {}\n", settlement_id);
    
    // Step 7: Beneficiaries receive settlement confirmation
    println!("Step 7: Beneficiaries receive settlement confirmation");
    
    // Receive at VASP and wallet
    let received_settle_vasp: Settle = beneficiary_vasp.receive_message(&packed_settle_vasp).await?;
    let _received_settle_wallet: Settle = beneficiary_wallet.receive_message(&packed_settle_wallet).await?;
    
    println!("Settlement received by both beneficiary VASP and wallet:");
    println!("  Transfer ID: {}", received_settle_vasp.transfer_id);
    println!("  Settlement ID: {}", received_settle_vasp.settlement_id);
    if let Some(amount) = received_settle_vasp.amount {
        println!("  Amount: {}", amount);
    }
    
    println!("\n=== Multi-agent transfer flow completed successfully ===");
    
    Ok(())
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
    ));
    
    // Create agent
    let agent = Arc::new(DefaultAgent::new(agent_config, message_packer));
    
    Ok((agent, did.to_string()))
}
