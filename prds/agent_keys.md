# PRD: Agent Keys & KeyManager Refactor

## 1. Background

Currently, cryptographic operations such as signing, encryption, and decryption are primarily handled in `tap-agent/src/crypto.rs`. The `tap-agent/src/key_manager.rs` is responsible for managing raw cryptographic secrets. This separation can lead to:

- Dispersed logic for key usage and cryptographic operations.
- Challenges in extending key management to support external key sources like Hardware Security Modules (HSMs) or other remote key services.
- Difficulty in managing key-specific metadata (e.g., algorithm, key identifiers for JWS/JWE) alongside the keys themselves.

This proposal outlines a refactor to consolidate cryptographic responsibilities within an enhanced `KeyManager` by introducing an `AgentKey` abstraction.

## 2. Goals

- **Consolidate Crypto Operations:** Centralize signing, verification, encryption, and decryption logic within the `KeyManager`'s scope of responsibility, leveraging the new `AgentKey` abstraction.
- **Introduce `AgentKey`:** Define an `AgentKey` struct and associated traits. This abstraction will encapsulate key material (or a reference to it) and the logic for its cryptographic use.
- **Store `AgentKey` Instances:** The `KeyManager` will store and manage instances of `AgentKey` instead of raw secrets.
- **Algorithm and Metadata Management:** The `AgentKey` itself will be responsible for determining appropriate algorithms and preparing necessary metadata (e.g., `kid`, `alg`) for JWS/JWE operations.
- **Enable Remote Key Operations:** The `AgentKey` design must allow for implementations that delegate cryptographic operations to remote services, such as HSMs, without exposing private key material to the `tap-agent`.
- **Improve Modularity and Testability:** Enhance the modularity of key management and cryptographic functions, making them easier to test and maintain.

## 3. Proposed Changes

### 3.1. `AgentKey` Trait and Structs

Define a core `AgentKey` trait and potentially specific sub-traits or concrete structs for different key types/operations.

```rust
// Example Trait (conceptual)
pub trait AgentKey: Send + Sync {
    fn key_id(&self) -> &str; // Unique identifier for the key
    fn public_key_jwk(&self) -> Result<serde_json::Value, AgentKeyError>; // Export public key as JWK
    // Potentially methods to indicate capabilities (sign, verify, encrypt, decrypt)
}

pub trait SigningKey: AgentKey {
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, AgentKeyError>;
    fn recommended_jws_alg(&self) -> JwsAlgorithm;
}

/// Represents a key (typically public) capable of verifying a JWS signature.
/// This trait might be implemented by a struct holding a public JWK,
/// or by an AgentKey that can expose its public verification capabilities.
pub trait VerificationKey: Send + Sync {
    fn key_id(&self) -> &str; // The kid associated with this verification key.
    fn public_key_jwk(&self) -> Result<serde_json::Value, AgentKeyError>;
    async fn verify_signature(&self, jws_payload: &[u8], jws_signature: &[u8], jws_protected_header: &JwsProtected) -> Result<bool, AgentKeyError>;
}

pub trait EncryptionKey: AgentKey {
    async fn encrypt(&self, plaintext: &[u8], aad: Option<&[u8]>) -> Result<Vec<u8>, AgentKeyError>;
    fn recommended_jwe_alg_enc(&self) -> (JweAlgorithm, JweEncryption);
}

pub trait DecryptionKey: AgentKey {
    async fn decrypt(&self, ciphertext: &[u8], aad: Option<&[u8]>) -> Result<Vec<u8>, AgentKeyError>;
}

// Example Struct for a local key
#[derive(Clone)] // If keys are to be cloned, secrets need careful handling (e.g. Arc)
pub struct LocalAgentKey {
    kid: String,
    secret: Arc<Secret>,
    // metadata like key type, curve, etc.
}

// Implement traits for LocalAgentKey
```

- **`AgentKey` Responsibilities:**
    - Storing or referencing key material.
    - Providing key identifiers (`kid`).
    - Exposing public key material (e.g., as JWK).
    - Managing algorithm selection (e.g., `recommended_jws_alg`, `recommended_jwe_alg_enc`).
    - Performing cryptographic operations (sign, verify, encrypt, decrypt).

### 3.2. `KeyManager` Refactor

