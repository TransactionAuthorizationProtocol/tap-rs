use didcomm::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;
use tap_msg::message::{DIDCommPresentation, TapMessageBody};
use tap_msg::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestVector {
    description: String,
    purpose: String,
    #[serde(rename = "shouldPass")]
    should_pass: bool,
    version: String,
    taips: Vec<String>,
    message: Value,
    #[serde(rename = "expectedResult")]
    expected_result: ExpectedResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExpectedResult {
    valid: bool,
    #[serde(default)]
    errors: Vec<TestVectorError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestVectorError {
    field: String,
    message: String,
}

/// Load test vectors from a directory
fn load_test_vectors(directory: &str) -> Vec<TestVector> {
    // Use the project root as the base path
    let root_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let path = root_path.join(directory);

    if !path.exists() || !path.is_dir() {
        panic!("Test vector directory not found: {:?}", path);
    }

    let mut test_vectors = Vec::new();

    // Load all JSON files in the directory
    for entry in fs::read_dir(path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
            let file_content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read file: {:?}", path));

            match serde_json::from_str::<TestVector>(&file_content) {
                Ok(test_vector) => test_vectors.push(test_vector),
                Err(e) => eprintln!("Failed to parse test vector {:?}: {}", path, e),
            }
        }
    }

    test_vectors
}

/// Test presentation message against a test vector
fn test_presentation_message(test_vector: &TestVector) -> Result<()> {
    // Convert the message to a DIDComm Message
    let didcomm_result = serde_json::from_str::<Message>(&test_vector.message.to_string());

    // First check - whether the message can be parsed as a DIDComm message
    if let Err(parse_error) = &didcomm_result {
        if !test_vector.should_pass {
            // This is expected failure for invalid format tests
            println!("✅ Expected parsing failure: {}", parse_error);
            return Ok(());
        } else {
            return Err(tap_msg::Error::Validation(format!(
                "Test vector '{}' should pass but failed to parse as DIDComm: {}",
                test_vector.description, parse_error
            )));
        }
    }

    // If we got here, message parsing succeeded
    let didcomm_message = didcomm_result.unwrap();

    // Convert to DIDCommPresentation
    let presentation_result = DIDCommPresentation::from_didcomm(&didcomm_message);

    // Second check - whether the message can be converted to DIDCommPresentation
    if let Err(conversion_error) = &presentation_result {
        if !test_vector.should_pass {
            // This is expected failure for invalid message content
            println!("✅ Expected conversion failure: {}", conversion_error);
            return Ok(());
        } else {
            return Err(tap_msg::Error::Validation(format!(
                "Test vector '{}' should pass but failed conversion: {}",
                test_vector.description, conversion_error
            )));
        }
    }

    // If we got here, conversion succeeded
    let presentation = presentation_result.unwrap();

    // Third check - validate the presentation
    let validation_result = presentation.validate();

    if let Err(validation_error) = &validation_result {
        if !test_vector.should_pass {
            // This is expected failure for messages that should fail validation
            println!("✅ Expected validation failure: {}", validation_error);
            return Ok(());
        } else {
            return Err(tap_msg::Error::Validation(format!(
                "Test vector '{}' should pass but failed validation: {}",
                test_vector.description, validation_error
            )));
        }
    }

    // If we got here, validation succeeded

    // For vectors that should fail but passed all checks so far, we have an issue
    if !test_vector.should_pass {
        return Err(tap_msg::Error::Validation(format!(
            "Test vector '{}' should fail but passed all checks",
            test_vector.description
        )));
    }

    // Final check for passing vectors - test round trip conversion
    let to_didcomm_result = presentation.to_didcomm(None);

    if let Err(round_trip_error) = &to_didcomm_result {
        return Err(tap_msg::Error::Validation(format!(
            "Test vector '{}' passed validation but failed round-trip conversion: {}",
            test_vector.description, round_trip_error
        )));
    }

    Ok(())
}

#[tokio::test]
async fn test_presentation_vectors() {
    let test_vectors = load_test_vectors("prds/taips/test-vectors/presentation");
    println!("Loaded {} test vectors", test_vectors.len());

    let mut success_count = 0;
    let mut error_count = 0;

    // Process each test vector
    for test_vector in &test_vectors {
        println!("\nTesting: {}", test_vector.description);
        println!("Should pass: {}", test_vector.should_pass);

        match test_presentation_message(test_vector) {
            Ok(_) => {
                println!("✅ Passed: {}", test_vector.description);
                success_count += 1;
            }
            Err(e) => {
                eprintln!("❌ Failed: {} - {}", test_vector.description, e);
                error_count += 1;
            }
        }
    }

    println!(
        "\nTest vectors summary: {} passed, {} failed, {} total",
        success_count,
        error_count,
        test_vectors.len()
    );

    // Ensure all tests passed
    assert_eq!(error_count, 0, "{} test vectors failed", error_count);
}

#[tokio::test]
async fn test_presentation_round_trip() {
    // Test with a valid presentation test vector only
    let test_vectors = load_test_vectors("prds/taips/test-vectors/presentation");

    // Find a valid test vector
    let valid_vector = test_vectors
        .iter()
        .find(|v| v.should_pass)
        .expect("No valid presentation test vector found");

    // Test round-trip conversion
    let message_str = valid_vector.message.to_string();
    let didcomm_message: Message =
        serde_json::from_str(&message_str).expect("Failed to parse message as DIDComm");

    // Convert to DIDCommPresentation
    let presentation = DIDCommPresentation::from_didcomm(&didcomm_message)
        .expect("Failed to convert DIDComm message to DIDCommPresentation");

    // Convert back to DIDComm
    let round_trip_message = presentation
        .to_didcomm("did:example:sender")
        .expect("Failed to convert DIDCommPresentation back to DIDComm message");

    // Convert again to DIDCommPresentation to verify integrity
    let round_trip_presentation = DIDCommPresentation::from_didcomm(&round_trip_message)
        .expect("Failed to convert round-trip DIDComm message back to DIDCommPresentation");

    // Verify key properties match
    assert_eq!(
        presentation.thid, round_trip_presentation.thid,
        "Thread ID does not match after round trip"
    );

    assert_eq!(
        presentation.attachments.len(),
        round_trip_presentation.attachments.len(),
        "Attachment count does not match after round trip"
    );

    // Verify the first attachment's properties if available
    if !presentation.attachments.is_empty() {
        assert_eq!(
            presentation.attachments[0].id, round_trip_presentation.attachments[0].id,
            "Attachment ID does not match after round trip"
        );

        assert_eq!(
            presentation.attachments[0].media_type,
            round_trip_presentation.attachments[0].media_type,
            "Attachment media type does not match after round trip"
        );
    }
}
