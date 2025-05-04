//! Benchmarks for TAP Agent functionality
//!
//! Run with: cargo bench --bench agent_benchmark

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use didcomm::secrets::{Secret, SecretMaterial, SecretType};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tap_agent::did::MultiResolver;
use tap_agent::{Agent, AgentConfig, BasicSecretResolver, DefaultAgent, DefaultMessagePacker};
use tap_caip::AssetId;
use tap_msg::{message::Transfer, Participant};

/// Create a test agent with a fresh keypair
async fn create_test_agent() -> (Arc<DefaultAgent>, String) {
    // Create a test DID
    let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string();

    // Create agent config
    let agent_config = AgentConfig::new(did.clone());

    // Create a secret resolver with the test key
    let mut secret_resolver = BasicSecretResolver::new();

    // Add a test Ed25519 key
    let secret = Secret {
        id: format!("{}#keys-1", did),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "kid": format!("{}#keys-1", did),
                "crv": "Ed25519",
                "x": "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo",
                "d": "nWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Yh"
            }),
        },
    };

    secret_resolver.add_secret(&did, secret);

    // Create DID resolver
    let did_resolver = Arc::new(MultiResolver::default());

    // Create message packer
    let message_packer = Arc::new(DefaultMessagePacker::new(
        did_resolver,
        Arc::new(secret_resolver),
    ));

    // Create agent
    let agent = Arc::new(DefaultAgent::new(agent_config, message_packer));

    (agent, did)
}

/// Create a test transfer message
async fn create_transfer_message(from_did: &str, to_did: &str) -> Transfer {
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
    }
}

/// Benchmark message sending
fn bench_send_message(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("agent_send_message");

    group.bench_function(BenchmarkId::new("send", "transfer"), |b| {
        b.iter(|| {
            rt.block_on(async {
                // Create agents
                let (agent1, did1) = create_test_agent().await;
                let (_, did2) = create_test_agent().await;

                // Create transfer message
                let transfer = create_transfer_message(&did1, &did2).await;

                // Send message
                let _ = agent1.send_message(&transfer, &did2).await.unwrap();
            });
        });
    });

    group.finish();
}

/// Benchmark message packing
fn bench_message_packing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("agent_message_packing");

    group.bench_function(BenchmarkId::new("pack", "transfer"), |b| {
        b.iter(|| {
            rt.block_on(async {
                // Create agents
                let (agent1, did1) = create_test_agent().await;
                let (agent2, did2) = create_test_agent().await;

                // Create transfer message
                let transfer = create_transfer_message(&did1, &did2).await;

                // Send and receive message
                let packed = agent1.send_message(&transfer, &did2).await.unwrap();
                let _: Transfer = agent2.receive_message(&packed).await.unwrap();
            });
        });
    });

    group.finish();
}

criterion_group!(agent_benches, bench_send_message, bench_message_packing);
criterion_main!(agent_benches);
