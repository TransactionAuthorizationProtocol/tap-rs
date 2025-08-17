//! Simple example demonstrating payment link creation
//!
//! This example shows the basic API for creating payment links
//! Run with: cargo run --example simple_payment_link

use serde_json::json;
use tap_agent::payment_link::{PaymentLinkConfig, DEFAULT_PAYMENT_SERVICE_URL};

fn main() {
    println!("=== Payment Link Configuration Example ===\n");

    // Example 1: Default configuration
    let default_config = PaymentLinkConfig::default();
    println!("Default configuration:");
    println!("  Service URL: {}", default_config.service_url);
    println!("  Metadata: {:?}", default_config.metadata);
    println!("  Goal: {:?}", default_config.goal);
    println!();

    // Example 2: Custom configuration
    let custom_config = PaymentLinkConfig::new()
        .with_service_url("https://pay.mystore.com/checkout")
        .with_metadata("store_id", json!("store-123"))
        .with_metadata("theme", json!("dark"))
        .with_metadata("return_url", json!("https://mystore.com/success"))
        .with_goal("Complete your purchase from MyStore");

    println!("Custom configuration:");
    println!("  Service URL: {}", custom_config.service_url);
    println!("  Metadata: {:?}", custom_config.metadata);
    println!("  Goal: {:?}", custom_config.goal);
    println!();

    // Example 3: Default service URL constant
    println!(
        "Default payment service URL: {}",
        DEFAULT_PAYMENT_SERVICE_URL
    );
    println!();

    // Example 4: Configuration with various metadata types
    let rich_config = PaymentLinkConfig::new()
        .with_service_url("https://flow-connect.notabene.dev/payin")
        .with_metadata("order_id", json!("ORD-2024-001"))
        .with_metadata("customer_type", json!("premium"))
        .with_metadata("expires_at", json!("2024-12-31T23:59:59Z"))
        .with_metadata("require_kyc", json!(true))
        .with_metadata("supported_currencies", json!(["USD", "EUR", "GBP"]))
        .with_goal("Complete payment for premium subscription");

    println!("Rich configuration example:");
    println!("  Service URL: {}", rich_config.service_url);
    println!("  Goal: {:?}", rich_config.goal);
    println!("  Metadata fields:");
    for (key, value) in &rich_config.metadata {
        println!("    {}: {}", key, value);
    }
    println!();

    println!("🎉 Payment link configuration examples completed!");
    println!();
    println!("In a real application, you would:");
    println!("1. Create a TapAgent instance");
    println!("2. Create a Payment message");
    println!("3. Use agent.create_payment_link(&payment, Some(config)) to generate a URL");
    println!("4. Share the URL with customers via QR code, email, etc.");
}
