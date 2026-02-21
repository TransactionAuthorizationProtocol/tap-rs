# TAP-CLI: Command-Line Interface for TAP Agent Operations

## Overview

A comprehensive CLI tool that provides full TAP agent functionality, mirroring the capabilities of `tap-mcp` but exposed as traditional command-line subcommands with JSON output. The CLI wraps a local `TapNode` instance with the same `TapIntegration` layer used by `tap-mcp`, providing agent management, transaction creation, transaction lifecycle actions, customer management, and message inspection.

## Architecture

### Core Design

- **Reuses `TapIntegration`** from `tap-mcp` (extracted to shared code or duplicated with same pattern)
- **Wraps `TapNode`** with SQLite storage, agent registration, and message routing
- **Same storage layout** as `tap-mcp`: `~/.tap/keys.json` for keys, `~/.tap/{sanitized_did}/tap.db` for per-agent databases
- **JSON output by default** for machine readability, with `--format text` for human-readable output
- **Async runtime** via tokio for all TapNode operations

### Global Flags

| Flag | Env Var | Default | Description |
|------|---------|---------|-------------|
| `--agent-did <DID>` | `TAP_AGENT_DID` | Default key | Primary agent DID |
| `--tap-root <PATH>` | `TAP_ROOT` / `TAP_HOME` | `~/.tap` | Storage root |
| `--debug` | - | false | Enable debug logging |
| `--format <FORMAT>` | - | `json` | Output format: `json` or `text` |

## Command Structure

### Agent Management

```
tap-cli agent create [--label <LABEL>]
tap-cli agent list [--limit N] [--offset N]
```

### Transaction Creation

```
tap-cli transfer --asset <CAIP-19> --amount <AMOUNT> \
    --originator <DID> --beneficiary <DID> \
    [--agents <JSON>] [--memo <TEXT>]

tap-cli payment --amount <AMOUNT> --merchant <DID> \
    (--asset <CAIP-19> | --currency <ISO4217>) \
    [--agents <JSON>] [--memo <TEXT>]

tap-cli connect --recipient <DID> --for <DID> \
    [--role <ROLE>] [--constraints <JSON>]

tap-cli escrow --amount <AMOUNT> \
    (--asset <CAIP-19> | --currency <ISO4217>) \
    --originator <DID> --beneficiary <DID> \
    --expiry <ISO8601> --agents <JSON> \
    [--agreement <URL>]

tap-cli capture --escrow-id <ID> \
    [--amount <AMOUNT>] [--settlement-address <CAIP-10>]
```

### Transaction Actions

```
tap-cli authorize --transaction-id <ID> \
    [--settlement-address <CAIP-10>] [--expiry <ISO8601>]

tap-cli reject --transaction-id <ID> --reason <TEXT>

tap-cli cancel --transaction-id <ID> --by <DID> [--reason <TEXT>]

tap-cli settle --transaction-id <ID> --settlement-id <CAIP-220> \
    [--amount <AMOUNT>]

tap-cli revert --transaction-id <ID> \
    --settlement-address <CAIP-10> --reason <TEXT>
```

### Queries

```
tap-cli transaction list [--agent-did <DID>] \
    [--type <MSG_TYPE>] [--thread-id <ID>] \
    [--from <DID>] [--to <DID>] \
    [--limit N] [--offset N]

tap-cli delivery list \
    (--recipient <DID> | --message <ID> | --thread <ID>) \
    [--agent-did <DID>] [--limit N] [--offset N]

tap-cli received list [--agent-did <DID>] [--limit N] [--offset N]
tap-cli received pending [--agent-did <DID>]
tap-cli received view <ID> [--agent-did <DID>]
```

### Customer Management

```
tap-cli customer list [--agent-did <DID>] [--limit N] [--offset N]
tap-cli customer create --customer-id <DID> --profile <JSON> [--agent-did <DID>]
tap-cli customer details --customer-id <ID> [--agent-did <DID>]
tap-cli customer update --customer-id <ID> --profile <JSON> [--agent-did <DID>]
tap-cli customer ivms101 --customer-id <ID> [--agent-did <DID>]
```

### Communication

```
tap-cli ping --recipient <DID>
tap-cli message --recipient <DID> --content <TEXT>
```

### DID Operations (from existing tap-agent-cli)

```
tap-cli did generate [--method key|web] [--key-type ed25519|p256|secp256k1] \
    [--save] [--default] [--label <LABEL>]
tap-cli did lookup <DID>
tap-cli did keys [list|view|set-default|delete|relabel]
```

## Output Format

### JSON Mode (default)

All successful operations output a JSON object:
```json
{
  "status": "success",
  "data": { ... }
}
```

Errors output:
```json
{
  "status": "error",
  "error": "description"
}
```

### Text Mode (`--format text`)

Human-readable tabular/formatted output with headers and clear field labels.

## Dependencies

Same TAP ecosystem dependencies as `tap-mcp`:
- `tap-node`, `tap-agent`, `tap-msg`, `tap-caip`
- `tokio` (full), `clap` (derive), `serde`/`serde_json`
- `tracing`, `tracing-subscriber`, `chrono`, `uuid`
- `sqlx` (sqlite), `dirs`

## Non-Goals

- No interactive/REPL mode in first version
- No HTTP client mode (connecting to remote tap-http) in first version
- No TUI (terminal UI) - pure command output
