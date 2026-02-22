# tap-cli

Command-line interface for TAP Agent operations. `tap-cli` enables you to manage agents, create and monitor transactions, perform Travel Rule compliance operations, and interact with the TAP ecosystem from the terminal.

## Installation

```bash
# Install from the repository
cargo install --path tap-cli

# Or build and run directly
cargo run --bin tap-cli -- --help
```

## Quick Start

```bash
# Create your first agent (generates a new DID automatically)
tap-cli agent create

# Or generate a DID explicitly
tap-cli did generate --save

# Create a transfer
tap-cli transaction transfer \
  --asset eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7 \
  --amount 100.0 \
  --originator did:key:z6Mk... \
  --beneficiary did:key:z6Mk...
```

## Global Flags

| Flag | Env Var | Description |
|------|---------|-------------|
| `--agent-did <DID>` | `TAP_AGENT_DID` | Use a specific agent DID for operations |
| `--tap-root <PATH>` | `TAP_ROOT` or `TAP_HOME` | Custom TAP data directory (default: `~/.tap`) |
| `--format <FORMAT>` | | Output format: `json` (default) or `text` |
| `--debug` / `-d` | | Enable debug logging to stderr |

If no agent is specified and no stored keys exist, a new agent with a generated DID is created automatically.

## Commands

### `agent` — Manage Agents

```bash
# Create a new agent
tap-cli agent create
tap-cli agent create --label "my-vasp"

# List all registered agents
tap-cli agent list
```

### `did` — DID Operations

```bash
# Generate a new DID (did:key by default)
tap-cli did generate
tap-cli did generate --method key --key-type ed25519 --save --label "primary"
tap-cli did generate --method web --domain example.com --save

# Resolve a DID to its DID Document
tap-cli did lookup did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK

# Manage stored keys
tap-cli did keys                        # list all keys
tap-cli did keys list
tap-cli did keys view did:key:z6Mk...
tap-cli did keys set-default did:key:z6Mk...
tap-cli did keys delete did:key:z6Mk...
tap-cli did keys relabel did:key:z6Mk... "new-label"
```

### `transaction` — Create and List Transactions

#### `transaction transfer` — TAIP-3 Transfer (VASP-to-VASP)

```bash
tap-cli transaction transfer \
  --asset eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7 \
  --amount 100.0 \
  --originator did:key:z6MkOriginator... \
  --beneficiary did:key:z6MkBeneficiary... \
  --memo "Invoice #1234"

# With agents
tap-cli transaction transfer \
  --asset eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 \
  --amount 500.0 \
  --originator did:key:z6MkOriginator... \
  --beneficiary did:key:z6MkBeneficiary... \
  --agents '[{"@id":"did:key:z6MkAgent...","role":"SourceAgent","for":"did:key:z6MkOriginator..."}]'
```

| Flag | Required | Description |
|------|----------|-------------|
| `--asset` | Yes | CAIP-19 asset identifier |
| `--amount` | Yes | Transfer amount |
| `--originator` | Yes | Originator DID |
| `--beneficiary` | Yes | Beneficiary DID |
| `--agents` | No | Agents as JSON array |
| `--memo` | No | Optional memo |
| `--expiry` | No | ISO 8601 expiry timestamp |
| `--transaction-value` | No | Fiat equivalent as `amount:currency` (e.g., `1000.00:USD`) |

#### `transaction payment` — TAIP-14 Payment Request

```bash
# Payment with asset
tap-cli transaction payment \
  --amount 99.99 \
  --merchant did:key:z6MkMerchant... \
  --asset eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48

# Payment with fiat currency
tap-cli transaction payment \
  --amount 99.99 \
  --merchant did:key:z6MkMerchant... \
  --currency USD \
  --memo "Order #5678"

# With expiry and invoice
tap-cli transaction payment \
  --amount 99.99 \
  --merchant did:key:z6MkMerchant... \
  --asset eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 \
  --expiry "2026-12-31T23:59:59Z" \
  --invoice-url "https://merchant.example.com/invoices/5678" \
  --fallback-addresses eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
```

