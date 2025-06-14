//! Customer and connection tools for TAP MCP

use super::schema;
use super::{default_limit, error_text_response, success_text_response, ToolHandler};
use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tap_msg::message::TapMessage;
use tap_node::customer::CustomerManager;
use tap_node::storage::models::{Customer, SchemaType};
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
                error!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                );
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
            if let Ok(tap_message) =
                serde_json::from_value::<TapMessage>(transaction.message_json.clone())
            {
                // Handle Transfer messages directly
                if let TapMessage::Transfer(ref transfer) = tap_message {
                    // Check if any agent acts for our target agent
                    for agent in &transfer.agents {
                        if agent.id == params.agent_did {
                            // Add all parties this agent acts for as customers
                            for party_id in agent.for_parties() {
                                let customer = customers
                                    .entry(party_id.to_string())
                                    .or_insert_with(|| CustomerInfo {
                                        id: party_id.to_string(),
                                        metadata: HashMap::new(),
                                        transaction_count: 0,
                                        transaction_ids: Vec::new(),
                                    });
                                customer.transaction_count += 1;
                                customer
                                    .transaction_ids
                                    .push(transaction.reference_id.clone());
                            }
                        }
                    }

                    // Also check for party metadata in the message
                    // Check originator
                    if let Some(originator) = &transfer.originator {
                        if let Some(customer) = customers.get_mut(&originator.id) {
                            for (key, value) in &originator.metadata {
                                customer.metadata.insert(key.clone(), value.clone());
                            }
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
                    if let Ok(Some(original_tx)) =
                        storage.get_transaction_by_id(&auth.transaction_id).await
                    {
                        if let Ok(TapMessage::Transfer(ref original_transfer)) =
                            serde_json::from_value::<TapMessage>(original_tx.message_json.clone())
                        {
                            // Check if any agent acts for our target agent
                            for agent in &original_transfer.agents {
                                if agent.id == params.agent_did {
                                    // Add all parties this agent acts for as customers
                                    for party_id in agent.for_parties() {
                                        let customer = customers
                                            .entry(party_id.to_string())
                                            .or_insert_with(|| CustomerInfo {
                                                id: party_id.to_string(),
                                                metadata: HashMap::new(),
                                                transaction_count: 0,
                                                transaction_ids: Vec::new(),
                                            });
                                        customer.transaction_count += 1;
                                        customer
                                            .transaction_ids
                                            .push(transaction.reference_id.clone());
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

        let response_json = serde_json::to_string_pretty(&response)
            .map_err(|e| Error::tool_execution(format!("Failed to serialize response: {}", e)))?;

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
                    debug!(
                        "Failed to get transactions for agent {}: {}",
                        agent_info.id, e
                    );
                    continue;
                }
            };

            // Process each transaction
            for transaction in transactions {
                if let Ok(TapMessage::Transfer(ref transfer)) =
                    serde_json::from_value::<TapMessage>(transaction.message_json.clone())
                {
                    let mut party_is_involved = false;
                    let mut counterparties = HashSet::new();

                    // Check if our party is the originator
                    if let Some(originator) = &transfer.originator {
                        if originator.id == params.party_id {
                            party_is_involved = true;
                            // Add beneficiary as counterparty
                            if let Some(ref beneficiary) = transfer.beneficiary {
                                counterparties.insert(beneficiary.id.clone());
                            }
                        }
                    }

                    // Check if our party is the beneficiary
                    if let Some(ref beneficiary) = transfer.beneficiary {
                        if beneficiary.id == params.party_id {
                            party_is_involved = true;
                            // Add originator as counterparty
                            if let Some(originator) = &transfer.originator {
                                counterparties.insert(originator.id.clone());
                            }
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
                            let connection = connections
                                .entry(counterparty_id.clone())
                                .or_insert_with(|| ConnectionInfo {
                                    id: counterparty_id.clone(),
                                    metadata: HashMap::new(),
                                    transaction_count: 0,
                                    transaction_ids: Vec::new(),
                                    roles: Vec::new(),
                                });
                            connection.transaction_count += 1;
                            connection
                                .transaction_ids
                                .push(transaction.reference_id.clone());

                            // Determine role of counterparty
                            if let Some(originator) = &transfer.originator {
                                if counterparty_id == originator.id
                                    && !connection.roles.contains(&"originator".to_string())
                                {
                                    connection.roles.push("originator".to_string());
                                }
                            }
                            if let Some(ref beneficiary) = transfer.beneficiary {
                                if counterparty_id == beneficiary.id
                                    && !connection.roles.contains(&"beneficiary".to_string())
                                {
                                    connection.roles.push("beneficiary".to_string());
                                }
                            }

                            // Add metadata from party objects
                            if let Some(originator) = &transfer.originator {
                                if counterparty_id == originator.id {
                                    for (key, value) in &originator.metadata {
                                        connection.metadata.insert(key.clone(), value.clone());
                                    }
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

        let response_json = serde_json::to_string_pretty(&response)
            .map_err(|e| Error::tool_execution(format!("Failed to serialize response: {}", e)))?;

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

/// Tool for getting customer details including IVMS101 data
pub struct GetCustomerDetailsTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for getting customer details
#[derive(Debug, Deserialize)]
struct GetCustomerDetailsParams {
    agent_did: String,
    customer_id: String,
}

/// Response for getting customer details
#[derive(Debug, Serialize)]
struct GetCustomerDetailsResponse {
    customer: Option<serde_json::Value>,
    ivms101_data: Option<serde_json::Value>,
}

impl GetCustomerDetailsTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for GetCustomerDetailsTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: GetCustomerDetailsParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Getting customer details for customer {} via agent {}",
            params.customer_id, params.agent_did
        );

        // Get storage for the agent
        let storage = match self
            .tap_integration()
            .storage_for_agent(&params.agent_did)
            .await
        {
            Ok(storage) => storage,
            Err(e) => {
                error!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                );
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                )));
            }
        };

        // Get customer data
        let customer = match storage.get_customer(&params.customer_id).await {
            Ok(customer) => customer,
            Err(e) => {
                debug!("Failed to get customer {}: {}", params.customer_id, e);
                None
            }
        };

        let response = if let Some(customer) = customer {
            // Convert customer to JSON value
            let customer_json = serde_json::to_value(&customer).map_err(|e| {
                Error::tool_execution(format!("Failed to serialize customer: {}", e))
            })?;

            let profile = customer_json.get("profile").cloned();
            let ivms101 = customer_json.get("ivms101_data").cloned();

            GetCustomerDetailsResponse {
                customer: Some(profile.unwrap_or(customer_json)),
                ivms101_data: ivms101,
            }
        } else {
            GetCustomerDetailsResponse {
                customer: None,
                ivms101_data: None,
            }
        };

        let response_json = serde_json::to_string_pretty(&response)
            .map_err(|e| Error::tool_execution(format!("Failed to serialize response: {}", e)))?;

        Ok(success_text_response(response_json))
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_get_customer_details".to_string(),
            description: "Gets detailed information about a specific customer including their profile and IVMS101 data if available.".to_string(),
            input_schema: schema::get_customer_details_schema(),
        }
    }
}

/// Tool for generating IVMS101 data for a customer
pub struct GenerateIvms101Tool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for generating IVMS101
#[derive(Debug, Deserialize)]
struct GenerateIvms101Params {
    agent_did: String,
    customer_id: String,
}

impl GenerateIvms101Tool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for GenerateIvms101Tool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: GenerateIvms101Params = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Generating IVMS101 data for customer {} via agent {}",
            params.customer_id, params.agent_did
        );

        // Get storage for the agent
        let storage = match self
            .tap_integration()
            .storage_for_agent(&params.agent_did)
            .await
        {
            Ok(storage) => storage,
            Err(e) => {
                error!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                );
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                )));
            }
        };

        // Create customer manager
        let customer_manager = CustomerManager::new(storage);

        // Generate IVMS101 data
        match customer_manager
            .generate_ivms101_data(&params.customer_id)
            .await
        {
            Ok(ivms_data) => {
                let response_json = serde_json::to_string_pretty(&ivms_data).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize IVMS101 data: {}", e))
                })?;
                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to generate IVMS101 data: {}", e);
                Ok(error_text_response(format!(
                    "Failed to generate IVMS101 data: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_generate_ivms101".to_string(),
            description: "Generates IVMS101 compliant data for a customer based on their stored profile information.".to_string(),
            input_schema: schema::generate_ivms101_schema(),
        }
    }
}

