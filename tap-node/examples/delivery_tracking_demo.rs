use std::sync::Arc;
use tap_node::{HttpPlainMessageSenderWithTracking, PlainMessageSender, Storage};

/// Example demonstrating message delivery tracking
///
/// This example shows how to use the HttpPlainMessageSenderWithTracking
/// to track delivery status, HTTP status codes, retry counts, and delivery URLs.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Create storage for tracking deliveries
    let storage = Arc::new(Storage::new(None).await?);

    // Create a sender with delivery tracking
    let sender =
        HttpPlainMessageSenderWithTracking::new("https://example.com".to_string(), storage.clone());

    // Simulate sending a message to multiple recipients
    let packed_message = r#"{"test": "message"}"#.to_string();
    let recipients = vec![
        "did:example:alice".to_string(),
        "did:example:bob".to_string(),
    ];

    println!(
        "Attempting to send message to {} recipients",
        recipients.len()
    );

    // Send the message (this will fail in practice since the URLs don't exist)
    // but delivery records will still be created and updated
    match sender.send(packed_message, recipients).await {
        Ok(_) => println!("Message sent successfully"),
        Err(e) => println!("Message delivery failed: {}", e),
    }

    // Query delivery records to see the tracking information
    println!("\nDelivery records:");

    // Get pending deliveries (for retry processing)
    let pending_deliveries = storage.get_pending_deliveries(5, 10).await?;
    println!("Found {} pending deliveries", pending_deliveries.len());

    for delivery in pending_deliveries {
        println!("Delivery ID: {}", delivery.id);
        println!("  Message ID: {}", delivery.message_id);
        println!("  Recipient: {}", delivery.recipient_did);
        println!("  URL: {:?}", delivery.delivery_url);
        println!("  Status: {:?}", delivery.status);
        println!("  Retry count: {}", delivery.retry_count);
        println!("  HTTP status: {:?}", delivery.last_http_status_code);
        println!("  Error: {:?}", delivery.error_message);
        println!("  Created: {}", delivery.created_at);
        println!("  Updated: {}", delivery.updated_at);
        println!("  Delivered: {:?}", delivery.delivered_at);
        println!();
    }

    // Get failed deliveries for a specific recipient
    let failed_deliveries = storage
        .get_failed_deliveries_for_recipient("did:example:alice", 10, 0)
        .await?;
    println!(
        "Found {} failed deliveries for did:example:alice",
        failed_deliveries.len()
    );

    Ok(())
}
