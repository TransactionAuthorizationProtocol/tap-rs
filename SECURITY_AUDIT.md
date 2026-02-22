# TAP-RS Security Audit Report

**Date**: 2026-02-22
**Scope**: Full codebase — all Rust crates and TypeScript packages
**Methodology**: Line-by-line static analysis of all source files, migrations, dependencies, and configurations

---

## Executive Summary

The `tap-rs` repository implements the Transaction Authorization Protocol (TAP) for financial messaging, including DIDComm v2 message packing/unpacking, state machine management, HTTP server endpoints, MCP tooling, WASM bindings, and CLI tooling. The codebase handles highly sensitive data including financial transaction details, private cryptographic keys, and Travel Rule PII (IVMS101).

After a comprehensive audit of all crates and TypeScript packages, **47 findings** were identified across severity levels. The codebase demonstrates generally strong security practices — SQL queries use parameterized bindings, the state machine enforces valid transitions, and cryptographic operations use well-tested libraries (DIDComm-rs, ed25519-dalek). However, several critical gaps exist around SQL injection in the MCP layer, financial input validation, key material protection, and authorization boundaries.

### Findings Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 1     |
| HIGH     | 9     |
| MEDIUM   | 18    |
| LOW      | 14    |
| INFO     | 5     |

---

## CRITICAL Findings

### C1: SQL Injection in tap-mcp Database Schema Tool

**Crate:** `tap-mcp`
**File:** `tap-mcp/src/tools/database_tools.rs`

The `tap_database_schema` tool constructs SQL queries using string formatting with user-controlled table names:

```rust
let query = format!("PRAGMA table_info({})", table_name);
```

The `table_name` parameter comes from MCP tool input and is not sanitized. An attacker with access to the MCP interface can inject arbitrary SQL:

```
table_name = "x); DROP TABLE transactions; --"
```

**Impact:** Full database compromise — read, modify, or delete any data including transactions, customer PII, and decision logs.

**Remediation:**
- Validate `table_name` against an allowlist of known tables
- Use a regex to reject anything that isn't `[a-zA-Z0-9_]`
- Consider removing direct SQL access entirely and using structured queries only

---

## HIGH Findings

### H1: NaN/Infinity Bypass in Financial Amount Validation

**Crate:** `tap-msg`
**File:** `tap-msg/src/message/transfer.rs`, `tap-msg/src/message/payment.rs`

The `amount` field in `Transfer` and `Payment` messages is a `String` type that undergoes validation via `validate_amount()`. However, this function only checks for a valid decimal pattern. The IEEE 754 special values `NaN`, `Infinity`, and `-Infinity` are not rejected and could propagate through financial calculations:

```rust
fn validate_amount(amount: &str) -> bool {
    // Checks for valid decimal pattern but doesn't reject NaN/Infinity
}
```

**Impact:** Financial calculation errors, potential bypass of balance checks if downstream systems parse "NaN" or "Infinity" as valid amounts.

**Remediation:** Explicitly reject `NaN`, `Infinity`, `-Infinity`, and negative amounts. Add maximum amount bounds.

---

### H2: Validation Not Enforced on Deserialization

**Crate:** `tap-msg`
**File:** `tap-msg/src/message/*.rs`

All TAP message types implement the `Validation` trait with checks for required fields, amount formats, DID formats, etc. However, validation is **not called during deserialization**. A caller using `serde_json::from_str::<Transfer>(...)` gets an unvalidated object. Only callers who explicitly call `.validate()` get the safety checks.

```rust
// This produces an unvalidated Transfer with amount = "" or asset_id = "garbage"
let transfer: Transfer = serde_json::from_str(json)?;
// Must explicitly call: transfer.validate()?;
```

**Impact:** Any code path that deserializes messages without calling `.validate()` processes unvalidated input, bypassing all field-level checks.

**Remediation:** Implement a custom `Deserialize` that calls validation, or use the `#[serde(try_from = "...")]` pattern to enforce validation at the type level.

---

### H3: DoS via Unbounded Agent Creation

**Crate:** `tap-http`
**File:** `tap-http/src/main.rs`

The `/.well-known/did.json` endpoint auto-creates a new `TapNode` instance per unique DID in the Host header, stored in a `DashMap` with no upper bound:

```rust
// No limit on how many agents can be created
let node = get_or_create_node(&nodes, &agent_did, &config).await?;
```

