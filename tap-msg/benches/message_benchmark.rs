//! Benchmarks for TAP Core message handling
//!
//! This file benchmarks the conversion between TAP message bodies
//! and DIDComm messages, as well as serialization/deserialization.
//!
//! Run with: cargo bench --bench message_benchmark

use criterion::{criterion_group, criterion_main, Criterion};
use didcomm::Message;
use std::collections::HashMap;
use tap_caip::AssetId;
use tap_msg::message::{Authorize, Participant, Reject, Settle, TapMessageBody, Transfer};

// Configure bench
criterion_group!(
    benches,
    bench_to_didcomm,
    bench_from_didcomm,
    bench_serialize_deserialize
);
criterion_main!(benches);

/// Create a test Transfer message body
fn create_transfer_body() -> Transfer {
    // Create participant information
    let originator = Participant {
        id: "did:example:alice".to_string(),
        role: Some("originator".to_string()),
    };

    let beneficiary = Participant {
        id: "did:example:bob".to_string(),
        role: Some("beneficiary".to_string()),
    };

    // Create asset ID properly - using a valid Ethereum address format
    let asset = AssetId::new(
        tap_caip::ChainId::new("eip155", "1").unwrap(),
        "erc20",
        "0x1234567890abcdef1234567890abcdef12345678",
    )
    .unwrap();

    // Return the transfer body
    Transfer {
        asset,
        originator,
        beneficiary: Some(beneficiary),
        amount: "10.00".to_string(),
        participants: vec![],
        settlement_id: None,
        memo: Some("Test transfer".to_string()),
        metadata: HashMap::new(),
    }
}

/// Create a test Authorize message body
fn create_authorize_body() -> Authorize {
    Authorize {
        transfer_id: "test-transfer-id".to_string(),
        note: Some("Transfer authorized".to_string()),
        metadata: HashMap::new(),
    }
}

/// Create a test Reject message body
fn create_reject_body() -> Reject {
    Reject {
        transfer_id: "test-transfer-id".to_string(),
        code: "COMPLIANCE_FAILURE".to_string(),
        description: "Unable to comply with transfer requirements".to_string(),
        note: Some("Further documentation needed".to_string()),
        metadata: HashMap::new(),
    }
}

/// Create a test Settle message body
fn create_settle_body() -> Settle {
    Settle {
        transfer_id: "test-transfer-id".to_string(),
        transaction_id: "0xabcdef1234567890".to_string(),
        transaction_hash: Some("0xabcdef1234567890".to_string()),
        block_height: Some(12345678),
        note: Some("Transaction completed".to_string()),
        metadata: HashMap::new(),
    }
}

/// Benchmark converting TAP message bodies to DIDComm messages
fn bench_to_didcomm(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_to_didcomm");

    // Create test message bodies
    let transfer_body = create_transfer_body();
    let authorize_body = create_authorize_body();
    let reject_body = create_reject_body();
    let settle_body = create_settle_body();

    // Benchmark Transfer messages
    group.bench_function("transfer", |b| {
        b.iter(|| {
            let _: Message = transfer_body.to_didcomm().unwrap();
        })
    });

    // Benchmark Authorize messages
    group.bench_function("authorize", |b| {
        b.iter(|| {
            let _: Message = authorize_body.to_didcomm().unwrap();
        })
    });

    // Benchmark Reject messages
    group.bench_function("reject", |b| {
        b.iter(|| {
            let _: Message = reject_body.to_didcomm().unwrap();
        })
    });

    // Benchmark Settle messages
    group.bench_function("settle", |b| {
        b.iter(|| {
            let _: Message = settle_body.to_didcomm().unwrap();
        })
    });

    group.finish();
}

/// Benchmark converting DIDComm messages to TAP message bodies
fn bench_from_didcomm(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_from_didcomm");

    // Create test message bodies and convert to DIDComm messages
    let transfer_body = create_transfer_body();
    let transfer_message = transfer_body.to_didcomm().unwrap();

    let authorize_body = create_authorize_body();
    let authorize_message = authorize_body.to_didcomm().unwrap();

    let reject_body = create_reject_body();
    let reject_message = reject_body.to_didcomm().unwrap();

    let settle_body = create_settle_body();
    let settle_message = settle_body.to_didcomm().unwrap();

    // Benchmark Transfer messages
    group.bench_function("transfer", |b| {
        b.iter(|| {
            let _: Transfer = Transfer::from_didcomm(&transfer_message).unwrap();
        })
    });

    // Benchmark Authorize messages
    group.bench_function("authorize", |b| {
        b.iter(|| {
            let _: Authorize = Authorize::from_didcomm(&authorize_message).unwrap();
        })
    });

    // Benchmark Reject messages
    group.bench_function("reject", |b| {
        b.iter(|| {
            let _: Reject = Reject::from_didcomm(&reject_message).unwrap();
        })
    });

    // Benchmark Settle messages
    group.bench_function("settle", |b| {
        b.iter(|| {
            let _: Settle = Settle::from_didcomm(&settle_message).unwrap();
        })
    });

    group.finish();
}

/// Benchmark serialization and deserialization of TAP message bodies
fn bench_serialize_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_serialize_deserialize");

    // Create test message bodies
    let transfer_body = create_transfer_body();
    let authorize_body = create_authorize_body();
    let reject_body = create_reject_body();
    let settle_body = create_settle_body();

    // Serialize the bodies to JSON
    let transfer_json = serde_json::to_string(&transfer_body).unwrap();
    let authorize_json = serde_json::to_string(&authorize_body).unwrap();
    let reject_json = serde_json::to_string(&reject_body).unwrap();
    let settle_json = serde_json::to_string(&settle_body).unwrap();

    // Benchmark Transfer serialization
    group.bench_function("transfer_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(&transfer_body).unwrap();
        })
    });

    // Benchmark Transfer deserialization
    group.bench_function("transfer_deserialize", |b| {
        b.iter(|| {
            let _: Transfer = serde_json::from_str(&transfer_json).unwrap();
        })
    });

    // Benchmark Authorize serialization
    group.bench_function("authorize_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(&authorize_body).unwrap();
        })
    });

    // Benchmark Authorize deserialization
    group.bench_function("authorize_deserialize", |b| {
        b.iter(|| {
            let _: Authorize = serde_json::from_str(&authorize_json).unwrap();
        })
    });

    // Benchmark Reject serialization
    group.bench_function("reject_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(&reject_body).unwrap();
        })
    });

    // Benchmark Reject deserialization
    group.bench_function("reject_deserialize", |b| {
        b.iter(|| {
            let _: Reject = serde_json::from_str(&reject_json).unwrap();
        })
    });

    // Benchmark Settle serialization
    group.bench_function("settle_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(&settle_body).unwrap();
        })
    });

    // Benchmark Settle deserialization
    group.bench_function("settle_deserialize", |b| {
        b.iter(|| {
            let _: Settle = serde_json::from_str(&settle_json).unwrap();
        })
    });

    group.finish();
}
