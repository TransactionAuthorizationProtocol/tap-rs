# TAP-RS Security and Code Quality Audit Report

**Date**: 2026-01-29
**Version Audited**: 0.5.0
**Auditor**: Automated Security Analysis

---

## Executive Summary

This audit covers security, functionality, and Rust code quality across the tap-rs workspace. The project implements the Transaction Authorization Protocol (TAP) for secure blockchain transaction authorization with Travel Rule compliance.

### Overall Assessment

| Category | Rating | Summary |
|----------|--------|---------|
| **Security** | ⚠️ CRITICAL | Multiple critical cryptographic vulnerabilities |
| **Functionality** | ✅ GOOD | Core protocol complete, some gaps in policy enforcement |
| **Code Quality** | ✅ GOOD | Idiomatic Rust with room for improvement |

### Critical Issues Count

| Severity | Count |
|----------|-------|
| 🔴 CRITICAL | 6 |
| 🟠 HIGH | 8 |
| 🟡 MEDIUM | 12 |
| 🟢 LOW | 7 |

---

## Part 1: Security Findings

### CRITICAL SEVERITY

#### SEC-001: XOR-Based Key Encryption (CWE-327)
**File**: `tap-agent/src/local_agent_key.rs:905-908, 990-993`

**Description**: Content Encryption Key (CEK) wrapping uses simple XOR instead of proper AES-KW key wrapping per RFC 3394.

```rust
// Encryption (line 905-908)
let mut encrypted_cek = cek;
for i in 0..cek.len() {
    encrypted_cek[i] ^= shared_bytes[i % shared_bytes.len()];
}

// Decryption (line 990-993)
cek[i] = private_key[i] ^ encrypted_key[i]
```

**Impact**: XOR provides no semantic security. An attacker with ciphertext can trivially recover the key.

**Recommendation**: Implement proper ECDH-ES+A256KW key wrapping per RFC 7518.

---

#### SEC-002: Plaintext Private Key Storage (CWE-312)
**File**: `tap-agent/src/storage.rs:462-470`

**Description**: Private keys are stored as plaintext JSON on disk without encryption.

```rust
pub fn save_to_path(&self, path: &Path) -> Result<()> {
    let contents = serde_json::to_string_pretty(self)?;
    fs::write(path, contents)?;  // No encryption, no permissions
    Ok(())
}
```

**Impact**: Any process or user with file access can read all private keys from `~/.tap/keys.json`.

**Recommendation**:
1. Encrypt keys at rest using envelope encryption
2. Set file permissions to 0o600 immediately after creation
3. Consider platform keychain integration (macOS Keychain, Windows DPAPI)

---

#### SEC-003: Missing File Permission Protection (CWE-276)
**File**: `tap-agent/src/storage.rs:466`

**Description**: Key storage files are created with default permissions (typically 0o644), making them readable by all users.

**Impact**: Multi-user systems expose private keys to other users.

**Recommendation**: Use `std::os::unix::fs::PermissionsExt` to set 0o600 permissions on Unix systems.

---

#### SEC-004: Mock Encryption in Production Code (CWE-327)
**File**: `tap-agent/src/local_agent_key.rs:92-139`

**Description**: The `encrypt_to_jwk()` method uses base64 encoding instead of actual encryption, with hardcoded test values.

```rust
let ciphertext = base64::engine::general_purpose::STANDARD.encode(plaintext);
ephemeral_key: EphemeralPublicKey::Ec {
    crv: "P-256".to_string(),
    x: "test".to_string(),  // HARDCODED
    y: "test".to_string(),  // HARDCODED
},
```

**Impact**: Messages marked as "encrypted" are actually plaintext.

---

#### SEC-005: Unchecked String Slicing (Panic/DoS)
**Files**:
- `tap-msg/src/settlement_address.rs:51, 70`
- `tap-agent/src/did.rs:297, 386, 592-593`

**Description**: String slicing operations without bounds checking cause panics on malformed input.

```rust
// settlement_address.rs:51 - Panics if URI < 8 chars
let after_scheme = &uri[8..];

// did.rs:297 - Panics if DID < 8 chars
let key_id = &did_key[8..];
```

**Impact**: Denial of service via malformed input.

**Recommendation**: Add length validation before all slice operations.

---

#### SEC-006: Array Indexing Without Bounds Check
**File**: `tap-agent/src/did.rs:592-593`

**Description**: Array access without verifying collection is non-empty.

