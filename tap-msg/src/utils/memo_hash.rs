//! TAIP-20 on-chain transfer correlation via memo hash.
//!
//! TAIP-20 defines a chain-agnostic correlation primitive that ties a TAP
//! transfer to its on-chain settlement. The primitive is `tap_hash =
//! SHA-256(UTF8(transfer_id))`. The hash is then carried on-chain in either:
//!
//! - **Profile A (text):** the canonical form `tap:1:<64-lowercase-hex>` placed
//!   in a UTF-8 memo / reference / comment field.
//! - **Profile B (binary):** the raw 32-byte hash placed in a fixed-length
//!   binary memo field.
//!
//! See `prds/taips/TAIPs/taip-20.md` for the full spec.

use sha2::{Digest, Sha256};

/// Compute the canonical TAP correlation hash for a transfer ID:
/// `SHA-256(UTF8(transfer_id))`.
pub fn tap_memo_hash(transfer_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(transfer_id.as_bytes());
    hasher.finalize().into()
}

/// Encode a transfer ID as a Profile A (text) memo: `tap:1:<64-lowercase-hex>`.
pub fn encode_text_memo(transfer_id: &str) -> String {
    format!("tap:1:{}", hex::encode(tap_memo_hash(transfer_id)))
}

/// Encode a transfer ID as a Profile B (binary) memo: the raw 32-byte hash.
pub fn encode_binary_memo(transfer_id: &str) -> [u8; 32] {
    tap_memo_hash(transfer_id)
}

/// Verify that a Profile A text memo correlates to the given transfer ID.
///
/// The memo MUST start with the literal prefix `tap:1:` and be followed by
/// exactly 64 lowercase hex characters that match `SHA-256(transfer_id)`.
/// Uppercase hex, truncated hex, or alternative version prefixes are
/// rejected per the spec.
pub fn verify_text_memo(memo: &str, transfer_id: &str) -> bool {
    let Some(tail) = memo.strip_prefix("tap:1:") else {
        return false;
    };
    if tail.len() != 64 {
        return false;
    }
    if tail.chars().any(|c| c.is_ascii_uppercase()) {
        return false;
    }
    let Ok(bytes) = hex::decode(tail) else {
        return false;
    };
    bytes.as_slice() == tap_memo_hash(transfer_id).as_slice()
}

/// Verify that a Profile B binary memo (raw bytes) correlates to the given
/// transfer ID. Memos that are not exactly 32 bytes long are rejected.
pub fn verify_binary_memo(memo: &[u8], transfer_id: &str) -> bool {
    memo.len() == 32 && memo == tap_memo_hash(transfer_id).as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TRANSFER_ID: &str = "3fa85f64-5717-4562-b3fc-2c963f66afa6";
    const SAMPLE_HEX: &str = "c7aa09cd25da8b6ab686f96da282d29ce9a7a2a0d7c27e1e359eb2cac6fbfaaf";

    #[test]
    fn text_and_binary_profiles_agree() {
        let text = encode_text_memo(SAMPLE_TRANSFER_ID);
        let bin = encode_binary_memo(SAMPLE_TRANSFER_ID);

        assert_eq!(text, format!("tap:1:{}", SAMPLE_HEX));
        assert_eq!(hex::encode(bin), SAMPLE_HEX);
    }

    #[test]
    fn round_trip_text_verifies() {
        let memo = encode_text_memo(SAMPLE_TRANSFER_ID);
        assert!(verify_text_memo(&memo, SAMPLE_TRANSFER_ID));
    }

    #[test]
    fn round_trip_binary_verifies() {
        let memo = encode_binary_memo(SAMPLE_TRANSFER_ID);
        assert!(verify_binary_memo(&memo, SAMPLE_TRANSFER_ID));
    }
}