| Flag | Required | Description |
|------|----------|-------------|
| `--amount` | Yes | Payment amount |
| `--merchant` | Yes | Merchant DID |
| `--asset` | No* | CAIP-19 asset identifier |
| `--currency` | No* | ISO 4217 currency code (e.g., USD) |
| `--agents` | No | Agents as JSON array |
| `--memo` | No | Optional memo |
| `--expiry` | No | ISO 8601 expiry timestamp |
| `--invoice-url` | No | Invoice URL (TAIP-16) |
| `--fallback-addresses` | No | Comma-separated fallback settlement addresses (CAIP-10) |

\* One of `--asset` or `--currency` must be specified.

#### `transaction connect` — TAIP-15 Connection Request

```bash
tap-cli transaction connect \
  --recipient did:key:z6MkRecipient... \
  --for did:key:z6MkParty... \
  --role SourceAgent

# With transaction limits and constraints
tap-cli transaction connect \
  --recipient did:key:z6MkRecipient... \
  --for did:key:z6MkParty... \
  --constraints '{"max_amount":"10000","daily_limit":"50000","allowed_assets":["eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"]}' \
  --expiry "2027-01-01T00:00:00Z" \
  --agreement "https://example.com/terms"
```

| Flag | Required | Description |
|------|----------|-------------|
| `--recipient` | Yes | Recipient agent DID |
| `--for` | Yes | Party DID this connection is for |
| `--role` | No | Agent role (e.g., SourceAgent) |
| `--constraints` | No | Constraints JSON (limits, allowed_beneficiaries, allowed_settlement_addresses, allowed_assets) |
| `--expiry` | No | ISO 8601 expiry timestamp |
| `--agreement` | No | URL to terms of service or agreement |

#### `transaction escrow` — TAIP-17 Escrow Request

```bash
tap-cli transaction escrow \
  --amount 1000.0 \
  --originator did:key:z6MkOriginator... \
  --beneficiary did:key:z6MkBeneficiary... \
  --expiry "2026-12-31T23:59:59Z" \
  --asset eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7 \
  --agents '[{"@id":"did:key:z6MkEscrow...","role":"EscrowAgent","for":"did:key:z6MkOriginator..."}]' \
  --agreement "https://example.com/escrow-terms"
```

#### `transaction capture` — Release Escrowed Funds

```bash
tap-cli transaction capture --escrow-id <ESCROW_TRANSACTION_ID>

# Partial capture
tap-cli transaction capture \
  --escrow-id <ESCROW_TRANSACTION_ID> \
  --amount 500.0 \
  --settlement-address eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
```

#### `transaction exchange` — TAIP-18 Exchange Request

```bash
tap-cli transaction exchange \
  --from-assets eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 \
  --to-assets eip155:137/erc20:0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 \
  --from-amount 1000.00 \
  --requester did:key:z6MkRequester...

# With a specific provider
tap-cli transaction exchange \
  --from-assets eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 \
  --to-assets eip155:137/erc20:0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 \
  --to-amount 999.50 \
  --requester did:key:z6MkRequester... \
  --provider did:key:z6MkProvider...
```

#### `transaction quote` — TAIP-18 Quote Response

```bash
tap-cli transaction quote \
  --exchange-id <EXCHANGE_TX_ID> \
  --from-asset eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 \
  --to-asset eip155:137/erc20:0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 \
  --from-amount 1000.00 \
  --to-amount 999.50 \
  --provider did:key:z6MkProvider... \
  --expires "2026-06-30T12:00:00Z"
```

#### `transaction list` — Query Transactions

```bash
# List all transactions for the current agent
tap-cli transaction list

# Filter by type
tap-cli transaction list --type Transfer

# Filter by thread ID
tap-cli transaction list --thread-id <THREAD_ID>

# Pagination
tap-cli transaction list --limit 20 --offset 40
```

