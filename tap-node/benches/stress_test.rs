//! Stress testing for TAP Node
//!
//! Run with: cargo bench --bench stress_test

use std::sync::Arc;
use std::time::Instant;
use tap_agent::{AgentConfig, TapAgent};
use tap_core::message::{TapMessageBuilder, TapMessageType};
use tap_node::message::ProcessorPoolConfig;
use tap_node::{NodeConfig, TapNode};

use criterion::{criterion_group, criterion_main, Criterion};

/// Create a test message
fn create_test_message(
    from_did: &str,
    to_did: &str,
    index: usize,
) -> tap_core::message::TapMessage {
    TapMessageBuilder::new()
        .id(format!("test-message-{}", index))
        .message_type(TapMessageType::Error)
        .from_did(Some(from_did.to_string()))
        .to_did(Some(to_did.to_string()))
        .body(serde_json::json!({
            "code": format!("TEST_ERROR_{}", index),
            "message": format!("Test error message {}", index),
            "transaction_id": None::<String>,
            "metadata": {
                "index": index,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }))
        .build()
        .unwrap()
}

fn stress_test(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup a node with processing pool
    let pool_config = ProcessorPoolConfig {
        max_concurrent_tasks: 16,
        queue_size: 1000,
    };

    let node_config = NodeConfig {
        debug: false,
        max_agents: None,
        enable_message_logging: false,
        log_message_content: false,
        processor_pool: Some(pool_config),
    };

    let agent1_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let agent2_did = "did:key:z6MkgYAFipwyqebCJagYs8XP6EPwXjiwLy8GZ6M1YyYAXMbh";

    let mut group = c.benchmark_group("tap_node_stress");
    group.sample_size(10); // Reduce sample size for stress tests

    // Test with different message batch sizes
    for &batch_size in &[10, 100, 1000] {
        group.bench_function(format!("process_{}_messages", batch_size), |b| {
            b.iter(|| {
                rt.block_on(async {
                    // Create a new node for each benchmark iteration
                    let node = TapNode::new(node_config.clone());

                    // Create and register agents
                    let agent_config1 = AgentConfig::new_with_did(agent1_did);
                    let agent1 = TapAgent::with_defaults(
                        agent_config1,
                        agent1_did.to_string(),
                        Some("Agent 1".to_string()),
                    )
                    .unwrap();

                    let agent_config2 = AgentConfig::new_with_did(agent2_did);
                    let agent2 = TapAgent::with_defaults(
                        agent_config2,
                        agent2_did.to_string(),
                        Some("Agent 2".to_string()),
                    )
                    .unwrap();

                    node.register_agent(Arc::new(agent1)).await.unwrap();
                    node.register_agent(Arc::new(agent2)).await.unwrap();

                    // Submit messages
                    let start = Instant::now();
                    let mut futures = Vec::with_capacity(batch_size);

                    for i in 0..batch_size {
                        let message = create_test_message(agent1_did, agent2_did, i);
                        futures.push(node.submit_message(message));
                    }

                    // Wait for all messages to be processed
                    futures::future::join_all(futures).await;

                    let duration = start.elapsed();
                    println!(
                        "Processed {} messages in {:.2?} ({:.2} msgs/sec)",
                        batch_size,
                        duration,
                        batch_size as f64 / duration.as_secs_f64()
                    );
                });
            })
        });
    }

    group.finish();
}

criterion_group!(benches, stress_test);
criterion_main!(benches);
