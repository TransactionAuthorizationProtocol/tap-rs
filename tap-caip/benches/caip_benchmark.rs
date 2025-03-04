//! Benchmarks for CAIP parsing and validation
//!
//! Run with: cargo bench --bench caip_benchmark

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use tap_caip::{ChainId, AccountId, AssetId};
use std::str::FromStr;

/// Benchmark for parsing and validating CAIP-2 Chain IDs
fn bench_chain_id(c: &mut Criterion) {
    let mut group = c.benchmark_group("caip_chain_id");
    
    // Test a variety of chain IDs
    let chain_ids = vec![
        "eip155:1",           // Ethereum Mainnet
        "eip155:137",         // Polygon
        "bip122:000000000019d6689c085ae165831e93", // Bitcoin
        "cosmos:cosmoshub-4", // Cosmos Hub
        "solana:4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ",  // Solana
        "polkadot:91b171bb158e2d3848fa23a9f1c25182", // Polkadot
    ];
    
    for chain_id in chain_ids {
        group.bench_with_input(
            BenchmarkId::new("parse", chain_id),
            &chain_id,
            |b, &chain_id| {
                b.iter(|| {
                    let _: ChainId = ChainId::from_str(chain_id).unwrap();
                })
            }
        );
        
        let parsed = ChainId::from_str(chain_id).unwrap();
        group.bench_with_input(
            BenchmarkId::new("to_string", chain_id),
            &parsed,
            |b, parsed| {
                b.iter(|| {
                    let _: String = parsed.to_string();
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark for parsing and validating CAIP-10 Account IDs
fn bench_account_id(c: &mut Criterion) {
    let mut group = c.benchmark_group("caip_account_id");
    
    // Test a variety of account IDs
    let account_ids = vec![
        "eip155:1:0x1234567890123456789012345678901234567890",  // Ethereum
        "eip155:137:0xabcdefabcdefabcdefabcdefabcdefabcdefabcd", // Polygon
        "bip122:000000000019d6689c085ae165831e93:128Lkh3S7CkDTBZ8W7BbpsN3YYizJMp8p6", // Bitcoin
        "cosmos:cosmoshub-4:cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd", // Cosmos
        "solana:4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ:mvines9iiHiQTysrwkJjGf2gb9Ex9jXJX8ns3qwf2kN", // Solana
    ];
    
    for account_id in account_ids {
        group.bench_with_input(
            BenchmarkId::new("parse", account_id),
            &account_id,
            |b, &account_id| {
                b.iter(|| {
                    let _: AccountId = AccountId::from_str(account_id).unwrap();
                })
            }
        );
        
        let parsed = AccountId::from_str(account_id).unwrap();
        group.bench_with_input(
            BenchmarkId::new("to_string", account_id),
            &parsed,
            |b, parsed| {
                b.iter(|| {
                    let _: String = parsed.to_string();
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark for parsing and validating CAIP-19 Asset IDs
fn bench_asset_id(c: &mut Criterion) {
    let mut group = c.benchmark_group("caip_asset_id");
    
    // Test a variety of asset IDs - make sure they match the expected format
    let asset_ids = vec![
        "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", // USDC on Ethereum
        "eip155:137/erc20:0x2791bca1f2de4661ed88a30c99a7a9449aa84174", // USDC on Polygon
        "cosmos:cosmoshub-4/slip44:118", // ATOM
    ];
    
    for asset_id in asset_ids {
        group.bench_with_input(
            BenchmarkId::new("parse", asset_id),
            &asset_id,
            |b, &asset_id| {
                b.iter(|| {
                    let _: AssetId = AssetId::from_str(asset_id).unwrap();
                })
            }
        );
        
        let parsed = AssetId::from_str(asset_id).unwrap();
        group.bench_with_input(
            BenchmarkId::new("to_string", asset_id),
            &parsed,
            |b, parsed| {
                b.iter(|| {
                    let _: String = parsed.to_string();
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark for validation functions of CAIP identifiers
fn bench_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("caip_validation");
    
    // CAIP-2 Chain ID validation
    let valid_chain_ids = vec![
        "eip155:1",
        "eip155:137",
        "bip122:000000000019d6689c085ae165831e93",
        "cosmos:cosmoshub-4",
    ];
    let invalid_chain_ids = vec![
        "eip155", // Missing reference
        "eip155:", // Empty reference
        ":1", // Missing namespace
        "eip155:1:extra", // Too many parts
        "INVALID:1", // Uppercase namespace
    ];
    
    group.bench_function("validate_chain_ids", |b| {
        b.iter(|| {
            for chain_id in &valid_chain_ids {
                let result = ChainId::from_str(chain_id).is_ok();
                assert!(result);
            }
            for chain_id in &invalid_chain_ids {
                let result = ChainId::from_str(chain_id).is_ok();
                assert!(!result);
            }
        })
    });
    
    // CAIP-10 Account ID validation
    let valid_account_ids = vec![
        "eip155:1:0x1234567890123456789012345678901234567890",
        "cosmos:cosmoshub-4:cosmos1t2uflqwqe0fsj0shcfkrvpukewcw40yjj6hdc0",
    ];
    let invalid_account_ids = vec![
        "eip155:1", // Missing address
        "eip155:1:", // Empty address
        "eip155::0x1234", // Missing chain reference
        ":1:0x1234", // Missing namespace
    ];
    
    group.bench_function("validate_account_ids", |b| {
        b.iter(|| {
            for account_id in &valid_account_ids {
                let result = AccountId::from_str(account_id).is_ok();
                assert!(result);
            }
            for account_id in &invalid_account_ids {
                let result = AccountId::from_str(account_id).is_ok();
                assert!(!result);
            }
        })
    });
    
    // CAIP-19 Asset ID validation
    let valid_asset_ids = vec![
        "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "cosmos:cosmoshub-4/slip44:118",
    ];
    let invalid_asset_ids = vec![
        "eip155:1", // Missing asset portion
        "eip155:1/", // Empty asset
        "eip155:1/erc20:", // Missing asset reference
        "/erc20:0x1234", // Missing chain ID
    ];
    
    group.bench_function("validate_asset_ids", |b| {
        b.iter(|| {
            for asset_id in &valid_asset_ids {
                let result = AssetId::from_str(asset_id).is_ok();
                assert!(result);
            }
            for asset_id in &invalid_asset_ids {
                let result = AssetId::from_str(asset_id).is_ok();
                assert!(!result);
            }
        })
    });
    
    group.finish();
}

criterion_group!(
    caip_benches, 
    bench_chain_id, 
    bench_account_id, 
    bench_asset_id,
    bench_validation
);
criterion_main!(caip_benches);
