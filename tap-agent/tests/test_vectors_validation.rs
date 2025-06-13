//! Test suite for validating TAP test vectors from prds/taips/test-vectors
//!
//! This test module reads all test vector JSON files and validates them
//! according to the TAP protocol specification.
//!
//! ## Known Issues
//! 
//! Several test vectors fail due to mismatches between the test vector format 
//! and the current tap-msg implementation:
//!
//! 1. **Field Naming Convention**: Test vectors use camelCase (e.g., `settlementAddress`)
//!    while Rust structs use snake_case (e.g., `settlement_address`) without proper
//!    serde rename attributes.
//!
//! 2. **Thread ID Mapping**: Fields marked with `#[tap(thread_id)]` in the Rust structs
//!    should map to the DIDComm message's `thid` field, not be part of the body.
//!    This test handles this by injecting the thread_id from thid into the body.
//!
//! 3. **Missing Required Fields**: Some test vectors are missing fields that the
//!    implementation marks as required (e.g., `by` field in Cancel messages).
//!
//! 4. **Different Message Structure**: Some messages like Transfer expect `originator`
//!    as a Party struct, but test vectors have it differently structured.
//!
//! ## Fixes Applied
//!
//! This test applies minimal fixes to handle known protocol differences:
//! 1. Injects `transaction_id` from `thid` for non-initiator messages
//! 2. Generates `transaction_id` for initiator messages using message ID

use serde::Deserialize;
use std::fs;
use std::path::Path;
use tap_msg::didcomm::PlainMessage;
use tap_msg::{
    AddAgents, Authorize, TapMessageBody, Transfer, Presentation, Reject, Settle,
    ErrorBody,
};
use tap_msg::message::{
    RemoveAgent, ReplaceAgent, ConfirmRelationship, Cancel, UpdateParty,
    UpdatePolicies, Revert, DIDCommPresentation, Payment, Connect, AuthorizationRequired
};

#[derive(Debug, Deserialize)]
struct TestVector {
    description: String,
    purpose: String,
    #[serde(rename = "shouldPass")]
    should_pass: bool,
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    taips: Vec<String>,
    message: PlainMessage,
    #[serde(rename = "expectedResult")]
    expected_result: ExpectedResult,
}

