//! TAP MCP tools implementation

mod agent_tools;
mod transaction_tools;
mod schema;

use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool, ToolContent};
use crate::tap_integration::TapIntegration;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error};

pub use agent_tools::*;
pub use transaction_tools::*;

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
            "tap.create_agent".to_string(),
            Box::new(CreateAgentTool::new(tap_integration.clone())),
        );
        tools.insert(
            "tap.list_agents".to_string(),
            Box::new(ListAgentsTool::new(tap_integration.clone())),
        );

        // Transaction creation tools
        tools.insert(
            "tap.create_transfer".to_string(),
            Box::new(CreateTransferTool::new(tap_integration.clone())),
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