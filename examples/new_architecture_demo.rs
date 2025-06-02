//! Demonstration of the new TAP architecture
//!
//! This example shows how the refactored TAP system works with:
//! - Standalone agents
//! - TAP Node integration
//! - Optimized message routing

use std::sync::Arc;
use serde_json::json;
use tap_node::{TapNode, NodeConfig};
use tap_agent::{TapAgent, PackOptions, Packable, SecurityMode, verify_jws, MultiResolver};
use tap_msg::didcomm::PlainMessage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 TAP Architecture Demo");
    println!("========================\n");

    // Demo 1: Standalone Agent Usage
    demo_standalone_agent().await?;
    
    // Demo 2: TAP Node with Multiple Agents
    demo_node_integration().await?;
    
    // Demo 3: Message Routing Optimization
    demo_message_routing().await?;
    
    println!("✅ Demo completed successfully!");
    Ok(())
}

async fn demo_standalone_agent() -> Result<(), Box<dyn std::error::Error>> {
    println!("📱 Demo 1: Standalone Agent Usage");
    println!("----------------------------------");
    
    // Create two standalone agents
    let (alice_agent, alice_did) = TapAgent::from_ephemeral_key().await?;
    let (bob_agent, bob_did) = TapAgent::from_ephemeral_key().await?;
    
    println!("👤 Alice DID: {}", alice_did);
    println!("👤 Bob DID: {}", bob_did);
    
    // Alice creates a message for Bob
    let message = PlainMessage {
        id: "standalone-demo-1".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/transfer".to_string(),
        body: json!({
            "amount": "25.00",
            "currency": "USD",
            "memo": "Coffee payment"
        }),
        from: alice_did.clone(),
        to: vec![bob_did.clone()],
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };
    
    // Alice encrypts the message for Bob
    let alice_kid = format!("{}#keys-1", alice_did);
    let bob_kid = format!("{}#keys-1", bob_did);
    
    let pack_options = PackOptions {
        security_mode: SecurityMode::AuthCrypt,
        sender_kid: Some(alice_kid),
        recipient_kid: Some(bob_kid),
    };
    
    println!("🔐 Alice encrypting message for Bob...");
    let encrypted_message = message.pack(&*alice_agent.key_manager(), pack_options).await?;
    
    // Bob receives and processes the message (standalone mode)
    println!("📨 Bob receiving encrypted message...");
    let decrypted_message = bob_agent.receive_message(&encrypted_message).await?;
    
    println!("✅ Bob successfully decrypted message:");
    println!("   ID: {}", decrypted_message.id);
    println!("   Type: {}", decrypted_message.type_);
    println!("   Amount: {}", decrypted_message.body["amount"]);
    println!();
    
    Ok(())
}

async fn demo_node_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏢 Demo 2: TAP Node Integration");
    println!("-------------------------------");
    
    // Create TAP Node
    let config = NodeConfig::default();
    let node = Arc::new(TapNode::new(config));
    println!("🏗️  Created TAP Node");
    
    // Create and register multiple agents
    let (alice_agent, alice_did) = TapAgent::from_ephemeral_key().await?;
    let (bob_agent, bob_did) = TapAgent::from_ephemeral_key().await?;
    let (charlie_agent, charlie_did) = TapAgent::from_ephemeral_key().await?;
    
    node.register_agent(Arc::new(alice_agent)).await?;
    node.register_agent(Arc::new(bob_agent)).await?;
    node.register_agent(Arc::new(charlie_agent)).await?;
    
    println!("👥 Registered 3 agents with the node:");
    println!("   Alice: {}", alice_did);
    println!("   Bob: {}", bob_did);
    println!("   Charlie: {}", charlie_did);
    
    // External sender creates encrypted message for Bob
    let (sender_agent, sender_did) = TapAgent::from_ephemeral_key().await?;
    
    let message = PlainMessage {
        id: "node-demo-1".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/payment".to_string(),
        body: json!({
            "payment_id": "pay-12345",
            "amount": "100.00",
            "currency": "EUR",
            "beneficiary": "Bob"
        }),
        from: sender_did.clone(),
        to: vec![bob_did.clone()],
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };
    
    // Encrypt for Bob
    let sender_kid = format!("{}#keys-1", sender_did);
    let bob_kid = format!("{}#keys-1", bob_did);
    
    let pack_options = PackOptions {
        security_mode: SecurityMode::AuthCrypt,
        sender_kid: Some(sender_kid),
        recipient_kid: Some(bob_kid),
    };
    
    let encrypted_message = message.pack(&*sender_agent.key_manager(), pack_options).await?;
    let message_value: serde_json::Value = serde_json::from_str(&encrypted_message)?;
    
    println!("🔐 External sender created encrypted message for Bob");
    
    // Node processes the message - automatically routes to Bob
    println!("🚀 TAP Node processing encrypted message...");
    node.receive_message(message_value).await?;
    
    println!("✅ Node successfully routed encrypted message to Bob!");
    println!("   (Bob's agent decrypted and processed the message)");
    println!();
    
    Ok(())
}

