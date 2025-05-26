# PRD: Transaction Storage (Transfers & Payments)

## 1. Background

`tap-node` currently routes and processes TAP messages in-memory. To enable searchability, auditing, and eventual state reconciliation, we need persistent storage for **Transfer** and **Payment** transactions. This PRD introduces a lightweight SQLite-based storage layer using the [`rusqlite`](https://crates.io/crates/rusqlite) crate. For the **first iteration**, storage will be **append-only**: each time a `Transfer` or `Payment` message is *created* **or** *received*, a corresponding row is inserted with status `pending` and the full message JSON.

## 2. Goals

1. Persist all Transfer & Payment messages processed by `tap-node`.
2. Provide a simple API for inserting new rows; no update logic (beyond the automatic `updated_at` trigger) is required in v0.
3. Keep the storage layer self-contained and ready for future expansion (status changes, indexing, joins, etc.).
4. Zero external dependencies beyond SQLite (bundled).

## 3. Non-Goals (Out of Scope)

- Changing the business logic of message validation or authorisation.
- Complex queries or reporting APIs.
- Support for databases other than SQLite.
- Message *status* updates beyond the default `pending` value.

## 4. Requirements

| # | Requirement |
|---|-------------|
| R1 | `tap-node` **must** embed and open a SQLite database file (default: `tap-node.db`) on startup. |
| R2 | `tap-node` **must** run schema migrations automatically at startup. |
| R3 | When a `Transfer` or `Payment` TAP message is **created locally** or **received from the network**, `tap-node` **must** insert a new row in the `transactions` table. |
| R4 | Inserted rows **must** have `status = "pending"` and contain the entire original message serialised as JSON. |
| R5 | The storage layer **must** expose an async-friendly API (compatible with `tokio` tasks). |
| R6 | The solution **must** compile to WASM (no `rusqlite` usage on WASM builds; feature-gated). |

## 5. Data Model

### 5.1. General Conventions
- **Primary keys** are `INTEGER` autoincrement (`id`).
- **Timestamps** use `TEXT` in RFC-3339 (UTC) format.
- **Status** is a `TEXT` column (initially only `pending`).
- **message_json** stores the raw serialized JSON of the DIDComm `Message`.

### 5.2. `transactions` Table
| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER PRIMARY KEY | |
| type | TEXT NOT NULL | `"transfer"` or `"payment"` |
| reference_id | TEXT NOT NULL | `TransferBody.transfer_id` or `PaymentBody.payment_id` |
| from_did | TEXT | Sender DID (nullable for privacy) |
| to_did | TEXT | Recipient DID (nullable for privacy) |
| thread_id | TEXT | DIDComm thread ID |
| message_type | TEXT NOT NULL | Full TAP message type URI |
| status | TEXT NOT NULL DEFAULT 'pending' | |
| message_json | TEXT NOT NULL | Full DIDComm message |
| created_at | TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')) | |
| updated_at | TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')) | Updated via trigger |

### 5.3. Triggers
A common `updated_at` trigger will refresh the timestamp on `UPDATE`.

```sql
CREATE TRIGGER set_updated_at
AFTER UPDATE ON transactions
BEGIN
  UPDATE transactions SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = NEW.id;
END;
```

## 6. Migrations

We will embed plain SQL migration files under `tap-node/migrations/` and apply them using [`rusqlite_migration`](https://crates.io/crates/rusqlite_migration) (lightweight, no `diesel`).

```
├── tap-node
│   └── migrations
│       └── 0001_create_transactions.sql
```

`MigrationManager` (new module) will:
1. Locate or create the database file.
2. Run pending migrations on startup.
3. Expose a `get_connection_pool()` helper (Feature-gated off for WASM).

## 7. Public API (Storage Layer)

```rust
// tap-node/src/storage/mod.rs
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub struct Storage { 
    pool: Pool<SqliteConnectionManager>,
}

impl Storage {
    pub async fn new(path: Option<PathBuf>) -> Result<Self, StorageError>; // runs migrations
    
    pub async fn insert_transaction(&self, message: &Message) -> Result<(), StorageError>;
    
    // Future-ready query methods
    pub async fn get_transaction_by_id(&self, reference_id: &str) -> Result<Option<Transaction>, StorageError>;
    pub async fn list_transactions(&self, limit: u32, offset: u32) -> Result<Vec<Transaction>, StorageError>;
}
```

Inserting uses `serde_json::to_string_pretty(&message)` to store the canonical JSON.

## 8. Impacted Modules

- `tap-node/src/main.rs` (initialisation).
- `tap-node/src/handlers/` (where messages are observed/created).
- **New** `tap-node/src/storage/` module.

## 9. Security Considerations

- The DB file inherits OS-level permissions of the process; document recommendations (e.g., 600).
- Message JSON may contain sensitive metadata; consider future encryption or redaction.

## 10. Performance

- SQLite writes are fast (<1 ms). Bulk throughput is well within our target (thousands msgs/s).
- Use a single connection with WAL mode to minimise contention.

## 11. Testing

- Unit tests for migration runner and insertion APIs (using `tempfile`).
- Integration test triggering a dummy Transfer & Payment through `tap-node` and asserting rows count.

## 12. Future Work

- **Status Updates:** reconciler to mark `confirmed`, `failed`, etc.
- **Query APIs:** search by `reference_id`, status.
- **Database Abstraction:** switch to Postgres with minimal code change.
- **Encryption at Rest:** optional field-level encryption for `message_json`.

## 13. Open Questions

1. Do we need unique constraints on `reference_id`? **Answer: Yes, add UNIQUE constraint to prevent duplicates**
2. Where should the DB path be configured? (env var, CLI arg, config file) **Answer: Use `TAP_NODE_DB_PATH` env var with CLI override**
3. How do we handle storage when compiled to WASM? (likely disabled) **Answer: Feature-gate with `storage` feature, disabled for WASM**

## 14. Checklist (from prds/v1.md)

- [ ] **Feature Complete**
- [ ] **Security Review**
- [ ] **Testing**
  - [ ] Unit Tests
  - [ ] Integration Tests
- [ ] **Documentation**
  - [ ] API Documentation
  - [ ] Usage Examples
- [ ] **Performance**
  - [ ] Benchmarks (if applicable)
- [ ] **WASM Compatibility**
- [ ] **Code Quality**
  - [ ] `cargo fmt`
  - [ ] `cargo clippy --all-targets --all-features -- -D warnings`
  - [ ] No new compiler warnings
- [ ] **Dependencies Review**
- [ ] **Protocol Compliance**
