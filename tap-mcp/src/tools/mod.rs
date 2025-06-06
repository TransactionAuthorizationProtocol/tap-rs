//! TAP MCP tools implementation

mod agent_tools;
mod communication_tools;
mod customer_tools;
mod delivery_tools;
mod received_tools;
mod schema;
mod transaction_tools;

use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool, ToolContent};
use crate::tap_integration::TapIntegration;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error};

pub use agent_tools::*;
pub use communication_tools::*;
pub use customer_tools::*;
pub use delivery_tools::*;
pub use received_tools::*;
pub use transaction_tools::*;

/// Default limit for pagination
pub fn default_limit() -> u32 {
    50
}

/// Registry for all available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ToolHandler>>,
}

/// Trait for handling tool calls
#[async_trait::async_trait]
pub trait ToolHandler: Send + Sync {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult>;
    fn get_definition(&self) -> Tool;
}

impl ToolRegistry {
    /// Create a new tool registry with all TAP tools
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        let mut tools: HashMap<String, Box<dyn ToolHandler>> = HashMap::new();

        // Agent management tools
        tools.insert(
            "tap_create_agent".to_string(),
            Box::new(CreateAgentTool::new(tap_integration.clone())),
        );
        tools.insert(
            "tap_list_agents".to_string(),
            Box::new(ListAgentsTool::new(tap_integration.clone())),
        );

        // Transaction creation tools
        tools.insert(
            "tap_create_transfer".to_string(),
            Box::new(CreateTransferTool::new(tap_integration.clone())),
        );

        // Transaction action tools
        tools.insert(
            "tap_authorize".to_string(),
            Box::new(AuthorizeTool::new(tap_integration.clone())),
        );
        tools.insert(
            "tap_reject".to_string(),
            Box::new(RejectTool::new(tap_integration.clone())),
        );
        tools.insert(
            "tap_cancel".to_string(),
            Box::new(CancelTool::new(tap_integration.clone())),
        );
        tools.insert(
            "tap_settle".to_string(),
            Box::new(SettleTool::new(tap_integration.clone())),
        );

        // Transaction management tools
        tools.insert(
            "tap_list_transactions".to_string(),
            Box::new(ListTransactionsTool::new(tap_integration.clone())),
        );

        // Communication tools
        tools.insert(
            "tap_trust_ping".to_string(),
            Box::new(TrustPingTool::new(tap_integration.clone())),
        );
        tools.insert(
            "tap_basic_message".to_string(),
            Box::new(BasicMessageTool::new(tap_integration.clone())),
        );

        // Delivery tools
        tools.insert(
            "tap_list_deliveries_by_recipient".to_string(),
            Box::new(ListDeliveriesByRecipientTool::new(tap_integration.clone())),
        );
        tools.insert(
            "tap_list_deliveries_by_message".to_string(),
            Box::new(ListDeliveriesByMessageTool::new(tap_integration.clone())),
        );
        tools.insert(
            "tap_list_deliveries_by_thread".to_string(),
            Box::new(ListDeliveriesByThreadTool::new(tap_integration.clone())),
        );

        // Customer and connection tools
        tools.insert(
            "tap_list_customers".to_string(),
            Box::new(ListCustomersTool::new(tap_integration.clone())),
        );
        tools.insert(
            "tap_list_connections".to_string(),
            Box::new(ListConnectionsTool::new(tap_integration.clone())),
        );

        // Received message tools
        tools.insert(
            "tap_list_received".to_string(),
            Box::new(ListReceivedTool::new(tap_integration.clone())),
        );
        tools.insert(
            "tap_get_pending_received".to_string(),
            Box::new(GetPendingReceivedTool::new(tap_integration.clone())),
        );
        tools.insert(
            "tap_view_raw_received".to_string(),
            Box::new(ViewRawReceivedTool::new(tap_integration)),
        );

        debug!("Initialized tool registry with {} tools", tools.len());

        Self { tools }
    }

    /// List all available tools
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools
            .values()
            .map(|handler| handler.get_definition())
            .collect()
    }

    /// Call a tool by name
    pub async fn call_tool(&self, name: &str, arguments: Option<Value>) -> Result<CallToolResult> {
        match self.tools.get(name) {
            Some(handler) => {
                debug!("Calling tool: {}", name);
                handler.handle(arguments).await
            }
            None => {
                error!("Tool not found: {}", name);
                Err(Error::tool_execution(format!("Tool not found: {}", name)))
            }
        }
    }
}

/// Helper function to create success text response
pub fn success_text_response(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![ToolContent::Text { text }],
        is_error: Some(false),
    }
}

/// Helper function to create error text response
pub fn error_text_response(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![ToolContent::Text { text }],
        is_error: Some(true),
    }
}
