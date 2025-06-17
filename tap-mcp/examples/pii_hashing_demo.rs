//! Demonstration of PII hashing for TAP transfers

use tap_msg::message::Party;
use tap_msg::utils::NameHashable;

fn main() {
    println!("TAP Transfer PII Hashing Demonstration\n");

    // Example 1: Natural Person - Alice Lee
    println!("1. Natural Person Example:");
    println!("   Name: Alice Lee");

    let alice = Party::new("did:example:alice")
        .with_name_hash("Alice Lee")
        .with_country("US");

    println!("   Party JSON:");
    let alice_json = serde_json::to_string_pretty(&alice).unwrap();
    println!("{}", alice_json);

    // Example 2: Organization - Example Corp
    println!("\n2. Organization Example:");
    println!("   Legal Name: Example Corporation Ltd.");
    println!("   LEI: 549300ZFEEJ2IP5VME73");

    let corp = Party::new("did:web:example.com")
        .with_lei("549300ZFEEJ2IP5VME73")
        .with_country("US");

    // For organizations, we could also add legal name in metadata
    let mut corp_with_name = corp;
    corp_with_name.add_metadata(
        "legalName".to_string(),
        serde_json::Value::String("Example Corporation Ltd.".to_string()),
    );

    println!("   Party JSON:");
    let corp_json = serde_json::to_string_pretty(&corp_with_name).unwrap();
    println!("{}", corp_json);

    // Example 3: Show the difference
    println!("\n3. Privacy Comparison:");
    println!("   Without hashing (DO NOT USE IN PRODUCTION):");
    let mut alice_with_pii = Party::new("did:example:alice");
    alice_with_pii.add_metadata("givenName".to_string(), serde_json::json!("Alice"));
    alice_with_pii.add_metadata("familyName".to_string(), serde_json::json!("Lee"));
    println!("{}", serde_json::to_string_pretty(&alice_with_pii).unwrap());

    println!("\n   With hashing (RECOMMENDED):");
    println!("{}", alice_json);

    // Example 4: Name hash verification
    println!("\n4. Name Hash Verification:");
    struct Hasher;
    impl NameHashable for Hasher {}

    let hash1 = Hasher::hash_name("Alice Lee");
    let hash2 = Hasher::hash_name("alice lee"); // Different case
    let hash3 = Hasher::hash_name("Alice  Lee"); // Extra space

    println!("   'Alice Lee'  -> {}", hash1);
    println!("   'alice lee'  -> {}", hash2);
    println!("   'Alice  Lee' -> {}", hash3);
    println!("   All hashes match: {}", hash1 == hash2 && hash2 == hash3);
}
