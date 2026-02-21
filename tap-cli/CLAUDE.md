# tap-cli Crate

Command-line interface for TAP Agent operations, providing a terminal tool to manage agents, create transactions, and interact with the TAP ecosystem.

## Purpose

The `tap-cli` crate provides:
- CLI binary (`tap-cli`) for TAP protocol operations
- Agent creation and key management via the terminal
- Transaction creation for all TAP message types (Transfer, Payment, Connect, Escrow)
- Transaction lifecycle actions (authorize, reject, cancel, settle, revert)
- Customer management with IVMS101 Travel Rule data generation
- Message delivery tracking and received message inspection
- DID generation and resolution

## Key Components

- `src/main.rs` — Entry point, CLI argument parsing, agent resolution
- `src/commands/` — Per-command handlers
  - `agent.rs` — `agent create`, `agent list`
  - `transaction.rs` — `transaction transfer/payment/connect/escrow/capture/list`
  - `transaction_actions.rs` — `action authorize/reject/cancel/settle/revert`
  - `communication.rs` — `comm ping`, `comm message`
  - `customer.rs` — `customer list/create/details/update/ivms101`
  - `delivery.rs` — `delivery list`
  - `received.rs` — `received list/pending/view`
  - `did.rs` — `did generate/lookup/keys`
- `src/tap_integration.rs` — Shared setup: TapNode + per-agent storage
- `src/output.rs` — JSON/text output formatting
- `src/error.rs` — CLI error types

## Build Commands

```bash
# Build the crate
cargo build -p tap-cli

# Run tests
cargo test -p tap-cli

# Run specific test
cargo test -p tap-cli test_name

# Run the CLI binary
cargo run --bin tap-cli -- --help

# Build release binary
cargo build -p tap-cli --release

# Install locally
cargo install --path tap-cli
```

## Adding a New Command

1. Add the subcommand enum variant in `src/main.rs` under `Commands`.
2. Create (or extend) `src/commands/<module>.rs` with:
   - A `#[derive(Subcommand)]` enum for subcommands
   - A `handle(cmd, format, agent_did, tap_integration)` async function
   - Serializable response structs using `serde::Serialize`
3. Wire the handler in `main.rs` inside the `match cli.command { ... }` block.
4. Export the module in `src/commands/mod.rs`.

### Command Handler Pattern

```rust
use crate::error::Result;
use crate::output::{print_success, OutputFormat};
use crate::tap_integration::TapIntegration;
use clap::Subcommand;
use serde::Serialize;

#[derive(Subcommand, Debug)]
pub enum MyCommands {
    /// Do something
    DoSomething {
        #[arg(long)]
        param: String,
    },
}

#[derive(Debug, Serialize)]
struct MyResponse {
    result: String,
}

pub async fn handle(
    cmd: &MyCommands,
    format: OutputFormat,
    agent_did: &str,
    tap_integration: &TapIntegration,
) -> Result<()> {
    match cmd {
        MyCommands::DoSomething { param } => {
            let response = MyResponse {
                result: format!("did something with {}", param),
            };
            print_success(format, &response);
            Ok(())
        }
    }
}
```

## Output Formatting

Use `print_success(format, &value)` where `value: impl Serialize`. For errors, return `Err(...)` and the top-level handler in `main.rs` calls `output::print_error`.

Available output formats (controlled by `--format`):
- `json` (default) — machine-readable, suitable for scripting with `jq`
- `text` — human-readable key=value format

## Error Handling

Use the error constructors in `src/error.rs`:
- `Error::invalid_parameter(msg)` — bad user input
- `Error::command_failed(msg)` — execution error
- `Error::configuration(msg)` — setup/config error

## TapIntegration

`TapIntegration` wraps a `TapNode` and provides:
- `tap_integration.node()` — access the underlying `TapNode`
- `tap_integration.list_agents()` — list registered agents
- `tap_integration.storage_for_agent(did)` — get per-agent storage

It is constructed in `main.rs` for all commands except `did` (which needs no node).

## DID Commands

DID commands do not need a `TapIntegration` and are handled before the node is initialized. They operate directly on `KeyStorage` (stored at `~/.tap/keys.json` by default).

## Development Guidelines

### CLI UX
- All output goes to stdout (via `print_success`)
- Debug/log output goes to stderr (via `tracing`)
- Exit code 0 on success, 1 on error
- Always return structured JSON unless `--format text` is specified

### Testing
- Use `tempfile::TempDir` for isolated storage in tests
- Set `TAP_HOME` to a temp directory to avoid touching `~/.tap`
- Use `#[serial]` for tests that share global state (env vars)
- Test both the happy path and error conditions

### Adding Storage Queries
Use `tap_integration.storage_for_agent(did)` to get an `AgentStorage` handle, then call methods on it (e.g., `list_messages`, `get_customer`, `list_received`). See `tap-node/src/storage/` for available operations.

## Testing

```bash
# Run all CLI tests
cargo test -p tap-cli

# Run with output visible
cargo test -p tap-cli -- --nocapture

# Run a specific test
cargo test -p tap-cli test_agent_create

# Clean up test databases if tests fail with DB errors
rm -rf /tmp/test-tap
```
