//! TAP Payment Flow Simulator
//!
//! This command-line tool creates an ephemeral agent and simulates payment flow messages
//! towards a TAP HTTP server using the DIDComm protocol.
//!
//! Usage:
//!   tap-payment-simulator --url <server-url> --did <server-agent-did>

use std::error::Error;
use std::process;
use tap_agent::{Agent, DefaultAgent};
use tap_msg::{PaymentRequest, Transfer, Participant};
use tap_node::DefaultAgentExt;
use tracing::{debug, info};
use std::collections::HashMap;

struct Args {
    url: String,
    recipient_did: String,
    amount: Option<f64>,
    currency: Option<String>,
    verbose: bool,
}

impl Args {
    fn parse() -> Result<Self, Box<dyn Error>> {
        let mut args = pico_args::Arguments::from_env();

        // Check for help flag first
        if args.contains(["-h", "--help"]) {
            print_help();
            process::exit(0);
        }

        // Check for version flag
        if args.contains("--version") {
            println!("tap-payment-simulator {}", env!("CARGO_PKG_VERSION"));
            process::exit(0);
        }

        let result = Args {
            url: match args.value_from_str("--url") {
                Ok(url) => url,
                Err(_) => {
                    return Err("Missing required argument: --url".into());
                }
            },
            recipient_did: match args.value_from_str("--to") {
                Ok(did) => did,
                Err(_) => {
                    return Err("Missing required argument: --to".into());
                }
            },
            amount: args.opt_value_from_str("--amount")?,
            currency: args.opt_value_from_str("--currency")?,
            verbose: args.contains(["-v", "--verbose"]),
        };

        // Check for any remaining arguments (which would be invalid)
        let remaining = args.finish();
        if !remaining.is_empty() {
            return Err(format!("Unknown arguments: {:?}", remaining).into());
        }

        Ok(result)
    }
}

fn print_help() {
    println!("TAP Payment Flow Simulator");
    println!("-------------------------");
    println!(
        "Creates an ephemeral agent and simulates payment flow messages towards a TAP HTTP server"
    );
    println!();
    println!("USAGE:");
    println!("    tap-payment-simulator --url <server-url> --did <server-agent-did> [OPTIONS]");
    println!();
    println!("REQUIRED ARGUMENTS:");
    println!("    --url <URL>                 URL of the TAP HTTP server's DIDComm endpoint");
    println!("    --did <DID>                 DID of the server's agent");
    println!();
    println!("OPTIONS:");
    println!("    --amount <AMOUNT>           Amount to transfer [default: 100.00]");
    println!("    --currency <CURRENCY>       Currency code [default: USD]");
    println!("    -v, --verbose               Enable verbose logging");
    println!("    --help                      Print help information");
    println!("    --version                   Print version information");
}