### `action` — Transaction Lifecycle Actions

#### `action authorize` — TAIP-4 Authorization

```bash
tap-cli action authorize --transaction-id <TX_ID>

# With settlement address
tap-cli action authorize \
  --transaction-id <TX_ID> \
  --settlement-address eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --expiry "2026-06-30T12:00:00Z"
```

#### `action reject` — TAIP-4 Rejection

```bash
tap-cli action reject \
  --transaction-id <TX_ID> \
  --reason "AML policy violation"
```

#### `action cancel` — TAIP-5 Cancellation

```bash
tap-cli action cancel \
  --transaction-id <TX_ID> \
  --by did:key:z6MkRequester... \
  --reason "Customer request"
```

#### `action settle` — TAIP-6 Settlement

```bash
tap-cli action settle \
  --transaction-id <TX_ID> \
  --settlement-id eip155:1:0xabcdef1234567890...

# Partial settlement
tap-cli action settle \
  --transaction-id <TX_ID> \
  --settlement-id eip155:1:0xabcdef1234567890... \
  --amount 75.0
```

#### `action revert` — TAIP-12 Revert

```bash
tap-cli action revert \
  --transaction-id <TX_ID> \
  --settlement-address eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --reason "Fraudulent transaction"
```

### `agent-mgmt` — Agent & Policy Management within Transactions

Manage agents and policies on existing transactions (TAIP-5, TAIP-7).

```bash
# Add agents to a transaction
tap-cli agent-mgmt add-agents \
  --transaction-id <TX_ID> \
  --agents '[{"@id":"did:key:z6MkNew...","role":"ComplianceOfficer","for":"did:key:z6MkParty..."}]'

# Remove an agent from a transaction
tap-cli agent-mgmt remove-agent \
  --transaction-id <TX_ID> \
  --agent-to-remove did:key:z6MkOldAgent...

# Replace an agent in a transaction
tap-cli agent-mgmt replace-agent \
  --transaction-id <TX_ID> \
  --original did:key:z6MkOldAgent... \
  --new-agent '{"@id":"did:key:z6MkNewAgent...","role":"SourceAgent","for":"did:key:z6MkParty..."}'

# Update transaction policies
tap-cli agent-mgmt update-policies \
  --transaction-id <TX_ID> \
  --policies '[{"@type":"RequireAuthorization"}]'
```

### `comm` — Communication

```bash
# Send a trust ping to verify agent connectivity
tap-cli comm ping --recipient did:key:z6MkRecipient...

# Send a basic text message
tap-cli comm message \
  --recipient did:key:z6MkRecipient... \
  --content "Hello, this is a test message"
```

### `customer` — Customer Management

```bash
# List customers
tap-cli customer list
tap-cli customer list --limit 20 --offset 0

# Create a customer with a Schema.org profile
tap-cli customer create \
  --customer-id did:key:z6MkCustomer... \
  --profile '{"@type":"Person","name":"Alice Smith","email":"alice@example.com"}'

# View customer details
tap-cli customer details --customer-id did:key:z6MkCustomer...

# Update a customer profile
tap-cli customer update \
  --customer-id did:key:z6MkCustomer... \
  --profile '{"@type":"Person","name":"Alice Smith","email":"new@example.com"}'

# Generate IVMS101 data for Travel Rule compliance
tap-cli customer ivms101 --customer-id did:key:z6MkCustomer...
```

### `decision` — Decision Log Management

Decisions are created when the TAP node reaches a decision point in the transaction lifecycle (e.g., authorization needed, settlement required). In poll mode, they accumulate in the database for external systems to act on.

