//! Stress testing for TAP Node
//!
//! Run with: cargo bench --bench stress_test

use didcomm::secrets::{Secret, SecretMaterial, SecretType};
use didcomm::Message as DIDCommMessage;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tap_agent::did::MultiResolver;
use tap_agent::{AgentConfig, BasicSecretResolver, DefaultAgent, DefaultMessagePacker};
use tap_msg::message::TapMessageBody;
use tap_msg::message::Transfer;
use tap_node::message::processor_pool::ProcessorPoolConfig;
use tap_node::{NodeConfig, TapNode};

use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use tokio::time::Duration;

/// Create a test message
async fn create_test_message(
    from_did: &str,
    to_did: &str,
    index: usize,
) -> (DIDCommMessage, Transfer) {
    // Create a simple transfer message
    let body = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset: tap_caip::AssetId::from_str(
            "eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f",
        )
        .unwrap(),
        originator: tap_msg::Participant {
            id: from_did.to_string(),
            role: Some("originator".to_string()),
            policies: None,
            leiCode: None,
        },
        beneficiary: Some(tap_msg::Participant {
            id: to_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
        }),
        amount: format!("{}.00", index),
        agents: vec![],
        settlement_id: None,
        memo: Some(format!("Test message {}", index)),
        metadata: HashMap::new(),
    };

    // Convert to DIDComm message
    let message = body.to_didcomm(Some(from_did)).unwrap();
    (message, body)
}

fn stress_test(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup a node with processing pool
    let pool_config = ProcessorPoolConfig {
        workers: 16,
        channel_capacity: 1000,
        worker_timeout: Duration::from_secs(30),
    };

    let node_config = NodeConfig {
        debug: false,
        max_agents: None,
        enable_message_logging: false,
        log_message_content: false,
        processor_pool: Some(pool_config),
    };

    // For testing, we'll create some DIDs that don't rely on external resolvers
    let mut group = c.benchmark_group("tap_node_stress");
    group.sample_size(10); // Reduce sample size for stress tests

    // Test with different message batch sizes
    for &batch_size in &[10, 100, 1000] {
        group.bench_function(format!("process_{}_messages", batch_size), |b| {
            b.iter(|| {
                rt.block_on(async {
                    // Create a new node for each benchmark iteration
                    let node = TapNode::new(node_config.clone());

                    // Create and register two test agents
                    // Create a test DID and keys for agent 1
                    let agent1_did =
                        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string();
                    let agent2_did =
                        "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp".to_string();

                    // Create secret resolvers
                    let mut resolver1 = BasicSecretResolver::new();
                    let mut resolver2 = BasicSecretResolver::new();

                    // Add test keys
                    let secret1 = Secret {
                        id: format!("{}#keys-1", agent1_did),
                        type_: SecretType::JsonWebKey2020,
                        secret_material: SecretMaterial::JWK {
                            private_key_jwk: serde_json::json!({
                                "kty": "OKP",
                                "kid": format!("{}#keys-1", agent1_did),
                                "crv": "Ed25519",
                                "x": "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo",
                                "d": "nWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Yh"
                            }),
                        },
                    };

                    let secret2 = Secret {
                        id: format!("{}#keys-1", agent2_did),
                        type_: SecretType::JsonWebKey2020,
                        secret_material: SecretMaterial::JWK {
                            private_key_jwk: serde_json::json!({
                                "kty": "OKP",
                                "kid": format!("{}#keys-1", agent2_did),
                                "crv": "Ed25519",
                                "x": "G1j6ccPJpxWGqOLEEUQYqrNHRF7xBWqfZPjxWQ2Aj6c",
                                "d": "Hk-QnVOyk2yd6PkY5-qpqJdJHe_lDmn7-3RVsLZR9QI"
                            }),
                        },
                    };

                    resolver1.add_secret(&agent1_did, secret1);
                    resolver2.add_secret(&agent2_did, secret2);

                    // Create DID resolver
                    let did_resolver1 = Arc::new(MultiResolver::default());
                    let did_resolver2 = Arc::new(MultiResolver::default());

                    // Create agents
                    let agent1_config = AgentConfig::new(agent1_did.clone());
                    let message_packer1 = Arc::new(DefaultMessagePacker::new(
                        did_resolver1,
                        Arc::new(resolver1),
                    ));
                    let agent1 = Arc::new(DefaultAgent::new(agent1_config, message_packer1));

                    let agent2_config = AgentConfig::new(agent2_did.clone());
                    let message_packer2 = Arc::new(DefaultMessagePacker::new(
                        did_resolver2,
                        Arc::new(resolver2),
                    ));
                    let agent2 = Arc::new(DefaultAgent::new(agent2_config, message_packer2));

                    // Register the agents with the node
                    node.register_agent(agent1).await.unwrap();
                    node.register_agent(agent2).await.unwrap();

                    // Submit messages
                    let start = Instant::now();
                    let mut futures = Vec::with_capacity(batch_size);

                    for i in 0..batch_size {
                        let (message, _) = create_test_message(&agent1_did, &agent2_did, i).await;
                        futures.push(node.receive_message(message));
                    }

                    // Wait for all messages to be processed
                    for future in futures {
                        let _ = future.await;
                    }

                    let duration = start.elapsed();
                    println!(
                        "Processed {} messages in {:?} ({:.2} msg/s)",
                        batch_size,
                        duration,
                        batch_size as f64 / duration.as_secs_f64()
                    );
                });
            });
        });
    }

    group.finish();
}

criterion_group!(benches, stress_test);
criterion_main!(benches);