/// Send a TAP message to the server
async fn send_tap_message<T: tap_msg::message::tap_message_trait::TapMessageBody + serde::Serialize + Send + Sync>(
    agent: &DefaultAgent,
    recipient_did: &str,
    recipient_url: &str,
    message: &T,
) -> Result<(), Box<dyn Error>> {
    // Create a DIDComm message from the TAP message using a custom approach
    info!("Creating DIDComm message for TAP type: {}", T::message_type());
    let from_did = agent.get_agent_did();
    
    // Convert the TAP message to JSON value
    let mut message_json = serde_json::to_value(message)
        .map_err(|e| format!("Failed to serialize message: {}", e))?;
    
    // Ensure the @type field is set correctly in the body
    if let Some(body_obj) = message_json.as_object_mut() {
        body_obj.insert("@type".to_string(), serde_json::Value::String(T::message_type().to_string()));
    }
    
    // Create a proper DIDComm message with correct typ and type fields - this is critical!
    let id = uuid::Uuid::new_v4().to_string();
    let didcomm_message = tap_msg::didcomm::Message {
        id,
        typ: T::message_type().to_string(), // Set this to the TAP message type, not the media type
        type_: T::message_type().to_string(),
        body: message_json,
        from: Some(from_did.to_string()),
        to: Some(vec![recipient_did.to_string()]),
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        extra_headers: std::collections::HashMap::new(),
        from_prior: None,
        attachments: None,
    };
        
    // Debug: Dump the raw message to see what's being sent
    println!("RAW DIDCOMM MESSAGE:\n{}", serde_json::to_string_pretty(&didcomm_message).unwrap_or_default());
    
    // Pack the message using the agent's send_serialized_message method
    info!("Packing message for recipient {}", recipient_did);
    let packed = agent.send_serialized_message(&didcomm_message, recipient_did).await?;
    debug!("Packed message size: {} bytes", packed.len());

    // Send to the server
    info!("Sending message to {}", recipient_url);
    let client = reqwest::Client::new();
    let response = client
        .post(recipient_url)
        .header("Content-Type", "application/didcomm-encrypted+json")
        .body(packed)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response".to_string());
        return Err(format!("Server returned error status: {} - {}", status, error_text).into());
    }

    info!("Message sent successfully, status: {}", status);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = Args::parse().unwrap_or_else(|e| {
        eprintln!("Error parsing arguments: {}", e);
        process::exit(1);
    });

    // Setup logging
    let log_level = if args.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    // Create ephemeral agent
    info!("Creating ephemeral agent for payment simulation");

    // Create multiple agents to verify they get different DIDs
    let (agent, agent_did) = DefaultAgent::new_ephemeral()?;

    info!("Using agent with DID: {}", agent_did);

    // Print the DID to stdout for easy copying
    println!("Payment simulator using agent DID: {}", agent_did);

    // Get amount and currency
    let amount = args.amount.unwrap_or(100.0);
    let currency = args.currency.unwrap_or_else(|| "USD".to_string());
    info!("Using amount: {} {}", amount, currency);

    // Create a unique transaction ID
    let transaction_id = uuid::Uuid::new_v4().to_string();
    info!("Transaction ID: {}", transaction_id);

    // Create payment request message using the proper type
    info!("Creating payment request message");
    
    // Create merchant and customer participants
    let merchant = Participant {
        id: args.recipient_did.clone(),
        role: Some("merchant".to_string()),
        policies: None,
        leiCode: None,
    };

    let customer = Participant {
        id: agent_did.clone(),
        role: Some("customer".to_string()),
        policies: None,
        leiCode: None,
    };
    
    // Create agent participants
    let sender_agent = Participant {
        id: agent_did.clone(),
        role: Some("sender".to_string()),
        policies: None,
        leiCode: None,
    };
    
    let recipient_agent = Participant {
        id: args.recipient_did.clone(),
        role: Some("recipient".to_string()),
        policies: None,
        leiCode: None,
    };
    
    // Create a settlement agent participant (required by validation)
    let settlement_agent = Participant {
        id: format!("did:pkh:eip155:1:0x{}", uuid::Uuid::new_v4().to_string().replace("-", "").get(0..40).unwrap_or("1234567890abcdef1234567890abcdef12345678")),
        role: Some("settlementAddress".to_string()),
        policies: None,
        leiCode: None,
    };

    // Create a payment request using the proper struct
    let payment_request = PaymentRequest {
        asset: Some(
            "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".parse().unwrap(),
        ),
        currency: Some(currency),
        amount: amount.to_string(),
        supported_assets: None,
        invoice: None,
        expiry: None,
        merchant,
        customer: Some(customer),
        agents: vec![sender_agent.clone(), recipient_agent.clone(), settlement_agent.clone()], // Include both DIDs plus settlement agent
        metadata: HashMap::new(),
    };

    // Send payment request message using the agent's proper method
    info!("Sending payment request message to server");
    send_tap_message(
        &agent,
        &args.recipient_did,
        &args.url,
        &payment_request,
    )
    .await?;

    // Wait a bit before sending the transfer
    info!("Waiting 2 seconds before sending transfer message...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Create transfer message using the proper type
    info!("Creating transfer message");
    
    // Create originator and beneficiary participants
    let originator = Participant {
        id: agent_did.clone(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let beneficiary = Participant {
        id: args.recipient_did.clone(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    // Create a transfer using the proper struct
    let transfer = Transfer {
        transaction_id: transaction_id.clone(),
        asset: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".parse().unwrap(),
        originator,
        beneficiary: Some(beneficiary),
        amount: amount.to_string(),
        agents: vec![sender_agent, recipient_agent, settlement_agent],  // Include both DIDs plus settlement agent
        settlement_id: None,
        memo: Some("Payment simulator transfer".to_string()),
        metadata: HashMap::new(),
    };

    // Send transfer message using the agent's proper method
    info!("Sending transfer message to server");
    send_tap_message(
        &agent,
        &args.recipient_did,
        &args.url,
        &transfer,
    )
    .await?;

    info!("Payment flow simulation completed successfully");
    Ok(())
}
