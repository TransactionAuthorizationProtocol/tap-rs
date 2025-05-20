//! Benchmarks for TAP Agent functionality
//!
//! Run with: cargo bench --bench agent_benchmark

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tap_agent::did::MultiResolver;
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_agent::{
    Agent, AgentConfig, BasicSecretResolver, DefaultMessagePacker, SyncDIDResolver, TapAgent
};
use tap_caip::AssetId;
use tap_msg::{message::Transfer, Participant};

/// Create a test agent with known key material for benchmarking
fn create_test_agent() -> (Arc<TapAgent>, String) {
    // Create a DID for the agent - using a fixed DID for predictability
    let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string();

    // Create agent config
    let agent_config = AgentConfig::new(did.clone());

    // Create a secret resolver with the test key
    let mut secret_resolver = BasicSecretResolver::new();

    // Add a test Ed25519 key with correctly sized private key (32 bytes)
    let secret = Secret {
        id: format!("{}#keys-1", did),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "kid": format!("{}#keys-1", did),
                "crv": "Ed25519",
                "x": "F74Yk9BrwnXVJUEKwDxBfNjOElv1eIHr9QypeZ2DQQg",
                "d": "9kVnxrZlZW6V2MrNfcXUL8sAle/XX9XBbOxmHKFbvs4="
            }),
        },
    };

    secret_resolver.add_secret(&did, secret);

    // Create DID resolver
    let did_resolver: Arc<dyn SyncDIDResolver> = Arc::new(MultiResolver::default());

    // Create message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(
        did_resolver,
        Arc::new(secret_resolver),
        false,
    ));

    // Create agent
    let agent = Arc::new(TapAgent::new(agent_config, message_packer));

    (agent, did)
}

/// Create a test transfer message
fn create_transfer_message(from_did: &str, to_did: &str) -> Transfer {
    // Create originator and beneficiary participants
    let originator = Participant {
        id: from_did.to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let beneficiary = Participant {
        id: to_did.to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    // Create a transfer body
    Transfer {
        asset: AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f")
            .unwrap(),
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents: vec![],
        settlement_id: None,
        memo: Some("Benchmark test transfer".to_string()),
        metadata: HashMap::new(),
        transaction_id: "benchmark-transfer-id".to_string(),
    }
}

/// Benchmark message sending
fn bench_send_message(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("agent_send_message");

    // Create the test agent once, outside the benchmark loop
    let (agent, did) = create_test_agent();
    let transfer = rt.block_on(async { create_transfer_message(&did, &did) });

    group.bench_function(BenchmarkId::new("send", "transfer"), |b| {
        b.iter(|| {
            rt.block_on(async {
                // Send message
                let (_, _) = agent
                    .send_message(&transfer, vec![&did], false)
                    .await
                    .unwrap();
            });
        });
    });

    group.finish();
}

criterion_group!(agent_benches, bench_send_message);
criterion_main!(agent_benches);
