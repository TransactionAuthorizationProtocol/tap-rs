# External Decision Executable for tap-http

## Overview

Add support for an external long-running executable to `tap-http` that receives TAP events and decisions via stdin and takes actions via stdout using the existing MCP tool interface. Decisions are durably persisted in a `decision_log` SQLite table (per-agent) so the external process can go down, restart, and catch up on missed decisions.

## Motivation

The existing `DecisionHandler` trait and `DecisionMode` in `tap-node` allow programmatic decision-making, but only within the same Rust process. Real-world deployments need external systems (compliance engines, agentic AI loops, human-in-the-loop UIs) to make authorization and settlement decisions. An external executable communicating via stdin/stdout provides:

- **Language agnostic**: Any language can implement the protocol
- **Process isolation**: Crashes in the decision logic don't take down tap-http
- **Familiar pattern**: Same stdin/stdout framing as MCP servers
- **Durable catch-up**: SQLite-backed decision queue survives process restarts

## Architecture

```
                          ┌─────────────────────────┐
                          │    External Executable   │
                          │  (compliance engine, AI  │
                          │   agent, rules engine)   │
                          └──────┬──────────▲────────┘
                           stdin │          │ stdout
                     (JSON-RPC         (JSON-RPC
                      notifications     tool calls:
                      + decision        authorize,
                      requests)         reject, query)
                          ┌──────▼──────────┴────────┐
                          │  ExternalDecisionManager  │
                          │  (new module in tap-http)  │
                          │                           │
                          │  - Spawns/restarts child  │
                          │  - Writes to decision_log │
                          │  - Replays on reconnect   │
                          │  - Routes tool calls to   │
                          │    ToolRegistry            │
                          └──────────┬────────────────┘
                                     │ implements
                          ┌──────────▼────────────────┐
                          │  DecisionHandler trait     │
                          │  + EventSubscriber trait   │
                          └──────────┬────────────────┘
                                     │
                     ┌───────────────┼───────────────┐
                     │               │               │
              ┌──────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐
              │ decision_log│ │  TapNode    │ │ToolRegistry │
              │ (SQLite)    │ │  FSM/Events │ │ (from       │
              │             │ │             │ │  tap-mcp)   │
              └─────────────┘ └─────────────┘ └─────────────┘
```

## Protocol

Uses JSON-RPC 2.0 over newline-delimited JSON on stdin/stdout, matching the MCP framing.

### tap-http → External Process (stdin)

#### Notifications (fire-and-forget, no response expected)

**Event notification** (when subscribed to "all"):
```json
{"jsonrpc":"2.0","method":"tap/event","params":{"event_type":"message_received","agent_did":"did:key:z...","data":{"message_id":"...","message_type":"...","from":"...","to":"...","body":{}},"timestamp":"2026-02-21T12:00:00Z"}}
```

Supported event types:
- `message_received` — incoming plain message processed
- `message_sent` — outgoing message sent
- `transaction_state_changed` — FSM state transition
- `customer_updated` — customer data extracted

#### Requests (response expected)

**Decision request**:
```json
{"jsonrpc":"2.0","id":1,"method":"tap/decision","params":{"decision_id":42,"transaction_id":"txn-123","agent_did":"did:key:z...","decision_type":"authorization_required","context":{"transaction_state":"Received","pending_agents":["did:key:z..."],"transaction":{"type":"transfer","asset":"eip155:1/slip44:60","amount":"100","originator":"did:key:z1...","beneficiary":"did:key:z2..."}},"created_at":"2026-02-21T12:00:00Z"}}
```

Decision types:
- `authorization_required` — new transaction needs approval
- `policy_satisfaction_required` — policies must be fulfilled
- `settlement_required` — all agents authorized, ready to settle

**Decision response** (from external process):
```json
{"jsonrpc":"2.0","id":1,"result":{"action":"authorize","detail":{"settlement_address":"eip155:1:0x..."}}}
```

Valid actions per decision type:
- `authorization_required`: `authorize`, `reject`, `update_policies`, `defer`
- `policy_satisfaction_required`: `present`, `reject`, `cancel`, `defer`
- `settlement_required`: `settle`, `cancel`, `defer`

The `defer` action means "I've seen it, don't send it again, I'll act on it later via a tool call." This marks the decision as `delivered` rather than `resolved`.

### External Process → tap-http (stdout)

The external process can call any MCP tool from the existing `ToolRegistry` (all 32+ tools from tap-mcp):

```json
{"jsonrpc":"2.0","id":100,"method":"tools/call","params":{"name":"tap_authorize","arguments":{"agent_did":"did:key:z...","transaction_id":"txn-123","settlement_address":"eip155:1:0x..."}}}
```

