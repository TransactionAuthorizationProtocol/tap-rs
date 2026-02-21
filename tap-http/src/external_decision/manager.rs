//! External Decision Manager
//!
//! Manages the lifecycle of an external decision-making process. The child
//! process communicates over stdin/stdout using JSON-RPC 2.0.

use super::protocol::*;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tap_mcp::mcp::protocol::ToolContent;
use tap_mcp::tools::ToolRegistry;
use tap_node::event::{EventSubscriber, NodeEvent};
use tap_node::state_machine::fsm::{DecisionHandler, TransactionContext, TransactionState};
use tap_node::storage::{DecisionStatus, DecisionType, Storage};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};

/// Subscribe mode — what events to forward to the external process
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscribeMode {
    /// Only forward decision points
    Decisions,
    /// Forward all events + decision points
    All,
}

impl std::str::FromStr for SubscribeMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "decisions" => Ok(SubscribeMode::Decisions),
            "all" => Ok(SubscribeMode::All),
            _ => Err(format!(
                "Invalid subscribe mode: {}. Expected 'decisions' or 'all'",
                s
            )),
        }
    }
}

/// Configuration for the external decision executable
#[derive(Debug, Clone)]
pub struct ExternalDecisionConfig {
    /// Path to the executable
    pub exec_path: String,
    /// Arguments to pass to the executable
    pub exec_args: Vec<String>,
    /// What events to forward
    pub subscribe_mode: SubscribeMode,
}

/// Manages the external decision process lifecycle
pub struct ExternalDecisionManager {
    config: ExternalDecisionConfig,
    agent_dids: Vec<String>,
    tool_registry: Arc<ToolRegistry>,
    storage: Arc<Storage>,
    /// Channel for sending lines to stdin writer task
    stdin_tx: RwLock<Option<mpsc::Sender<String>>>,
    /// Whether the process is currently running
    is_running: AtomicBool,
    /// Pending RPC responses — maps request_id to a oneshot sender
    pending_responses:
        Arc<Mutex<std::collections::HashMap<i64, tokio::sync::oneshot::Sender<Value>>>>,
    /// Handle to the management task (for shutdown)
    management_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl std::fmt::Debug for ExternalDecisionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalDecisionManager")
            .field("config", &self.config)
            .field("agent_dids", &self.agent_dids)
            .field("is_running", &self.is_running.load(Ordering::Relaxed))
            .finish()
    }
}

impl ExternalDecisionManager {
    /// Create a new ExternalDecisionManager
    pub fn new(
        config: ExternalDecisionConfig,
        agent_dids: Vec<String>,
        tool_registry: Arc<ToolRegistry>,
        storage: Arc<Storage>,
    ) -> Self {
        Self {
            config,
            agent_dids,
            tool_registry,
            storage,
            stdin_tx: RwLock::new(None),
            is_running: AtomicBool::new(false),
            pending_responses: Arc::new(Mutex::new(std::collections::HashMap::new())),
            management_handle: Mutex::new(None),
        }
    }

    /// Start the external process and management tasks
    pub async fn start(self: &Arc<Self>) {
        let this = Arc::clone(self);
        let handle = tokio::spawn(async move {
            this.run_process_loop().await;
        });
        *self.management_handle.lock().await = Some(handle);
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) {
        info!("Shutting down external decision manager");
        self.is_running.store(false, Ordering::SeqCst);

        // Close stdin to signal the child
        {
            let mut tx = self.stdin_tx.write().await;
            *tx = None;
        }

        // Cancel management task
        if let Some(handle) = self.management_handle.lock().await.take() {
            handle.abort();
        }
    }

    /// Main process lifecycle loop with restart and backoff
    async fn run_process_loop(&self) {
        let mut backoff_secs = 1u64;
        let max_backoff = 30u64;

        loop {
            info!(
                "Spawning external decision process: {} {:?}",
                self.config.exec_path, self.config.exec_args
            );

            match self.spawn_and_run().await {
                Ok(()) => {
                    info!("External decision process exited normally");
                }
                Err(e) => {
                    error!("External decision process error: {}", e);
                }
            }

            // Clear running state
            self.is_running.store(false, Ordering::SeqCst);
            {
                let mut tx = self.stdin_tx.write().await;
                *tx = None;
            }

            // Backoff before restart
            info!("Restarting external decision process in {}s", backoff_secs);
            tokio::time::sleep(Duration::from_secs(backoff_secs)).await;

            // Increase backoff (cap at max)
            backoff_secs = (backoff_secs * 2).min(max_backoff);
        }
    }

