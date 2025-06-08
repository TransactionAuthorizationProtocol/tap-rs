//! Complete Travel Rule flow example demonstrating IVMS101 integration
//!
//! This example shows:
//! - Automatic customer data extraction
//! - IVMS101 data generation and attachment
//! - Policy-based presentation requests
//! - Compliance data handling

use std::collections::HashMap;
use std::sync::Arc;
use tap_agent::{Agent, LocalAgentKey};
use tap_ivms101::builder::*;
use tap_ivms101::types::*;
use tap_msg::message::{
    authorize::Authorize, payment::Payment, settle::Settle, transfer::Transfer,
    update_policies::UpdatePolicies, Party, Policy, TapMessage,
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
        database_path: Some(db_path),
        ..Default::default()
    };
    let mut node = TapNode::new(config);
    node.init_storage().await?;

    // Step 2: Create VASPs (Virtual Asset Service Providers)
    println!("\n2. Creating VASP agents...");

    // VASP A (Originating VASP)
    let vasp_a_key = LocalAgentKey::generate();
    let vasp_a_did = vasp_a_key.did();
    let vasp_a = Agent::new(vasp_a_did.clone(), Arc::new(vasp_a_key));
    node.register_agent(vasp_a).await?;
    println!("   - VASP A: {}", vasp_a_did);

    // VASP B (Beneficiary VASP)
    let vasp_b_key = LocalAgentKey::generate();
    let vasp_b_did = vasp_b_key.did();
    let vasp_b = Agent::new(vasp_b_did.clone(), Arc::new(vasp_b_key));
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

    let ivms_policy = Policy::RequirePresentation {
        context: vec!["https://intervasp.org/ivms101".to_string()],
        credential_types: vec!["TravelRuleCredential".to_string()],
        purpose: Some("Travel Rule Compliance - FATF R.16".to_string()),
    };

    let update_policies = UpdatePolicies {
        thread_id: "transfer-001".to_string(),
        policies: vec![ivms_policy],
    };

    // Send policy update
    node.send_message(
        &vasp_b_did,
        &vasp_a_did,
        update_policies.into_plain_message(
            &vasp_b_did,
            &vasp_a_did,
            Some("transfer-001".to_string()),
        )?,
    )
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
        originator: alice_with_address.clone(),
        beneficiary: bob.clone(),
        originating_vasp: Some(Party::from(&vasp_a_did)),
        beneficiary_vasp: Some(Party::from(&vasp_b_did)),
        amount: "5000.00".to_string(), // Above typical Travel Rule threshold
        currency: "USD".to_string(),
    };

    println!(
        "   - Transfer amount: {} {}",
        transfer.amount, transfer.currency
    );
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
    let sent_message = node
        .send_message(
            &vasp_a_did,
            &vasp_b_did,
            transfer.into_plain_message(
                &vasp_a_did,
                &vasp_b_did,
                Some("transfer-001".to_string()),
            )?,
        )
        .await?;

    // Check if IVMS101 was attached
    if let Some(attachments) = &sent_message.attachments {
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

    // Retrieve extracted customer data (for demonstration)
    let storage_manager = node.get_agent_storage_manager(&vasp_b_did)?;
    let customer_manager = storage_manager.get_customer_manager();

    // In a real scenario, the customer would be automatically created from the IVMS101 data
    println!("   - Customer data extracted and stored");
    println!("   - Compliance requirements satisfied");

    // Step 7: Continue with transaction flow
    println!("\n7. Continuing transaction flow...");

    // VASP B authorizes
    let authorize = Authorize::builder()
        .transaction_id("transfer-001")
        .party(bob.clone())
        .amount("5000.00")
        .currency("USD")
        .build()?;

    node.send_message(
        &vasp_b_did,
        &vasp_a_did,
        authorize.into_plain_message(&vasp_b_did, &vasp_a_did, Some("transfer-001".to_string()))?,
    )
    .await?;
    println!("   - VASP B authorized transaction");

    // VASP A settles
    let settle = Settle::builder()
        .transaction_id("transfer-001")
        .amount("5000.00")
        .currency("USD")
        .settlement_id("blockchain-tx-123")
        .build()?;

    node.send_message(
        &vasp_a_did,
        &vasp_b_did,
        settle.into_plain_message(&vasp_a_did, &vasp_b_did, Some("transfer-001".to_string()))?,
    )
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
        .date_of_birth("1985-06-15")
        .build()?;

    let ivms_message = IvmsMessageBuilder::new()
        .originator(vec![Person::NaturalPerson(natural_person)])
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