#[derive(Debug, Deserialize)]
struct ExpectedResult {
    valid: bool,
    #[serde(default)]
    #[allow(dead_code)]
    errors: Vec<ValidationError>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ValidationError {
    field: String,
    message: String,
}

/// Validate a TAP message based on its type
fn validate_tap_message(message: &PlainMessage) -> Result<(), String> {
    // Basic validation
    if message.id.is_empty() {
        return Err("Message ID is required".to_string());
    }
    
    // For messages that need thread_id, inject it from thid into the body for validation
    let mut body_with_thread_id = message.body.clone();
    
    // List of message types that use thread_id (non-initiator messages)
    let thread_id_messages = [
        "https://tap.rsvp/schema/1.0#Authorize",
        "https://tap.rsvp/schema/1.0#Reject", 
        "https://tap.rsvp/schema/1.0#Settle",
        "https://tap.rsvp/schema/1.0#Cancel",
        "https://tap.rsvp/schema/1.0#Revert",
        "https://tap.rsvp/schema/1.0#RemoveAgent",
        "https://tap.rsvp/schema/1.0#UpdateParty",
        "https://tap.rsvp/schema/1.0#UpdatePolicies",
        "https://tap.rsvp/schema/1.0#ConfirmRelationship",
        "https://tap.rsvp/schema/1.0#ReplaceAgent",
    ];
    
    // List of initiator messages that generate their own transaction_id
    let initiator_messages = [
        "https://tap.rsvp/schema/1.0#Transfer",
        "https://tap.rsvp/schema/1.0#Payment",
        "https://tap.rsvp/schema/1.0#Connect",
    ];
    
    if thread_id_messages.contains(&message.type_.as_str()) {
        if let Some(thid) = &message.thid {
            if let Some(obj) = body_with_thread_id.as_object_mut() {
                // Add transaction_id for most messages
                if message.type_ == "https://tap.rsvp/schema/1.0#ConfirmRelationship" {
                    obj.insert("transfer_id".to_string(), serde_json::Value::String(thid.clone()));
                } else {
                    obj.insert("transaction_id".to_string(), serde_json::Value::String(thid.clone()));
                }
            }
        }
    } else if initiator_messages.contains(&message.type_.as_str()) {
        // For initiator messages, generate a transaction_id if not present
        if let Some(obj) = body_with_thread_id.as_object_mut() {
            if !obj.contains_key("transaction_id") {
                // Use the message ID as transaction_id for test purposes
                obj.insert("transaction_id".to_string(), serde_json::Value::String(message.id.clone()));
            }
        }
    }
    
    // Validate body based on message type
    match message.type_.as_str() {
        "https://tap.rsvp/schema/1.0#Transfer" => {
            let transfer: Transfer = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse Transfer: {}", e))?;
            transfer.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#Presentation" => {
            let presentation: Presentation = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse Presentation: {}", e))?;
            presentation.validate().map_err(|e| e.to_string())
        }
        "https://didcomm.org/present-proof/3.0/presentation" => {
            let didcomm_presentation: DIDCommPresentation = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse DIDCommPresentation: {}", e))?;
            didcomm_presentation.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#Authorize" => {
            let authorize: Authorize = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse Authorize: {}", e))?;
            authorize.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#Reject" => {
            let reject: Reject = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse Reject: {}", e))?;
            reject.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#Settle" => {
            let settle: Settle = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse Settle: {}", e))?;
            settle.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#AddAgents" => {
            let add_agents: AddAgents = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse AddAgents: {}", e))?;
            add_agents.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#RemoveAgent" => {
            let remove_agent: RemoveAgent = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse RemoveAgent: {}", e))?;
            remove_agent.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#ReplaceAgent" => {
            let replace_agent: ReplaceAgent = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse ReplaceAgent: {}", e))?;
            replace_agent.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#Error" => {
            let error: ErrorBody = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse Error: {}", e))?;
            error.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#ConfirmRelationship" => {
            let confirm: ConfirmRelationship = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse ConfirmRelationship: {}", e))?;
            confirm.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#Cancel" => {
            let cancel: Cancel = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse Cancel: {}", e))?;
            cancel.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#UpdateParty" => {
            let update_party: UpdateParty = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse UpdateParty: {}", e))?;
            update_party.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#UpdatePolicies" => {
            let update_policies: UpdatePolicies = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse UpdatePolicies: {}", e))?;
            update_policies.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#Revert" => {
            let revert: Revert = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse Revert: {}", e))?;
            revert.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#Payment" => {
            let payment: Payment = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse Payment: {}", e))?;
            payment.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#Connect" => {
            let connect: Connect = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse Connect: {}", e))?;
            connect.validate().map_err(|e| e.to_string())
        }
        "https://tap.rsvp/schema/1.0#AuthorizationRequired" => {
            let auth_required: AuthorizationRequired = serde_json::from_value(body_with_thread_id.clone())
                .map_err(|e| format!("Failed to parse AuthorizationRequired: {}", e))?;
            auth_required.validate().map_err(|e| e.to_string())
        }
        "https://didcomm.org/out-of-band/2.0/invitation" => {
            // Out-of-band messages don't have body validation
            Ok(())
        }
        _ => {
            // For unknown types, we can't perform specific validation
            Ok(())
        }
    }
}

fn load_test_vectors_from_directory(dir_path: &Path) -> Vec<(String, TestVector)> {
    let mut test_vectors = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Skip README files
                if path.file_name().unwrap().to_str().unwrap().starts_with("README") {
                    continue;
                }
                
                if let Ok(contents) = fs::read_to_string(&path) {
                    match serde_json::from_str::<TestVector>(&contents) {
                        Ok(test_vector) => {
                            let test_name = format!(
                                "{}::{}",
                                dir_path.file_name().unwrap().to_str().unwrap(),
                                path.file_stem().unwrap().to_str().unwrap()
                            );
                            test_vectors.push((test_name, test_vector));
                        }
                        Err(e) => {
                            eprintln!(
                                "Failed to parse test vector {}: {}",
                                path.display(),
                                e
                            );
                        }
                    }
                }
            } else if path.is_dir() {
                // Recursively load from subdirectories
                test_vectors.extend(load_test_vectors_from_directory(&path));
            }
        }
    }
    
    test_vectors
}

