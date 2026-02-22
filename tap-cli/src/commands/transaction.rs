use crate::error::{Error, Result};
use crate::output::{print_success, OutputFormat};
use crate::tap_integration::TapIntegration;
use clap::Subcommand;
use serde::Serialize;
use std::collections::HashMap;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{
    Agent, Capture, Connect, ConnectionConstraints, Escrow, Exchange, Party, Payment, Quote,
    TransactionLimits, Transfer,
};
use tracing::debug;

#[derive(Subcommand, Debug)]
pub enum TransactionCommands {
    /// Create a new transfer transaction (TAIP-3)
    #[command(long_about = "\
Create a new VASP-to-VASP transfer transaction (TAIP-3).

Initiates a transfer of a crypto asset between an originator and beneficiary. \
The asset must be specified in CAIP-19 format. Optionally include agents \
(VASPs, compliance providers) as a JSON array.

Examples:
  tap-cli transaction transfer \\
    --asset eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7 \\
    --amount 100.0 --originator did:key:z6Mk... --beneficiary did:key:z6Mk...

  # With agents
  tap-cli transaction transfer --asset eip155:1/slip44:60 --amount 500.0 \\
    --originator did:key:z6Mk... --beneficiary did:key:z6Mk... \\
    --agents '[{\"@id\":\"did:key:z6MkAgent...\",\"role\":\"SourceAgent\",\"for\":\"did:key:z6Mk...\"}]'")]
    Transfer {
        /// CAIP-19 asset identifier (e.g., eip155:1/erc20:0x... or eip155:1/slip44:60)
        #[arg(long)]
        asset: String,
        /// Transfer amount
        #[arg(long)]
        amount: String,
        /// Originator DID (the sender)
        #[arg(long)]
        originator: String,
        /// Beneficiary DID (the receiver)
        #[arg(long)]
        beneficiary: String,
        /// Agents as JSON array of objects with @id, role, and for fields
        #[arg(long)]
        agents: Option<String>,
        /// Optional memo text
        #[arg(long)]
        memo: Option<String>,
    },
    /// Create a new payment request (TAIP-14)
    #[command(long_about = "\
Create a new payment request (TAIP-14).

Initiates a payment from a customer to a merchant. Specify either --asset \
(CAIP-19) for crypto payments or --currency (ISO 4217) for fiat-denominated payments.

Examples:
  tap-cli transaction payment --amount 99.99 --merchant did:key:z6Mk... \\
    --asset eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48

  tap-cli transaction payment --amount 99.99 --merchant did:key:z6Mk... \\
    --currency USD --memo \"Order #5678\"")]
    Payment {
        /// Payment amount
        #[arg(long)]
        amount: String,
        /// Merchant DID (payment recipient)
        #[arg(long)]
        merchant: String,
        /// CAIP-19 asset identifier (mutually exclusive with --currency)
        #[arg(long, conflicts_with = "currency")]
        asset: Option<String>,
        /// ISO 4217 currency code, e.g., USD, EUR (mutually exclusive with --asset)
        #[arg(long, conflicts_with = "asset")]
        currency: Option<String>,
        /// Agents as JSON array
        #[arg(long)]
        agents: Option<String>,
        /// Optional memo text
        #[arg(long)]
        memo: Option<String>,
    },
    /// Create a new connection request (TAIP-15)
    #[command(long_about = "\
Create a new connection request (TAIP-15).

Establishes a relationship between agents for a party. Used to set up agent \
relationships before initiating transfers.

Examples:
  tap-cli transaction connect --recipient did:key:z6Mk... --for did:key:z6Mk... --role SourceAgent")]
    Connect {
        /// Recipient DID (the agent to connect with)
        #[arg(long)]
        recipient: String,
        /// Party DID this connection is for
        #[arg(long, name = "for")]
        for_party: String,
        /// Role in the connection (e.g., SourceAgent, DestinationAgent)
        #[arg(long)]
        role: Option<String>,
        /// Connection constraints as JSON (e.g., max_amount, daily_limit)
        #[arg(long)]
        constraints: Option<String>,
    },
    /// Create a new escrow request (TAIP-17)
    #[command(long_about = "\