- The `KeyManager` will be modified to store a collection of `Box<dyn AgentKey>` or similar (e.g., `Arc<dyn AgentKey>`), potentially specialized for signing and encryption (e.g., `Box<dyn SigningKey>`, `Box<dyn EncryptionKey>`).
- Methods like `add_secret` will be replaced or augmented by `add_agent_key(key: Box<dyn AgentKey>)`.
- New methods will be exposed by `KeyManager` for performing cryptographic operations, which will delegate to the appropriate `AgentKey` instance based on a `kid` or other criteria.

```rust
// Example KeyManager modification (conceptual)
pub struct KeyManager {
    // Using a map for easier lookup by kid
    signing_keys: Arc<RwLock<HashMap<String, Arc<dyn SigningKey + Send + Sync>>>>,
    encryption_keys: Arc<RwLock<HashMap<String, Arc<dyn EncryptionKey + Send + Sync>>>>,
    decryption_keys: Arc<RwLock<HashMap<String, Arc<dyn DecryptionKey + Send + Sync>>>>,
    // Potentially verification keys if they are managed separately or derived
}

impl KeyManager {
    // ... constructor, add_key methods ...

    pub async fn sign_with_key(&self, kid: &str, data: &[u8]) -> Result<Jws, KeyManagerError> { ... }
    pub async fn encrypt_with_key(&self, kid: &str, plaintext: &[u8], aad: Option<&[u8]>) -> Result<Jwe, KeyManagerError> { ... }
    // ... corresponding verify and decrypt methods ...
}
```

### 3.3. Relocation of `crypto.rs` Logic

- Functions currently in `tap-agent/src/crypto.rs` (e.g., `sign_message`, `encrypt_message`, `decrypt_message`, `verify_signature`) will be refactored.
- Their core logic for signing and encryption/decryption will move into the implementations of the `SigningKey`, `EncryptionKey`, and `DecryptionKey` traits (e.g., `LocalAgentKey`).
- The `verify_signature` logic will be integrated into the `unpack` method of the `Unpackable` trait implementation for the `Jws` struct (see Section 3.6). This method will use the `KeyManager` to resolve the necessary public verification key.
- `KeyManager` will become the primary entry point for cryptographic operations that require private key material (signing, decryption) or key resolution for public key operations (verification, encryption).
- `crypto.rs` might be significantly reduced, possibly repurposed for cryptographic utility functions or removed if all logic is cleanly integrated.

### 3.4. Remote `AgentKey` Implementation

A new struct, e.g., `RemoteAgentKey`, will implement the `AgentKey` (and sub-traits). Instead of holding private key material, it will hold configuration for connecting to a remote service (e.g., HSM endpoint, API key).

```rust
// Example RemoteAgentKey (conceptual)
pub struct RemoteAgentKey {
    kid: String,
    remote_service_client: Arc<RemoteCryptoServiceClient>, // Client for HSM/remote service
    // metadata
}

// Implement AgentKey traits for RemoteAgentKey, delegating calls to the remote service
```

### 3.5. Updating `Agent` Struct

The `Agent` struct will be updated to use the refactored `KeyManager` for all cryptographic needs. Direct calls to `crypto.rs` functions will be replaced with calls to `KeyManager` methods.

### 3.6. Message Packing and Unpacking Utilities

To standardize how messages are prepared for transmission (packed) and processed upon receipt (unpacked), we will introduce `Packable` and `Unpackable` traits. These traits will provide a consistent interface for converting between raw message objects (e.g., `didcomm::Message`) and their secured representations (JWS/JWE), or handling plaintext messages.

These utilities will reside in `tap-agent/src/message.rs`.

**Core Concepts:**

-   **`MessageError` Enum:** A dedicated error type for packing and unpacking operations, covering issues like serialization, cryptographic failures, invalid formats, or missing keys.
    ```rust
    #[derive(Debug, thiserror::Error)]
    pub enum MessageError {
        #[error("Serialization error: {0}")]
        Serialization(#[from] serde_json::Error),
        #[error("Key manager error: {0}")]
        KeyManager(String), // Or integrate with KeyManagerError
        #[error("Crypto operation failed: {0}")]
        Crypto(String), // Or integrate with AgentKeyError
        #[error("Invalid message format: {0}")]
        InvalidFormat(String),
        #[error("Unsupported security mode: {0:?}")]
        UnsupportedSecurityMode(SecurityMode),
        #[error("Missing required parameter: {0}")]
        MissingParameter(String),
        #[error("Key not found: {0}")]
        KeyNotFound(String),
        #[error("Verification failed")]
        VerificationFailed,
        #[error("Decryption failed")]
        DecryptionFailed,
    }
    ```

