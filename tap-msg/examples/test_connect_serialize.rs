use tap_msg::message::connection::Connect;

fn main() {
    // Create a simple Connect message using the backward compatible constructor
    let connect = Connect::new(
        "test-transaction-123",
        "did:example:agent1",
        "did:example:principal",
        Some("originator"),
    );

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&connect).unwrap();

    println!("Connect message serialized to JSON:");
    println!("{}", json);

    // Check if transaction_id is present
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();
    if json_value.get("transaction_id").is_some() {
        println!("\n✗ transaction_id IS present in the JSON (it should not be)");
    } else {
        println!("\n✓ transaction_id is NOT present in the JSON (as expected)");
    }

    // Also check what fields ARE present
    println!("\nFields present in JSON:");
    if let Some(obj) = json_value.as_object() {
        for key in obj.keys() {
            println!("  - {}", key);
        }
    }
}
