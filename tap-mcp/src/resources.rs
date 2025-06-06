//! MCP resources for read-only data access

use crate::error::{Error, Result};
use crate::mcp::protocol::{Resource, ResourceContent};
use crate::tap_integration::TapIntegration;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;
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
                uri: "tap://schemas".to_string(),
                name: "TAP Schemas".to_string(),
                description: "JSON schemas for TAP message types".to_string(),
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
            Some("schemas") => self.read_schemas_resource(url.path()).await,
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

    /// Read schemas resource
    async fn read_schemas_resource(&self, _path: &str) -> Result<Vec<ResourceContent>> {
        let schemas = json!({
            "schemas": {
                "Transfer": {
                    "description": "TAP Transfer message (TAIP-3)",
                    "message_type": "https://tap.rsvp/schema/1.0#Transfer",
                    "properties": {
                        "transaction_id": { "type": "string" },
                        "asset": { "type": "string", "description": "CAIP-19 asset identifier" },
                        "amount": { "type": "string" },
                        "originator": {
                            "type": "object",
                            "properties": {
                                "@id": { "type": "string" }
                            }
                        },
                        "beneficiary": {
                            "type": "object",
                            "properties": {
                                "@id": { "type": "string" }
                            }
                        },
                        "agents": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "@id": { "type": "string" },
                                    "role": { "type": "string" },
                                    "for": { "type": "string" }
                                }
                            }
                        }
                    }
                },
                "Authorize": {
                    "description": "TAP Authorize message (TAIP-4)",
                    "message_type": "https://tap.rsvp/schema/1.0#Authorize",
                    "properties": {
                        "transaction_id": { "type": "string" },
                        "settlement_address": { "type": "string", "description": "CAIP-10 address" },
                        "expiry": { "type": "string", "description": "ISO 8601 timestamp" }
                    }
                },
                "Reject": {
                    "description": "TAP Reject message (TAIP-4)",
                    "message_type": "https://tap.rsvp/schema/1.0#Reject",
                    "properties": {
                        "transaction_id": { "type": "string" },
                        "reason": { "type": "string" }
                    }
                }
            }
        });

        Ok(vec![ResourceContent {
            uri: "tap://schemas".to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(serde_json::to_string_pretty(&schemas)?),
            blob: None,
        }])
    }
}