Create a new escrow request (TAIP-17).

Places funds in escrow with an escrow agent. The agents JSON array must \
include at least one agent with the 'EscrowAgent' role.

Examples:
  tap-cli transaction escrow --amount 1000.0 \\
    --originator did:key:z6Mk... --beneficiary did:key:z6Mk... \\
    --expiry 2026-12-31T23:59:59Z \\
    --asset eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7 \\
    --agents '[{\"@id\":\"did:key:z6MkEscrow...\",\"role\":\"EscrowAgent\",\"for\":\"did:key:z6Mk...\"}]'")]
    Escrow {
        /// Escrow amount
        #[arg(long)]
        amount: String,
        /// Originator DID
        #[arg(long)]
        originator: String,
        /// Beneficiary DID
        #[arg(long)]
        beneficiary: String,
        /// Expiry timestamp (ISO 8601, e.g., 2026-12-31T23:59:59Z)
        #[arg(long)]
        expiry: String,
        /// Agents as JSON array (must include one EscrowAgent)
        #[arg(long)]
        agents: String,
        /// CAIP-19 asset identifier (mutually exclusive with --currency)
        #[arg(long, conflicts_with = "currency")]
        asset: Option<String>,
        /// ISO 4217 currency code (mutually exclusive with --asset)
        #[arg(long, conflicts_with = "asset")]
        currency: Option<String>,
        /// Agreement URL
        #[arg(long)]
        agreement: Option<String>,
    },
    /// Capture escrowed funds (TAIP-17)
    #[command(long_about = "\
Release escrowed funds (TAIP-17).

Captures funds held in escrow. Supports partial capture by specifying an amount \
less than the escrowed total.

Examples:
  tap-cli transaction capture --escrow-id <ESCROW_TX_ID>
  tap-cli transaction capture --escrow-id <ESCROW_TX_ID> --amount 500.0 \\
    --settlement-address eip155:1:0x742d35Cc...")]
    Capture {
        /// Escrow transaction ID to capture from
        #[arg(long)]
        escrow_id: String,
        /// Amount to capture (for partial capture; omit for full capture)
        #[arg(long)]
        amount: Option<String>,
        /// Settlement address (CAIP-10 format)
        #[arg(long)]
        settlement_address: Option<String>,
    },
    /// Create a new exchange request (TAIP-18)
    Exchange {
        /// Source asset identifiers (comma-separated CAIP-19, DTI, or ISO 4217)
        #[arg(long, value_delimiter = ',')]
        from_assets: Vec<String>,
        /// Target asset identifiers (comma-separated CAIP-19, DTI, or ISO 4217)
        #[arg(long, value_delimiter = ',')]
        to_assets: Vec<String>,
        /// Amount of source asset to exchange
        #[arg(long, conflicts_with = "to_amount")]
        from_amount: Option<String>,
        /// Amount of target asset desired
        #[arg(long, conflicts_with = "from_amount")]
        to_amount: Option<String>,
        /// Requester DID
        #[arg(long)]
        requester: String,
        /// Provider DID (optional, omit to broadcast)
        #[arg(long)]
        provider: Option<String>,
        /// Agents as JSON array
        #[arg(long)]
        agents: Option<String>,
    },
    /// Respond with a quote to an exchange request (TAIP-18)
    Quote {
        /// Exchange transaction ID to quote against
        #[arg(long)]
        exchange_id: String,
        /// Source asset identifier
        #[arg(long)]
        from_asset: String,
        /// Target asset identifier
        #[arg(long)]
        to_asset: String,
        /// Amount of source asset
        #[arg(long)]
        from_amount: String,
        /// Amount of target asset
        #[arg(long)]
        to_amount: String,
        /// Provider DID
        #[arg(long)]
        provider: String,
        /// Agents as JSON array
        #[arg(long)]
        agents: Option<String>,
        /// ISO 8601 expiry timestamp
        #[arg(long)]
        expires: String,
    },
    /// List transactions
    #[command(long_about = "\
