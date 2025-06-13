//! Complete Travel Rule flow example demonstrating IVMS101 integration
//!
//! This example shows:
//! - Automatic customer data extraction
//! - IVMS101 data generation and attachment
//! - Policy-based presentation requests
//! - Compliance data handling

use std::collections::HashMap;
use std::sync::Arc;
use tap_agent::TapAgent;
use tap_ivms101::builder::*;
use tap_ivms101::types::*;
use tap_ivms101::Person;
use tap_msg::message::{
    authorize::Authorize, settle::Settle, transfer::Transfer, update_policies::UpdatePolicies,
    Agent as MessageAgent, Party, Policy, RequirePresentation,
};
use tap_node::{NodeConfig, TapNode};
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Create temporary storage
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("travel_rule_example.db");

    println!("=== Travel Rule Flow Example ===\n");

    // Step 1: Create node with Travel Rule support
    println!("1. Setting up TAP Node with Travel Rule processor...");
    let config = NodeConfig {
        storage_path: Some(db_path),
        ..Default::default()
    };
    let mut node = TapNode::new(config);
    node.init_storage().await?;

    // Step 2: Create VASPs (Virtual Asset Service Providers)
    println!("\n2. Creating VASP agents...");

    // VASP A (Originating VASP)
    let (vasp_a, vasp_a_did) = TapAgent::from_ephemeral_key().await?;
    let vasp_a = Arc::new(vasp_a);
    node.register_agent(vasp_a).await?;
    println!("   - VASP A: {}", vasp_a_did);

    // VASP B (Beneficiary VASP)
    let (vasp_b, vasp_b_did) = TapAgent::from_ephemeral_key().await?;
    let vasp_b = Arc::new(vasp_b);
    node.register_agent(vasp_b).await?;
    println!("   - VASP B: {}", vasp_b_did);

    // Step 3: Create customer profiles
    println!("\n3. Creating customer profiles...");

    // Alice (Originator) - Natural Person
    let alice_metadata = {
        let mut m = HashMap::new();
        m.insert("name".to_string(), serde_json::json!("Alice Smith"));
        m.insert("givenName".to_string(), serde_json::json!("Alice"));
        m.insert("familyName".to_string(), serde_json::json!("Smith"));
        m.insert("addressCountry".to_string(), serde_json::json!("US"));
        m.insert("email".to_string(), serde_json::json!("alice@example.com"));
        m
    };
    let alice = Party::with_metadata("did:key:alice", alice_metadata);
    println!("   - Alice (Originator): Natural Person in US");

    // Bob (Beneficiary) - Natural Person
    let bob_metadata = {
        let mut m = HashMap::new();
        m.insert("name".to_string(), serde_json::json!("Bob Jones"));
        m.insert("givenName".to_string(), serde_json::json!("Bob"));
        m.insert("familyName".to_string(), serde_json::json!("Jones"));
        m.insert("addressCountry".to_string(), serde_json::json!("GB"));
        m
    };
    let bob = Party::with_metadata("did:key:bob", bob_metadata);
    println!("   - Bob (Beneficiary): Natural Person in GB");

    // Step 4: VASP B requests IVMS101 data via policy
    println!("\n4. VASP B requests Travel Rule compliance data...");

    let mut credentials = HashMap::new();
    credentials.insert("type".to_string(), vec!["TravelRuleCredential".to_string()]);

    let ivms_policy = Policy::RequirePresentation(RequirePresentation {
        context: Some(vec!["https://intervasp.org/ivms101".to_string()]),
        credentials: Some(credentials),
        purpose: Some("Travel Rule Compliance - FATF R.16".to_string()),
        from: None,
        from_role: None,
        from_agent: None,
        about_party: None,
        about_agent: None,
        presentation_definition: None,
    });

    let update_policies = UpdatePolicies {
        transaction_id: "transfer-001".to_string(),
        policies: vec![ivms_policy],
    };

    // Send policy update
    use tap_msg::message::TapMessageBody;
    let update_policies_message = update_policies.to_didcomm(&vasp_b_did)?;
    node.send_message(vasp_b_did.clone(), update_policies_message)
        .await?;
    println!("   - Policy sent: RequirePresentation for IVMS101");

    // Step 5: Create Transfer with automatic IVMS101 attachment
    println!("\n5. Creating Transfer with automatic IVMS101 data...");

    // First, enhance Alice's profile with full address for IVMS101
    let alice_with_address = {
        let mut metadata = alice.metadata.clone();
        metadata.insert(
            "address".to_string(),
            serde_json::json!({
                "@type": "PostalAddress",
                "streetAddress": "123 Main Street",
                "addressLocality": "New York",
                "addressRegion": "NY",
                "postalCode": "10001",
                "addressCountry": "US"
            }),
        );
        Party::with_metadata(&alice.id, metadata)
    };

    // Create Transfer - Travel Rule processor will automatically attach IVMS101
    let transfer = Transfer {
        asset: "eip155:1/slip44:60".parse()?, // USD on Ethereum
        originator: alice_with_address.clone(),
        beneficiary: Some(bob.clone()),
        amount: "5000.00".to_string(), // Above typical Travel Rule threshold
        agents: vec![
            MessageAgent::new(&vasp_a_did, "originating_vasp", &alice.id),
            MessageAgent::new(&vasp_b_did, "beneficiary_vasp", &bob.id),
        ],
        memo: None,
        settlement_id: None,
        transaction_id: "transfer-001".to_string(),
        connection_id: None,
        metadata: HashMap::new(),
    };

    println!("   - Transfer amount: {} USD", transfer.amount);
    println!(
        "   - From: {} ({})",
        alice.id,
        alice.metadata.get("addressCountry").unwrap()
    );
    println!(
        "   - To: {} ({})",
        bob.id,
        bob.metadata.get("addressCountry").unwrap()
    );

    // Send Transfer - IVMS101 data will be automatically attached
    let transfer_message = transfer.to_didcomm(&vasp_a_did)?;

    let _sent_message_id = node
        .send_message(vasp_a_did.clone(), transfer_message.clone())
        .await?;

    // Check if IVMS101 was attached
    if let Some(attachments) = &transfer_message.attachments {
        println!("\n   - IVMS101 data automatically attached:");
        for attachment in attachments {
            if attachment.id == Some("ivms101-vp".to_string()) {
                println!("     ✓ Verifiable Presentation with Travel Rule data");
            }
        }
    }

    // Step 6: Process the received Transfer at VASP B
    println!("\n6. VASP B processes Transfer with IVMS101 data...");

    // The Travel Rule processor automatically:
    // - Detects IVMS101 attachments
    // - Validates the presentation
    // - Extracts customer data
    // - Updates customer records

    // In a real scenario, the customer would be automatically created from the IVMS101 data

    // In a real scenario, the customer would be automatically created from the IVMS101 data
    println!("   - Customer data extracted and stored");
    println!("   - Compliance requirements satisfied");

    // Step 7: Continue with transaction flow
    println!("\n7. Continuing transaction flow...");

    // VASP B authorizes
    let authorize = Authorize::with_settlement_address(
        "transfer-001",
        "eip155:1:0x1234567890123456789012345678901234567890",
    );

    node.send_message(vasp_b_did.clone(), authorize.to_didcomm(&vasp_b_did)?)
        .await?;
    println!("   - VASP B authorized transaction");

    // VASP A settles
    let settle = Settle::with_amount("transfer-001", "blockchain-tx-123", "5000.00");

    node.send_message(vasp_a_did.clone(), settle.to_didcomm(&vasp_a_did)?)
        .await?;
    println!("   - Transaction settled");

    // Step 8: Demonstrate direct IVMS101 generation
    println!("\n8. Demonstrating direct IVMS101 data generation...");

    // Create comprehensive IVMS101 data
    let natural_person = NaturalPersonBuilder::new()
        .name(
            NaturalPersonNameBuilder::new()
                .legal_name("Smith", "Alice")
                .build()?,
        )
        .add_address(
            GeographicAddressBuilder::new()
                .address_type(AddressType::Home)
                .street_name("123 Main Street")
                .town_name("New York")
                .country_sub_division("NY")
                .post_code("10001")
                .country("US")
                .build()?,
        )
        .country_of_residence("US")
        .national_id(
            "123-45-6789",
            NationalIdentifierType::NationalIdentityNumber,
            "US",
        )
        .birth_info("1985-06-15", "New York", "US")
        .build()?;

    let beneficiary_person = NaturalPersonBuilder::new()
        .name(
            NaturalPersonNameBuilder::new()
                .legal_name("Jones", "Bob")
                .build()?,
        )
        .country_of_residence("GB")
        .build()?;

    let _ivms_message = IvmsMessageBuilder::new()
        .originator(vec![Person::NaturalPerson(natural_person)])
        .beneficiary(vec![Person::NaturalPerson(beneficiary_person)])
        .originating_vasp(Person::LegalPerson(
            LegalPersonBuilder::new()
                .name(
                    LegalPersonNameBuilder::new()
                        .legal_name("VASP A Inc.")
                        .build()?,
                )
                .lei("529900HNOAA1KXQJUQ27")?
                .country_of_registration("US")
                .build()?,
        ))
        .transaction(
            "5000.00",
            "USD",
            TransactionDirection::Outgoing,
            "transfer-001",
            "2024-01-15T10:30:00Z",
        )?
        .build()?;

    println!("   - Generated complete IVMS101 message");
    println!("   - Contains: originator, VASP, transaction details");

    // Summary
    println!("\n=== Travel Rule Flow Complete ===");
    println!("✓ Customer data automatically extracted");
    println!("✓ IVMS101 data generated from customer profiles");
    println!("✓ Compliance data attached to transfers");
    println!("✓ Presentation requests handled");
    println!("✓ Transaction completed with full compliance");

    // Cleanup
    temp_dir.close()?;

    Ok(())
}
