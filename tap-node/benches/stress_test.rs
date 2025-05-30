//! Stress testing for TAP Node
//!
//! Run with: cargo bench --bench stress_test

use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tap_agent::TapAgent;
use tap_msg::message::TapMessageBody;
use tap_msg::message::Transfer;
use tap_msg::PlainMessage;
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
) -> (PlainMessage, Transfer) {
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
            name: None,
        },
        beneficiary: Some(tap_msg::Participant {
            id: to_did.to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
            name: None,
        }),
        amount: format!("{}.00", index),
        agents: vec![],
        settlement_id: None,
        memo: Some(format!("Test message {}", index)),
        metadata: HashMap::new(),
        connection_id: None,
    };

    // Convert to DIDComm message
    let message = body.to_didcomm(from_did).unwrap();
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
        event_logger: None,
        #[cfg(feature = "storage")]
        storage_path: None,
        #[cfg(feature = "storage")]
        agent_did: None,
        #[cfg(feature = "storage")]
        tap_root: None,
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

                    // Create two ephemeral agents for the test
                    let (agent1, agent1_did) = TapAgent::from_ephemeral_key().await.unwrap();
                    let (agent2, agent2_did) = TapAgent::from_ephemeral_key().await.unwrap();

                    let agent1 = Arc::new(agent1);
                    let agent2 = Arc::new(agent2);

                    // Register the agents with the node
                    node.register_agent(agent1).await.unwrap();
                    node.register_agent(agent2).await.unwrap();

                    // Submit messages
                    let start = Instant::now();
                    let mut futures = Vec::with_capacity(batch_size);

                    for i in 0..batch_size {
                        let (message, _) = create_test_message(&agent1_did, &agent2_did, i).await;
                        let message_value = serde_json::to_value(message).unwrap();
                        futures.push(node.receive_message(message_value));
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