An attacker can send requests with millions of different Host headers, each creating a new agent with its own SQLite database, consuming disk space and memory until the system fails.

**Impact:** Denial of service — memory exhaustion and disk space exhaustion.

**Remediation:** Add an upper bound on the number of agents. Validate the Host header against a configured allowlist of expected DIDs.

---

### H4: No Request Body Size Limit on DIDComm Endpoint

**Crate:** `tap-http`
**File:** `tap-http/src/main.rs`

The `/didcomm` POST endpoint reads the full request body without size limits:

```rust
async fn handle_didcomm(body: String, ...) -> impl IntoResponse {
```

Axum's default body size limit is very permissive. An attacker can send a multi-gigabyte POST body to exhaust memory.

**Impact:** Denial of service via memory exhaustion.

**Remediation:** Add `axum::extract::DefaultBodyLimit::max(MAX_BODY_SIZE)` middleware with a reasonable limit (e.g., 1MB).

---

### H5: Internal Error Details Leaked to HTTP Clients

**Crate:** `tap-http`
**File:** `tap-http/src/main.rs`

Error responses include internal implementation details:

```rust
(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to receive message: {}", e))
```

This exposes database errors, file paths, and internal state information to external callers.

**Impact:** Information disclosure that aids attackers in understanding internal architecture and crafting targeted exploits.

**Remediation:** Return generic error messages to clients. Log detailed errors server-side only.

---

### H6: SQL Read-Only Filter Bypass in tap-mcp

**Crate:** `tap-mcp`
**File:** `tap-mcp/src/tools/database_tools.rs`

The `tap_database_query` tool attempts to enforce read-only queries by checking for a `SELECT` prefix, but the check is easily bypassed:

```rust
if !query.trim().to_uppercase().starts_with("SELECT") {
    return Err("Only SELECT queries are allowed");
}
```

This can be bypassed with: `SELECT 1; DROP TABLE transactions;` or using CTEs: `WITH x AS (DELETE FROM transactions RETURNING *) SELECT * FROM x;`

**Impact:** Full database modification/deletion through what should be a read-only interface.

**Remediation:** Execute queries within a read-only transaction (`BEGIN DEFERRED; PRAGMA query_only = ON;`), or remove raw SQL query capability entirely.

---

### H7: Private Key Stored as Plaintext String in WASM Heap

**Crate:** `tap-wasm`
**File:** `tap-wasm/src/lib.rs`

The `WasmTapAgent` stores the private key as a `String` field in the WASM heap:

```rust
pub struct WasmTapAgent {
    private_key: String,  // Plaintext in WASM linear memory
    // ...
}
```

WASM linear memory is a contiguous ArrayBuffer accessible from JavaScript. The private key persists in memory after the agent is dropped because WASM does not zero memory on deallocation.

**Impact:** Private key exposure through JavaScript memory inspection, browser devtools, memory dumps, or extensions.

**Remediation:** Use `zeroize` crate for key material. Minimize the lifetime of plaintext keys. Consider storing keys in WebCrypto `CryptoKey` objects which are opaque to JavaScript.

---

### H8: Example Code Promotes Insecure Key Storage

**Crate:** `tap-ts`
**File:** `tap-ts/examples/`

Example code demonstrates storing private keys in `localStorage`:

```typescript
localStorage.setItem('agent_private_key', privateKey);
```

`localStorage` is accessible to any JavaScript on the same origin (XSS), has no expiration, and persists across sessions. Developers following examples often copy patterns directly.

**Impact:** Key theft via XSS attacks in applications that follow this pattern.

**Remediation:** Replace with `sessionStorage` at minimum, or demonstrate using the Web Crypto API `extractable: false` pattern. Add security warnings.

---

### H9: No Encryption at Rest for Sensitive Data

**Crate:** `tap-node`
**File:** `tap-node/src/storage/db.rs`, `tap-node/migrations/007_create_customers.sql`

The SQLite database stores highly sensitive data in plaintext:
- Customer PII (names, addresses) in the `customers` table
- IVMS101 Travel Rule compliance data (full identity documents)
- Full message bodies including financial details
- Customer relationship proofs

The database file sits on disk at `~/.tap/{did}/transactions.db` with no encryption.

