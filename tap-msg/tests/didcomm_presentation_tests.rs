use didcomm::Message;
use serde_json::{json, Value};
use std::collections::HashMap;
use tap_msg::message::{Attachment, AttachmentData, DIDCommPresentation, TapMessageBody};

#[tokio::test]
async fn test_didcomm_presentation_deserialization() {
    // Create a sample DIDComm present-proof message based on the test vector format
    let message_json = json!({
        "from": "did:web:originator.vasp",
        "type": "https://didcomm.org/present-proof/3.0/presentation",
        "id": "f1ca8245-ab2d-4d9c-8d7d-94bf310314ef",
        "thid": "95e63a5f-73e1-46ac-b269-48bb22591bfa",
        "to": ["did:web:beneficiary.vasp"],
        "created_time": 1516269022,
        "expires_time": 1516385931,
        "body": {
            "comment": "Here is the requested presentation",
            "goal_code": "kyc.beneficiary.individual"
        },
        "attachments": [
            {
                "id": "2a3f1c4c-623c-44e6-b159-179048c51260",
                "media_type": "application/json",
                "format": "dif/presentation-exchange/submission@v1.0",
                "data": {
                    "json": {
                        "@context": [
                            "https://www.w3.org/2018/credentials/v1",
                            "https://identity.foundation/presentation-exchange/submission/v1"
                        ],
                        "type": [
                            "VerifiablePresentation",
                            "PresentationSubmission"
                        ],
                        "presentation_submission": {
                            "id": "a30e3b91-fb77-4d22-95fa-871689c322e2",
                            "definition_id": "32f54163-7166-48f1-93d8-ff217bdb0653",
                            "descriptor_map": [
                                {
                                    "id": "beneficiary_vp",
                                    "format": "jwt_vc",
                                    "path": "$.verifiableCredential[0]"
                                }
                            ]
                        },
                        "verifiableCredential": [
                            {
                                "@context": ["https://www.w3.org/2018/credentials/v1","https://schema.org/Person"],
                                "type": ["VerifiableCredential", "Person"],
                                "issuer": "did:web:originator.vasp",
                                "issuanceDate": "2022-01-01T19:23:24Z",
                                "credentialSubject": {
                                    "id": "did:eg:bob",
                                    "givenName": "Bob",
                                    "familyName": "Smith"
                                }
                            }
                        ]
                    }
                }
            }
        ]
    });

    // Convert to a DIDComm Message
    let message_str = message_json.to_string();
    let message: Message = serde_json::from_str(&message_str).expect("Failed to parse message");

    // Test deserialization
    let presentation = DIDCommPresentation::from_didcomm(&message)
        .expect("Failed to convert to DIDCommPresentation");

    // Verify the presentation attributes
    assert_eq!(
        presentation.thid,
        Some("95e63a5f-73e1-46ac-b269-48bb22591bfa".to_string())
    );
    assert_eq!(
        presentation.comment,
        Some("Here is the requested presentation".to_string())
    );
    assert_eq!(
        presentation.goal_code,
        Some("kyc.beneficiary.individual".to_string())
    );
    assert_eq!(presentation.attachments.len(), 1);
    assert_eq!(
        presentation.attachments[0].id,
        "2a3f1c4c-623c-44e6-b159-179048c51260"
    );
    assert_eq!(presentation.attachments[0].media_type, "application/json");
}

