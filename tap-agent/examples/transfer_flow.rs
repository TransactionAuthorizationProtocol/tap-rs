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
use std::sync::Arc;

use didcomm::secrets::{Secret, SecretMaterial, SecretType};
use tap_agent::agent::{Agent, DefaultAgent};
use tap_agent::config::AgentConfig;
use tap_agent::crypto::{BasicSecretResolver, DefaultMessagePacker};
use tap_agent::did::{KeyResolver, MultiResolver};
use tap_caip::AssetId;
use tap_msg::message::{Authorize, Settle, Transfer};
use tap_msg::Participant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TAIP-3 Transfer Flow with TAIP-4 Authorization ===\n");
    
    // Create originator agent
    let (originator_agent, originator_did) = create_agent(
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo",
        "nWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Yh",
    ).await;
    
    // Create beneficiary agent
    let (beneficiary_agent, beneficiary_did) = create_agent(
        "did:key:z6MkhFvVnYxkqLNEiWQmUwhQuVpXiCfNmRUVi5yZ4Cg9w15k",
        "8zYZK2vvsAyVYpNpnYzTnUPjBuWdWpYmPpQmwErV9XQg",
        "8zYZK2vvsAyVYpNpnYzTnUPjBuWdWpYmPpQmwErV9XQg",
    ).await;
    
    println!("Created originator agent with DID: {}", originator_did);
    println!("Created beneficiary agent with DID: {}\n", beneficiary_did);
    
    // Create a settlement address (in a real scenario, this would be a blockchain address)
    let settlement_address = "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb";
    
    // Step 1: Originator creates and sends a transfer request
    println!("Step 1: Originator creates a transfer request");
    
    let transfer = create_transfer_message(&originator_did, &beneficiary_did, settlement_address);
    println!("Transfer details:");
    println!("  Asset: {}", transfer.asset);
    println!("  Amount: {}", transfer.amount);
    println!("  From: {}", transfer.originator.id);
    println!("  To: {}\n", transfer.beneficiary.as_ref().unwrap().id);
    
    // Pack the transfer message
    let packed_transfer = originator_agent.send_message(&transfer, &beneficiary_did).await?;
    println!("Originator sends the transfer request to the beneficiary\n");
    
    // Step 2: Beneficiary receives and processes the transfer request
    println!("Step 2: Beneficiary receives and processes the transfer request");
    
    let received_transfer: Transfer = beneficiary_agent.receive_message(&packed_transfer).await?;
    println!("Beneficiary received transfer request:");
    println!("  Asset: {}", received_transfer.asset);
    println!("  Amount: {}", received_transfer.amount);
    println!("  From: {}", received_transfer.originator.id);
    println!("  To: {}\n", received_transfer.beneficiary.as_ref().unwrap().id);
    
    // Step 3: Beneficiary authorizes the transfer
    println!("Step 3: Beneficiary authorizes the transfer");
    
    // Generate a unique transfer ID (in a real scenario, this would be from the original transfer)
    let transfer_id = uuid::Uuid::new_v4().to_string();
    
    let authorize = Authorize {
        transfer_id: transfer_id.clone(),
        note: Some(format!("Authorizing transfer to settlement address: {}", settlement_address)),
    };
    
    let packed_authorize = beneficiary_agent.send_message(&authorize, &originator_did).await?;
    println!("Beneficiary sends authorization to the originator\n");
    
    // Step 4: Originator receives the authorization
    println!("Step 4: Originator receives the authorization");
    
    let received_authorize: Authorize = originator_agent.receive_message(&packed_authorize).await?;
    println!("Originator received authorization:");
    println!("  Transfer ID: {}", received_authorize.transfer_id);
    if let Some(note) = received_authorize.note {
        println!("  Note: {}\n", note);
    }
    
    // Step 5: Originator settles the transfer
    println!("Step 5: Originator settles the transfer");
    
    // In a real scenario, the originator would submit the transaction to the blockchain
    // and get a transaction ID. Here we simulate it with a mock transaction ID.
    let settlement_id = "eip155:1:tx/0x3edb98c24d46d148eb926c714f4fbaa117c47b0c0821f38bfce9763604457c33";
    
    let settle = Settle {
        transfer_id: transfer_id.clone(),
        settlement_id: settlement_id.to_string(),
        amount: Some(transfer.amount.clone()),
    };
    
    let packed_settle = originator_agent.send_message(&settle, &beneficiary_did).await?;
    println!("Originator sends settlement confirmation to the beneficiary");
    println!("  Settlement ID: {}\n", settlement_id);
    
    // Step 6: Beneficiary receives the settlement confirmation
    println!("Step 6: Beneficiary receives the settlement confirmation");
    
    let received_settle: Settle = beneficiary_agent.receive_message(&packed_settle).await?;
    println!("Beneficiary received settlement confirmation:");
    println!("  Transfer ID: {}", received_settle.transfer_id);
    println!("  Settlement ID: {}", received_settle.settlement_id);
    println!("  Amount: {}\n", received_settle.amount.unwrap_or_default());
    
    println!("=== Transfer flow completed successfully ===");
    
    Ok(())
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
    let message_packer = Arc::new(DefaultMessagePacker::new(
        did_resolver,
        secret_resolver,
    ));
    
    // Create agent
    let agent = Arc::new(DefaultAgent::new(agent_config, message_packer));
    
    (agent, did.to_string())
}

/// Create a transfer message
fn create_transfer_message(
    originator_did: &str,
    beneficiary_did: &str,
    settlement_address: &str,
) -> Transfer {
    // Create originator and beneficiary participants
    let originator = Participant {
        id: originator_did.to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };
    
    let beneficiary = Participant {
        id: beneficiary_did.to_string(),
        role: Some("beneficiary".to_string()),
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
    
    // Create a transfer message
    Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents: vec![settlement_agent],
        settlement_id: None,
        memo: Some("Example transfer".to_string()),
        metadata: HashMap::new(),
    }
}