**Impact:** An attacker with filesystem read access obtains all PII and Travel Rule data, potentially violating GDPR, travel rule regulations, and data protection requirements.

**Remediation:** Use SQLCipher for database-level encryption. Encrypt sensitive columns at the application layer. Use OS-level filesystem encryption as defense-in-depth.

---

## MEDIUM Findings

### M1: `#[serde(untagged)]` Type Confusion in Message Enums

**Crate:** `tap-msg`
**Files:** `tap-msg/src/message/transfer.rs`, `tap-msg/src/message/payment.rs`

Several enums use `#[serde(untagged)]` which tries variants in order. Since variants share common fields, a value intended for one type can silently deserialize as another:

```rust
#[derive(Deserialize)]
#[serde(untagged)]
enum SupportedAsset {
    Slip44(Slip44Asset),   // { slip44: u32 }
    Caip19(Caip19Asset),   // { caip19: String }
}
```

**Impact:** Silent type misinterpretation could cause funds to be sent to the wrong asset type.

**Remediation:** Use `#[serde(tag = "type")]` internally tagged enums to make disambiguation explicit.

---

### M2: Panicking in Library Code

**Crate:** `tap-msg`
**Files:** Multiple files in `tap-msg/src/`

Several `.unwrap()` and `.expect()` calls exist in library code, particularly in CAIP-2 parsing and serialization paths. Library code should never panic.

**Impact:** Denial of service if a caller passes malformed input that triggers a panic.

**Remediation:** Replace all `.unwrap()`/`.expect()` with proper `Result` error propagation.

---

### M3: No Input Size Limits on String Fields

**Crate:** `tap-msg`
**Files:** `tap-msg/src/message/*.rs`

TAP message types have `String` fields with no maximum length validation:
- `memo` field on transfers (arbitrary length)
- `note` field on payments
- DID fields
- CAIP identifiers

**Impact:** Memory exhaustion or storage bloat from oversized messages.

**Remediation:** Add maximum length validation to all string fields in the `Validation` trait implementations.

---

### M4: No Security Headers on HTTP Responses

**Crate:** `tap-http`
**File:** `tap-http/src/main.rs`

The HTTP server does not set security headers: `X-Content-Type-Options`, `X-Frame-Options`, `Strict-Transport-Security`, `Content-Security-Policy`, `X-XSS-Protection`.

**Impact:** Increases attack surface for content sniffing, clickjacking, and downgrade attacks.

**Remediation:** Add security headers middleware via `tower-http`.

---

### M5: No CORS Policy

**Crate:** `tap-http`
**File:** `tap-http/src/main.rs`

No CORS headers are configured. If browser clients need to interact with the API, CORS will need to be configured; if they don't, an explicit deny is still better than relying on defaults.

**Impact:** Either blocks legitimate browser clients or, if CORS is loosely added later, enables cross-origin attacks.

**Remediation:** Configure explicit CORS policy with allowlisted origins.

---

### M6: No Rate Limiting

**Crate:** `tap-http`
**File:** `tap-http/src/main.rs`

No rate limiting on any endpoint. The `/didcomm` endpoint processes every incoming message, and `/.well-known/did.json` creates new agent instances.

**Impact:** Denial of service through message flooding.

**Remediation:** Add per-IP and per-DID rate limiting using `tower::limit` or a dedicated rate limiter.

---

### M7: TLS Not Implemented

**Crate:** `tap-http`
**File:** `tap-http/src/main.rs`

The server binds with plain HTTP. No TLS configuration exists.

**Impact:** All traffic including DIDComm messages and PII is transmitted in cleartext.

**Remediation:** Add TLS support via `axum-server` with `rustls`, or document that a reverse proxy (nginx, caddy) is required.

---

### M8: External Process Tool Call Responses Not Returned

**Crate:** `tap-http`
**File:** `tap-http/src/external_decision/manager.rs`

When the external decision process sends tool call requests via stdout, the results of those tool calls are computed but never sent back to the external process. The response is logged but dropped.

**Impact:** External decision processes cannot get feedback on their actions, leading to blind operation and potential inconsistency.

**Remediation:** Send tool call results back via stdin to the external process.

---

### M9: No Authorization Boundary Between Agent DIDs in MCP

**Crate:** `tap-mcp`
**File:** `tap-mcp/src/tools/`