#[test]
fn validate_all_test_vectors() {
    let test_vectors_dir = Path::new("../prds/taips/test-vectors");
    let test_vectors = load_test_vectors_from_directory(test_vectors_dir);
    
    assert!(!test_vectors.is_empty(), "No test vectors found!");
    
    let mut passed = 0;
    let mut failed = 0;
    let mut unexpected_results = Vec::new();
    
    for (test_name, test_vector) in &test_vectors {
        println!("\nRunning test: {}", test_name);
        println!("  Description: {}", test_vector.description);
        println!("  Purpose: {}", test_vector.purpose);
        println!("  Should Pass: {}", test_vector.should_pass);
        
        let validation_result = validate_tap_message(&test_vector.message);
        let is_valid = validation_result.is_ok();
        
        println!("  Validation Result: {}", if is_valid { "VALID" } else { "INVALID" });
        
        if let Err(e) = &validation_result {
            println!("  Error: {}", e);
        }
        
        // Check if the result matches expected
        if is_valid == test_vector.expected_result.valid {
            println!("  ✓ Result matches expected");
            passed += 1;
        } else {
            println!("  ✗ Result does NOT match expected!");
            println!("    Expected valid={}, got valid={}", test_vector.expected_result.valid, is_valid);
            failed += 1;
            unexpected_results.push((test_name.clone(), test_vector.expected_result.valid, is_valid));
        }
    }
    
    println!("\n========== TEST SUMMARY ==========");
    println!("Total test vectors: {}", test_vectors.len());
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    
    if !unexpected_results.is_empty() {
        println!("\nFailed tests:");
        for (name, expected, actual) in &unexpected_results {
            println!("  - {}: expected valid={}, got valid={}", name, expected, actual);
        }
        panic!("{} test vectors produced unexpected results", failed);
    }
    
    println!("\nAll test vectors validated successfully! ✓");
}

#[test]
fn validate_specific_message_types() {
    // Test specific message types individually
    let test_vectors_dir = Path::new("../prds/taips/test-vectors");
    
    // Test Transfer messages
    test_message_type(test_vectors_dir, "transfer", "Transfer");
    
    // Test Authorize messages
    test_message_type(test_vectors_dir, "authorize", "Authorize");
    
    // Test Presentation messages
    test_message_type(test_vectors_dir, "presentation", "Presentation");
    
    // Test Reject messages
    test_message_type(test_vectors_dir, "reject", "Reject");
    
    // Test Settle messages
    test_message_type(test_vectors_dir, "settle", "Settle");
    
    // Test agent management messages
    test_message_type(test_vectors_dir, "add-agents", "AddAgents");
    test_message_type(test_vectors_dir, "remove-agent", "RemoveAgent");
    test_message_type(test_vectors_dir, "replace-agent", "ReplaceAgent");
}

fn test_message_type(base_dir: &Path, dir_name: &str, message_type: &str) {
    let type_dir = base_dir.join(dir_name);
    if !type_dir.exists() {
        println!("Skipping {} tests - directory not found", message_type);
        return;
    }
    
    println!("\n========== Testing {} messages ==========", message_type);
    let test_vectors = load_test_vectors_from_directory(&type_dir);
    
    for (test_name, test_vector) in &test_vectors {
        println!("  {} - {}", test_name, if validate_tap_message(&test_vector.message).is_ok() == test_vector.expected_result.valid { "✓" } else { "✗" });
    }
}

#[test]
fn test_invalid_messages_fail() {
    let test_vectors_dir = Path::new("../prds/taips/test-vectors");
    let test_vectors = load_test_vectors_from_directory(test_vectors_dir);
    
    // Filter for test vectors that should fail
    let invalid_vectors: Vec<_> = test_vectors
        .into_iter()
        .filter(|(_, tv)| !tv.expected_result.valid)
        .collect();
    
    println!("\nTesting {} invalid test vectors", invalid_vectors.len());
    
    for (test_name, test_vector) in &invalid_vectors {
        let validation_result = validate_tap_message(&test_vector.message);
        assert!(
            validation_result.is_err(),
            "Test {} should have failed validation but passed",
            test_name
        );
        println!("  {} correctly failed validation ✓", test_name);
    }
}

#[test]
fn test_valid_messages_pass() {
    let test_vectors_dir = Path::new("../prds/taips/test-vectors");
    let test_vectors = load_test_vectors_from_directory(test_vectors_dir);
    
    // Filter for test vectors that should pass
    let valid_vectors: Vec<_> = test_vectors
        .into_iter()
        .filter(|(_, tv)| tv.expected_result.valid)
        .collect();
    
    println!("\nTesting {} valid test vectors", valid_vectors.len());
    
    for (test_name, test_vector) in &valid_vectors {
        let validation_result = validate_tap_message(&test_vector.message);
        if let Err(e) = &validation_result {
            println!("  {} failed with error: {}", test_name, e);
        }
        assert!(
            validation_result.is_ok(),
            "Test {} should have passed validation but failed: {:?}",
            test_name,
            validation_result.err()
        );
        println!("  {} correctly passed validation ✓", test_name);
    }
}