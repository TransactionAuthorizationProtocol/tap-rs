//! Customer and connection tools for TAP MCP

use super::schema;
use super::{default_limit, error_text_response, success_text_response, ToolHandler};
use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tap_msg::message::TapMessage;
use tracing::{debug, error};

/// Tool for listing customers (parties that an agent acts for)
pub struct ListCustomersTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for listing customers
#[derive(Debug, Deserialize)]
struct ListCustomersParams {
    agent_did: String,
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

/// Response for listing customers
#[derive(Debug, Serialize)]
struct ListCustomersResponse {
    customers: Vec<CustomerInfo>,
    total: usize,
}

#[derive(Debug, Serialize)]
struct CustomerInfo {
    #[serde(rename = "@id")]
    id: String,
    metadata: HashMap<String, serde_json::Value>,
    transaction_count: usize,
    transaction_ids: Vec<String>,
}

impl ListCustomersTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for ListCustomersTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: ListCustomersParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Listing customers for agent {} with limit={}, offset={}",
            params.agent_did, params.limit, params.offset
        );

        // Get storage for the agent
        let storage = match self
            .tap_integration()
            .storage_for_agent(&params.agent_did)
            .await
        {
            Ok(storage) => storage,
            Err(e) => {
                error!("Failed to get storage for agent {}: {}", params.agent_did, e);
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                )));
            }
        };

        // Get all transactions for this agent
        let transactions = match storage.list_transactions(1000, 0).await {
            Ok(transactions) => transactions,
            Err(e) => {
                error!("Failed to get transactions: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to get transactions: {}",
                    e
                )));
            }
        };

        // Track customers and their metadata
        let mut customers: HashMap<String, CustomerInfo> = HashMap::new();

        // Process each transaction to find customers
        for transaction in transactions {
            if let Ok(tap_message) = serde_json::from_value::<TapMessage>(transaction.message_json.clone()) {
                // Handle Transfer messages directly
                if let TapMessage::Transfer(ref transfer) = tap_message {
                    // Check if any agent acts for our target agent
                    for agent in &transfer.agents {
                        if agent.id == params.agent_did {
                            // Add all parties this agent acts for as customers
                            for party_id in agent.for_parties() {
                                let customer = customers.entry(party_id.to_string()).or_insert_with(|| {
                                    CustomerInfo {
                                        id: party_id.to_string(),
                                        metadata: HashMap::new(),
                                        transaction_count: 0,
                                        transaction_ids: Vec::new(),
                                    }
                                });
                                customer.transaction_count += 1;
                                customer.transaction_ids.push(transaction.reference_id.clone());
                            }
                        }
                    }

                    // Also check for party metadata in the message
                    // Check originator
                    if let Some(customer) = customers.get_mut(&transfer.originator.id) {
                        for (key, value) in &transfer.originator.metadata {
                            customer.metadata.insert(key.clone(), value.clone());
                        }
                    }
                    // Check beneficiary
                    if let Some(ref beneficiary) = transfer.beneficiary {
                        if let Some(customer) = customers.get_mut(&beneficiary.id) {
                            for (key, value) in &beneficiary.metadata {
                                customer.metadata.insert(key.clone(), value.clone());
                            }
                        }
                    }
                }
                
                // Handle Authorize messages by looking up the original transfer
                else if let TapMessage::Authorize(ref auth) = tap_message {
                    if let Ok(Some(original_tx)) = storage.get_transaction_by_id(&auth.transaction_id).await {
                        if let Ok(TapMessage::Transfer(ref original_transfer)) = serde_json::from_value::<TapMessage>(original_tx.message_json.clone()) {
                            // Check if any agent acts for our target agent
                            for agent in &original_transfer.agents {
                                if agent.id == params.agent_did {
                                    // Add all parties this agent acts for as customers
                                    for party_id in agent.for_parties() {
                                        let customer = customers.entry(party_id.to_string()).or_insert_with(|| {
                                            CustomerInfo {
                                                id: party_id.to_string(),
                                                metadata: HashMap::new(),
                                                transaction_count: 0,
                                                transaction_ids: Vec::new(),
                                            }
                                        });
                                        customer.transaction_count += 1;
                                        customer.transaction_ids.push(transaction.reference_id.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let total = customers.len();
        
        // Apply pagination and sort by ID for consistent ordering
        let mut customer_list: Vec<CustomerInfo> = customers.into_values().collect();
        customer_list.sort_by(|a, b| a.id.cmp(&b.id));
        
        let paginated_customers: Vec<CustomerInfo> = customer_list
            .into_iter()
            .skip(params.offset as usize)
            .take(params.limit as usize)
            .collect();

        let response = ListCustomersResponse {
            customers: paginated_customers,
            total,
        };

        let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
            Error::tool_execution(format!("Failed to serialize response: {}", e))
        })?;

        Ok(success_text_response(response_json))
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_list_customers".to_string(),
            description: "Lists customers (parties) that a specific agent acts on behalf of. Includes metadata about each party and transaction history.".to_string(),
            input_schema: schema::list_customers_schema(),
        }
    }
}

/// Tool for listing connections (counterparties with transaction history)
pub struct ListConnectionsTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for listing connections
#[derive(Debug, Deserialize)]
struct ListConnectionsParams {
    party_id: String,
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

/// Response for listing connections
#[derive(Debug, Serialize)]
struct ListConnectionsResponse {
    connections: Vec<ConnectionInfo>,
    total: usize,
}

#[derive(Debug, Serialize)]
struct ConnectionInfo {
    #[serde(rename = "@id")]
    id: String,
    metadata: HashMap<String, serde_json::Value>,
    transaction_count: usize,
    transaction_ids: Vec<String>,
    roles: Vec<String>, // Roles this counterparty has played
}

impl ListConnectionsTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for ListConnectionsTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: ListConnectionsParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Listing connections for party {} with limit={}, offset={}",
            params.party_id, params.limit, params.offset
        );

        // We need to search across all agent storages to find transactions involving this party
        let agent_infos = match self.tap_integration().list_agents().await {
            Ok(agents) => agents,
            Err(e) => {
                error!("Failed to list agents: {}", e);
                return Ok(error_text_response(format!("Failed to list agents: {}", e)));
            }
        };

        let mut connections: HashMap<String, ConnectionInfo> = HashMap::new();

        // Search through each agent's storage
        for agent_info in agent_infos {
            let storage = match self
                .tap_integration()
                .storage_for_agent(&agent_info.id)
                .await
            {
                Ok(storage) => storage,
                Err(e) => {
                    debug!("Failed to get storage for agent {}: {}", agent_info.id, e);
                    continue;
                }
            };

            let transactions = match storage.list_transactions(1000, 0).await {
                Ok(transactions) => transactions,
                Err(e) => {
                    debug!("Failed to get transactions for agent {}: {}", agent_info.id, e);
                    continue;
                }
            };

            // Process each transaction
            for transaction in transactions {
                if let Ok(tap_message) = serde_json::from_value::<TapMessage>(transaction.message_json.clone()) {
                    match tap_message {
                        TapMessage::Transfer(ref transfer) => {
                            let mut party_is_involved = false;
                            let mut counterparties = HashSet::new();

                            // Check if our party is the originator
                            if transfer.originator.id == params.party_id {
                                party_is_involved = true;
                                // Add beneficiary as counterparty
                                if let Some(ref beneficiary) = transfer.beneficiary {
                                    counterparties.insert(beneficiary.id.clone());
                                }
                            }

                            // Check if our party is the beneficiary
                            if let Some(ref beneficiary) = transfer.beneficiary {
                                if beneficiary.id == params.party_id {
                                    party_is_involved = true;
                                    // Add originator as counterparty
                                    counterparties.insert(transfer.originator.id.clone());
                                }
                            }

                            // Check if our party is represented by any agent
                            for agent in &transfer.agents {
                                if agent.for_parties().contains(&params.party_id) {
                                    party_is_involved = true;
                                    // Add other parties represented by other agents as counterparties
                                    for other_agent in &transfer.agents {
                                        if other_agent.id != agent.id {
                                            for other_party in other_agent.for_parties() {
                                                if other_party != &params.party_id {
                                                    counterparties.insert(other_party.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // If this party is involved, record the counterparties
                            if party_is_involved {
                                for counterparty_id in counterparties {
                                    let connection = connections.entry(counterparty_id.clone()).or_insert_with(|| {
                                        ConnectionInfo {
                                            id: counterparty_id.clone(),
                                            metadata: HashMap::new(),
                                            transaction_count: 0,
                                            transaction_ids: Vec::new(),
                                            roles: Vec::new(),
                                        }
                                    });
                                    connection.transaction_count += 1;
                                    connection.transaction_ids.push(transaction.reference_id.clone());

                                    // Determine role of counterparty
                                    if counterparty_id == transfer.originator.id {
                                        if !connection.roles.contains(&"originator".to_string()) {
                                            connection.roles.push("originator".to_string());
                                        }
                                    }
                                    if let Some(ref beneficiary) = transfer.beneficiary {
                                        if counterparty_id == beneficiary.id {
                                            if !connection.roles.contains(&"beneficiary".to_string()) {
                                                connection.roles.push("beneficiary".to_string());
                                            }
                                        }
                                    }

                                    // Add metadata from party objects
                                    if counterparty_id == transfer.originator.id {
                                        for (key, value) in &transfer.originator.metadata {
                                            connection.metadata.insert(key.clone(), value.clone());
                                        }
                                    }
                                    if let Some(ref beneficiary) = transfer.beneficiary {
                                        if counterparty_id == beneficiary.id {
                                            for (key, value) in &beneficiary.metadata {
                                                connection.metadata.insert(key.clone(), value.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
        }

        let total = connections.len();
        
        // Apply pagination and sort by ID for consistent ordering
        let mut connection_list: Vec<ConnectionInfo> = connections.into_values().collect();
        connection_list.sort_by(|a, b| a.id.cmp(&b.id));
        
        let paginated_connections: Vec<ConnectionInfo> = connection_list
            .into_iter()
            .skip(params.offset as usize)
            .take(params.limit as usize)
            .collect();

        let response = ListConnectionsResponse {
            connections: paginated_connections,
            total,
        };

        let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
            Error::tool_execution(format!("Failed to serialize response: {}", e))
        })?;

        Ok(success_text_response(response_json))
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_list_connections".to_string(),
            description: "Lists all counterparties (connections) of a specific party. Includes metadata about each counterparty and transaction history.".to_string(),
            input_schema: schema::list_connections_schema(),
        }
    }
}