//! MCP server implementation

use crate::error::Result;
use crate::mcp::protocol::*;
use crate::mcp::transport::StdioTransport;
use crate::resources::ResourceRegistry;
use crate::tap_integration::TapIntegration;
use crate::tools::ToolRegistry;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// MCP server for TAP functionality
pub struct McpServer {
    transport: StdioTransport,
    tool_registry: ToolRegistry,
    resource_registry: ResourceRegistry,
    initialized: bool,
}

impl McpServer {
    /// Create a new MCP server
    pub async fn new(tap_integration: TapIntegration) -> Result<Self> {
        let tap_integration = Arc::new(tap_integration);
        let tool_registry = ToolRegistry::new(tap_integration.clone());
        let resource_registry = ResourceRegistry::new(tap_integration.clone());

        Ok(Self {
            transport: StdioTransport::new(),
            tool_registry,
            resource_registry,
            initialized: false,
        })
    }

    /// Run the MCP server
    pub async fn run(mut self) -> Result<()> {
        info!("MCP server started, waiting for requests");

        loop {
            match self.transport.read_request().await {
                Ok(Some(request)) => {
                    if let Err(e) = self.handle_request(request).await {
                        error!("Error handling request: {}", e);
                    }
                }
                Ok(None) => {
                    info!("Client disconnected, shutting down");
                    break;
                }
                Err(e) => {
                    error!("Transport error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle an incoming JSON-RPC request
    async fn handle_request(&mut self, request: JsonRpcRequest) -> Result<()> {
        debug!("Handling request: {}", request.method);

        let response = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id, request.params).await,
            "initialized" => {
                // Client confirms initialization
                self.initialized = true;
                info!("Client initialization confirmed");
                return Ok(());
            }
            "tools/list" => self.handle_list_tools(request.id, request.params).await,
            "tools/call" => self.handle_call_tool(request.id, request.params).await,
            "resources/list" => self.handle_list_resources(request.id, request.params).await,
            "resources/read" => self.handle_read_resource(request.id, request.params).await,
            _ => {
                warn!("Unknown method: {}", request.method);
                JsonRpcResponse::error(request.id, JsonRpcError::method_not_found(request.method))
            }
        };

        self.transport.write_response(response).await?;
        Ok(())
    }

    /// Handle initialize request
    async fn handle_initialize(
        &mut self,
        id: Option<Value>,
        params: Option<Value>,
    ) -> JsonRpcResponse {
        let params: InitializeParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(id, JsonRpcError::invalid_params(e.to_string()));
                }
            },
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing parameters"),
                );
            }
        };

        info!(
            "Initializing with client: {} v{}",
            params.client_info.name, params.client_info.version
        );

        // Check protocol version compatibility
        if params.protocol_version != MCP_VERSION {
            warn!(
                "Protocol version mismatch: client={}, server={}",
                params.protocol_version, MCP_VERSION
            );
        }

        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities {
                logging: None,
                prompts: None,
                resources: Some(ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                experimental: None,
            },
            server_info: ServerInfo {
                name: "tap-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        match serde_json::to_value(result) {
            Ok(value) => JsonRpcResponse::success(id, value),
            Err(e) => JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string())),
        }
    }

    /// Handle list tools request
    async fn handle_list_tools(
        &self,
        id: Option<Value>,
        _params: Option<Value>,
    ) -> JsonRpcResponse {
        if !self.initialized {
            return JsonRpcResponse::error(id, JsonRpcError::invalid_request("Not initialized"));
        }

        let tools = self.tool_registry.list_tools();
        let result = ListToolsResult {
            tools,
            next_cursor: None,
        };

        match serde_json::to_value(result) {
            Ok(value) => JsonRpcResponse::success(id, value),
            Err(e) => JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string())),
        }
    }

    /// Handle call tool request
    async fn handle_call_tool(&self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        if !self.initialized {
            return JsonRpcResponse::error(id, JsonRpcError::invalid_request("Not initialized"));
        }

        let params: CallToolParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(id, JsonRpcError::invalid_params(e.to_string()));
                }
            },
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing parameters"),
                );
            }
        };

        match self
            .tool_registry
            .call_tool(&params.name, params.arguments)
            .await
        {
            Ok(result) => match serde_json::to_value(result) {
                Ok(value) => JsonRpcResponse::success(id, value),
                Err(e) => JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string())),
            },
            Err(e) => {
                error!("Tool execution failed: {}", e);
                let result = CallToolResult {
                    content: vec![ToolContent::Text {
                        text: format!("Error: {}", e),
                    }],
                    is_error: Some(true),
                };
                match serde_json::to_value(result) {
                    Ok(value) => JsonRpcResponse::success(id, value),
                    Err(e) => {
                        JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string()))
                    }
                }
            }
        }
    }

    /// Handle list resources request
    async fn handle_list_resources(
        &self,
        id: Option<Value>,
        _params: Option<Value>,
    ) -> JsonRpcResponse {
        if !self.initialized {
            return JsonRpcResponse::error(id, JsonRpcError::invalid_request("Not initialized"));
        }

        let resources = self.resource_registry.list_resources().await;
        let result = ListResourcesResult {
            resources,
            next_cursor: None,
        };

        match serde_json::to_value(result) {
            Ok(value) => JsonRpcResponse::success(id, value),
            Err(e) => JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string())),
        }
    }

    /// Handle read resource request
    async fn handle_read_resource(
        &self,
        id: Option<Value>,
        params: Option<Value>,
    ) -> JsonRpcResponse {
        if !self.initialized {
            return JsonRpcResponse::error(id, JsonRpcError::invalid_request("Not initialized"));
        }

        let params: ReadResourceParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(id, JsonRpcError::invalid_params(e.to_string()));
                }
            },
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing parameters"),
                );
            }
        };

        match self.resource_registry.read_resource(&params.uri).await {
            Ok(contents) => {
                let result = ReadResourceResult { contents };
                match serde_json::to_value(result) {
                    Ok(value) => JsonRpcResponse::success(id, value),
                    Err(e) => {
                        JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string()))
                    }
                }
            }
            Err(e) => {
                error!("Resource read failed: {}", e);
                JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string()))
            }
        }
    }

    /// Handle a request directly (for testing)
    #[allow(dead_code)]
    pub async fn handle_request_direct(
        &mut self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse> {
        debug!("Handling direct request: {}", request.method);

        let response = match request.method.as_str() {
            "initialize" => {
                let response = self.handle_initialize(request.id, request.params).await;
                // For testing, automatically mark as initialized after successful initialize
                if matches!(
                    response,
                    JsonRpcResponse {
                        result: Some(_),
                        error: None,
                        ..
                    }
                ) {
                    self.initialized = true;
                }
                response
            }
            "initialized" => {
                // Client confirms initialization
                self.initialized = true;
                info!("Client initialization confirmed");
                return Ok(JsonRpcResponse::success(request.id, serde_json::json!({})));
            }
            "tools/list" => self.handle_list_tools(request.id, request.params).await,
            "tools/call" => self.handle_call_tool(request.id, request.params).await,
            "resources/list" => self.handle_list_resources(request.id, request.params).await,
            "resources/read" => self.handle_read_resource(request.id, request.params).await,
            _ => {
                warn!("Unknown method: {}", request.method);
                JsonRpcResponse::error(request.id, JsonRpcError::method_not_found(request.method))
            }
        };

        Ok(response)
    }
}