MCP tools operate on a `TapIntegration` that has a `default_agent_did`. However, many tools accept an `agent_did` parameter, allowing the MCP client to query or act as any agent DID known to the system without authorization checks.

**Impact:** An MCP client authenticated for one agent can access or act as any other agent.

**Remediation:** Validate that the requesting client is authorized for the specified `agent_did`, or restrict operations to the configured default agent only.

---

### M10: Full PII Exposed to AI Models via MCP

**Crate:** `tap-mcp`
**File:** `tap-mcp/src/tools/customer_tools.rs`

Customer tools return full PII (names, addresses, national IDs) as MCP tool results that flow directly to AI models:

```rust
// Returns full customer profile including PII to the AI model
serde_json::to_string_pretty(&customer)?
```

**Impact:** PII leakage to AI model providers. May violate data processing agreements and privacy regulations.

**Remediation:** Return only customer IDs and minimal metadata. Require explicit user confirmation before revealing PII.

---

### M11: LIKE Pattern Injection in Customer Search

**Crate:** `tap-node`
**File:** `tap-node/src/storage/db.rs`

```rust
let search_pattern = format!("%{}%", query);
```

LIKE metacharacters (`%`, `_`) in the search query are not escaped, enabling pattern-based enumeration of the customer database.

**Impact:** Data enumeration through crafted search patterns.

**Remediation:** Escape `%` and `_` characters and add `ESCAPE '\'` to the SQL LIKE clause.

---

### M12: Panic-Inducing `unwrap()` on Database Deserialization

**Crate:** `tap-node`
**File:** `tap-node/src/storage/db.rs`

Over a dozen `.unwrap()` calls on `serde_json::from_str()` when reading customer profiles, IVMS101 data, relationship proofs, and decision details from the database:

```rust
profile: serde_json::from_str(&row.get::<String, _>("profile")).unwrap(),
```

**Impact:** Malformed JSON in the database (from corruption or bugs) crashes the entire node process.

**Remediation:** Replace with `.map_err(|e| StorageError::Deserialization(...))?`.

---

### M13: Race Condition in State Machine Transitions (TOCTOU)

**Crate:** `tap-node`
**File:** `tap-node/src/state_machine/fsm.rs`

The FSM reads state, computes transition, then writes — without holding a lock. Concurrent messages for the same transaction can race.

**Impact:** In multi-agent transactions, concurrent authorizations can be lost, leaving transactions stuck or causing duplicate settlements.

**Remediation:** Use `BEGIN EXCLUSIVE TRANSACTION` in SQLite or implement optimistic concurrency control with version numbers.

---

### M14: DID Sanitization for Filesystem Paths is Insufficient

**Crate:** `tap-node`
**File:** `tap-node/src/storage/db.rs`

```rust
let sanitized_did = agent_did.replace(':', "_");
```

Only `:` is replaced. A DID containing `../` sequences could escape the intended directory.

**Impact:** Path traversal — database created outside the `.tap` directory tree.

