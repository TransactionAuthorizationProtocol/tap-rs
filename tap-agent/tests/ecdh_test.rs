use base64::Engine;
use p256::ecdh::EphemeralSecret;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use rand::rngs::OsRng;

#[test]
fn test_ephemeral_key_generation() {
    // Generate an ephemeral key pair for ECDH
    let ephemeral_secret = EphemeralSecret::random(&mut OsRng);
    let ephemeral_public_key = ephemeral_secret.public_key();

    // Convert the public key to coordinates
    let point = ephemeral_public_key.to_encoded_point(false); // Uncompressed format
    let x_bytes = point.x().unwrap().to_vec();
    let y_bytes = point.y().unwrap().to_vec();

    // Base64 encode the coordinates for the ephemeral public key
    let x_b64 = base64::engine::general_purpose::STANDARD.encode(&x_bytes);
    let y_b64 = base64::engine::general_purpose::STANDARD.encode(&y_bytes);

    // Print the coordinates
    println!("x coordinate (base64): {}", x_b64);
    println!("y coordinate (base64): {}", y_b64);

    // Verify the coordinates are valid base64 and have appropriate length for P-256
    assert!(!x_b64.is_empty(), "x coordinate should not be empty");
    assert!(!y_b64.is_empty(), "y coordinate should not be empty");

    // Try decoding the base64 to make sure they're valid
    let x_decoded = base64::engine::general_purpose::STANDARD
        .decode(&x_b64)
        .unwrap();
    let y_decoded = base64::engine::general_purpose::STANDARD
        .decode(&y_b64)
        .unwrap();

    // P-256 coordinates should be 32 bytes each
    assert_eq!(x_decoded.len(), 32, "x coordinate should be 32 bytes");
    assert_eq!(y_decoded.len(), 32, "y coordinate should be 32 bytes");

    println!("✅ Ephemeral key generation test passed");
}