#[tokio::test]
async fn test_didcomm_presentation_validation() {
    // Create a valid presentation
    let presentation = DIDCommPresentation {
        thid: Some("test-thread-id".to_string()),
        comment: Some("Test comment".to_string()),
        goal_code: Some("test.goal".to_string()),
        attachments: vec![Attachment {
            id: "test-attachment-id".to_string(),
            media_type: "application/json".to_string(),
            data: Some(AttachmentData {
                base64: None,
                json: Some(json!({
                    "test": "data"
                })),
            }),
        }],
        metadata: HashMap::new(),
    };

    // Validate the presentation
    assert!(presentation.validate().is_ok());

    // Create an invalid presentation (no attachments)
    let invalid_presentation = DIDCommPresentation {
        thid: Some("test-thread-id".to_string()),
        comment: Some("Test comment".to_string()),
        goal_code: Some("test.goal".to_string()),
        attachments: vec![],
        metadata: HashMap::new(),
    };

    // Validation should fail
    assert!(invalid_presentation.validate().is_err());

    // Create an invalid presentation (empty attachment ID)
    let invalid_presentation2 = DIDCommPresentation {
        thid: Some("test-thread-id".to_string()),
        comment: Some("Test comment".to_string()),
        goal_code: Some("test.goal".to_string()),
        attachments: vec![Attachment {
            id: "".to_string(),
            media_type: "application/json".to_string(),
            data: None,
        }],
        metadata: HashMap::new(),
    };

    // Validation should fail
    assert!(invalid_presentation2.validate().is_err());
}

#[tokio::test]
async fn test_didcomm_presentation_to_didcomm() {
    // Create a presentation
    let presentation = DIDCommPresentation {
        thid: Some("test-thread-id".to_string()),
        comment: Some("Test comment".to_string()),
        goal_code: Some("test.goal".to_string()),
        attachments: vec![Attachment {
            id: "test-attachment-id".to_string(),
            media_type: "application/json".to_string(),
            data: Some(AttachmentData {
                base64: None,
                json: Some(json!({
                    "test": "data"
                })),
            }),
        }],
        metadata: HashMap::new(),
    };

    // Convert to DIDComm message
    let message = presentation
        .to_didcomm()
        .expect("Failed to convert to DIDComm");

    // Verify message attributes
    assert_eq!(
        message.type_,
        "https://didcomm.org/present-proof/3.0/presentation"
    );
    assert_eq!(message.thid, Some("test-thread-id".to_string()));

    // Verify body contains comment and goal_code
    let body = message.body.as_object().unwrap();
    assert_eq!(
        body.get("comment").unwrap().as_str().unwrap(),
        "Test comment"
    );
    assert_eq!(
        body.get("goal_code").unwrap().as_str().unwrap(),
        "test.goal"
    );

    // Verify attachments
    assert!(message.attachments.is_some());
    let attachments = message.attachments.as_ref().unwrap();
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].id, Some("test-attachment-id".to_string()));
    assert_eq!(
        attachments[0].media_type,
        Some("application/json".to_string())
    );
}

#[tokio::test]
async fn test_round_trip_conversion() {
    // Create a presentation
    let original = DIDCommPresentation {
        thid: Some("test-thread-id".to_string()),
        comment: Some("Test comment".to_string()),
        goal_code: Some("test.goal".to_string()),
        attachments: vec![Attachment {
            id: "test-attachment-id".to_string(),
            media_type: "application/json".to_string(),
            data: Some(AttachmentData {
                base64: None,
                json: Some(json!({
                    "test": "data"
                })),
            }),
        }],
        metadata: HashMap::new(),
    };

    // Convert to DIDComm message
    let message = original.to_didcomm().expect("Failed to convert to DIDComm");

    // Convert back to DIDCommPresentation
    let roundtrip = DIDCommPresentation::from_didcomm(&message)
        .expect("Failed to convert back to DIDCommPresentation");

    // Verify attributes match
    assert_eq!(roundtrip.thid, original.thid);
    assert_eq!(roundtrip.comment, original.comment);
    assert_eq!(roundtrip.goal_code, original.goal_code);
    assert_eq!(roundtrip.attachments.len(), original.attachments.len());
    assert_eq!(roundtrip.attachments[0].id, original.attachments[0].id);
    assert_eq!(
        roundtrip.attachments[0].media_type,
        original.attachments[0].media_type
    );
}