Response from tap-http:
```json
{"jsonrpc":"2.0","id":100,"result":{"content":[{"type":"text","text":"{\"transaction_id\":\"txn-123\",\"status\":\"authorized\"}"}],"isError":false}}
```

Additional methods available:
- `tools/list` — list all available tools (same as MCP)
- `tools/call` — call any tool (same as MCP)
- `tap/list_pending_decisions` — query unresolved decisions from `decision_log`
- `tap/resolve_decision` — explicitly mark a decision as resolved with an action

### Initialization Handshake

When the external process starts, tap-http sends an `initialize` notification:
```json
{"jsonrpc":"2.0","method":"tap/initialize","params":{"version":"0.1.0","agent_dids":["did:key:z..."],"subscribe_mode":"decisions","capabilities":{"tools":true,"decisions":true}}}
```

The external process responds with readiness (optional, tap-http proceeds after a timeout if no response):
```json
{"jsonrpc":"2.0","method":"tap/ready","params":{"version":"1.0.0","name":"my-compliance-engine"}}
```

After initialization, tap-http replays all unresolved decisions from `decision_log`.

## Decision Log (SQLite)

New migration `008_create_decision_log.sql` in `tap-node/migrations/`:

```sql
CREATE TABLE IF NOT EXISTS decision_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_id TEXT NOT NULL,
    agent_did TEXT NOT NULL,
    decision_type TEXT NOT NULL CHECK (decision_type IN (
        'authorization_required',
        'policy_satisfaction_required',
        'settlement_required'
    )),
    context_json JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending',
        'delivered',
        'resolved',
        'expired'
    )),
    resolution TEXT,
    resolution_detail JSONB,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    delivered_at TEXT,
    resolved_at TEXT
);

CREATE INDEX idx_decision_log_transaction_id ON decision_log(transaction_id);
CREATE INDEX idx_decision_log_agent_did ON decision_log(agent_did);
CREATE INDEX idx_decision_log_status ON decision_log(status);
CREATE INDEX idx_decision_log_decision_type ON decision_log(decision_type);
CREATE INDEX idx_decision_log_created_at ON decision_log(created_at);
CREATE INDEX idx_decision_log_status_created ON decision_log(status, created_at);
```

Lives in the existing per-agent SQLite database alongside transactions, messages, deliveries, etc.

### Status Transitions

```
pending ──► delivered ──► resolved
   │            │
   │            └──► expired  (transaction reached terminal state)
   │
   └──► expired  (transaction reached terminal state before delivery)
```

- `pending`: Written to DB when FSM produces a decision. Not yet sent to external process (process may be down).
- `delivered`: Sent to external process via stdin. Awaiting action. Also set when process responds with `defer`.
- `resolved`: External process took action (authorize, reject, settle, etc.). `resolution` and `resolution_detail` populated.
- `expired`: Transaction moved to a terminal state (Rejected, Cancelled, Reverted) before this decision was resolved. No action needed.

### Expiration Logic

When a `TransactionStateChanged` event moves a transaction to a terminal state, all `pending` or `delivered` decisions for that `transaction_id` are marked `expired`. This prevents the external process from acting on stale decisions after restart.

## Process Lifecycle

### Startup
1. tap-http spawns the external executable as a child process
2. stdin/stdout pipes connected
3. stderr forwarded to tap-http's log output
4. `tap/initialize` sent on stdin
5. Unresolved decisions replayed from `decision_log`

### Health Monitoring
- tap-http monitors the child process (poll process status)
- If stdout returns EOF or the process exits, mark it as down
- Decisions continue to accumulate in `decision_log` with status=`pending`

### Restart with Backoff
- On process exit, restart after backoff: 1s, 2s, 4s, 8s, 16s, 30s (capped)
- Reset backoff counter after 60s of continuous uptime
- Log each restart attempt

### Graceful Shutdown
- On tap-http shutdown (SIGTERM/Ctrl-C), send EOF on stdin
- Wait up to 5 seconds for process to exit
- SIGTERM if still running, SIGKILL after another 5 seconds

## Configuration

### CLI Flags

| Flag | Env Var | Default | Description |
|------|---------|---------|-------------|
| `--decision-exec <PATH>` | `TAP_DECISION_EXEC` | none | Path to the external executable |
| `--decision-exec-args <ARGS>` | `TAP_DECISION_EXEC_ARGS` | none | Arguments for the executable (comma-separated) |
| `--decision-subscribe <MODE>` | `TAP_DECISION_SUBSCRIBE` | `decisions` | What to forward: `decisions` (only decision points) or `all` (all events + decisions) |

When `--decision-exec` is provided:
- `NodeConfig.decision_mode` is set to `DecisionMode::Custom(...)` with the external handler
- `auto_act` is set to `false` on the `StandardTransactionProcessor` (the external process decides)