List transactions stored in the agent's database.

Returns transactions with their type, direction, and status. Supports filtering \
by message type, thread ID, sender, or recipient.

Examples:
  tap-cli transaction list
  tap-cli transaction list --type Transfer --limit 20
  tap-cli transaction list --thread-id <THREAD_ID>")]
    List {
        /// Agent DID to list transactions for (defaults to --agent-did global flag)
        #[arg(long)]
        agent_did: Option<String>,
        /// Filter by message type (e.g., Transfer, Payment, Authorize, Reject)
        #[arg(long, name = "type")]
        msg_type: Option<String>,
        /// Filter by thread ID
        #[arg(long)]
        thread_id: Option<String>,
        /// Filter by sender DID
        #[arg(long)]
        from: Option<String>,
        /// Filter by recipient DID
        #[arg(long)]
        to: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "50")]
        limit: u32,
        /// Offset for pagination
        #[arg(long, default_value = "0")]
        offset: u32,
    },
}

#[derive(Debug, Serialize)]
struct TransactionResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    created_at: String,
}

#[derive(Debug, serde::Deserialize)]
struct AgentInput {
    #[serde(rename = "@id")]
    id: String,
    role: String,
    #[serde(rename = "for")]
    for_party: String,
}

#[derive(Debug, serde::Deserialize)]
struct ConstraintsInput {
    #[serde(default)]
    max_amount: Option<String>,
    #[serde(default)]
    daily_limit: Option<String>,
}