    /// Spawn the process and run until it exits
    async fn spawn_and_run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut child = Command::new(&self.config.exec_path)
            .args(&self.config.exec_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Forward stderr to tap-http log output
            .spawn()?;

        let stdin = child.stdin.take().ok_or("Failed to open stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;

        // Create stdin writer channel
        let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(256);
        {
            let mut tx = self.stdin_tx.write().await;
            *tx = Some(stdin_tx);
        }
        self.is_running.store(true, Ordering::SeqCst);

        // Stdin writer task
        let stdin_handle = tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(line) = stdin_rx.recv().await {
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.write_all(b"\n").await.is_err() {
                    break;
                }
                if stdin.flush().await.is_err() {
                    break;
                }
            }
        });

        // Stdout reader task
        let tool_registry = Arc::clone(&self.tool_registry);
        let pending_responses = Arc::clone(&self.pending_responses);
        let storage = Arc::clone(&self.storage);
        let stdout_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }

                debug!("Received from external process: {}", line);

                Self::handle_stdout_message(&line, &tool_registry, &pending_responses, &storage)
                    .await;
            }
            debug!("External process stdout closed");
        });

        // Send initialization
        self.send_initialize().await;

        // Replay pending decisions
        self.replay_pending_decisions().await;

        // Wait for child to exit
        let status = child.wait().await?;
        info!("External decision process exited with status: {}", status);

        // Clean up tasks
        stdin_handle.abort();
        stdout_handle.abort();

        Ok(())
    }

    /// Send the initialization message
    async fn send_initialize(&self) {
        let params = InitializeParams {
            version: env!("CARGO_PKG_VERSION").to_string(),
            agent_dids: self.agent_dids.clone(),
            subscribe_mode: match self.config.subscribe_mode {
                SubscribeMode::Decisions => "decisions".to_string(),
                SubscribeMode::All => "all".to_string(),
            },
            capabilities: InitializeCapabilities {
                tools: true,
                decisions: true,
            },
        };

        let notif = JsonRpcNotification::new(
            "tap/initialize",
            Some(serde_json::to_value(&params).unwrap()),
        );

        self.send_line(&serde_json::to_string(&notif).unwrap())
            .await;
    }

    /// Replay all pending/delivered decisions from the decision log
    async fn replay_pending_decisions(&self) {
        // Get all pending/delivered decisions across agents
        for did in &self.agent_dids {
            // List pending
            match self
                .storage
                .list_decisions(Some(did), Some(DecisionStatus::Pending), None, 1000)
                .await
            {
                Ok(entries) => {
                    for entry in entries {
                        self.send_decision_request(&entry).await;
                    }
                }
                Err(e) => {
                    error!("Failed to list pending decisions for {}: {}", did, e);
                }
            }

            // List delivered (re-send in case external process lost them)
            match self
                .storage
                .list_decisions(Some(did), Some(DecisionStatus::Delivered), None, 1000)
                .await
            {
                Ok(entries) => {
                    for entry in entries {
                        self.send_decision_request(&entry).await;
                    }
                }
                Err(e) => {
                    error!("Failed to list delivered decisions for {}: {}", did, e);
                }
            }
        }
    }

    /// Send a decision request for a DecisionLogEntry
    async fn send_decision_request(&self, entry: &tap_node::storage::DecisionLogEntry) {
        let params = DecisionRequestParams {
            decision_id: entry.id,
            transaction_id: entry.transaction_id.clone(),
            agent_did: entry.agent_did.clone(),
            decision_type: entry.decision_type.to_string(),
            context: entry.context_json.clone(),
            created_at: entry.created_at.clone(),
        };

        let req = JsonRpcRequest::new(
            entry.id,
            "tap/decision",
            Some(serde_json::to_value(&params).unwrap()),
        );

        let line = serde_json::to_string(&req).unwrap();
        self.send_line(&line).await;

        // Mark as delivered
        if entry.status == DecisionStatus::Pending {
            if let Err(e) = self
                .storage
                .update_decision_status(entry.id, DecisionStatus::Delivered, None, None)
                .await
            {
                error!("Failed to mark decision {} as delivered: {}", entry.id, e);
            }
        }
    }

    /// Handle a line from stdout
    async fn handle_stdout_message(
        line: &str,
        tool_registry: &ToolRegistry,
        pending_responses: &Mutex<
            std::collections::HashMap<i64, tokio::sync::oneshot::Sender<Value>>,
        >,
        _storage: &Storage,
    ) {
        // Try to parse as incoming message
        match serde_json::from_str::<IncomingMessage>(line) {
            Ok(IncomingMessage::Request(req)) => {
                Self::handle_tool_call(req, tool_registry, pending_responses).await;
            }
            Ok(IncomingMessage::Notification(notif)) => {
                debug!(
                    "Received notification from external process: {}",
                    notif.method
                );
                // Handle tap/ready or other notifications
            }
            Err(_) => {
                // Try as a JSON-RPC response (from decision requests)
                if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(line) {
                    let id = resp.id.as_i64().unwrap_or(-1);
                    let mut pending = pending_responses.lock().await;
                    if let Some(sender) = pending.remove(&id) {
                        let _ = sender.send(resp.result);
                    }
                } else {
                    warn!("Unrecognized message from external process: {}", line);
                }
            }
        }
    }

    /// Handle a tool call from the external process
    async fn handle_tool_call(
        req: JsonRpcRequest,
        tool_registry: &ToolRegistry,
        _pending_responses: &Mutex<
            std::collections::HashMap<i64, tokio::sync::oneshot::Sender<Value>>,
        >,
    ) {
        match req.method.as_str() {
            "tools/call" => {
                let params = req.params.unwrap_or(json!({}));
                let tool_name = params["name"].as_str().unwrap_or("");
                let arguments = params.get("arguments").cloned();

                debug!("External process calling tool: {}", tool_name);

                match tool_registry.call_tool(tool_name, arguments).await {
                    Ok(result) => {
                        let response = JsonRpcResponse::new(
                            req.id,
                            json!({
                                "content": result.content.iter().map(|c| match c {
                                    ToolContent::Text { text } => json!({"type": "text", "text": text}),
                                    _ => json!({"type": "unknown"}),
                                }).collect::<Vec<_>>(),
                                "isError": result.is_error.unwrap_or(false),
                            }),
                        );
                        debug!("Tool call response: {:?}", response);
                    }
                    Err(e) => {
                        error!("Tool call failed: {}", e);
                    }
                }
            }
            "tools/list" => {
                let tools = tool_registry.list_tools();
                let response = JsonRpcResponse::new(req.id, json!({ "tools": tools }));
                debug!("Tools list response: {:?}", response);
            }
            _ => {
                warn!("Unknown method from external process: {}", req.method);
            }
        }
    }

    /// Send a line to stdin
    async fn send_line(&self, line: &str) {
        let tx = self.stdin_tx.read().await;
        if let Some(tx) = tx.as_ref() {
            if let Err(e) = tx.send(line.to_string()).await {
                debug!("Failed to send to stdin (process may be down): {}", e);
            }
        }
    }
}