```rust
let path_segments: Vec<&str> = domain_path.split(':').collect();
let domain = path_segments[0];  // Panics if empty
```

**Impact**: Panic on malformed did:web URIs.

---

### HIGH SEVERITY

#### SEC-007: No Secret Zeroization (CWE-226)
**Files**: Throughout `local_agent_key.rs`, `key_manager.rs`, `agent_key_manager.rs`

**Description**: Private keys stored in plain `Vec<u8>` without zeroization after use.

**Recommendation**: Add `zeroize` crate dependency and implement `Zeroize` trait for all key material.

---

#### SEC-008: Incomplete ECDH Key Derivation
**File**: `tap-agent/src/local_agent_key.rs:897-910`

**Description**: Raw ECDH shared secret used directly without proper KDF.

**Recommendation**: Implement Concat KDF per NIST SP 800-56A.

---

#### SEC-009: Potential SSRF in did:web Resolution
**File**: `tap-agent/src/did.rs:589-598`

**Description**: did:web URLs are constructed without validating the domain, potentially allowing SSRF to internal services.

```rust
let url = format!("https://{}/{}/did.json", domain, path);
// No validation of domain (could be localhost, 127.0.0.1, etc.)
```

**Recommendation**: Implement domain allowlist/denylist and block private IP ranges.

---

#### SEC-010: Slice Without Length Validation
**File**: `tap-agent/src/did.rs:1107-1122`

**Description**: Public key slicing assumes exact 64-byte length without validation.

```rust
"x": encode(&key.public_key[0..32]),
"y": encode(&key.public_key[32..64]),
```

---

### MEDIUM SEVERITY

#### SEC-011: Excessive Regex Compilation
**Files**: `tap-caip/src/chain_id.rs`, `account_id.rs`, `asset_id.rs`

**Description**: Same regex patterns recompiled on every validation call instead of using `lazy_static!` or `OnceCell`.

**Impact**: Performance degradation under load.

---

#### SEC-012: Lock Poisoning Not Handled
**File**: `tap-agent/src/agent_key_manager.rs`

**Description**: RwLock errors return generic error without distinguishing poisoned locks.

---

#### SEC-013: Loose CAIP-10 Validation in Settlement Addresses
**File**: `tap-msg/src/settlement_address.rs:110-114`

**Description**: CAIP-10 addresses validated loosely (just checks for colons) unlike strict validation in tap-caip.

---

#### SEC-014: No Request Body Size Limits
**File**: `tap-http/src/handler.rs`

**Description**: HTTP handlers accept unlimited body sizes, enabling memory exhaustion.

---

#### SEC-015: Multiple Clones of Private Keys
**Files**: `agent_key_manager.rs`, `local_agent_key.rs`

**Description**: Private keys frequently cloned without clearing originals.

---

---

## Part 2: Functionality Findings

### Protocol Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Message Types (TAIP-3,4,5,6,7,8,9) | ✅ Complete | All message types implemented |
| DIDComm v2 | ✅ Complete | from_prior JWT not implemented |
| IVMS 101.2023 | ✅ Complete | Travel Rule data model |
| State Machine | ⚠️ Partial | Policy enforcement missing |
| Travel Rule Processor | ⚠️ Partial | Threshold checking not implemented |

### FUNC-001: Policy Enforcement Not Implemented
**File**: `tap-node/src/state_machine/mod.rs:521-525`

```rust
TapMessage::UpdatePolicies(_) => {
    log::debug!("UpdatePolicies message received, but policy storage not implemented");
}
```

**Impact**: RequireAuthorization, RequirePresentation, RequireProofOfControl policies are defined but not enforced.

---

### FUNC-002: Travel Rule Threshold Not Enforced
**File**: `tap-node/src/message/travel_rule_processor.rs:181-190`

```rust
async fn should_attach_ivms101(&self, _message: &PlainMessage) -> bool {
    // In a production system, this would check:
    // 1. Regulatory requirements for the jurisdiction
    // 2. Transaction amount thresholds
    // ... For now, we'll attach IVMS101 data to all transfers
    true
}
```

**Impact**: Cannot enforce jurisdiction-specific Travel Rule thresholds.

---

### FUNC-003: Escrow Expiry Not Enforced
**Description**: Escrow messages have `expires` field but no automatic release mechanism on expiry.

---

### FUNC-004: from_prior JWT Not Implemented
**File**: Throughout `tap-msg`, `tap-agent`

