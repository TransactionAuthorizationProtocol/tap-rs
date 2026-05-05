//! Integration tests for TAIP-20 memo-hash helpers.
//!
//! TAIP-20 defines a deterministic on-chain correlation primitive:
//! `tap_hash = SHA-256(UTF8(transfer_id))`. The hash can be carried in two
//! profiles: a text profile (`tap:1:<64-lowercase-hex>`) and a 32-byte
//! binary profile.

use tap_msg::utils::memo_hash::{
    encode_binary_memo, encode_text_memo, tap_memo_hash, verify_binary_memo, verify_text_memo,
};

const TRANSFER_ID: &str = "3fa85f64-5717-4562-b3fc-2c963f66afa6";

/// SHA-256 of `b"3fa85f64-5717-4562-b3fc-2c963f66afa6"`, computed with
/// `shasum -a 256`. Used as the canonical reference vector for these tests.
const EXPECTED_HEX: &str = "c7aa09cd25da8b6ab686f96da282d29ce9a7a2a0d7c27e1e359eb2cac6fbfaaf";

#[test]
fn test_tap_memo_hash_matches_reference_vector() {
    let hash = tap_memo_hash(TRANSFER_ID);
    assert_eq!(hex::encode(hash), EXPECTED_HEX);
}

#[test]
fn test_encode_text_memo_produces_canonical_form() {
    let memo = encode_text_memo(TRANSFER_ID);
    assert_eq!(memo, format!("tap:1:{}", EXPECTED_HEX));

    // The hex tail MUST be exactly 64 lowercase hex chars.
    let tail = memo.strip_prefix("tap:1:").unwrap();
    assert_eq!(tail.len(), 64);
    assert!(tail.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
}

#[test]
fn test_encode_binary_memo_returns_32_bytes() {
    let bytes = encode_binary_memo(TRANSFER_ID);
    assert_eq!(bytes.len(), 32);
    assert_eq!(hex::encode(bytes), EXPECTED_HEX);
}

#[test]
fn test_verify_text_memo_accepts_canonical_form() {
    let memo = encode_text_memo(TRANSFER_ID);
    assert!(verify_text_memo(&memo, TRANSFER_ID));
}

#[test]
fn test_verify_text_memo_rejects_wrong_transfer_id() {
    let memo = encode_text_memo(TRANSFER_ID);
    assert!(!verify_text_memo(&memo, "other-transfer-id"));
}

#[test]
fn test_verify_text_memo_rejects_missing_prefix() {
    // Hex without the `tap:1:` prefix MUST NOT verify.
    assert!(!verify_text_memo(EXPECTED_HEX, TRANSFER_ID));
}

#[test]
fn test_verify_text_memo_rejects_uppercase_hex() {
    let upper = format!("tap:1:{}", EXPECTED_HEX.to_uppercase());
    assert!(!verify_text_memo(&upper, TRANSFER_ID));
}

#[test]
fn test_verify_text_memo_rejects_truncated_hex() {
    let truncated = format!("tap:1:{}", &EXPECTED_HEX[..32]);
    assert!(!verify_text_memo(&truncated, TRANSFER_ID));
}

#[test]
fn test_verify_text_memo_rejects_wrong_version_prefix() {
    let v2 = format!("tap:2:{}", EXPECTED_HEX);
    assert!(!verify_text_memo(&v2, TRANSFER_ID));
}

#[test]
fn test_verify_binary_memo_accepts_32_byte_hash() {
    let bytes = encode_binary_memo(TRANSFER_ID);
    assert!(verify_binary_memo(&bytes, TRANSFER_ID));
}

#[test]
fn test_verify_binary_memo_rejects_wrong_length() {
    let bytes = encode_binary_memo(TRANSFER_ID);
    assert!(!verify_binary_memo(&bytes[..16], TRANSFER_ID));
}

#[test]
fn test_verify_binary_memo_rejects_wrong_transfer_id() {
    let bytes = encode_binary_memo(TRANSFER_ID);
    assert!(!verify_binary_memo(&bytes, "other-transfer-id"));
}

#[test]
fn test_distinct_transfer_ids_produce_distinct_hashes() {
    let a = tap_memo_hash("transfer-a");
    let b = tap_memo_hash("transfer-b");
    assert_ne!(a, b);
}