-   **`PackOptions` Struct:** Specifies parameters for packing a message.
    ```rust
    #[derive(Debug, Clone)]
    pub struct PackOptions {
        pub security_mode: SecurityMode,
        pub recipient_kid: Option<String>, // For JWE, identifies recipient's key
        pub sender_kid: Option<String>,    // For JWS (signer's key) and JWE (sender's key for authcrypt)
        // Potentially: JWE `alg` and `enc` if not derived from key, AAD, etc.
    }
    ```

-   **`UnpackOptions` Struct:** Specifies parameters for unpacking a message.
    ```rust
    #[derive(Debug, Clone)]
    pub struct UnpackOptions {
        // e.g., SecurityMode::Any to attempt unpacking as JWE then JWS, or a specific mode.
        pub expected_security_mode: SecurityMode,
        pub expected_recipient_kid: Option<String>, // To verify JWE recipient
        // Potentially: expected_issuer_kid for JWS
    }
    ```

-   **`Packable` Trait:**
    ```rust
    pub trait Packable<Output>: Sized {
        async fn pack(
            &self,
            key_manager: &KeyManager,      // For key access
            options: PackOptions,
        ) -> Result<Output, MessageError>; // Output could be Jws, Jwe, or String
    }
    ```
    -   Implemented by the raw message type (e.g., `didcomm::Message`).
    -   Based on `PackOptions.security_mode`, it will either:
        -   Return the message as a plain string/object (for `SecurityMode::Plain`).
        -   Sign the message using `KeyManager` and return a `Jws` object or its string representation.
        -   Encrypt (and potentially sign, i.e., authcrypt) the message using `KeyManager` and return a `Jwe` object or its string representation.

-   **`Unpackable` Trait:**
    ```rust
    pub trait Unpackable<Input>: Sized {
        async fn unpack(
            packed_message: &Input,        // Input could be &Jws, &Jwe, or &str
            key_manager: &KeyManager,      // For key access
            options: UnpackOptions,
        ) -> Result<didcomm::Message, MessageError>; // Returns the raw inner message
    }
    ```
    -   Implemented by `Jws` and `Jwe` structs (and potentially a wrapper for plain string messages).
    -   For `Jws`: Parses the JWS, uses the `KeyManager` to resolve the `kid` from the JWS header to a public `VerificationKey` (or its JWK). It then calls the `VerificationKey::verify_signature` method (or uses a crypto library directly with the JWK) to verify the signature against the JWS payload and protected header. If successful, it extracts and returns the payload (raw `didcomm::Message`).
    -   For `Jwe`: Decrypts the message using `KeyManager` (which internally uses an appropriate `DecryptionKey`) and extracts the plaintext (raw `didcomm::Message`).

**Integration:**

- The `Agent` struct will utilize these traits via helper methods or directly for preparing outgoing messages and processing incoming messages.
- This approach centralizes the logic for applying and removing security layers from messages, making the `Agent`'s message handling flow cleaner.

## 4. Key Management Interface (API)

**`KeyManager` API additions/changes:**

- `add_signing_key(key: Arc<dyn SigningKey + Send + Sync>)`
- `add_encryption_key(key: Arc<dyn EncryptionKey + Send + Sync>)`
- `add_decryption_key(key: Arc<dyn DecryptionKey + Send + Sync>)`
- `get_signing_key(kid: &str) -> Option<Arc<dyn SigningKey + Send + Sync>>`
- `get_encryption_key(kid: &str) -> Option<Arc<dyn EncryptionKey + Send + Sync>>`
- `get_decryption_key(kid: &str) -> Option<Arc<dyn DecryptionKey + Send + Sync>>`
- `resolve_verification_key(kid: &str) -> Result<Arc<dyn VerificationKey + Send + Sync>, KeyManagerError>` // New method for resolving verification keys

- `sign_jws(kid: &str, payload: &[u8], protected_header: Option<JwsHeader>) -> Result<String, KeyManagerError>` // Uses SigningKey
- `verify_jws(jws: &str, expected_kid: Option<&str>) -> Result<Vec<u8>, KeyManagerError>` // This method's role changes. It might become a convenience wrapper around `Jws::unpack()`, or be removed if `Jws::unpack()` is preferred.
- `encrypt_jwe(recipient_kid: &str, plaintext: &[u8], protected_header: Option<JweHeader>, aad: Option<&[u8]>) -> Result<String, KeyManagerError>` // Uses EncryptionKey (or resolves recipient's public key for encryption)
- `decrypt_jwe(jwe: &str, expected_kid: Option<&str>) -> Result<Vec<u8>, KeyManagerError>` // Uses DecryptionKey