```bash
# List all pending decisions
tap-cli decision list --status pending

# List all decisions (any status)
tap-cli decision list

# Paginate through decisions
tap-cli decision list --since-id 100 --limit 20

# Resolve a decision by authorizing the transaction
tap-cli decision resolve --decision-id 42 --action authorize

# Resolve with additional detail
tap-cli decision resolve --decision-id 42 --action authorize \
  --detail '{"settlement_address":"eip155:1:0xABC"}'

# Reject a decision
tap-cli decision resolve --decision-id 42 --action reject

# Defer a decision (mark as seen, will act later)
tap-cli decision resolve --decision-id 42 --action defer
```

Decision types:
- `authorization_required` — New transaction needs approval
- `policy_satisfaction_required` — Policies must be fulfilled
- `settlement_required` — All agents authorized, ready to settle

Decision statuses: `pending`, `delivered`, `resolved`, `expired`

**Note:** The `action` commands (`authorize`, `reject`, `settle`, `cancel`, `revert`) automatically resolve matching decisions when they succeed. You can use either `decision resolve` for fine-grained control or `action` commands for the common case.

### `delivery` — Message Delivery Tracking

```bash
# List deliveries for a recipient
tap-cli delivery list --recipient did:key:z6MkRecipient...

# List deliveries for a message
tap-cli delivery list --message <MESSAGE_ID>

# List deliveries for a transaction thread
tap-cli delivery list --thread <THREAD_ID>

# With pagination
tap-cli delivery list --recipient did:key:z6MkRecipient... --limit 20 --offset 0
```

### `received` — Received Message Inspection

```bash
# List all received messages
tap-cli received list

# List pending (unprocessed) messages
tap-cli received pending

# View a specific message by its numeric ID
tap-cli received view 42

# With explicit agent DID
tap-cli received list --agent-did did:key:z6Mk...
```

## Output Formats

All commands output JSON by default. Use `--format text` for a more readable format in interactive sessions.

```bash
# JSON output (default, good for scripting)
tap-cli --format json transaction list

# Text output (good for human reading)
tap-cli --format text agent list
```

JSON output can be piped through `jq` for filtering:

```bash
tap-cli transaction list | jq '.transactions[] | select(.type | contains("Transfer"))'
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `TAP_ROOT` | TAP data directory (also `TAP_HOME`) |
| `TAP_AGENT_DID` | Default agent DID to use |

## Storage

`tap-cli` stores data under `~/.tap/` by default:

```
~/.tap/
├── keys.json           # Stored DID keys
└── agents/
    └── <did-hash>/
        └── storage.db  # Per-agent SQLite database
```

Use `--tap-root` or `TAP_ROOT` to point to a different directory.

## Complete Workflow Example

```bash
# 1. Generate and save a DID
tap-cli did generate --method key --save --label "vasp-agent"

# 2. Verify it's stored
tap-cli did keys list

# 3. Initiate a transfer
RESULT=$(tap-cli transaction transfer \
  --asset eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 \
  --amount 250.00 \
  --originator did:key:z6MkOriginator... \
  --beneficiary did:key:z6MkBeneficiary...)

TX_ID=$(echo "$RESULT" | jq -r '.transaction_id')

# 4. Authorize the transaction
tap-cli action authorize \
  --transaction-id "$TX_ID" \
  --settlement-address eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb

# 5. Settle once on-chain confirmation is received
tap-cli action settle \
  --transaction-id "$TX_ID" \
  --settlement-id eip155:1:0xabcdef1234...

# 6. Review the transaction log
tap-cli transaction list | jq '.transactions[] | {id, type, direction, created_at}'
```

## Related Packages

- [tap-agent](../tap-agent/README.md) — Core agent and key management
- [tap-node](../tap-node/README.md) — TAP node orchestration
- [tap-msg](../tap-msg/README.md) — TAP message types
- [tap-http](../tap-http/README.md) — HTTP server for DIDComm transport
- [tap-mcp](../tap-mcp/README.md) — AI assistant integration via MCP

## License

MIT — see [LICENSE-MIT](../LICENSE-MIT).