**Description**: `from_prior` field exists but is always `None`. DID delegation chain not supported.

---

### FUNC-005: Settlement Amount Validation Missing
**Description**: No validation that Settle amount <= original Transfer amount.

---

---

## Part 3: Rust Code Quality Findings

### Positive Findings

1. **Error Handling**: Consistent use of `thiserror` with proper `Result<T>` types
2. **Trait Design**: Clean separation of `TapMessageBody`, `TapMessage`, `Authorizable`, `Transaction`
3. **Async Patterns**: Proper use of `async-trait` with `Send + Sync` bounds
4. **Minimal Unsafe**: Only 5 justified `unsafe` blocks
5. **Module Organization**: Clear separation of concerns with feature-gated exports

### CODE-001: Panic Points in Derive Macro
**File**: `tap-msg-derive/src/lib.rs:901-1113`

**Description**: Generated code uses `.expect()` instead of returning Results.

```rust
let original_message = self
    .to_didcomm(creator_did)
    .expect("Failed to create DIDComm message");  // Panic!
```

**Recommendation**: Generated code should propagate errors with `?`.

---

### CODE-002: Unimplemented body_as Method
**File**: `tap-msg-derive/src/lib.rs:417-421`

```rust
fn body_as<T: TapMessageBody>(&self) -> Result<T> {
    unimplemented!()
}
```

---

### CODE-003: High Clone Count in tap-agent
**Observation**: 209 `.clone()` calls in 19 source files suggests opportunities for:
- Better use of `Arc` for shared ownership
- Borrowing instead of cloning
- `Cow<str>` for optional ownership

---

### CODE-004: RwLock Contention Pattern
**File**: `tap-agent/src/agent_key_manager.rs:28-38`

**Description**: 5 separate RwLocks for different key types could use `dashmap` for better concurrent access.

---

### CODE-005: Missing Derives
**Observation**: Several types could benefit from:
- `#[derive(Clone)]` for ergonomics
- `#[derive(Default)]` where sensible
- `#[derive(PartialEq, Eq)]` for testing

---

---

## Recommendations Summary

### Immediate (Critical Security)

1. **Replace XOR encryption** with proper AES-KW per RFC 3394
2. **Encrypt keys at rest** or integrate with platform keychain
3. **Set file permissions** to 0o600 on key storage
4. **Remove mock encryption code** - implement real ECDH-ES+A256KW
5. **Add bounds checking** before all string slicing operations
6. **Add zeroize** crate for secret memory cleanup

### Short-Term (High Priority)

1. Implement proper Concat KDF for ECDH
2. Add SSRF protection for did:web resolution
3. Add request body size limits to HTTP handlers
4. Implement policy storage and enforcement
5. Add Travel Rule threshold configuration

### Medium-Term (Functionality)

1. Implement from_prior JWT validation
2. Add escrow expiry enforcement
3. Implement settlement amount validation
4. Complete LegalPerson mapping for customers
5. Add ISO validation (country codes, LEI checksums)

### Long-Term (Code Quality)

1. Replace RwLocks with dashmap where appropriate
2. Reduce unnecessary cloning with better Arc usage
3. Fix derive macro to return Results instead of panicking
4. Add comprehensive documentation examples
5. Cache regex patterns with lazy_static

---

## Test Coverage Assessment

| Crate | Tests | Coverage |
|-------|-------|----------|
| tap-msg | 101 | Good - all message types |
| tap-agent | 13+ | Moderate - key operations |
| tap-node | 33 | Good - state machine, routing |
| tap-caip | Fuzz tests | Good - property testing |
| tap-ivms101 | 4+ | Good - validation |
| tap-http | 4 | Basic - integration |

**Overall**: Good test coverage with comprehensive test vectors. Missing negative testing for security edge cases.

---

## Conclusion

TAP-RS is a well-structured Rust implementation of the Transaction Authorization Protocol with:

- **Complete message type implementation** per TAIP specifications
- **Good DIDComm v2 compliance** (except from_prior)
- **Comprehensive IVMS 101.2023 support**
- **Solid Rust idioms** and async patterns

However, **critical security vulnerabilities in cryptographic operations** must be addressed before production deployment. The XOR-based key encryption and plaintext key storage are particularly severe.

**Production Readiness**:
- Basic transfers: ⚠️ After security fixes
- Regulated deployments: ❌ Requires policy enforcement and Travel Rule thresholds

---

*This audit focused on code review. Penetration testing and dynamic analysis were not performed.*
