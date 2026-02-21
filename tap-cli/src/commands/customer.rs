use crate::error::{Error, Result};
use crate::output::{print_success, OutputFormat};
use crate::tap_integration::TapIntegration;
use clap::Subcommand;
use serde::Serialize;
use tap_node::customer::CustomerManager;
use tap_node::storage::models::{Customer, SchemaType};

#[derive(Subcommand, Debug)]
pub enum CustomerCommands {
    /// List customers
    List {
        /// Agent DID for storage lookup
        #[arg(long)]
        agent_did: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "50")]
        limit: u32,
        /// Offset for pagination
        #[arg(long, default_value = "0")]
        offset: u32,
    },
    /// Create a new customer record
    Create {
        /// Customer identifier (DID or CAIP-10 address)
        #[arg(long)]
        customer_id: String,
        /// Customer profile as JSON
        #[arg(long)]
        profile: String,
        /// Agent DID for storage lookup
        #[arg(long)]
        agent_did: Option<String>,
    },
    /// View customer details
    Details {
        /// Customer identifier
        #[arg(long)]
        customer_id: String,
        /// Agent DID for storage lookup
        #[arg(long)]
        agent_did: Option<String>,
    },
    /// Update a customer profile
    Update {
        /// Customer identifier
        #[arg(long)]
        customer_id: String,
        /// Updated profile as JSON
        #[arg(long)]
        profile: String,
        /// Agent DID for storage lookup
        #[arg(long)]
        agent_did: Option<String>,
    },
    /// Generate IVMS101 data for a customer
    Ivms101 {
        /// Customer identifier
        #[arg(long)]
        customer_id: String,
        /// Agent DID for storage lookup
        #[arg(long)]
        agent_did: Option<String>,
    },
}

#[derive(Debug, Serialize)]
struct CustomerInfo {
    id: String,
    display_name: Option<String>,
    schema_type: String,
    address_country: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct CustomerListResponse {
    customers: Vec<CustomerInfo>,
    total: usize,
}

pub async fn handle(
    cmd: &CustomerCommands,
    format: OutputFormat,
    default_agent_did: &str,
    tap_integration: &TapIntegration,
) -> Result<()> {
    match cmd {
        CustomerCommands::List {
            agent_did,
            limit,
            offset,
        } => {
            let effective_did = agent_did.as_deref().unwrap_or(default_agent_did);
            let storage = tap_integration.storage_for_agent(effective_did).await?;
            let customers = storage
                .list_customers(effective_did, *limit, *offset)
                .await?;

            let customer_infos: Vec<CustomerInfo> = customers
                .iter()
                .map(|c| CustomerInfo {
                    id: c.id.clone(),
                    display_name: c.display_name.clone(),
                    schema_type: format!("{:?}", c.schema_type),
                    address_country: c.address_country.clone(),
                    created_at: c.created_at.clone(),
                    updated_at: c.updated_at.clone(),
                })
                .collect();

            let response = CustomerListResponse {
                total: customer_infos.len(),
                customers: customer_infos,
            };
            print_success(format, &response);
            Ok(())
        }
        CustomerCommands::Create {
            customer_id,
            profile,
            agent_did,
        } => {
            let effective_did = agent_did.as_deref().unwrap_or(default_agent_did);
            let storage = tap_integration.storage_for_agent(effective_did).await?;

            let profile_json: serde_json::Value = serde_json::from_str(profile)
                .map_err(|e| Error::invalid_parameter(format!("Invalid profile JSON: {}", e)))?;

            let schema_type =
                if profile_json.get("@type").and_then(|v| v.as_str()) == Some("Organization") {
                    SchemaType::Organization
                } else {
                    SchemaType::Person
                };

            let customer = Customer {
                id: customer_id.clone(),
                agent_did: effective_did.to_string(),
                schema_type,
                given_name: profile_json
                    .get("givenName")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                family_name: profile_json
                    .get("familyName")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                display_name: profile_json
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                legal_name: profile_json
                    .get("legalName")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                lei_code: profile_json
                    .get("leiCode")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                mcc_code: profile_json
                    .get("mccCode")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                address_country: profile_json
                    .get("addressCountry")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                address_locality: profile_json
                    .get("addressLocality")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                postal_code: profile_json
                    .get("postalCode")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                street_address: profile_json
                    .get("streetAddress")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                profile: profile_json,
                ivms101_data: None,
                verified_at: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };

            storage
                .upsert_customer(&customer)
                .await
                .map_err(|e| Error::command_failed(format!("Failed to create customer: {}", e)))?;

            #[derive(Serialize)]
            struct Created {
                customer_id: String,
                status: String,
            }
            let response = Created {
                customer_id: customer_id.clone(),
                status: "created".to_string(),
            };
            print_success(format, &response);
            Ok(())
        }
        CustomerCommands::Details {
            customer_id,
            agent_did,
        } => {
            let effective_did = agent_did.as_deref().unwrap_or(default_agent_did);
            let storage = tap_integration.storage_for_agent(effective_did).await?;
            let customer = storage.get_customer(customer_id).await?;

            match customer {
                Some(c) => {
                    print_success(format, &c);
                    Ok(())
                }
                None => Err(Error::command_failed(format!(
                    "Customer '{}' not found",
                    customer_id
                ))),
            }
        }
        CustomerCommands::Update {
            customer_id,
            profile,
            agent_did,
        } => {
            let effective_did = agent_did.as_deref().unwrap_or(default_agent_did);
            let storage = tap_integration.storage_for_agent(effective_did).await?;

            let profile_json: serde_json::Value = serde_json::from_str(profile)
                .map_err(|e| Error::invalid_parameter(format!("Invalid profile JSON: {}", e)))?;

            // Get existing customer to preserve fields
            let existing = storage.get_customer(customer_id).await?.ok_or_else(|| {
                Error::command_failed(format!("Customer '{}' not found", customer_id))
            })?;

            let customer = Customer {
                profile: profile_json,
                updated_at: chrono::Utc::now().to_rfc3339(),
                ..existing
            };

            storage
                .upsert_customer(&customer)
                .await
                .map_err(|e| Error::command_failed(format!("Failed to update customer: {}", e)))?;

            #[derive(Serialize)]
            struct Updated {
                customer_id: String,
                status: String,
            }
            let response = Updated {
                customer_id: customer_id.clone(),
                status: "updated".to_string(),
            };
            print_success(format, &response);
            Ok(())
        }
        CustomerCommands::Ivms101 {
            customer_id,
            agent_did,
        } => {
            let effective_did = agent_did.as_deref().unwrap_or(default_agent_did);
            let storage = tap_integration.storage_for_agent(effective_did).await?;
            let customer_manager = CustomerManager::new(storage);

            let ivms_data = customer_manager
                .generate_ivms101_data(customer_id)
                .await
                .map_err(|e| {
                    Error::command_failed(format!("Failed to generate IVMS101 data: {}", e))
                })?;
            print_success(format, &ivms_data);
            Ok(())
        }
    }
}