// Implement DecisionHandler so the FSM can delegate decisions to us
#[async_trait]
impl DecisionHandler for ExternalDecisionManager {
    async fn handle_decision(
        &self,
        ctx: &TransactionContext,
        decision: &tap_node::state_machine::fsm::Decision,
    ) {
        let (decision_type, context_json) = match decision {
            tap_node::state_machine::fsm::Decision::AuthorizationRequired {
                transaction_id,
                pending_agents,
            } => (
                DecisionType::AuthorizationRequired,
                json!({
                    "transaction_state": ctx.state.to_string(),
                    "pending_agents": pending_agents,
                    "transaction_id": transaction_id,
                }),
            ),
            tap_node::state_machine::fsm::Decision::PolicySatisfactionRequired {
                transaction_id,
                requested_by,
            } => (
                DecisionType::PolicySatisfactionRequired,
                json!({
                    "transaction_state": ctx.state.to_string(),
                    "requested_by": requested_by,
                    "transaction_id": transaction_id,
                }),
            ),
            tap_node::state_machine::fsm::Decision::SettlementRequired { transaction_id } => (
                DecisionType::SettlementRequired,
                json!({
                    "transaction_state": ctx.state.to_string(),
                    "transaction_id": transaction_id,
                }),
            ),
        };

        let agent_did = self.agent_dids.first().cloned().unwrap_or_default();

        // Insert into decision log
        match self
            .storage
            .insert_decision(
                &ctx.transaction_id,
                &agent_did,
                decision_type,
                &context_json,
            )
            .await
        {
            Ok(decision_id) => {
                debug!(
                    "Inserted decision {} for transaction {}",
                    decision_id, ctx.transaction_id
                );

                // If process is running, send immediately
                if self.is_running.load(Ordering::Relaxed) {
                    let entry = self.storage.get_decision_by_id(decision_id).await;
                    if let Ok(Some(entry)) = entry {
                        self.send_decision_request(&entry).await;
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to insert decision for transaction {}: {}",
                    ctx.transaction_id, e
                );
            }
        }
    }
}

// Implement EventSubscriber so we can forward events to the external process
#[async_trait]
impl EventSubscriber for ExternalDecisionManager {
    async fn handle_event(&self, event: NodeEvent) {
        // In "decisions" mode, we only handle TransactionStateChanged for expiration
        // In "all" mode, we also forward all events

        // Always handle terminal state transitions for expiration
        if let NodeEvent::TransactionStateChanged {
            ref transaction_id,
            ref new_state,
            ..
        } = event
        {
            if let Ok(state) = new_state.parse::<TransactionState>() {
                if state.is_terminal() {
                    if let Err(e) = self
                        .storage
                        .expire_decisions_for_transaction(transaction_id)
                        .await
                    {
                        error!(
                            "Failed to expire decisions for transaction {}: {}",
                            transaction_id, e
                        );
                    }
                }
            }
        }

        // In "all" mode, forward all events
        if self.config.subscribe_mode == SubscribeMode::All
            && self.is_running.load(Ordering::Relaxed)
        {
            let (event_type, agent_did, data) = match &event {
                NodeEvent::PlainMessageReceived { message } => {
                    ("message_received", None, message.clone())
                }
                NodeEvent::PlainMessageSent { message, from, to } => (
                    "message_sent",
                    Some(from.clone()),
                    json!({"message": message, "to": to}),
                ),
                NodeEvent::TransactionStateChanged {
                    transaction_id,
                    old_state,
                    new_state,
                    agent_did,
                } => (
                    "transaction_state_changed",
                    agent_did.clone(),
                    json!({
                        "transaction_id": transaction_id,
                        "old_state": old_state,
                        "new_state": new_state,
                    }),
                ),
                NodeEvent::CustomerUpdated {
                    customer_id,
                    agent_did,
                    update_type,
                } => (
                    "customer_updated",
                    Some(agent_did.clone()),
                    json!({
                        "customer_id": customer_id,
                        "update_type": update_type,
                    }),
                ),
                NodeEvent::MessageReceived { message, source } => (
                    "message_received",
                    None,
                    json!({
                        "message_id": message.id,
                        "message_type": message.type_,
                        "from": message.from,
                        "source": source,
                    }),
                ),
                NodeEvent::MessageSent {
                    message,
                    destination,
                } => (
                    "message_sent",
                    None,
                    json!({
                        "message_id": message.id,
                        "message_type": message.type_,
                        "destination": destination,
                    }),
                ),
                _ => return, // Skip events we don't forward
            };

            let params = EventNotificationParams {
                event_type: event_type.to_string(),
                agent_did,
                data,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            let notif =
                JsonRpcNotification::new("tap/event", Some(serde_json::to_value(&params).unwrap()));

            self.send_line(&serde_json::to_string(&notif).unwrap())
                .await;
        }
    }
}
