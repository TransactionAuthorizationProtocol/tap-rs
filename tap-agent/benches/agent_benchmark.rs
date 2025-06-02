//! Benchmarks for TAP Agent functionality
//!
//! Run with: cargo bench --bench agent_benchmark

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tap_agent::{Agent, TapAgent};
use tap_caip::AssetId;
use tap_msg::message::{Party, Transfer};
use uuid::Uuid;

/// Create a test agent with ephemeral key for benchmarking
fn create_test_agent() -> (Arc<TapAgent>, String) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Create agent with ephemeral key
    let (agent, did) = rt.block_on(async {
        let (agent, did) = TapAgent::from_ephemeral_key().await.unwrap();
        (Arc::new(agent), did)
    });

    (agent, did)
}

/// Create a test transfer message
fn create_transfer_message(from_did: &str, to_did: &str) -> Transfer {
    // Create originator and beneficiary parties
    let originator = Party::new(from_did);

    let beneficiary = Party::new(to_did);

    // Create a transfer body using the builder pattern
    Transfer::builder()
        .asset(
            AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        )
        .originator(originator)
        .beneficiary(beneficiary)
        .amount("100.00".to_string())
        .transaction_id(format!("test-{}", Uuid::new_v4()))
        .build()
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
