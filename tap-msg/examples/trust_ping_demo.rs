//! Example demonstrating Trust Ping protocol usage

use tap_msg::message::{TapMessage, TrustPing, TrustPingResponse};
use tap_msg::{create_tap_message, TapMessageBody};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Trust Ping Protocol Demo");
    println!("========================\n");

    // Example 1: Create a simple trust ping
    println!("1. Simple Trust Ping (no response requested):");
    let simple_ping = TrustPing::new().response_requested(false);
    println!("   Response requested: {}", simple_ping.response_requested);
    println!("   Comment: {:?}", simple_ping.comment);

    // Convert to DIDComm message
    let didcomm_msg = simple_ping.to_didcomm("did:example:alice")?;
    println!("   Message type: {}", didcomm_msg.type_);
    println!("   From: {}", didcomm_msg.from);
    println!();

    // Example 2: Trust ping with response requested
    println!("2. Trust Ping with response requested:");
    let ping_with_response = TrustPing::new().response_requested(true);
    println!(
        "   Response requested: {}",
        ping_with_response.response_requested
    );

    // Example 3: Trust ping with comment
    println!("\n3. Trust Ping with comment:");
    let ping_with_comment = TrustPing::with_comment("Checking if you're still online".to_string());
    println!(
        "   Response requested: {}",
        ping_with_comment.response_requested
    );
    println!("   Comment: {:?}", ping_with_comment.comment);

    // Example 4: Trust ping response
    println!("\n4. Trust Ping Response:");
    let response = TrustPingResponse::new("original-ping-message-id".to_string());
    println!("   Thread ID: {}", response.thread_id);
    println!("   Comment: {:?}", response.comment);

    // Example 5: Trust ping response with comment
    println!("\n5. Trust Ping Response with comment:");
    let response_with_comment = TrustPingResponse::with_comment(
        "original-ping-message-id".to_string(),
        "Yes, I'm here!".to_string(),
    );
    println!("   Thread ID: {}", response_with_comment.thread_id);
    println!("   Comment: {:?}", response_with_comment.comment);

    // Example 6: Validate messages
    println!("\n6. Message validation:");
    assert!(ping_with_comment.validate_trustping().is_ok());
    assert!(response_with_comment.validate_trustpingresponse().is_ok());
    println!("   All messages validated successfully!");

    // Example 7: Create using the function
    println!("\n7. Using create_tap_message function:");
    let body = TrustPing::with_comment("Are you there?".to_string());
    let tap_message = create_tap_message(&body, None, "did:example:alice", &["did:example:bob"])?;
    println!("   Created TAP message with ID: {}", tap_message.id);
    println!("   Type: {}", tap_message.type_);

    // Example 8: Serialization
    println!("\n8. JSON Serialization:");
    let json = serde_json::to_string_pretty(&ping_with_comment)?;
    println!("{}", json);

    // Example 9: Parsing from TapMessage enum
    println!("\n9. Using TapMessage enum:");
    let tap_enum = TapMessage::TrustPing(TrustPing::new().response_requested(true));
    match tap_enum {
        TapMessage::TrustPing(ping) => {
            println!(
                "   Received Trust Ping, response requested: {}",
                ping.response_requested
            );
        }
        _ => unreachable!(),
    }

    println!("\nDemo completed successfully!");
    Ok(())
}
