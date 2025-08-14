//! MCP resources for read-only data access

use crate::error::{Error, Result};
use crate::mcp::protocol::{Resource, ResourceContent};
use crate::tap_integration::TapIntegration;
use serde_json::json;
use sqlx::{Connection, Row, SqliteConnection};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error};
use url::Url;

/// Registry for all available resources
pub struct ResourceRegistry {
    tap_integration: Arc<TapIntegration>,
}

impl ResourceRegistry {
    /// Create a new resource registry
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }

    /// List all available resources
    pub async fn list_resources(&self) -> Vec<Resource> {
        vec![
            Resource {
                uri: "tap://agents".to_string(),
                name: "TAP Agents".to_string(),
                description: "List of all configured TAP agents".to_string(),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "tap://messages".to_string(),
                name: "TAP Messages".to_string(),
                description: "TAP messages from agent storage. Query parameters: ?agent_did=<did>&direction=<incoming|outgoing>&type=<message_type>&thread_id=<id>&limit=<n>&offset=<n>".to_string(),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "tap://deliveries".to_string(),
                name: "TAP Deliveries".to_string(),
                description: "Message delivery tracking from agent storage. Query parameters: ?agent_did=<did>&message_id=<id>&recipient_did=<did>&delivery_type=<https|internal|return_path|pickup>&status=<pending|success|failed>&limit=<n>&offset=<n>".to_string(),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "tap://database-schema".to_string(),
                name: "Database Schema".to_string(),
                description: "Database schema information for agent storage. Query parameters: ?agent_did=<did>&table_name=<table>".to_string(),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "tap://schemas".to_string(),
                name: "TAP Schemas".to_string(),
                description: "JSON schemas for TAP message types. Use tap://schemas/{MessageType} to get specific schema (e.g., tap://schemas/Transfer, tap://schemas/Authorize)".to_string(),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "tap://received".to_string(),
                name: "TAP Received Messages".to_string(),
                description: "Raw received messages before processing. Query parameters: ?agent_did=<did>&source_type=<https|internal|websocket|return_path|pickup>&status=<pending|processed|failed>&limit=<n>&offset=<n>".to_string(),
                mime_type: Some("application/json".to_string()),
            },
        ]
    }

    /// Read a resource by URI
    pub async fn read_resource(&self, uri: &str) -> Result<Vec<ResourceContent>> {
        debug!("Reading resource: {}", uri);

        let url = Url::parse(uri)
            .map_err(|e| Error::resource_not_found(format!("Invalid URI: {}", e)))?;

        if url.scheme() != "tap" {
            return Err(Error::resource_not_found("Only tap:// URIs are supported"));
        }

        match url.host_str() {
            Some("agents") => self.read_agents_resource(url.path(), url.query()).await,
            Some("messages") => self.read_messages_resource(url.path(), url.query()).await,
            Some("deliveries") => self.read_deliveries_resource(url.path(), url.query()).await,
            Some("database-schema") => {
                self.read_database_schema_resource(url.path(), url.query())
                    .await
            }
            Some("schemas") => self.read_schemas_resource(url.path()).await,
            Some("received") => self.read_received_resource(url.path(), url.query()).await,
            _ => Err(Error::resource_not_found(format!(
                "Unknown resource: {}",
                uri
            ))),
        }
    }

    /// Read agents resource
    async fn read_agents_resource(
        &self,
        _path: &str,
        query: Option<&str>,
    ) -> Result<Vec<ResourceContent>> {
        let agents = self.tap_integration().list_agents().await?;

        // Parse query parameters for filtering
        let mut role_filter = None;
        let mut for_filter = None;

        if let Some(query_str) = query {
            let params: HashMap<String, String> = url::form_urlencoded::parse(query_str.as_bytes())
                .into_owned()
                .collect();

            role_filter = params.get("role").cloned();
            for_filter = params.get("for").cloned();
        }

        // Apply filters
        let filtered_agents: Vec<_> = agents
            .into_iter()
            .filter(|agent| {
                if let Some(ref role) = role_filter {
                    if agent.role != *role {
                        return false;
                    }
                }
                if let Some(ref for_party) = for_filter {
                    if agent.for_party != *for_party {
                        return false;
                    }
                }
                true
            })
            .collect();

        let content = json!({
            "agents": filtered_agents.iter().map(|agent| json!({
                "@id": agent.id,
                "role": agent.role,
                "for": agent.for_party,
                "policies": agent.policies,
                "metadata": agent.metadata
            })).collect::<Vec<_>>(),
            "total": filtered_agents.len(),
            "query_applied": query.is_some()
        });

        Ok(vec![ResourceContent {
            uri: format!(
                "tap://agents{}",
                if query.is_some() {
                    format!("?{}", query.unwrap())
                } else {
                    String::new()
                }
            ),
            mime_type: Some("application/json".to_string()),
            text: Some(serde_json::to_string_pretty(&content)?),
            blob: None,
        }])
    }

    /// Read messages resource
    async fn read_messages_resource(
        &self,
        path: &str,
        query: Option<&str>,
    ) -> Result<Vec<ResourceContent>> {
        // Parse path for specific message ID
        if !path.is_empty() && path != "/" {
            let message_id = path.trim_start_matches('/');
            return self.read_specific_message(message_id).await;
        }

        // Parse query parameters
        let mut thread_id_filter = None;
        let mut message_type_filter = None;
        let mut direction_filter = None;
        let mut agent_did_filter = None;
        let mut limit = 50;
        let mut offset = 0;

        if let Some(query_str) = query {
            let params: HashMap<String, String> = url::form_urlencoded::parse(query_str.as_bytes())
                .into_owned()
                .collect();

            thread_id_filter = params.get("thread_id").cloned();
            message_type_filter = params.get("type").cloned();
            agent_did_filter = params.get("agent_did").cloned();

            // Parse direction filter if provided
            if let Some(direction_str) = params.get("direction") {
                match direction_str.as_str() {
                    "incoming" => {
                        direction_filter = Some(tap_node::storage::MessageDirection::Incoming)
                    }
                    "outgoing" => {
                        direction_filter = Some(tap_node::storage::MessageDirection::Outgoing)
                    }
                    _ => {} // Invalid direction, ignore
                }
            }

            if let Some(limit_str) = params.get("limit") {
                if let Ok(l) = limit_str.parse::<u32>() {
                    limit = l.min(1000); // Cap at 1000
                }
            }

            if let Some(offset_str) = params.get("offset") {
                if let Ok(o) = offset_str.parse::<u32>() {
                    offset = o;
                }
            }
        }

        // Get messages from storage
        let messages = if let Some(agent_did) = agent_did_filter.as_ref() {
            // Use agent-specific storage
            let agent_storage = self
                .tap_integration()
                .storage_for_agent(agent_did)
                .await
                .map_err(|e| {
                    Error::resource_not_found(format!("Failed to get agent storage: {}", e))
                })?;
            agent_storage
                .list_messages(limit, offset, direction_filter.clone())
                .await?
        } else {
            // If no agent specified, try to get messages from centralized storage
            // This is fallback behavior for backwards compatibility
            let storage = self
                .tap_integration()
                .storage()
                .ok_or_else(|| Error::resource_not_found("Storage not initialized and no agent_did specified. Please specify ?agent_did=<did> to get messages for a specific agent."))?;
            storage
                .list_messages(limit, offset, direction_filter.clone())
                .await?
        };

        // Apply additional filters
        let filtered_messages: Vec<_> = messages
            .into_iter()
            .filter(|msg| {
                if let Some(ref thread_id) = thread_id_filter {
                    if msg.thread_id.as_ref() != Some(thread_id) {
                        return false;
                    }
                }
                if let Some(ref msg_type) = message_type_filter {
                    if !msg.message_type.contains(msg_type) {
                        return false;
                    }
                }
                true
            })
            .collect();

        let content = json!({
            "messages": filtered_messages.iter().map(|msg| json!({
                "id": msg.message_id,
                "type": msg.message_type,
                "thread_id": msg.thread_id,
                "from": msg.from_did,
                "to": msg.to_did,
                "direction": msg.direction.to_string(),
                "created_at": msg.created_at,
                "body": msg.message_json
            })).collect::<Vec<_>>(),
            "total": filtered_messages.len(),
            "applied_filters": {
                "agent_did": agent_did_filter,
                "direction": direction_filter.as_ref().map(|d| d.to_string()),
                "thread_id": thread_id_filter,
                "message_type": message_type_filter,
                "limit": limit,
                "offset": offset
            }
        });

        Ok(vec![ResourceContent {
            uri: format!(
                "tap://messages{}",
                if query.is_some() {
                    format!("?{}", query.unwrap())
                } else {
                    String::new()
                }
            ),
            mime_type: Some("application/json".to_string()),
            text: Some(serde_json::to_string_pretty(&content)?),
            blob: None,
        }])
    }

    /// Read a specific message by ID
    async fn read_specific_message(&self, message_id: &str) -> Result<Vec<ResourceContent>> {
        // For specific message lookup, we'll try to find it in any agent's storage
        // First try centralized storage for backwards compatibility
        if let Some(storage) = self.tap_integration().storage() {
            if let Ok(Some(message)) = storage.get_message_by_id(message_id).await {
                let content = json!({
                    "message": {
                        "id": message.message_id,
                        "type": message.message_type,
                        "thread_id": message.thread_id,
                        "parent_thread_id": message.parent_thread_id,
                        "from": message.from_did,
                        "to": message.to_did,
                        "direction": message.direction.to_string(),
                        "created_at": message.created_at,
                        "body": message.message_json
                    }
                });

                return Ok(vec![ResourceContent {
                    uri: format!("tap://messages/{}", message_id),
                    mime_type: Some("application/json".to_string()),
                    text: Some(serde_json::to_string_pretty(&content)?),
                    blob: None,
                }]);
            }
        }

        // If not found in centralized storage, search all agent storages
        let agents = self.tap_integration().list_agents().await?;
        for agent in agents {
            if let Ok(agent_storage) = self.tap_integration().storage_for_agent(&agent.id).await {
                if let Ok(Some(message)) = agent_storage.get_message_by_id(message_id).await {
                    let content = json!({
                        "message": {
                            "id": message.message_id,
                            "type": message.message_type,
                            "thread_id": message.thread_id,
                            "parent_thread_id": message.parent_thread_id,
                            "from": message.from_did,
                            "to": message.to_did,
                            "direction": message.direction.to_string(),
                            "created_at": message.created_at,
                            "body": message.message_json
                        },
                        "found_in_agent": agent.id
                    });

                    return Ok(vec![ResourceContent {
                        uri: format!("tap://messages/{}", message_id),
                        mime_type: Some("application/json".to_string()),
                        text: Some(serde_json::to_string_pretty(&content)?),
                        blob: None,
                    }]);
                }
            }
        }

        Err(Error::resource_not_found(format!(
            "Message not found: {}",
            message_id
        )))
    }

    /// Read deliveries resource
    async fn read_deliveries_resource(
        &self,
        path: &str,
        query: Option<&str>,
    ) -> Result<Vec<ResourceContent>> {
        // Parse path for specific delivery ID
        if !path.is_empty() && path != "/" {
            let delivery_id = path.trim_start_matches('/');
            return self.read_specific_delivery(delivery_id).await;
        }

        // Parse query parameters
        let mut message_id_filter = None;
        let mut recipient_did_filter = None;
        let mut delivery_type_filter = None;
        let mut status_filter = None;
        let mut agent_did_filter = None;
        let mut limit = 50;
        let mut offset = 0;

        if let Some(query_str) = query {
            let params: HashMap<String, String> = url::form_urlencoded::parse(query_str.as_bytes())
                .into_owned()
                .collect();

            message_id_filter = params.get("message_id").cloned();
            recipient_did_filter = params.get("recipient_did").cloned();
            agent_did_filter = params.get("agent_did").cloned();

            // Parse delivery type filter if provided
            if let Some(delivery_type_str) = params.get("delivery_type") {
                match delivery_type_str.as_str() {
                    "https" => delivery_type_filter = Some("https".to_string()),
                    "internal" => delivery_type_filter = Some("internal".to_string()),
                    "return_path" => delivery_type_filter = Some("return_path".to_string()),
                    "pickup" => delivery_type_filter = Some("pickup".to_string()),
                    _ => {} // Invalid delivery type, ignore
                }
            }

            // Parse status filter if provided
            if let Some(status_str) = params.get("status") {
                match status_str.as_str() {
                    "pending" => status_filter = Some("pending".to_string()),
                    "success" => status_filter = Some("success".to_string()),
                    "failed" => status_filter = Some("failed".to_string()),
                    _ => {} // Invalid status, ignore
                }
            }

            if let Some(limit_str) = params.get("limit") {
                if let Ok(l) = limit_str.parse::<u32>() {
                    limit = l.min(1000); // Cap at 1000
                }
            }

            if let Some(offset_str) = params.get("offset") {
                if let Ok(o) = offset_str.parse::<u32>() {
                    offset = o;
                }
            }
        }

        // Get deliveries from storage
        let deliveries = if let Some(agent_did) = agent_did_filter.as_ref() {
            // Use agent-specific storage
            let agent_storage = self
                .tap_integration()
                .storage_for_agent(agent_did)
                .await
                .map_err(|e| {
                    Error::resource_not_found(format!("Failed to get agent storage: {}", e))
                })?;

            // If message_id is specified, get deliveries for that message
            if let Some(message_id) = message_id_filter.as_ref() {
                agent_storage.get_deliveries_for_message(message_id).await?
            } else {
                // For now, we'll get pending deliveries as a default
                // TODO: Implement a general get_all_deliveries method
                agent_storage
                    .get_pending_deliveries(10, limit) // max_retry_count=10
                    .await?
            }
        } else {
            return Err(Error::resource_not_found(
                "agent_did parameter is required to view deliveries",
            ));
        };

        // Apply additional filters
        let filtered_deliveries: Vec<_> = deliveries
            .into_iter()
            .filter(|delivery| {
                if let Some(ref message_id) = message_id_filter {
                    if &delivery.message_id != message_id {
                        return false;
                    }
                }
                if let Some(ref recipient_did) = recipient_did_filter {
                    if &delivery.recipient_did != recipient_did {
                        return false;
                    }
                }
                if let Some(ref delivery_type) = delivery_type_filter {
                    if &delivery.delivery_type.to_string() != delivery_type {
                        return false;
                    }
                }
                if let Some(ref status) = status_filter {
                    if &delivery.status.to_string() != status {
                        return false;
                    }
                }
                true
            })
            .collect();

        let content = json!({
            "deliveries": filtered_deliveries.iter().map(|delivery| json!({
                "id": delivery.id,
                "message_id": delivery.message_id,
                "message_text": delivery.message_text,
                "recipient_did": delivery.recipient_did,
                "delivery_url": delivery.delivery_url,
                "delivery_type": delivery.delivery_type.to_string(),
                "status": delivery.status.to_string(),
                "retry_count": delivery.retry_count,
                "last_http_status_code": delivery.last_http_status_code,
                "error_message": delivery.error_message,
                "created_at": delivery.created_at,
                "updated_at": delivery.updated_at,
                "delivered_at": delivery.delivered_at
            })).collect::<Vec<_>>(),
            "total": filtered_deliveries.len(),
            "applied_filters": {
                "agent_did": agent_did_filter,
                "message_id": message_id_filter,
                "recipient_did": recipient_did_filter,
                "delivery_type": delivery_type_filter,
                "status": status_filter,
                "limit": limit,
                "offset": offset
            }
        });

        Ok(vec![ResourceContent {
            uri: format!(
                "tap://deliveries{}",
                if query.is_some() {
                    format!("?{}", query.unwrap())
                } else {
                    String::new()
                }
            ),
            mime_type: Some("application/json".to_string()),
            text: Some(serde_json::to_string_pretty(&content)?),
            blob: None,
        }])
    }

    /// Read a specific delivery by ID
    async fn read_specific_delivery(&self, delivery_id: &str) -> Result<Vec<ResourceContent>> {
        // Parse delivery_id as i64
        let id: i64 = delivery_id
            .parse()
            .map_err(|_| Error::resource_not_found("Delivery ID must be a valid number"))?;

        // Search all agent storages for the delivery
        let agents = self.tap_integration().list_agents().await?;
        for agent in agents {
            if let Ok(agent_storage) = self.tap_integration().storage_for_agent(&agent.id).await {
                if let Ok(Some(delivery)) = agent_storage.get_delivery_by_id(id).await {
                    let content = json!({
                        "delivery": {
                            "id": delivery.id,
                            "message_id": delivery.message_id,
                            "message_text": delivery.message_text,
                            "recipient_did": delivery.recipient_did,
                            "delivery_url": delivery.delivery_url,
                            "delivery_type": delivery.delivery_type.to_string(),
                            "status": delivery.status.to_string(),
                            "retry_count": delivery.retry_count,
                            "last_http_status_code": delivery.last_http_status_code,
                            "error_message": delivery.error_message,
                            "created_at": delivery.created_at,
                            "updated_at": delivery.updated_at,
                            "delivered_at": delivery.delivered_at
                        },
                        "found_in_agent": agent.id
                    });

                    return Ok(vec![ResourceContent {
                        uri: format!("tap://deliveries/{}", delivery_id),
                        mime_type: Some("application/json".to_string()),
                        text: Some(serde_json::to_string_pretty(&content)?),
                        blob: None,
                    }]);
                }
            }
        }

        Err(Error::resource_not_found(format!(
            "Delivery not found: {}",
            delivery_id
        )))
    }

    /// Read received messages resource
    async fn read_received_resource(
        &self,
        path: &str,
        query: Option<&str>,
    ) -> Result<Vec<ResourceContent>> {
        // Parse path for specific received ID
        if !path.is_empty() && path != "/" {
            let received_id = path.trim_start_matches('/');
            return self.read_specific_received(received_id).await;
        }

        // Parse query parameters
        let mut source_type_filter = None;
        let mut status_filter = None;
        let mut agent_did_filter = None;
        let mut limit = 50;
        let mut offset = 0;

        if let Some(query_str) = query {
            let params: HashMap<String, String> = url::form_urlencoded::parse(query_str.as_bytes())
                .into_owned()
                .collect();

            agent_did_filter = params.get("agent_did").cloned();

            // Parse source type filter if provided
            if let Some(source_type_str) = params.get("source_type") {
                source_type_filter = match source_type_str.as_str() {
                    "https" => Some(tap_node::storage::SourceType::Https),
                    "internal" => Some(tap_node::storage::SourceType::Internal),
                    "websocket" => Some(tap_node::storage::SourceType::WebSocket),
                    "return_path" => Some(tap_node::storage::SourceType::ReturnPath),
                    "pickup" => Some(tap_node::storage::SourceType::Pickup),
                    _ => None,
                };
            }

            // Parse status filter if provided
            if let Some(status_str) = params.get("status") {
                status_filter = match status_str.as_str() {
                    "pending" => Some(tap_node::storage::ReceivedStatus::Pending),
                    "processed" => Some(tap_node::storage::ReceivedStatus::Processed),
                    "failed" => Some(tap_node::storage::ReceivedStatus::Failed),
                    _ => None,
                };
            }

            if let Some(limit_str) = params.get("limit") {
                if let Ok(l) = limit_str.parse::<u32>() {
                    limit = l.min(1000); // Cap at 1000
                }
            }

            if let Some(offset_str) = params.get("offset") {
                if let Ok(o) = offset_str.parse::<u32>() {
                    offset = o;
                }
            }
        }

        // Get received messages from storage
        let received_messages = if let Some(agent_did) = agent_did_filter.as_ref() {
            // Use agent-specific storage
            let agent_storage = self
                .tap_integration()
                .storage_for_agent(agent_did)
                .await
                .map_err(|e| {
                    Error::resource_not_found(format!("Failed to get agent storage: {}", e))
                })?;
            agent_storage
                .list_received(
                    limit,
                    offset,
                    source_type_filter.clone(),
                    status_filter.clone(),
                )
                .await?
        } else {
            return Err(Error::resource_not_found(
                "agent_did parameter is required to view received messages",
            ));
        };

        let content = json!({
            "received_messages": received_messages.iter().map(|msg| json!({
                "id": msg.id,
                "message_id": msg.message_id,
                "source_type": format!("{:?}", msg.source_type).to_lowercase(),
                "source_identifier": msg.source_identifier,
                "status": format!("{:?}", msg.status).to_lowercase(),
                "error_message": msg.error_message,
                "received_at": msg.received_at,
                "processed_at": msg.processed_at,
                "processed_message_id": msg.processed_message_id,
                // Include a preview of the raw message (first 200 chars)
                "raw_message_preview": if msg.raw_message.len() > 200 {
                    format!("{}...", &msg.raw_message[..200])
                } else {
                    msg.raw_message.clone()
                }
            })).collect::<Vec<_>>(),
            "total": received_messages.len(),
            "applied_filters": {
                "agent_did": agent_did_filter,
                "source_type": source_type_filter.as_ref().map(|s| format!("{:?}", s).to_lowercase()),
                "status": status_filter.as_ref().map(|s| format!("{:?}", s).to_lowercase()),
                "limit": limit,
                "offset": offset
            }
        });

        Ok(vec![ResourceContent {
            uri: format!(
                "tap://received{}",
                if query.is_some() {
                    format!("?{}", query.unwrap())
                } else {
                    String::new()
                }
            ),
            mime_type: Some("application/json".to_string()),
            text: Some(serde_json::to_string_pretty(&content)?),
            blob: None,
        }])
    }

    /// Read a specific received message by ID
    async fn read_specific_received(&self, received_id: &str) -> Result<Vec<ResourceContent>> {
        // Parse received_id as i64
        let id: i64 = received_id
            .parse()
            .map_err(|_| Error::resource_not_found("Received ID must be a valid number"))?;

        // Search all agent storages for the received message
        let agents = self.tap_integration().list_agents().await?;
        for agent in agents {
            if let Ok(agent_storage) = self.tap_integration().storage_for_agent(&agent.id).await {
                if let Ok(Some(received)) = agent_storage.get_received_by_id(id).await {
                    // Try to parse raw message as JSON
                    let raw_json =
                        serde_json::from_str::<serde_json::Value>(&received.raw_message).ok();

                    let content = json!({
                        "received": {
                            "id": received.id,
                            "message_id": received.message_id,
                            "source_type": format!("{:?}", received.source_type).to_lowercase(),
                            "source_identifier": received.source_identifier,
                            "status": format!("{:?}", received.status).to_lowercase(),
                            "error_message": received.error_message,
                            "received_at": received.received_at,
                            "processed_at": received.processed_at,
                            "processed_message_id": received.processed_message_id,
                            "raw_message": received.raw_message,
                            "raw_message_json": raw_json
                        },
                        "found_in_agent": agent.id
                    });

                    return Ok(vec![ResourceContent {
                        uri: format!("tap://received/{}", received_id),
                        mime_type: Some("application/json".to_string()),
                        text: Some(serde_json::to_string_pretty(&content)?),
                        blob: None,
                    }]);
                }
            }
        }

        Err(Error::resource_not_found(format!(
            "Received message not found: {}",
            received_id
        )))
    }

    /// Read database schema resource
    async fn read_database_schema_resource(
        &self,
        _path: &str,
        query: Option<&str>,
    ) -> Result<Vec<ResourceContent>> {
        // Parse query parameters
        let mut agent_did_filter = None;
        let mut table_name_filter = None;

        if let Some(query_str) = query {
            let params: HashMap<String, String> = url::form_urlencoded::parse(query_str.as_bytes())
                .into_owned()
                .collect();

            agent_did_filter = params.get("agent_did").cloned();
            table_name_filter = params.get("table_name").cloned();
        }

        let agent_did = agent_did_filter.clone().ok_or_else(|| {
            Error::resource_not_found("agent_did parameter is required to view database schema")
        })?;

        // Get agent storage
        let storage = self
            .tap_integration()
            .storage_for_agent(&agent_did)
            .await
            .map_err(|e| {
                Error::resource_not_found(format!("Failed to get agent storage: {}", e))
            })?;

        // Get database path from storage
        let db_path = storage.db_path();
        let db_url = format!("sqlite://{}?mode=ro", db_path.display());

        // Connect to database in read-only mode
        let mut conn = SqliteConnection::connect(&db_url).await.map_err(|e| {
            error!("Failed to connect to database: {}", e);
            Error::resource_not_found(format!("Failed to connect to database: {}", e))
        })?;

        let mut tables = Vec::new();

        // Get list of tables
        let table_query = if let Some(ref table_name) = table_name_filter {
            format!(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='{}' ORDER BY name",
                table_name
            )
        } else {
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name".to_string()
        };

        let table_rows = sqlx::query(&table_query)
            .fetch_all(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get tables: {}", e);
                Error::resource_not_found(format!("Failed to get tables: {}", e))
            })?;

        for table_row in table_rows {
            let table_name: String = table_row.try_get("name").unwrap_or_default();

            // Get columns for this table
            let column_query = format!("PRAGMA table_info('{}')", table_name);
            let column_rows = sqlx::query(&column_query)
                .fetch_all(&mut conn)
                .await
                .map_err(|e| {
                    error!("Failed to get columns for table {}: {}", table_name, e);
                    Error::resource_not_found(format!(
                        "Failed to get columns for table {}: {}",
                        table_name, e
                    ))
                })?;

            let mut columns = Vec::new();
            for col_row in column_rows {
                columns.push(json!({
                    "cid": col_row.try_get::<i32, _>("cid").unwrap_or(0),
                    "name": col_row.try_get::<String, _>("name").unwrap_or_default(),
                    "type": col_row.try_get::<String, _>("type").unwrap_or_default(),
                    "notnull": col_row.try_get::<i32, _>("notnull").unwrap_or(0) != 0,
                    "dflt_value": col_row.try_get::<Option<String>, _>("dflt_value").ok().flatten(),
                    "pk": col_row.try_get::<i32, _>("pk").unwrap_or(0) != 0,
                }));
            }

            // Get indexes for this table
            let index_query = format!("PRAGMA index_list('{}')", table_name);
            let index_rows = sqlx::query(&index_query)
                .fetch_all(&mut conn)
                .await
                .unwrap_or_default();

            let mut indexes = Vec::new();
            for idx_row in index_rows {
                indexes.push(json!({
                    "name": idx_row.try_get::<String, _>("name").unwrap_or_default(),
                    "unique": idx_row.try_get::<i32, _>("unique").unwrap_or(0) != 0,
                    "origin": idx_row.try_get::<String, _>("origin").unwrap_or_default(),
                    "partial": idx_row.try_get::<i32, _>("partial").unwrap_or(0) != 0,
                }));
            }

            // Get row count
            let count_query = format!("SELECT COUNT(*) as count FROM '{}'", table_name);
            let row_count = sqlx::query(&count_query)
                .fetch_one(&mut conn)
                .await
                .ok()
                .and_then(|row| row.try_get::<i64, _>("count").ok())
                .unwrap_or(0);

            tables.push(json!({
                "name": table_name,
                "columns": columns,
                "indexes": indexes,
                "row_count": row_count,
            }));
        }

        let content = json!({
            "database_path": db_path.display().to_string(),
            "agent_did": agent_did,
            "tables": tables,
            "applied_filters": {
                "agent_did": agent_did_filter,
                "table_name": table_name_filter,
            }
        });

        Ok(vec![ResourceContent {
            uri: format!(
                "tap://database-schema{}",
                if query.is_some() {
                    format!("?{}", query.unwrap())
                } else {
                    String::new()
                }
            ),
            mime_type: Some("application/json".to_string()),
            text: Some(serde_json::to_string_pretty(&content)?),
            blob: None,
        }])
    }

    /// Read schemas resource  
    async fn read_schemas_resource(&self, path: &str) -> Result<Vec<ResourceContent>> {
        // Check if requesting a specific message type schema
        if !path.is_empty() && path != "/" {
            let message_type = path.trim_start_matches('/');
            return self.read_specific_schema(message_type).await;
        }

        let schemas = self.get_all_schemas();

        Ok(vec![ResourceContent {
            uri: "tap://schemas".to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(serde_json::to_string_pretty(&schemas)?),
            blob: None,
        }])
    }

    /// Get all schemas as JSON value
    fn get_all_schemas(&self) -> serde_json::Value {
        json!({
            "version": "1.0",
            "description": "JSON schemas for TAP (Transfer Authorization Protocol) message types as defined in various TAIPs",
            "schemas": {
                "Transfer": {
                    "description": "TAP Transfer message (TAIP-3) - Initiates a new transfer between parties",
                    "message_type": "https://tap.rsvp/schema/1.0#Transfer",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "Unique transaction identifier" },
                        "asset": { "type": "string", "description": "CAIP-19 asset identifier" },
                        "amount": { "type": "string", "description": "Transfer amount as decimal string" },
                        "originator": {
                            "type": "object",
                            "description": "Party initiating the transfer",
                            "properties": {
                                "@id": { "type": "string", "description": "DID or identifier of the originator" }
                            }
                        },
                        "beneficiary": {
                            "type": "object",
                            "description": "Party receiving the transfer",
                            "properties": {
                                "@id": { "type": "string", "description": "DID or identifier of the beneficiary" }
                            }
                        },
                        "agents": {
                            "type": "array",
                            "description": "List of agents involved in the transaction",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "@id": { "type": "string", "description": "Agent DID" },
                                    "role": { "type": "string", "description": "Agent role (e.g., SettlementAddress)" },
                                    "for": { "type": "string", "description": "DID of party agent acts for" }
                                }
                            }
                        },
                        "memo": { "type": "string", "description": "Optional transaction memo" },
                        "settlement_id": { "type": "string", "description": "Optional pre-existing settlement ID" },
                        "connection_id": { "type": "string", "description": "Optional connection ID" }
                    },
                    "required": ["transaction_id", "asset", "amount"]
                },
                "Authorize": {
                    "description": "TAP Authorize message (TAIP-8) - Authorizes a transaction to proceed",
                    "message_type": "https://tap.rsvp/schema/1.0#Authorize",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "ID of transaction to authorize" },
                        "settlement_address": { "type": "string", "description": "Optional CAIP-10 settlement address" },
                        "expiry": { "type": "string", "description": "Optional ISO 8601 expiry timestamp" }
                    },
                    "required": ["transaction_id"]
                },
                "Reject": {
                    "description": "TAP Reject message (TAIP-10) - Rejects a transaction",
                    "message_type": "https://tap.rsvp/schema/1.0#Reject",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "ID of transaction to reject" },
                        "reason": { "type": "string", "description": "Optional reason for rejection" }
                    },
                    "required": ["transaction_id"]
                },
                "Settle": {
                    "description": "TAP Settle message (TAIP-9) - Confirms settlement of a transaction",
                    "message_type": "https://tap.rsvp/schema/1.0#Settle",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "ID of transaction to settle" },
                        "settlement_id": { "type": "string", "description": "Optional CAIP-220 settlement identifier" },
                        "amount": { "type": "string", "description": "Optional amount settled" }
                    },
                    "required": ["transaction_id"]
                },
                "Cancel": {
                    "description": "TAP Cancel message (TAIP-11) - Cancels a transaction",
                    "message_type": "https://tap.rsvp/schema/1.0#Cancel",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "ID of transaction to cancel" },
                        "by": { "type": "string", "description": "Party requesting cancellation" },
                        "reason": { "type": "string", "description": "Optional reason for cancellation" }
                    },
                    "required": ["transaction_id", "by"]
                },
                "Revert": {
                    "description": "TAP Revert message (TAIP-12) - Requests reversal of a settled transaction",
                    "message_type": "https://tap.rsvp/schema/1.0#Revert",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "ID of transaction to revert" },
                        "settlement_address": { "type": "string", "description": "CAIP-10 address to return funds to" },
                        "reason": { "type": "string", "description": "Reason for reversal request" }
                    },
                    "required": ["transaction_id", "settlement_address", "reason"]
                },
                "Complete": {
                    "description": "TAP Complete message (TAIP-13) - Confirms completion of a payment",
                    "message_type": "https://tap.rsvp/schema/1.0#Complete",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "ID of transaction to complete" },
                        "settlement_address": { "type": "string", "description": "CAIP-10 settlement address" },
                        "amount": { "type": "string", "description": "Optional amount completed" }
                    },
                    "required": ["transaction_id", "settlement_address"]
                },
                "Payment": {
                    "description": "TAP Payment message (TAIP-13) - Initiates a payment request",
                    "message_type": "https://tap.rsvp/schema/1.0#Payment",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "Unique transaction identifier" },
                        "invoice": {
                            "description": "Either an invoice URL or invoice object",
                            "oneOf": [
                                { "type": "string", "description": "URL to an invoice" },
                                {
                                    "type": "object",
                                    "description": "Structured invoice object",
                                    "properties": {
                                        "amount": { "type": "string" },
                                        "currency": { "type": "string" },
                                        "payee": { "type": "object" }
                                    }
                                }
                            ]
                        },
                        "payer": {
                            "type": "object",
                            "description": "Optional payer party",
                            "properties": {
                                "@id": { "type": "string" }
                            }
                        },
                        "payee": {
                            "type": "object",
                            "description": "Payee party",
                            "properties": {
                                "@id": { "type": "string" }
                            }
                        },
                        "agent_id": { "type": "string", "description": "Optional agent ID" }
                    },
                    "required": ["transaction_id", "invoice", "payee"]
                },
                "Connect": {
                    "description": "TAP Connect message (TAIP-2) - Establishes a connection between parties",
                    "message_type": "https://tap.rsvp/schema/1.0#Connect",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "Unique transaction identifier" },
                        "agent": {
                            "type": "object",
                            "description": "Agent requesting connection",
                            "properties": {
                                "@id": { "type": "string", "description": "Agent DID" },
                                "name": { "type": "string", "description": "Optional agent name" },
                                "type": { "type": "string", "description": "Optional agent type" },
                                "serviceUrl": { "type": "string", "description": "Optional service URL" }
                            }
                        },
                        "principal": {
                            "type": "object",
                            "description": "Principal party for the connection",
                            "properties": {
                                "@id": { "type": "string", "description": "Principal DID" }
                            }
                        },
                        "constraints": {
                            "type": "object",
                            "description": "Connection constraints",
                            "properties": {
                                "purposes": { "type": "array", "items": { "type": "string" } },
                                "categoryPurposes": { "type": "array", "items": { "type": "string" } },
                                "limits": {
                                    "type": "object",
                                    "properties": {
                                        "per_transaction": { "type": "string" },
                                        "daily": { "type": "string" },
                                        "currency": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "required": ["transaction_id", "constraints"]
                },
                "AuthorizationRequired": {
                    "description": "TAP AuthorizationRequired message (TAIP-2) - Indicates authorization is needed",
                    "message_type": "https://tap.rsvp/schema/1.0#AuthorizationRequired",
                    "properties": {
                        "authorization_url": { "type": "string", "description": "URL where authorization can be completed" },
                        "agent_id": { "type": "string", "description": "Optional agent ID" },
                        "expires": { "type": "string", "description": "Optional expiry date/time" }
                    },
                    "required": ["authorization_url"]
                },
                "ConfirmRelationship": {
                    "description": "TAP ConfirmRelationship message (TAIP-14) - Confirms a relationship between parties",
                    "message_type": "https://tap.rsvp/schema/1.0#ConfirmRelationship",
                    "properties": {
                        "transfer_id": { "type": "string", "description": "Transaction ID (maps to thid)" },
                        "@id": { "type": "string", "description": "Agent ID" },
                        "for": { "type": "string", "description": "Entity this relationship is for" },
                        "role": { "type": "string", "description": "Optional role in relationship" }
                    },
                    "required": ["@id", "for"]
                },
                "AddAgents": {
                    "description": "TAP AddAgents message (TAIP-5) - Adds agents to a transaction",
                    "message_type": "https://tap.rsvp/schema/1.0#AddAgents",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "ID of transaction to add agents to" },
                        "agents": {
                            "type": "array",
                            "description": "List of agents to add",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "@id": { "type": "string", "description": "Agent DID" },
                                    "role": { "type": "string", "description": "Agent role" },
                                    "for": { "type": "string", "description": "DID of party agent represents" }
                                }
                            }
                        }
                    },
                    "required": ["transaction_id", "agents"]
                },
                "RemoveAgent": {
                    "description": "TAP RemoveAgent message (TAIP-5) - Removes an agent from a transaction",
                    "message_type": "https://tap.rsvp/schema/1.0#RemoveAgent",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "ID of transaction to remove agent from" },
                        "agent": { "type": "string", "description": "DID of agent to remove" }
                    },
                    "required": ["transaction_id", "agent"]
                },
                "ReplaceAgent": {
                    "description": "TAP ReplaceAgent message (TAIP-5) - Replaces an agent in a transaction",
                    "message_type": "https://tap.rsvp/schema/1.0#ReplaceAgent",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "ID of transaction to replace agent in" },
                        "original": { "type": "string", "description": "DID of agent to replace" },
                        "replacement": {
                            "type": "object",
                            "description": "New agent details",
                            "properties": {
                                "@id": { "type": "string", "description": "New agent DID" },
                                "role": { "type": "string", "description": "Agent role" },
                                "for": { "type": "string", "description": "DID of party agent represents" }
                            }
                        }
                    },
                    "required": ["transaction_id", "original", "replacement"]
                },
                "UpdateParty": {
                    "description": "TAP UpdateParty message (TAIP-4) - Updates party information",
                    "message_type": "https://tap.rsvp/schema/1.0#UpdateParty",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "ID of transaction to update party in" },
                        "party": {
                            "type": "object",
                            "description": "Updated party information",
                            "properties": {
                                "@id": { "type": "string", "description": "Party DID" }
                            }
                        },
                        "role": { "type": "string", "description": "Party role (originator, beneficiary, etc.)" }
                    },
                    "required": ["transaction_id", "party", "role"]
                },
                "UpdatePolicies": {
                    "description": "TAP UpdatePolicies message (TAIP-7) - Updates agent policies",
                    "message_type": "https://tap.rsvp/schema/1.0#UpdatePolicies",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "ID of transaction to update policies for" },
                        "policies": {
                            "type": "array",
                            "description": "List of policies to update",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "@type": { "type": "string", "description": "Policy type" }
                                }
                            }
                        }
                    },
                    "required": ["transaction_id", "policies"]
                },
                "Presentation": {
                    "description": "TAP Presentation message (TAIP-6) - Presents verifiable data",
                    "message_type": "https://tap.rsvp/schema/1.0#Presentation",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "Optional transaction ID" },
                        "attachments": {
                            "type": "array",
                            "description": "List of attachments containing presented data",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": { "type": "string" },
                                    "media_type": { "type": "string" },
                                    "format": { "type": "string" },
                                    "data": { "type": "object" }
                                }
                            }
                        }
                    },
                    "required": ["attachments"]
                },
                "DIDCommPresentation": {
                    "description": "DIDComm Presentation message - Presents proof data",
                    "message_type": "https://didcomm.org/present-proof/3.0/presentation",
                    "properties": {
                        "transaction_id": { "type": "string", "description": "Optional transaction ID" },
                        "formats": {
                            "type": "array",
                            "description": "List of attachment formats",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "attach_id": { "type": "string" },
                                    "format": { "type": "string" }
                                }
                            }
                        }
                    }
                },
                "TrustPing": {
                    "description": "DIDComm Trust Ping message - Tests connectivity",
                    "message_type": "https://didcomm.org/trust-ping/2.0/ping",
                    "properties": {
                        "response_requested": { "type": "boolean", "description": "Whether a response is requested" },
                        "comment": { "type": "string", "description": "Optional comment" }
                    }
                },
                "TrustPingResponse": {
                    "description": "DIDComm Trust Ping Response - Responds to trust ping",
                    "message_type": "https://didcomm.org/trust-ping/2.0/ping-response",
                    "properties": {
                        "comment": { "type": "string", "description": "Optional comment" }
                    }
                },
                "BasicMessage": {
                    "description": "DIDComm Basic Message - Simple text message",
                    "message_type": "https://didcomm.org/basicmessage/2.0/message",
                    "properties": {
                        "content": { "type": "string", "description": "Message content" },
                        "locale": { "type": "string", "description": "Optional locale (e.g., en, fr)" },
                        "sent_time": { "type": "string", "description": "Optional ISO 8601 timestamp" }
                    },
                    "required": ["content"]
                },
                "OutOfBand": {
                    "description": "TAP Out of Band invitation",
                    "message_type": "https://tap.rsvp/schema/1.0#OutOfBand",
                    "properties": {
                        "goal_code": { "type": "string", "description": "Goal code for invitation" },
                        "goal": { "type": "string", "description": "Human-readable goal" },
                        "service": { "type": "string", "description": "DID or endpoint URL" },
                        "accept": { "type": "array", "items": { "type": "string" }, "description": "Optional accepted media types" },
                        "handshake_protocols": { "type": "array", "items": { "type": "string" }, "description": "Optional handshake protocols" }
                    },
                    "required": ["goal_code", "goal", "service"]
                },
                "Error": {
                    "description": "TAP Error message - Reports an error",
                    "message_type": "https://tap.rsvp/schema/1.0#Error",
                    "properties": {
                        "error_code": { "type": "string", "description": "Error code" },
                        "error_description": { "type": "string", "description": "Human-readable error description" },
                        "error_details": { "type": "object", "description": "Optional additional error details" }
                    },
                    "required": ["error_code", "error_description"]
                }
            }
        })
    }

    /// Read a specific schema by message type
    async fn read_specific_schema(&self, message_type: &str) -> Result<Vec<ResourceContent>> {
        let all_schemas = self.get_all_schemas();

        // Look for the specific schema
        if let Some(schema) = all_schemas["schemas"].get(message_type) {
            let content = json!({
                "message_type": message_type,
                "schema": schema,
                "version": all_schemas["version"],
                "description": all_schemas["description"]
            });

            Ok(vec![ResourceContent {
                uri: format!("tap://schemas/{}", message_type),
                mime_type: Some("application/json".to_string()),
                text: Some(serde_json::to_string_pretty(&content)?),
                blob: None,
            }])
        } else {
            // Also check by message_type URL
            for (name, schema_def) in all_schemas["schemas"]
                .as_object()
                .unwrap_or(&serde_json::Map::new())
            {
                if let Some(schema_message_type) = schema_def.get("message_type") {
                    if schema_message_type.as_str() == Some(message_type)
                        || schema_message_type
                            .as_str()
                            .map(|s| s.contains(message_type))
                            .unwrap_or(false)
                    {
                        let content = json!({
                            "message_type": name,
                            "schema": schema_def,
                            "version": all_schemas["version"],
                            "description": all_schemas["description"]
                        });

                        return Ok(vec![ResourceContent {
                            uri: format!("tap://schemas/{}", message_type),
                            mime_type: Some("application/json".to_string()),
                            text: Some(serde_json::to_string_pretty(&content)?),
                            blob: None,
                        }]);
                    }
                }
            }

            Err(Error::resource_not_found(format!(
                "Schema not found for message type: {}",
                message_type
            )))
        }
    }
}