When `--decision-exec` is NOT provided:
- Existing behavior preserved (`DecisionMode::AutoApprove` with `auto_act = true`)

## Implementation Details

### New Code in tap-node

1. **Migration** `008_create_decision_log.sql` — the table definition above
2. **Storage methods** in `db.rs`:
   - `insert_decision(transaction_id, agent_did, decision_type, context_json) -> Result<i64>`
   - `update_decision_status(id, status, resolution?, resolution_detail?) -> Result<()>`
   - `list_pending_decisions(agent_did?, since_id?, limit?) -> Result<Vec<DecisionLogEntry>>`
   - `expire_decisions_for_transaction(transaction_id) -> Result<u64>`
3. **Model** `DecisionLogEntry` in `models.rs`
4. **Event handler** that listens for `TransactionStateChanged` to terminal states and expires pending decisions

### New Code in tap-http

1. **Module** `external_decision.rs`:
   - `ExternalDecisionManager` struct — owns the child process, ToolRegistry, and Storage
   - Implements `DecisionHandler` — writes to `decision_log`, sends over stdin
   - Implements `EventSubscriber` — forwards events when in "all" mode
   - Spawns async tasks for: reading stdout (tool calls), monitoring process health, replaying decisions on reconnect
   - Uses `ToolRegistry` from tap-mcp to execute tool calls from the external process

2. **Integration in main.rs**:
   - Parse new CLI flags
   - When `--decision-exec` is set, create `ExternalDecisionManager`
   - Pass its `DecisionHandler` to `NodeConfig.decision_mode`
   - Subscribe it to the event bus
   - Spawn the child process after server starts
   - Graceful shutdown on SIGTERM

### Reuse from tap-mcp

The `ToolRegistry` and all tool implementations are reused directly. The external process gets the exact same tool interface that an MCP client would get. This means:
- No new tool code to write for the action side
- Any MCP-compatible client could be used as the external executable
- Future tools added to tap-mcp automatically become available

To enable this reuse, `TapIntegration` and `ToolRegistry` (and the `mcp::protocol` types) need to be importable from tap-mcp as a library dependency of tap-http, or extracted to a shared crate. The simplest path is adding tap-mcp as a dependency of tap-http and using its public API.

### New MCP Tools

Two new tools added to the `ToolRegistry`:

1. **`tap_list_pending_decisions`**
   - Input: `{ agent_did: string, status?: string, since_id?: number, limit?: number }`
   - Output: `{ decisions: DecisionLogEntry[], total: number }`
   - Allows the external process to query outstanding decisions

2. **`tap_resolve_decision`**
   - Input: `{ agent_did: string, decision_id: number, action: string, detail?: object }`
   - Output: `{ decision_id: number, status: "resolved", resolved_at: string }`
   - Marks a decision as resolved and executes the corresponding action
   - The `action` triggers the appropriate TapNode operation (authorize, reject, settle, etc.)

## Catch-up Scenarios

### Process restarts cleanly
1. Process exits → tap-http detects EOF on stdout
2. Decisions accumulate in `decision_log` with status=`pending`
3. After backoff, tap-http restarts the process
4. `tap/initialize` sent, then all `pending`/`delivered` decisions replayed in order
5. External process responds to each or uses `tap_list_pending_decisions` to pull

### Process is slow
1. Decisions sent but no response within timeout (configurable, default 60s)
2. Decision stays `delivered` — no automatic expiry for slow responses
3. External process can respond late, or call `tap_resolve_decision` whenever ready

### Transaction resolves while process is down
1. Another agent rejects the transaction while external process is down
2. `TransactionStateChanged` triggers expiration of pending decisions
3. When process restarts, those decisions are `expired` and not replayed
4. Process only sees actionable decisions

### External process wants to catch up manually
1. Process starts, receives `tap/initialize`
2. Instead of waiting for replay, calls `tap_list_pending_decisions`
3. Gets all unresolved decisions with full context
4. Acts on them via `tap_resolve_decision` or direct tool calls

## Non-Goals

- No WebSocket or HTTP callback transport (stdin/stdout only in v1)
- No multi-process support (single external executable)
- No built-in authentication between tap-http and the external process (process isolation via OS)
- No custom protocol — reuse JSON-RPC 2.0 and MCP tool format
- No modifications to the FSM itself — uses existing `DecisionHandler` trait

## Testing Strategy

- Unit tests for `decision_log` storage methods (insert, update, list, expire)
- Unit tests for JSON-RPC serialization/deserialization of decision protocol messages
- Integration tests using a mock external executable (simple shell script or Rust binary that auto-approves)
- Integration tests for catch-up: start process, send decisions, kill process, accumulate decisions, restart, verify replay
- Integration tests for expiration: send decision, move transaction to terminal state, verify decision expired
