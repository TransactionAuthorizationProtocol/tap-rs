//! Benchmarks for tap-wasm native binding functions
//!
//! These benchmarks test the Rust functions that will be exposed to WASM
//! without the overhead of the WASM bridge, to measure core performance.
//!
//! Run with: cargo bench --bench wasm_binding_benchmark

use criterion::{criterion_group, criterion_main, Criterion};
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::message::{Authorize, Party, Reject, TapMessageBody, Transfer};
use tap_msg::PlainMessage;

/// Create a test transfer message body
fn create_transfer_body() -> Transfer {
    // Create originator and beneficiary parties
    let originator = Party::new("did:example:alice");

    let beneficiary = Party::new("did:example:bob");

    // Create a transfer body using the builder pattern
    Transfer::builder()
        .asset(
            AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        )
        .originator(originator)
        .beneficiary(beneficiary)
        .amount("100.0".to_string())
        .memo("Benchmark test transfer".to_string())
        .transaction_id("benchmark-transfer-id".to_string())
        .build()
}

/// Create a message for testing
fn create_test_message() -> PlainMessage {
    let transfer_body = create_transfer_body();
    transfer_body.to_didcomm("did:example:alice").unwrap()
}

/// Benchmark JSON serialization and deserialization performance
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_serialization");

    // Create a test message
    let message = create_test_message();

    // Test JSON serialization (mimics WASM export)
    group.bench_function("message_to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&message).unwrap();
            assert!(!json.is_empty());
        })
    });

    // Serialize once for deserialization test
    let json = serde_json::to_string(&message).unwrap();

    // Test JSON deserialization (mimics WASM import)
    group.bench_function("json_to_message", |b| {
        b.iter(|| {
            let _: PlainMessage = serde_json::from_str(&json).unwrap();
        })
    });

    group.finish();
}

/// Benchmark message type detection and conversion
fn bench_message_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_message_conversion");

    // Create messages of different types
    let transfer_body = create_transfer_body();
    let transfer_message = transfer_body.to_didcomm("did:example:alice").unwrap();

    let authorize_body = Authorize {
        transaction_id: "test-transfer-id".to_string(),
        settlement_address: None,
        expiry: None,
    };
    let authorize_message = authorize_body.to_didcomm("did:example:alice").unwrap();

    let reject_body = Reject {
        transaction_id: "test-transfer-id".to_string(),
        reason:
            "COMPLIANCE_FAILURE: Unable to comply with transfer requirements. Rejected for testing."
                .to_string(),
    };
    let reject_message = reject_body.to_didcomm("did:example:alice").unwrap();

    // Serialize messages
    let transfer_json = serde_json::to_string(&transfer_message).unwrap();
    let authorize_json = serde_json::to_string(&authorize_message).unwrap();
    let reject_json = serde_json::to_string(&reject_message).unwrap();

    // Benchmark message type detection and conversion
    group.bench_function("detect_transfer", |b| {
        b.iter(|| {
            let parsed: PlainMessage = serde_json::from_str(&transfer_json).unwrap();
            let _: Transfer = Transfer::from_didcomm(&parsed).unwrap();
        })
    });

    group.bench_function("detect_authorize", |b| {
        b.iter(|| {
            let parsed: PlainMessage = serde_json::from_str(&authorize_json).unwrap();
            let _: Authorize = Authorize::from_didcomm(&parsed).unwrap();
        })
    });

    group.bench_function("detect_reject", |b| {
        b.iter(|| {
            let parsed: PlainMessage = serde_json::from_str(&reject_json).unwrap();
            let _: Reject = Reject::from_didcomm(&parsed).unwrap();
        })
    });

    group.finish();
}

/// Benchmark common WASM operations
fn bench_wasm_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_operations");

    // Benchmark string handling
    let test_string = "did:example:alice".to_string();

    group.bench_function("string_clone", |b| {
        b.iter(|| {
            let cloned = test_string.clone();
            assert_eq!(cloned, "did:example:alice");
        })
    });

    // Benchmark JSON object creation (common for WASM bridge)
    group.bench_function("create_json_object", |b| {
        b.iter(|| {
            let obj = serde_json::json!({
                "id": "test-id",
                "from": "did:example:alice",
                "to": ["did:example:bob"],
                "type": "https://example.com/protocols/test/1.0",
                "body": {
                    "amount": "10.00",
                    "note": "Test payment"
                }
            });
            assert!(obj.is_object());
        })
    });

    group.finish();
}

criterion_group!(
    wasm_benches,
    bench_serialization,
    bench_message_conversion,
    bench_wasm_operations
);
criterion_main!(wasm_benches);
