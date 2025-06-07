use tap_node::storage::{ReceivedStatus, SourceType, Storage};
use tempfile::tempdir;

#[tokio::test]
async fn test_received_table_operations() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage = Storage::new(Some(db_path)).await.unwrap();

    // Test creating received records for different message types
    let plain_message = r#"{"id":"msg1","type":"test","body":{}}"#;
    let jws_message =
        r#"{"payload":"eyJ0ZXN0IjoidGVzdCJ9","signatures":[{"protected":"eyJ0ZXN0IjoidGVzdCJ9"}]}"#;
    let jwe_message = r#"{"protected":"eyJ0ZXN0IjoidGVzdCJ9","recipients":[{"header":{"kid":"did:key:test#key1"}}]}"#;

    // Create received records
    let plain_id = storage
        .create_received(plain_message, SourceType::Internal, Some("did:key:sender1"))
        .await
        .unwrap();

    let jws_id = storage
        .create_received(
            jws_message,
            SourceType::Https,
            Some("https://example.com/sender"),
        )
        .await
        .unwrap();

    let jwe_id = storage
        .create_received(
            jwe_message,
            SourceType::WebSocket,
            Some("ws://example.com:8080"),
        )
        .await
        .unwrap();

    // Verify records were created
    assert!(plain_id > 0);
    assert!(jws_id > 0);
    assert!(jwe_id > 0);

    // Get pending received messages
    let pending = storage.get_pending_received(10).await.unwrap();
    assert_eq!(pending.len(), 3);

    // Verify the plain message record
    let plain_record = pending.iter().find(|r| r.id == plain_id).unwrap();
    assert_eq!(plain_record.message_id, Some("msg1".to_string()));
    assert_eq!(plain_record.source_type, SourceType::Internal);
    assert_eq!(
        plain_record.source_identifier,
        Some("did:key:sender1".to_string())
    );
    assert_eq!(plain_record.status, ReceivedStatus::Pending);

    // Update one to processed
    storage
        .update_received_status(plain_id, ReceivedStatus::Processed, Some("msg1"), None)
        .await
        .unwrap();

    // Update one to failed
    storage
        .update_received_status(
            jws_id,
            ReceivedStatus::Failed,
            None,
            Some("Signature verification failed"),
        )
        .await
        .unwrap();

    // Check pending again - should only have 1
    let pending = storage.get_pending_received(10).await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, jwe_id);

    // Get the processed record
    let processed_record = storage.get_received_by_id(plain_id).await.unwrap().unwrap();
    assert_eq!(processed_record.status, ReceivedStatus::Processed);
    assert_eq!(
        processed_record.processed_message_id,
        Some("msg1".to_string())
    );
    assert!(processed_record.processed_at.is_some());

    // Get the failed record
    let failed_record = storage.get_received_by_id(jws_id).await.unwrap().unwrap();
    assert_eq!(failed_record.status, ReceivedStatus::Failed);
    assert_eq!(
        failed_record.error_message,
        Some("Signature verification failed".to_string())
    );
    assert!(failed_record.processed_at.is_some());

    // Test list with filters
    let all_https = storage
        .list_received(10, 0, Some(SourceType::Https), None)
        .await
        .unwrap();
    assert_eq!(all_https.len(), 1);
    assert_eq!(all_https[0].id, jws_id);

    let all_failed = storage
        .list_received(10, 0, None, Some(ReceivedStatus::Failed))
        .await
        .unwrap();
    assert_eq!(all_failed.len(), 1);
    assert_eq!(all_failed[0].id, jws_id);
}

#[tokio::test]
async fn test_received_message_id_extraction() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage = Storage::new(Some(db_path)).await.unwrap();

    // Test with valid JSON containing ID
    let msg_with_id = r#"{"id":"test-123","type":"test"}"#;
    let id1 = storage
        .create_received(msg_with_id, SourceType::Internal, None)
        .await
        .unwrap();

    // Test with valid JSON without ID
    let msg_without_id = r#"{"type":"test"}"#;
    let id2 = storage
        .create_received(msg_without_id, SourceType::Internal, None)
        .await
        .unwrap();

    // Test with invalid JSON
    let invalid_json = r#"not valid json"#;
    let id3 = storage
        .create_received(invalid_json, SourceType::Internal, None)
        .await
        .unwrap();

    // Verify message ID extraction
    let rec1 = storage.get_received_by_id(id1).await.unwrap().unwrap();
    assert_eq!(rec1.message_id, Some("test-123".to_string()));

    let rec2 = storage.get_received_by_id(id2).await.unwrap().unwrap();
    assert_eq!(rec2.message_id, None);

    let rec3 = storage.get_received_by_id(id3).await.unwrap().unwrap();
    assert_eq!(rec3.message_id, None);
}

#[tokio::test]
async fn test_received_table_pagination() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage = Storage::new(Some(db_path)).await.unwrap();

    // Create 5 records
    for i in 0..5 {
        let msg = format!(r#"{{"id":"msg{}","type":"test"}}"#, i);
        storage
            .create_received(&msg, SourceType::Internal, None)
            .await
            .unwrap();
    }

    // Test pagination
    let page1 = storage.list_received(2, 0, None, None).await.unwrap();
    assert_eq!(page1.len(), 2);

    let page2 = storage.list_received(2, 2, None, None).await.unwrap();
    assert_eq!(page2.len(), 2);

    let page3 = storage.list_received(2, 4, None, None).await.unwrap();
    assert_eq!(page3.len(), 1);

    // Verify ordering (newest first)
    assert_eq!(page1[0].message_id, Some("msg4".to_string()));
    assert_eq!(page1[1].message_id, Some("msg3".to_string()));
}