/// Tool for updating customer profile
pub struct UpdateCustomerProfileTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for updating customer profile
#[derive(Debug, Deserialize)]
struct UpdateCustomerProfileParams {
    agent_did: String,
    customer_id: String,
    profile_data: Value,
}

impl UpdateCustomerProfileTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for UpdateCustomerProfileTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: UpdateCustomerProfileParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Updating profile for customer {} via agent {}",
            params.customer_id, params.agent_did
        );

        // Get storage for the agent
        let storage = match self
            .tap_integration()
            .storage_for_agent(&params.agent_did)
            .await
        {
            Ok(storage) => storage,
            Err(e) => {
                error!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                );
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                )));
            }
        };

        // Create customer manager
        let customer_manager = CustomerManager::new(storage);

        // Update customer profile
        match customer_manager
            .update_customer_profile(&params.customer_id, params.profile_data)
            .await
        {
            Ok(_) => Ok(success_text_response(format!(
                "Successfully updated profile for customer {}",
                params.customer_id
            ))),
            Err(e) => {
                error!("Failed to update customer profile: {}", e);
                Ok(error_text_response(format!(
                    "Failed to update customer profile: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_update_customer_profile".to_string(),
            description: "Updates the schema.org profile data for a customer. The profile_data should be a JSON object with schema.org fields.".to_string(),
            input_schema: schema::update_customer_profile_schema(),
        }
    }
}

/// Tool for creating a new customer
pub struct CreateCustomerTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for creating a customer
#[derive(Debug, Deserialize)]
struct CreateCustomerParams {
    agent_did: String,
    customer_id: String,
    profile_data: Value,
}

impl CreateCustomerTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for CreateCustomerTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: CreateCustomerParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Creating customer {} via agent {}",
            params.customer_id, params.agent_did
        );

        // Get storage for the agent
        let storage = match self
            .tap_integration()
            .storage_for_agent(&params.agent_did)
            .await
        {
            Ok(storage) => storage,
            Err(e) => {
                error!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                );
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                )));
            }
        };

        // Create customer manager
        let customer_manager = CustomerManager::new(storage.clone());

        // Check if customer already exists
        let existing = match storage.get_customer(&params.customer_id).await {
            Ok(existing) => existing,
            Err(e) => {
                error!("Failed to check existing customer: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to check existing customer: {}",
                    e
                )));
            }
        };

        if existing.is_none() {
            // Create new customer
            let display_name = params
                .profile_data
                .get("givenName")
                .and_then(|v| v.as_str())
                .map(|given| {
                    if let Some(family) = params
                        .profile_data
                        .get("familyName")
                        .and_then(|v| v.as_str())
                    {
                        format!("{} {}", given, family)
                    } else {
                        given.to_string()
                    }
                });

            // Create customer profile from schema.org data
            let mut profile = json!({
                "@context": "https://schema.org",
                "@type": "Person",
                "identifier": params.customer_id.clone(),
            });

            // Merge provided profile data
            if let Value::Object(profile_obj) = &mut profile {
                if let Value::Object(data_obj) = &params.profile_data {
                    for (key, value) in data_obj {
                        profile_obj.insert(key.clone(), value.clone());
                    }
                }
            }

            // Determine schema type based on provided data
            let schema_type = if params.profile_data.get("@type").and_then(|v| v.as_str())
                == Some("Organization")
            {
                SchemaType::Organization
            } else {
                SchemaType::Person
            };

            // Create Customer struct
            let customer = Customer {
                id: params.customer_id.clone(),
                agent_did: params.agent_did.clone(),
                schema_type,
                given_name: params
                    .profile_data
                    .get("givenName")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                family_name: params
                    .profile_data
                    .get("familyName")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                display_name,
                legal_name: params
                    .profile_data
                    .get("legalName")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                lei_code: params
                    .profile_data
                    .get("leiCode")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                mcc_code: params
                    .profile_data
                    .get("mccCode")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                address_country: params
                    .profile_data
                    .get("addressCountry")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                address_locality: params
                    .profile_data
                    .get("addressLocality")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                postal_code: params
                    .profile_data
                    .get("postalCode")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                street_address: params
                    .profile_data
                    .get("streetAddress")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                profile,
                ivms101_data: None,
                verified_at: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };

            // Create the customer
            match storage.upsert_customer(&customer).await {
                Ok(_) => {
                    debug!("Created new customer {}", params.customer_id);
                    Ok(success_text_response(format!(
                        "Successfully created customer {}",
                        params.customer_id
                    )))
                }
                Err(e) => {
                    error!("Failed to create customer: {}", e);
                    Ok(error_text_response(format!(
                        "Failed to create customer: {}",
                        e
                    )))
                }
            }
        } else {
            // Update existing customer
            match customer_manager
                .update_customer_profile(&params.customer_id, params.profile_data)
                .await
            {
                Ok(_) => Ok(success_text_response(format!(
                    "Successfully updated existing customer {}",
                    params.customer_id
                ))),
                Err(e) => {
                    error!("Failed to update customer: {}", e);
                    Ok(error_text_response(format!(
                        "Failed to update customer: {}",
                        e
                    )))
                }
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_create_customer".to_string(),
            description: "Creates a new customer profile for an agent. The customer_id should be a DID or unique identifier. The profile_data should be a JSON object with schema.org fields (e.g., givenName, familyName, addressCountry). If a customer with the same ID already exists, their profile will be updated.".to_string(),
            input_schema: schema::create_customer_schema(),
        }
    }
}

