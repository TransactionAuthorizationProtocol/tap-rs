//! Benchmarks for TAP Agent functionality
//!
//! Run with: cargo bench --bench agent_benchmark

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tap_agent::{Agent, TapAgent};
use tap_caip::AssetId;
use tap_msg::{message::Transfer, Participant};

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
    // Create originator and beneficiary participants
    let originator = Participant {
        id: from_did.to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let beneficiary = Participant {
        id: to_did.to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
        name: None,
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
