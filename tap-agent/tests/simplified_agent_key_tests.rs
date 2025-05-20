use tap_agent::{
    agent_key::{AgentKey, SigningKey, VerificationKey, JwsAlgorithm},
    error::Result,
};
use serde_json::Value;

/// Basic test to check that we can import and use the AgentKey traits
#[tokio::test]
async fn test_agent_key_trait_imports() -> Result<()> {
    // This test just verifies that we can import and use the AgentKey traits
    // The types and traits should be defined correctly
    
    // Create a simple struct that implements AgentKey for testing
    struct TestKey {
        id: String,
        did: String,
    }
    
    impl std::fmt::Debug for TestKey {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TestKey")
                .field("id", &self.id)
                .field("did", &self.did)
                .finish()
        }
    }
    
    impl AgentKey for TestKey {
        fn key_id(&self) -> &str {
            &self.id
        }
        
        fn public_key_jwk(&self) -> Result<Value> {
            // Return a dummy JWK
            Ok(serde_json::json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "kid": self.id,
                "x": "base64url-encoded-public-key"
            }))
        }
        
        fn did(&self) -> &str {
            &self.did
        }
        
        fn key_type(&self) -> &str {
            "Ed25519"
        }
    }
    
    // Create a test key
    let test_key = TestKey {
        id: "test-key-1".to_string(),
        did: "did:key:z6Mkw4Kh1MgzkBsNzSMVviFGTqsjqzeYL4Bktj8BAAjGsF8R".to_string(),
    };
    
    // Verify the key properties
    assert_eq!(test_key.key_id(), "test-key-1");
    assert_eq!(test_key.did(), "did:key:z6Mkw4Kh1MgzkBsNzSMVviFGTqsjqzeYL4Bktj8BAAjGsF8R");
    assert_eq!(test_key.key_type(), "Ed25519");
    
    // Get the JWK
    let jwk = test_key.public_key_jwk()?;
    assert_eq!(jwk["kid"], "test-key-1");
    
    Ok(())
}