pub async fn handle(
    cmd: &TransactionCommands,
    format: OutputFormat,
    agent_did: &str,
    tap_integration: &TapIntegration,
) -> Result<()> {
    match cmd {
        TransactionCommands::Transfer {
            asset,
            amount,
            originator,
            beneficiary,
            agents,
            memo,
        } => {
            handle_transfer(
                agent_did,
                asset,
                amount,
                originator,
                beneficiary,
                agents.as_deref(),
                memo.clone(),
                format,
                tap_integration,
            )
            .await
        }
        TransactionCommands::Payment {
            amount,
            merchant,
            asset,
            currency,
            agents,
            memo,
        } => {
            handle_payment(
                agent_did,
                amount,
                merchant,
                asset.as_deref(),
                currency.as_deref(),
                agents.as_deref(),
                memo.clone(),
                format,
                tap_integration,
            )
            .await
        }
        TransactionCommands::Connect {
            recipient,
            for_party,
            role,
            constraints,
        } => {
            handle_connect(
                agent_did,
                recipient,
                for_party,
                role.as_deref(),
                constraints.as_deref(),
                format,
                tap_integration,
            )
            .await
        }
        TransactionCommands::Escrow {
            amount,
            originator,
            beneficiary,
            expiry,
            agents,
            asset,
            currency,
            agreement,
        } => {
            handle_escrow(
                agent_did,
                amount,
                originator,
                beneficiary,
                expiry,
                agents,
                asset.as_deref(),
                currency.as_deref(),
                agreement.as_deref(),
                format,
                tap_integration,
            )
            .await
        }
        TransactionCommands::Capture {
            escrow_id,
            amount,
            settlement_address,
        } => {
            handle_capture(
                agent_did,
                escrow_id,
                amount.as_deref(),
                settlement_address.as_deref(),
                format,
                tap_integration,
            )
            .await
        }
        TransactionCommands::Exchange {
            from_assets,
            to_assets,
            from_amount,
            to_amount,
            requester,
            provider,
            agents,
        } => {
            handle_exchange(
                agent_did,
                from_assets,
                to_assets,
                from_amount.as_deref(),
                to_amount.as_deref(),
                requester,
                provider.as_deref(),
                agents.as_deref(),
                format,
                tap_integration,
            )
            .await
        }
        TransactionCommands::Quote {
            exchange_id,
            from_asset,
            to_asset,
            from_amount,
            to_amount,
            provider,
            agents,
            expires,
        } => {
            handle_quote(
                agent_did,
                exchange_id,
                from_asset,
                to_asset,
                from_amount,
                to_amount,
                provider,
                agents.as_deref(),
                expires,
                format,
                tap_integration,
            )
            .await
        }
        TransactionCommands::List {
            agent_did: list_agent_did,
            msg_type,
            thread_id,
            from,
            to,
            limit,
            offset,
        } => {
            let effective_did = list_agent_did.as_deref().unwrap_or(agent_did);
            handle_list(
                effective_did,
                msg_type.as_deref(),
                thread_id.as_deref(),
                from.as_deref(),
                to.as_deref(),
                *limit,
                *offset,
                format,
                tap_integration,
            )
            .await
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_transfer(
    agent_did: &str,
    asset: &str,
    amount: &str,
    originator_did: &str,
    beneficiary_did: &str,
    agents_json: Option<&str>,
    memo: Option<String>,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let asset_id = asset
        .parse::<AssetId>()
        .map_err(|e| Error::invalid_parameter(format!("Invalid asset ID: {}", e)))?;

    let originator = Party::new(originator_did);
    let beneficiary = Party::new(beneficiary_did);
    let agents = parse_agents(agents_json)?;

    let transfer = Transfer {
        transaction_id: None,
        asset: asset_id,
        originator: Some(originator),
        beneficiary: Some(beneficiary),
        amount: amount.to_string(),
        agents,
        memo,
        settlement_id: None,
        expiry: None,
        transaction_value: None,
        connection_id: None,
        metadata: HashMap::new(),
    };

    transfer
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("Transfer validation failed: {}", e)))?;

    let didcomm_message = transfer
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    debug!("Sending transfer from {}", agent_did);
    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send transfer: {}", e)))?;

    let response = TransactionResponse {
        transaction_id: didcomm_message
            .thid
            .clone()
            .unwrap_or(didcomm_message.id.clone()),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_payment(
    agent_did: &str,
    amount: &str,
    merchant_did: &str,
    asset: Option<&str>,
    currency: Option<&str>,
    agents_json: Option<&str>,
    memo: Option<String>,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let merchant = Party::new(merchant_did);
    let agents = parse_agents(agents_json)?;

    let mut payment = if let Some(asset) = asset {
        let asset_id = asset
            .parse::<AssetId>()
            .map_err(|e| Error::invalid_parameter(format!("Invalid asset ID: {}", e)))?;
        Payment::with_asset(asset_id, amount.to_string(), merchant, agents)
    } else if let Some(currency) = currency {
        Payment::with_currency(currency.to_string(), amount.to_string(), merchant, agents)
    } else {
        return Err(Error::invalid_parameter(
            "Either --asset or --currency must be specified",
        ));
    };

    if let Some(memo) = memo {
        payment.memo = Some(memo);
    }

    payment
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("Payment validation failed: {}", e)))?;

    let didcomm_message = payment
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send payment: {}", e)))?;

    let response = TransactionResponse {
        transaction_id: didcomm_message.id.clone(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_connect(
    agent_did: &str,
    recipient: &str,
    for_party: &str,
    role: Option<&str>,
    constraints_json: Option<&str>,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let transaction_id = format!("connect-{}", uuid::Uuid::new_v4());
    let mut connect = Connect::new(&transaction_id, agent_did, for_party, role);

    if let Some(json) = constraints_json {
        let input: ConstraintsInput = serde_json::from_str(json)
            .map_err(|e| Error::invalid_parameter(format!("Invalid constraints JSON: {}", e)))?;

        let mut constraints = ConnectionConstraints {
            purposes: None,
            category_purposes: None,
            limits: None,
            allowed_beneficiaries: None,
            allowed_settlement_addresses: None,
            allowed_assets: None,
        };

        let mut limits = TransactionLimits {
            per_transaction: None,
            per_day: None,
            per_week: None,
            per_month: None,
            per_year: None,
            currency: None,
        };
        limits.per_transaction = input.max_amount;
        limits.per_day = input.daily_limit;
        constraints.limits = Some(limits);
        connect.constraints = Some(constraints);
    }

    connect
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("Connect validation failed: {}", e)))?;

    let mut didcomm_message = connect
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    didcomm_message.to = vec![recipient.to_string()];

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send connect: {}", e)))?;

    let response = TransactionResponse {
        transaction_id: didcomm_message.id.clone(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_escrow(
    agent_did: &str,
    amount: &str,
    originator_did: &str,
    beneficiary_did: &str,
    expiry: &str,
    agents_json: &str,
    asset: Option<&str>,
    currency: Option<&str>,
    agreement: Option<&str>,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let originator = Party::new(originator_did);
    let beneficiary = Party::new(beneficiary_did);
    let agents = parse_agents(Some(agents_json))?;

    let escrow_agent_count = agents
        .iter()
        .filter(|a| a.role == Some("EscrowAgent".to_string()))
        .count();
    if escrow_agent_count != 1 {
        return Err(Error::invalid_parameter(format!(
            "Escrow must have exactly one EscrowAgent, found {}",
            escrow_agent_count
        )));
    }

    let mut escrow = if let Some(asset) = asset {
        Escrow::new_with_asset(
            asset.to_string(),
            amount.to_string(),
            originator,
            beneficiary,
            expiry.to_string(),
            agents,
        )
    } else if let Some(currency) = currency {
        Escrow::new_with_currency(
            currency.to_string(),
            amount.to_string(),
            originator,
            beneficiary,
            expiry.to_string(),
            agents,
        )
    } else {
        return Err(Error::invalid_parameter(
            "Either --asset or --currency must be specified",
        ));
    };

    if let Some(agreement) = agreement {
        escrow = escrow.with_agreement(agreement.to_string());
    }

    escrow
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("Escrow validation failed: {}", e)))?;

    let didcomm_message = escrow
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send escrow: {}", e)))?;

    let response = TransactionResponse {
        transaction_id: didcomm_message.id.clone(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

async fn handle_capture(
    agent_did: &str,
    escrow_id: &str,
    amount: Option<&str>,
    settlement_address: Option<&str>,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let mut capture = if let Some(amount) = amount {
        Capture::with_amount(amount.to_string())
    } else {
        Capture::new()
    };

    if let Some(address) = settlement_address {
        capture = capture.with_settlement_address(address.to_string());
    }

    capture
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("Capture validation failed: {}", e)))?;

    let mut didcomm_message = capture
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    didcomm_message.thid = Some(escrow_id.to_string());

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send capture: {}", e)))?;

    let response = TransactionResponse {
        transaction_id: escrow_id.to_string(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_exchange(
    agent_did: &str,
    from_assets: &[String],
    to_assets: &[String],
    from_amount: Option<&str>,
    to_amount: Option<&str>,
    requester_did: &str,
    provider_did: Option<&str>,
    agents_json: Option<&str>,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    if from_amount.is_none() && to_amount.is_none() {
        return Err(Error::invalid_parameter(
            "Either --from-amount or --to-amount must be specified",
        ));
    }

    let requester = Party::new(requester_did);
    let agents = parse_agents(agents_json)?;

    let mut exchange = if let Some(amount) = from_amount {
        Exchange::new_from(
            from_assets.to_vec(),
            to_assets.to_vec(),
            amount.to_string(),
            requester,
            agents,
        )
    } else {
        Exchange::new_to(
            from_assets.to_vec(),
            to_assets.to_vec(),
            to_amount.unwrap().to_string(),
            requester,
            agents,
        )
    };

    if let Some(provider) = provider_did {
        exchange = exchange.with_provider(Party::new(provider));
    }

    exchange
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("Exchange validation failed: {}", e)))?;

    let didcomm_message = exchange
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send exchange: {}", e)))?;

    let response = TransactionResponse {
        transaction_id: didcomm_message.id.clone(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_quote(
    agent_did: &str,
    exchange_id: &str,
    from_asset: &str,
    to_asset: &str,
    from_amount: &str,
    to_amount: &str,
    provider_did: &str,
    agents_json: Option<&str>,
    expires: &str,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let provider = Party::new(provider_did);
    let agents = parse_agents(agents_json)?;

    let quote = Quote::new(
        from_asset.to_string(),
        to_asset.to_string(),
        from_amount.to_string(),
        to_amount.to_string(),
        provider,
        agents,
        expires.to_string(),
    );

    quote
        .validate()
        .map_err(|e| Error::invalid_parameter(format!("Quote validation failed: {}", e)))?;

    let mut didcomm_message = quote
        .to_didcomm(agent_did)
        .map_err(|e| Error::command_failed(format!("Failed to create DIDComm message: {}", e)))?;

    didcomm_message.thid = Some(exchange_id.to_string());

    tap_integration
        .node()
        .send_message(agent_did.to_string(), didcomm_message.clone())
        .await
        .map_err(|e| Error::command_failed(format!("Failed to send quote: {}", e)))?;

    let response = TransactionResponse {
        transaction_id: exchange_id.to_string(),
        message_id: didcomm_message.id,
        status: "sent".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    print_success(format, &response);
    Ok(())
}

#[derive(Debug, Serialize)]
struct TransactionListResponse {
    transactions: Vec<TransactionInfo>,
    total: usize,
}

#[derive(Debug, Serialize)]
struct TransactionInfo {
    id: String,
    #[serde(rename = "type")]
    message_type: String,
    thread_id: Option<String>,
    from: Option<String>,
    to: Option<String>,
    direction: String,
    created_at: String,
    body: serde_json::Value,
}

#[allow(clippy::too_many_arguments)]
async fn handle_list(
    agent_did: &str,
    msg_type: Option<&str>,
    thread_id: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
    limit: u32,
    offset: u32,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let storage = tap_integration.storage_for_agent(agent_did).await?;
    let direction_filter = None;
    let messages = storage
        .list_messages(limit, offset, direction_filter)
        .await?;

    let filtered: Vec<_> = messages
        .into_iter()
        .filter(|msg| {
            if let Some(mt) = msg_type {
                if !msg.message_type.contains(mt) {
                    return false;
                }
            }
            if let Some(tid) = thread_id {
                if msg.thread_id.as_ref() != Some(&tid.to_string()) {
                    return false;
                }
            }
            if let Some(f) = from {
                if msg.from_did.as_ref() != Some(&f.to_string()) {
                    return false;
                }
            }
            if let Some(t) = to {
                if msg.to_did.as_ref() != Some(&t.to_string()) {
                    return false;
                }
            }
            true
        })
        .collect();

    let transactions: Vec<TransactionInfo> = filtered
        .iter()
        .map(|msg| TransactionInfo {
            id: msg.message_id.clone(),
            message_type: msg.message_type.clone(),
            thread_id: msg.thread_id.clone(),
            from: msg.from_did.clone(),
            to: msg.to_did.clone(),
            direction: msg.direction.to_string(),
            created_at: msg.created_at.clone(),
            body: msg.message_json.clone(),
        })
        .collect();

    let response = TransactionListResponse {
        total: transactions.len(),
        transactions,
    };
    print_success(format, &response);
    Ok(())
}

fn parse_agents(json: Option<&str>) -> Result<Vec<Agent>> {
    match json {
        Some(j) => {
            let inputs: Vec<AgentInput> = serde_json::from_str(j)
                .map_err(|e| Error::invalid_parameter(format!("Invalid agents JSON: {}", e)))?;
            Ok(inputs
                .iter()
                .map(|a| Agent::new(&a.id, &a.role, &a.for_party))
                .collect())
        }
        None => Ok(vec![]),
    }
}