**Remediation:** Replace `/`, `\`, and `..` sequences. Verify the resolved path is under the root directory after canonicalization.

---

### M15: Agent Authorization Validator Accepts on Missing Transaction ID

**Crate:** `tap-node`
**File:** `tap-node/src/validation/agent_validator.rs`

```rust
None => {
    // Can't find transaction ID
    return ValidationResult::Accept;  // Fail-open!
}
```

When the validator cannot extract a transaction ID from a message, it defaults to accepting the message.

**Impact:** Malformed authorization messages bypass agent validation checks.

**Remediation:** Return `ValidationResult::Reject` for messages that should have a transaction ID but don't.

---

### M16: No Key Zeroization in WASM

**Crate:** `tap-wasm`
**File:** `tap-wasm/src/lib.rs`

Key material in `String` and `Vec<u8>` types is not zeroized on drop. WASM linear memory does not zero freed allocations.

**Impact:** Key material persists in WASM memory after agent disposal.

**Remediation:** Use `zeroize::Zeroizing<String>` for all key material.

---

### M17: PII Data in Debug/Display Implementations

**Crate:** `tap-ivms101`
**Files:** `tap-ivms101/src/*.rs`

IVMS101 types derive `Debug` which includes full PII (names, birth dates, national IDs, addresses) in debug output.

**Impact:** PII leakage through logging, error messages, and debug output.

**Remediation:** Implement custom `Debug` that redacts PII fields. Use `secrecy` crate for sensitive fields.

---

### M18: No Agent-Level Data Isolation Enforcement

**Crate:** `tap-node`
**File:** `tap-node/src/storage/db.rs`

Methods like `get_transaction_by_id` don't filter by `agent_did`. If the centralized storage path is used, one agent can access another's transactions.

**Impact:** Cross-agent data access in shared storage configurations.

**Remediation:** Add `agent_did` as a required parameter to all query methods. Use per-agent storage exclusively.

---

## LOW Findings

### L1: `#[serde(flatten)]` with HashMap Captures Unknown Fields Silently

**Crate:** `tap-msg` — Unknown fields are silently accepted rather than rejected, allowing malformed messages to pass without error.

### L2: Panic on Missing Home Directory

**Crate:** `tap-node` — `dirs::home_dir().expect(...)` panics in containers or CI environments without home directories.

### L3: Insufficient URL Encoding in HTTP/WebSocket Senders

**Crate:** `tap-node` — Hand-rolled URL encoder only handles `:` and `/`, missing `#`, `?`, `@`, `=`, `&`, `+`, spaces.

### L4: `PRAGMA synchronous = NORMAL` Reduces Durability

**Crate:** `tap-node` — Reduced durability guarantees for a financial transaction database; committed data can be lost on OS crash.

### L5: Message Content Logged at Debug Level

**Crate:** `tap-node` — Full message contents (potentially containing PII and financial data) logged at debug level.

### L6: `Closure::forget()` Memory Leak in WASM WebSocket Sender

**Crate:** `tap-node` — WASM WebSocket callbacks leak memory for each connection via `Closure::forget()`.

### L7: Missing Input Validation on Status and Role Strings

**Crate:** `tap-node` — Storage methods accept free-form strings for status and role values instead of validated enums.

### L8: Pervasive Use of `any` Type in TypeScript

**Crate:** `tap-ts` — Extensive use of `any` bypasses TypeScript's type safety, particularly in WASM interface layer.

### L9: `dispose()` Does Not Clear Private Key References

**Crate:** `tap-ts` — The TypeScript `dispose()` method frees WASM resources but doesn't clear the JavaScript-side private key string references.

### L10: No Country Code Whitelist Validation

**Crate:** `tap-ivms101` — Country codes are not validated against ISO 3166-1 alpha-2, accepting arbitrary 2-character strings.

### L11: Key Storage as Plaintext JSON on Filesystem

**Crate:** `tap-agent` — `AgentKeyManager` stores private keys as plaintext JSON files on disk. Documented as a known limitation.

### L12: No Recipient Verification During Message Unpack

**Crate:** `tap-wasm` — Messages are unpacked without verifying the recipient DID matches the local agent, allowing processing of messages intended for other agents.

### L13: No Input Length Validation for Messages to WASM

**Crate:** `tap-wasm` — No maximum size check on messages passed to WASM pack/unpack functions.

### L14: WASM Module State in Global Mutable Variables

**Crate:** `tap-wasm` — Global mutable state could cause issues in multi-threaded WASM environments (SharedArrayBuffer).

---

## INFO Findings

### I1: No Rate Limiting on Message Processing

**Crate:** `tap-node` — `TapNode::receive_message` processes all incoming messages without per-sender rate limiting.

### I2: Missing `#[must_use]` on Validation Results

**Crate:** `tap-msg` — `ValidationResult` does not have `#[must_use]`, so callers can silently ignore validation failures.

### I3: No Signature Verification Enforcement During Unpack

**Crate:** `tap-wasm` — Signature verification during `unpack_message` depends on DID resolution, which may silently skip verification if resolution fails.

### I4: External Process Receives Full Transaction State

**Crate:** `tap-http` — Decision requests to external processes include full transaction details; a compromised external process gets complete visibility.

### I5: No Audit Trail for MCP Tool Invocations

**Crate:** `tap-mcp` — Tool invocations including customer queries and transaction actions are not logged to an audit trail.

---

## Positive Findings (Strengths)

1. **SQL Injection Protection:** All SQL queries throughout the codebase use parameterized bindings via `sqlx::query().bind()`. No string interpolation into SQL was found (except the PRAGMA table_info case in tap-mcp).

2. **Real Cryptography:** The codebase uses established cryptographic libraries (DIDComm-rs, ed25519-dalek, x25519-dalek) with proper key generation. No homebrew crypto or placeholder implementations.

3. **State Machine Design:** The FSM is a pure-logic component with no I/O. Terminal states reject all further events. Transition rules are comprehensive and correct for the TAP specification.

4. **Message Validation Pipeline:** The `CompositeValidator` provides defense-in-depth with timestamp validation, replay prevention (message uniqueness), and agent authorization.

5. **Database Schema Constraints:** Migration files include `CHECK` constraints on critical enum columns, providing database-level defense against invalid data.

6. **Error Type Design:** `thiserror`-based error hierarchies avoid exposing internal details in most error paths.

7. **Dependency Quality:** The project uses well-established, well-audited dependencies (sqlx, tokio, serde, reqwest, axum, DIDComm-rs).

8. **Per-Agent Database Isolation:** The `AgentStorageManager` creates separate SQLite databases per agent DID, providing strong isolation when used correctly.

9. **Comprehensive Test Coverage:** Extensive test suites across all crates, including interoperability tests with Veramo.

---

## Prioritized Remediation Roadmap

### Immediate (Sprint 1) — Quick Wins

| # | Finding | Effort | Impact |
|---|---------|--------|--------|
| 1 | C1 — SQL injection in MCP database tool | Low | Critical |
| 2 | H6 — SQL read-only filter bypass | Low | High |
| 3 | H5 — Internal error details leaked | Low | High |
| 4 | H4 — No body size limit | Low | High |
| 5 | M15 — Fail-open authorization validator | Low | Medium |
| 6 | M14 — Path traversal in DID sanitization | Low | Medium |

### Short-term (Sprint 2) — Core Security

| # | Finding | Effort | Impact |
|---|---------|--------|--------|
| 7 | H3 — Unbounded agent creation | Medium | High |
| 8 | H1 — NaN/Infinity in financial amounts | Low | High |
| 9 | H2 — Validation not enforced on deser | Medium | High |
| 10 | M12 — Panicking unwrap on DB reads | Medium | Medium |
| 11 | M2 — Panicking in library code | Medium | Medium |
| 12 | M6 — Rate limiting | Medium | Medium |
| 13 | M11 — LIKE pattern injection | Low | Medium |

### Medium-term (Sprint 3-4) — Defense in Depth

| # | Finding | Effort | Impact |
|---|---------|--------|--------|
| 14 | H9 — No encryption at rest | High | High |
| 15 | H7 — Plaintext key in WASM heap | Medium | High |
| 16 | M13 — State machine race condition | Medium | Medium |
| 17 | M9 — No MCP authorization boundary | Medium | Medium |
| 18 | M4 — Security headers | Low | Medium |
| 19 | M7 — TLS support | Medium | Medium |
| 20 | M17 — PII in Debug output | Medium | Medium |

### Long-term (Backlog) — Hardening

| # | Finding | Effort | Impact |
|---|---------|--------|--------|
| 21 | M10 — PII exposed to AI models | Medium | Medium |
| 22 | M16 — Key zeroization in WASM | Low | Medium |
| 23 | M1 — Untagged enum type confusion | Medium | Medium |
| 24 | H8 — Insecure example code | Low | High |
| 25 | All LOW and INFO findings | Varies | Low |

---

## Methodology

This audit was conducted through static analysis of all source code in the repository. Each crate was reviewed independently with focus on:

- **Cryptographic correctness:** Key generation, signing, encryption, and verification
- **Input validation:** All external inputs including messages, HTTP requests, CLI arguments, and MCP tool parameters
- **SQL security:** Query construction, parameterization, and access control
- **Authorization:** Agent-level isolation, DID validation, and access boundaries
- **Data protection:** PII handling, key material lifecycle, and encryption at rest
- **Error handling:** Panic safety, error information leakage, and fail-open vs fail-closed design
- **Denial of service:** Resource limits, rate limiting, and unbounded operations
- **Dependencies:** Known vulnerabilities and supply chain risks

**Crates audited:** tap-agent, tap-msg, tap-msg-derive, tap-node, tap-http, tap-mcp, tap-wasm, tap-cli, tap-caip, tap-ivms101
**Packages audited:** tap-ts (@taprsvp/agent)

---

*This audit focused on static code review. Penetration testing and dynamic analysis were not performed.*
