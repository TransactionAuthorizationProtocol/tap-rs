use std::collections::HashMap;
use tap_caip::{AssetId, ChainId};
use tap_msg::message::agent::Agent;
use tap_msg::message::connection::Connect;
use tap_msg::message::party::Party;
use tap_msg::message::transfer::Transfer;

fn main() {
    println!("Testing transaction_id serialization in TAP messages\n");

    // Test Connect message
    println!("1. Testing Connect message:");
    let connect = Connect::new(
        "test-transaction-123",
        "did:example:agent1",
        "did:example:principal",
        Some("originator"),
    );

    let connect_json = serde_json::to_string_pretty(&connect).unwrap();
    println!("Connect JSON:");
    println!("{}", connect_json);

    let connect_value: serde_json::Value = serde_json::from_str(&connect_json).unwrap();
    if connect_value.get("transaction_id").is_some() {
        println!("❌ transaction_id IS present in Connect JSON (should be skipped)");
    } else {
        println!("✅ transaction_id is NOT present in Connect JSON");
    }

    // Test Transfer message
    println!("\n2. Testing Transfer message:");

    // Create chain ID and asset ID properly
    let chain_id = ChainId::new("eip155", "1").unwrap();
    let asset = AssetId::new(
        chain_id,
        "erc20",
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
    )
    .unwrap();

    let transfer = Transfer {
        transaction_id: Some("transfer-tx-456".to_string()),
        originator: Some(Party::new("did:example:originator")),
        beneficiary: Some(Party::new("did:example:beneficiary")),
        agents: vec![Agent::new(
            "did:example:agent1",
            "originator",
            "did:example:originator",
        )],
        asset,
        amount: "100.50".to_string(),
        settlement_id: None,
        memo: None,
        connection_id: None,
        metadata: HashMap::new(),
    };

    let transfer_json = serde_json::to_string_pretty(&transfer).unwrap();
    println!("Transfer JSON:");
    println!("{}", transfer_json);

    let transfer_value: serde_json::Value = serde_json::from_str(&transfer_json).unwrap();
    if transfer_value.get("transaction_id").is_some() {
        println!("❌ transaction_id IS present in Transfer JSON (should be skipped)");
    } else {
        println!("✅ transaction_id is NOT present in Transfer JSON");
    }

    println!("\nConclusion:");
    println!("According to CLAUDE.local.md guidelines:");
    println!("- transaction_id should NOT be serialized in the message body");
    println!("- It should map to 'thid' in the parent didcomm message");
    println!("- Or to 'id' if it's an Initiator message like Transfer, Payment, or Connect");
}