async fn demo_message_routing() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗺️  Demo 3: Message Routing Optimization");
    println!("----------------------------------------");
    
    // Create node and agents
    let config = NodeConfig::default();
    let node = Arc::new(TapNode::new(config));
    
    let (agent1, agent1_did) = TapAgent::from_ephemeral_key().await?;
    let (agent2, agent2_did) = TapAgent::from_ephemeral_key().await?;
    let (signer_agent, signer_did) = TapAgent::from_ephemeral_key().await?;
    
    node.register_agent(Arc::new(agent1)).await?;
    node.register_agent(Arc::new(agent2)).await?;
    
    println!("📋 Testing different message types:");
    
    // Test 1: Plain Message
    println!("\n1️⃣  Plain Message Routing:");
    let plain_message = json!({
        "id": "plain-routing-test",
        "typ": "application/didcomm-plain+json",
        "type": "https://example.org/test",
        "body": {"content": "Plain message for agent1"},
        "from": "did:example:external",
        "to": [agent1_did.clone()],
        "created_time": chrono::Utc::now().timestamp()
    });
    
    node.receive_message(plain_message).await?;
    println!("   ✅ Plain message routed to agent1");
    
    // Test 2: Signed Message (would need proper DID resolution in production)
    println!("\n2️⃣  Signed Message Routing:");
    println!("   📝 Note: Signature verification requires DID resolution");
    println!("   🔍 In production, the node resolver would verify signatures");
    println!("   ⚡ Optimization: Signed messages verified ONCE for all agents");
    
    // Test 3: Encrypted Message Routing
    println!("\n3️⃣  Encrypted Message Routing:");
    
    // Create encrypted message for agent2
    let secret_message = PlainMessage {
        id: "encrypted-routing-test".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0/confidential".to_string(),
        body: json!({
            "secret_data": "This is confidential for agent2",
            "classification": "restricted"
        }),
        from: signer_did.clone(),
        to: vec![agent2_did.clone()],
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };
    
    let signer_kid = format!("{}#keys-1", signer_did);
    let agent2_kid = format!("{}#keys-1", agent2_did);
    
    let pack_options = PackOptions {
        security_mode: SecurityMode::AuthCrypt,
        sender_kid: Some(signer_kid),
        recipient_kid: Some(agent2_kid),
    };
    
    let encrypted = secret_message.pack(&*signer_agent.key_manager(), pack_options).await?;
    let jwe_value: serde_json::Value = serde_json::from_str(&encrypted)?;
    
    node.receive_message(jwe_value).await?;
    println!("   ✅ Encrypted message automatically routed to agent2");
    println!("   🔑 Agent2 decrypted the message with its private key");
    println!("   🛡️  Agent1 cannot access the encrypted content");
    
    // Test 4: Multi-recipient scenario
    println!("\n4️⃣  Multi-Recipient Optimization:");
    println!("   📨 Encrypted messages: Each recipient agent handles decryption");
    println!("   📝 Signed messages: Verified once, then routed to all recipients");
    println!("   ⚡ This provides both security and efficiency");
    
    println!("\n🎯 Key Architecture Benefits:");
    println!("   • Standalone agents remain fully functional");
    println!("   • Node provides efficient centralized routing");
    println!("   • Signatures verified once, not per-agent");
    println!("   • Encrypted messages naturally distributed");
    println!("   • Clean separation of concerns");
    
    Ok(())
}

// Utility function to demonstrate verification
async fn demo_standalone_verification() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Bonus: Standalone Verification Demo");
    println!("------------------------------------");
    
    // Create agent and sign a message
    let (agent, agent_did) = TapAgent::from_ephemeral_key().await?;
    
    let message = PlainMessage {
        id: "verification-demo".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://example.org/verify-test".to_string(),
        body: json!({"data": "This message will be verified"}),
        from: agent_did.clone(),
        to: vec!["did:example:recipient".to_string()],
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: Default::default(),
    };
    
    // Sign the message
    let agent_kid = agent.key_manager().list_keys()?[0].clone();
    let generated_key = agent.key_manager().get_key(&agent_kid)?;
    let vm_id = generated_key.did_doc.verification_method[0].id.clone();
    
    let pack_options = PackOptions::new().with_sign(&vm_id);
    let signed_message = message.pack(&*agent.key_manager(), pack_options).await?;
    
    println!("📝 Created signed message with DID: {}", agent_did);
    
    // Parse as JWS
    let jws: tap_agent::Jws = serde_json::from_str(&signed_message)?;
    
    // In a real scenario, you would use a DID resolver
    // let resolver = MultiResolver::default();
    // let verified = verify_jws(&jws, &resolver).await?;
    
    println!("🔍 Signature can be verified using verify_jws() with a DID resolver");
    println!("   This enables nodes to verify signatures without private keys");
    
    Ok(())
}