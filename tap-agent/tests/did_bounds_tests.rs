//! Tests for DID resolution bounds checking
//!
//! These tests verify that malformed DIDs do not cause panics.

#[cfg(feature = "native")]
use tap_agent::did::DIDDoc;
#[cfg(feature = "native")]
use tap_agent::{MultiResolver, SyncDIDResolver};

/// Test that WebResolver handles empty domain without panic
#[cfg(feature = "native")]
#[tokio::test]
async fn test_web_resolver_empty_domain_no_panic() {
    let resolver = MultiResolver::new();

    // Empty domain after did:web:
    let result: Result<Option<DIDDoc>, _> = resolver.resolve("did:web:").await;
    // Should return Ok(None) or Err, not panic
    assert!(
        result.is_ok() || result.is_err(),
        "did:web: with empty domain should not panic"
    );
}

/// Test that WebResolver handles malformed paths without panic
#[cfg(feature = "native")]
#[tokio::test]
async fn test_web_resolver_malformed_path_no_panic() {
    let resolver = MultiResolver::new();

    let malformed_dids = vec![
        "did:web:",           // Empty domain
        "did:web",            // Missing trailing colon
        "did:web::",          // Double colon, empty segment
        "did:web:::",         // Triple colon
        "did:web:example.com::path", // Double colon in path
    ];

    for did in malformed_dids {
        let result: Result<Option<DIDDoc>, _> = resolver.resolve(did).await;
        // Should handle gracefully, not panic
        assert!(
            result.is_ok() || result.is_err(),
            "DID '{}' should not cause panic",
            did
        );
    }
}

/// Test that KeyResolver handles short DIDs without panic
#[cfg(feature = "native")]
#[tokio::test]
async fn test_key_resolver_short_did_no_panic() {
    let resolver = MultiResolver::new();

    let short_dids = vec![
        "",
        "d",
        "di",
        "did",
        "did:",
        "did:k",
        "did:ke",
        "did:key",
        "did:key:",
    ];

    for did in short_dids {
        // This should not panic - Ok or Err is fine
        let result: Result<Option<DIDDoc>, _> = resolver.resolve(did).await;
        // Just accessing the result proves it didn't panic
        let _ = result;
    }
}

/// Test WASM KeyResolver with short DIDs (sync version)
#[cfg(target_arch = "wasm32")]
#[test]
fn test_key_resolver_short_did_wasm_no_panic() {
    use tap_agent::did::{DIDMethodResolver, KeyResolver};

    let resolver = KeyResolver::new();

    let short_dids = vec!["", "d", "did", "did:key", "did:key:"];

    for did in short_dids {
        let result = resolver.resolve_method(did);
        assert!(
            result.is_ok(),
            "DID '{}' should not cause panic in WASM",
            did
        );
    }
}