/// Tool for updating customer from IVMS101 data
pub struct UpdateCustomerFromIvms101Tool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for updating customer from IVMS101
#[derive(Debug, Deserialize)]
struct UpdateCustomerFromIvms101Params {
    agent_did: String,
    customer_id: String,
    ivms101_data: Value,
}

impl UpdateCustomerFromIvms101Tool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for UpdateCustomerFromIvms101Tool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: UpdateCustomerFromIvms101Params = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Updating customer {} from IVMS101 data via agent {}",
            params.customer_id, params.agent_did
        );

        // Get storage for the agent
        let storage = match self
            .tap_integration()
            .storage_for_agent(&params.agent_did)
            .await
        {
            Ok(storage) => storage,
            Err(e) => {
                error!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                );
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                )));
            }
        };

        // Create customer manager
        let customer_manager = CustomerManager::new(storage);

        // Update customer from IVMS101 data
        match customer_manager
            .update_customer_from_ivms101(&params.customer_id, &params.ivms101_data)
            .await
        {
            Ok(_) => Ok(success_text_response(format!(
                "Successfully updated customer {} from IVMS101 data",
                params.customer_id
            ))),
            Err(e) => {
                error!("Failed to update customer from IVMS101: {}", e);
                Ok(error_text_response(format!(
                    "Failed to update customer from IVMS101: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_update_customer_from_ivms101".to_string(),
            description: "Updates a customer's profile using IVMS101 data. This extracts name, address and other fields from IVMS101 format.".to_string(),
            input_schema: schema::update_customer_from_ivms101_schema(),
        }
    }
}