**`AgentKey` related traits:** (as defined in 3.1)
- `AgentKey`
- `SigningKey`
- `VerificationKey` (Represents a public key or capability to verify signatures; resolved by `KeyManager`)
- `EncryptionKey`
- `DecryptionKey`

## 5. Impacted Modules

- `tap-agent/src/key_manager.rs`: Major refactor.
- `tap-agent/src/crypto.rs`: Significant reduction, potential removal or repurposing.
- `tap-agent/src/agent.rs`: Update to use the new `KeyManager` API.
- All test files related to cryptographic operations and key management.
- Examples using agent functionalities involving crypto.

## 6. Security Considerations

- **Secret Handling:** `LocalAgentKey` must continue to handle raw key material with utmost care, leveraging secure types like `didcomm::Secret` or similar, and ensuring secrets are not unnecessarily cloned or exposed.
- **Remote Key Security:** For `RemoteAgentKey`, communication with the external service must be secure (e.g., mTLS). Authentication and authorization mechanisms for accessing the remote service are critical.
- **Algorithm Agility:** The design should allow for the addition of new key types and cryptographic algorithms over time.
- **Error Handling:** Robust error handling is crucial to distinguish between local errors, cryptographic failures, and remote service errors.

## 7. Future Considerations

- **HSM Integration:** Detailed implementation of `RemoteAgentKey` for specific HSM vendors or generic HSM interfaces (e.g., PKCS#11).
- **Key Rotation:** Policies and mechanisms for key rotation managed through `KeyManager`.
- **Advanced Key Usages:** Support for more complex cryptographic protocols or key derivation schemes if needed.
- **Key Discovery/Registration:** How `KeyManager` discovers or allows registration of different `AgentKey` implementations (local vs. remote, different types).

## 8. Open Questions

1.  Should `VerificationKey` be a separate trait managed by `KeyManager`, or should verification rely on public key material (e.g., JWKs) obtained from `AgentKey` instances or DID documents? (Clarified: `VerificationKey` trait represents the capability, `KeyManager` resolves to it or its material. `Jws::unpack` orchestrates.)
2.  What is the best way to manage `kid` generation and ensure uniqueness?
3.  How should the `KeyManager` handle requests for operations where a suitable `AgentKey` (by `kid`) is not found?
4.  What specific JWS/JWE libraries will be used for constructing and parsing messages, and how will they integrate with `AgentKey` methods and `Unpackable` implementations?
5.  Error type design: A new `AgentKeyError` and potentially refined `KeyManagerError` will be needed.

## 9. Success Criteria

- All cryptographic operations (signing, verification, encryption, decryption) are performed via `KeyManager` and `AgentKey` instances.
- The `crypto.rs` module is either removed or its responsibilities significantly reduced to only fundamental crypto primitives if any.
- All existing tests related to agent cryptographic functions pass after the refactor.
- New unit tests for `AgentKey` implementations and `KeyManager` are added, achieving high code coverage.
- The design clearly supports the future implementation of a `RemoteAgentKey` for HSMs or other external services without requiring changes to the core `Agent` logic.
- The codebase is more modular, with clearer separation of concerns regarding key management and cryptographic operations.
- Documentation for `KeyManager` and `AgentKey` is updated.

## 10. Checklist (from prds/v1.md)

- [ ] **Feature Complete:** (To be checked upon completion)
- [ ] **Security Review:** (To be checked after implementation and review)
- [ ] **Testing:**
    - [ ] Unit Tests
    - [ ] Integration Tests
    - [ ] Fuzz Tests (if applicable)
- [ ] **Documentation:**
    - [ ] API Documentation
    - [ ] Usage Examples
- [ ] **Performance:**
    - [ ] Benchmarks (if applicable)
- [ ] **WASM Compatibility:** (Ensure changes are compatible or appropriately handled for WASM builds)
- [ ] **Code Quality:**
    - [ ] `cargo fmt`
    - [ ] `cargo clippy --all-targets --all-features -- -D warnings`
    - [ ] No new compiler warnings
- [ ] **Dependencies:** (Review any new dependencies for security and maintenance)
- [ ] **Protocol Compliance:** (Ensure changes align with any relevant DID/TAP/TAIP specs)
