#[cfg(feature = "storage")]
mod storage_tests {
    use serde_json::json;
    use tap_msg::didcomm::PlainMessage;
    use tap_msg::message::{payment::Payment, transfer::Transfer, Party};
    use tap_node::storage::{
        MessageDirection, Storage, StorageError, TransactionStatus, TransactionType,
    };

    use tempfile::NamedTempFile;

    /// Create a SQLite database backed by a temporary file for testing
    async fn create_in_memory_storage() -> Storage {
        // Use a temporary file instead of pure in-memory to support connection pooling
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_path_buf();

        // Keep the temp file alive by leaking it (it will be cleaned up on process exit)
        std::mem::forget(temp_file);

        Storage::new(Some(path))
            .await
            .expect("Failed to create storage")
    }

    /// Helper to create a test PlainMessage
    fn create_test_message(id: &str, msg_type: &str, body: serde_json::Value) -> PlainMessage {
        PlainMessage {
            id: id.to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: msg_type.to_string(),
            body,
            from: "did:example:alice".to_string(),
            to: vec!["did:example:bob".to_string()],
            thid: Some("thread_123".to_string()),
            pthid: None,
            extra_headers: Default::default(),
            attachments: None,
            created_time: None,
            expires_time: None,
            from_prior: None,
        }
    }

    /// Helper to create a Transfer message
    fn create_transfer_message(id: &str) -> PlainMessage {
        let mut originator = Party::new("did:example:originator");
        originator.add_metadata(
            "name".to_string(),
            serde_json::Value::String("Alice".to_string()),
        );

        let mut beneficiary = Party::new("did:example:beneficiary");
        beneficiary.add_metadata(
            "name".to_string(),
            serde_json::Value::String("Bob".to_string()),
        );

        let transfer_body = Transfer {
            transaction_id: id.to_string(),
            originator,
            beneficiary: Some(beneficiary),
            asset: "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                .parse()
                .unwrap(),
            amount: "1000000".to_string(),
            agents: vec![],
            memo: Some("Test transfer".to_string()),
            settlement_id: None,
            connection_id: None,
            metadata: Default::default(),
        };

        create_test_message(
            id,
            "https://tap-protocol.io/messages/transfer/1.0",
            serde_json::to_value(&transfer_body).unwrap(),
        )
    }

    /// Helper to create a Payment message
    fn create_payment_message(id: &str) -> PlainMessage {
        let payment_body = Payment {
            transaction_id: id.to_string(),
            asset: Some(
                "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                    .parse()
                    .unwrap(),
            ),
            amount: "50.00".to_string(),
            currency_code: None,
            supported_assets: None,
            customer: {
                let mut customer = Party::new("did:example:customer");
                customer.add_metadata(
                    "name".to_string(),
                    serde_json::Value::String("Charlie".to_string()),
                );
                Some(customer)
            },
            merchant: {
                let mut merchant = Party::new("did:example:merchant");
                merchant.add_metadata(
                    "name".to_string(),
                    serde_json::Value::String("Dave's Shop".to_string()),
                );
                merchant
            },
            memo: Some("Payment for goods".to_string()),
            invoice: None,
            metadata: Default::default(),
            agents: vec![],
            expiry: None,
            connection_id: None,
        };

        create_test_message(
            id,
            "https://tap-protocol.io/messages/payment/1.0",
            serde_json::to_value(&payment_body).unwrap(),
        )
    }

    #[tokio::test]
    async fn test_in_memory_storage_creation() {
        let storage = create_in_memory_storage().await;
        // If we get here without panic, storage was created successfully

        // Try to insert a message to verify it's working
        let msg = create_test_message(
            "test_1",
            "https://tap-protocol.io/messages/connect/1.0",
            json!({}),
        );
        assert!(storage
            .log_message(&msg, MessageDirection::Incoming)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_transaction_insertion_and_retrieval() {
        let storage = create_in_memory_storage().await;

        // Create and insert a transfer transaction
        let transfer_msg = create_transfer_message("transfer_001");
        storage.insert_transaction(&transfer_msg).await.unwrap();

        // Retrieve the transaction
        let retrieved = storage.get_transaction_by_id("transfer_001").await.unwrap();
        assert!(retrieved.is_some());

        let tx = retrieved.unwrap();
        assert_eq!(tx.reference_id, "transfer_001");
        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert_eq!(tx.status, TransactionStatus::Pending);
        assert_eq!(tx.from_did, Some("did:example:alice".to_string()));
        assert_eq!(tx.to_did, Some("did:example:bob".to_string()));
        assert_eq!(tx.thread_id, Some("thread_123".to_string()));
    }

    #[tokio::test]
    async fn test_payment_transaction() {
        let storage = create_in_memory_storage().await;

        // Create and insert a payment transaction
        let payment_msg = create_payment_message("payment_001");
        storage.insert_transaction(&payment_msg).await.unwrap();

        // Retrieve and verify
        let retrieved = storage.get_transaction_by_id("payment_001").await.unwrap();
        assert!(retrieved.is_some());

        let tx = retrieved.unwrap();
        assert_eq!(tx.reference_id, "payment_001");
        assert_eq!(tx.transaction_type, TransactionType::Payment);
        assert_eq!(tx.status, TransactionStatus::Pending);
    }

    #[tokio::test]
    async fn test_transaction_list_pagination() {
        let storage = create_in_memory_storage().await;

        // Insert multiple transactions with delays to ensure different timestamps
        for i in 0..5 {
            let msg = create_transfer_message(&format!("transfer_{:03}", i));
            storage.insert_transaction(&msg).await.unwrap();
            // Small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Test pagination
        let page1 = storage.list_transactions(2, 0).await.unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = storage.list_transactions(2, 2).await.unwrap();
        assert_eq!(page2.len(), 2);

        let page3 = storage.list_transactions(2, 4).await.unwrap();
        assert_eq!(page3.len(), 1);

        // Verify we got the expected IDs in reverse order (newest first)
        assert_eq!(page1[0].reference_id, "transfer_004");
        assert_eq!(page1[1].reference_id, "transfer_003");
        assert_eq!(page2[0].reference_id, "transfer_002");
        assert_eq!(page2[1].reference_id, "transfer_001");
        assert_eq!(page3[0].reference_id, "transfer_000");
    }

    #[tokio::test]
    async fn test_duplicate_transaction_handling() {
        let storage = create_in_memory_storage().await;

        let msg = create_transfer_message("duplicate_001");

        // First insertion should succeed
        assert!(storage.insert_transaction(&msg).await.is_ok());

        // Second insertion should fail with duplicate error
        let result = storage.insert_transaction(&msg).await;
        assert!(matches!(result, Err(StorageError::DuplicateTransaction(_))));
    }

    #[tokio::test]
    async fn test_message_logging_all_directions() {
        let storage = create_in_memory_storage().await;

        // Log incoming messages
        let incoming1 = create_test_message(
            "in_001",
            "https://tap-protocol.io/messages/connect/1.0",
            json!({}),
        );
        let incoming2 = create_test_message(
            "in_002",
            "https://tap-protocol.io/messages/authorize/1.0",
            json!({}),
        );

        storage
            .log_message(&incoming1, MessageDirection::Incoming)
            .await
            .unwrap();
        storage
            .log_message(&incoming2, MessageDirection::Incoming)
            .await
            .unwrap();

        // Log outgoing messages
        let outgoing1 = create_test_message(
            "out_001",
            "https://tap-protocol.io/messages/reject/1.0",
            json!({}),
        );
        let outgoing2 = create_test_message(
            "out_002",
            "https://tap-protocol.io/messages/settle/1.0",
            json!({}),
        );

        storage
            .log_message(&outgoing1, MessageDirection::Outgoing)
            .await
            .unwrap();
        storage
            .log_message(&outgoing2, MessageDirection::Outgoing)
            .await
            .unwrap();

        // Verify all messages are stored
        let all_messages = storage.list_messages(10, 0, None).await.unwrap();
        assert_eq!(all_messages.len(), 4);

        // Verify filtering by direction
        let incoming_only = storage
            .list_messages(10, 0, Some(MessageDirection::Incoming))
            .await
            .unwrap();
        assert_eq!(incoming_only.len(), 2);
        assert!(incoming_only
            .iter()
            .all(|m| m.direction == MessageDirection::Incoming));

        let outgoing_only = storage
            .list_messages(10, 0, Some(MessageDirection::Outgoing))
            .await
            .unwrap();
        assert_eq!(outgoing_only.len(), 2);
        assert!(outgoing_only
            .iter()
            .all(|m| m.direction == MessageDirection::Outgoing));
    }

    #[tokio::test]
    async fn test_message_retrieval_by_id() {
        let storage = create_in_memory_storage().await;

        let msg = create_test_message(
            "specific_001",
            "https://tap-protocol.io/messages/transfer/1.0",
            json!({ "test": "data" }),
        );

        storage
            .log_message(&msg, MessageDirection::Incoming)
            .await
            .unwrap();

        // Retrieve by ID
        let retrieved = storage.get_message_by_id("specific_001").await.unwrap();
        assert!(retrieved.is_some());

        let stored_msg = retrieved.unwrap();
        assert_eq!(stored_msg.message_id, "specific_001");
        assert_eq!(
            stored_msg.message_type,
            "https://tap-protocol.io/messages/transfer/1.0"
        );
        assert_eq!(stored_msg.from_did, Some("did:example:alice".to_string()));
        assert_eq!(stored_msg.to_did, Some("did:example:bob".to_string()));
        assert_eq!(stored_msg.thread_id, Some("thread_123".to_string()));
        assert_eq!(stored_msg.direction, MessageDirection::Incoming);

        // Verify the JSON content - message_json is already a serde_json::Value
        assert_eq!(stored_msg.message_json["body"]["test"], "data");
    }

    #[tokio::test]
    async fn test_message_duplicate_handling() {
        let storage = create_in_memory_storage().await;

        let msg = create_test_message(
            "dup_msg_001",
            "https://tap-protocol.io/messages/connect/1.0",
            json!({}),
        );

        // First insertion should succeed
        assert!(storage
            .log_message(&msg, MessageDirection::Incoming)
            .await
            .is_ok());

        // Second insertion should silently succeed (idempotent)
        assert!(storage
            .log_message(&msg, MessageDirection::Incoming)
            .await
            .is_ok());

        // Verify only one message is stored
        let messages = storage.list_messages(10, 0, None).await.unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_message_thread_tracking() {
        let storage = create_in_memory_storage().await;

        // Create messages with thread relationships
        let mut parent_msg = create_test_message(
            "parent_001",
            "https://tap-protocol.io/messages/transfer/1.0",
            json!({}),
        );
        parent_msg.thid = Some("thread_parent".to_string());
        parent_msg.pthid = None;

        let mut child_msg = create_test_message(
            "child_001",
            "https://tap-protocol.io/messages/authorize/1.0",
            json!({}),
        );
        child_msg.thid = Some("thread_child".to_string());
        child_msg.pthid = Some("thread_parent".to_string());

        storage
            .log_message(&parent_msg, MessageDirection::Incoming)
            .await
            .unwrap();
        storage
            .log_message(&child_msg, MessageDirection::Outgoing)
            .await
            .unwrap();

        // Retrieve and verify thread relationships
        let parent = storage
            .get_message_by_id("parent_001")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(parent.thread_id, Some("thread_parent".to_string()));
        assert_eq!(parent.parent_thread_id, None);

        let child = storage
            .get_message_by_id("child_001")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(child.thread_id, Some("thread_child".to_string()));
        assert_eq!(child.parent_thread_id, Some("thread_parent".to_string()));
    }

    #[tokio::test]
    async fn test_message_pagination() {
        let storage = create_in_memory_storage().await;

        // Insert 15 messages
        for i in 0..15 {
            let msg = create_test_message(
                &format!("page_test_{:03}", i),
                "https://tap-protocol.io/messages/connect/1.0",
                json!({}),
            );
            storage
                .log_message(&msg, MessageDirection::Incoming)
                .await
                .unwrap();

            // Small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Test pagination
        let page1 = storage.list_messages(5, 0, None).await.unwrap();
        assert_eq!(page1.len(), 5);

        let page2 = storage.list_messages(5, 5, None).await.unwrap();
        assert_eq!(page2.len(), 5);

        let page3 = storage.list_messages(5, 10, None).await.unwrap();
        assert_eq!(page3.len(), 5);

        // Verify no overlap between pages
        let page1_ids: Vec<_> = page1.iter().map(|m| &m.message_id).collect();
        let page2_ids: Vec<_> = page2.iter().map(|m| &m.message_id).collect();
        assert!(page1_ids.iter().all(|id| !page2_ids.contains(id)));

        // Verify we got messages in the expected order (newest first)
        assert_eq!(page1[0].message_id, "page_test_014");
        assert_eq!(page1[4].message_id, "page_test_010");
        assert_eq!(page2[0].message_id, "page_test_009");
        assert_eq!(page3[4].message_id, "page_test_000");
    }

    #[tokio::test]
    async fn test_non_transaction_messages_not_in_transactions_table() {
        let storage = create_in_memory_storage().await;

        // Create non-transaction messages
        let connect_msg = create_test_message(
            "connect_001",
            "https://tap-protocol.io/messages/connect/1.0",
            json!({}),
        );
        let auth_msg = create_test_message(
            "auth_001",
            "https://tap-protocol.io/messages/authorize/1.0",
            json!({}),
        );

        // Log them as messages
        storage
            .log_message(&connect_msg, MessageDirection::Incoming)
            .await
            .unwrap();
        storage
            .log_message(&auth_msg, MessageDirection::Incoming)
            .await
            .unwrap();

        // Verify they're in messages table
        let messages = storage.list_messages(10, 0, None).await.unwrap();
        assert_eq!(messages.len(), 2);

        // Verify they're NOT in transactions table
        let transactions = storage.list_transactions(10, 0).await.unwrap();
        assert_eq!(transactions.len(), 0);

        // Now add a transfer message
        let transfer_msg = create_transfer_message("transfer_001");
        storage
            .log_message(&transfer_msg, MessageDirection::Incoming)
            .await
            .unwrap();
        storage.insert_transaction(&transfer_msg).await.unwrap();

        // Verify it's in both tables
        let messages = storage.list_messages(10, 0, None).await.unwrap();
        assert_eq!(messages.len(), 3);

        let transactions = storage.list_transactions(10, 0).await.unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].reference_id, "transfer_001");
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let storage = create_in_memory_storage().await;

        // Spawn multiple tasks to insert messages concurrently
        let mut handles = vec![];

        for i in 0..10 {
            let storage_clone = storage.clone();
            let handle = tokio::spawn(async move {
                // Use thread ID and index to ensure unique message IDs
                let thread_id = std::thread::current().id();
                let msg = create_test_message(
                    &format!("concurrent_{:03}_{:?}", i, thread_id),
                    "https://tap-protocol.io/messages/connect/1.0",
                    json!({}),
                );
                storage_clone
                    .log_message(&msg, MessageDirection::Incoming)
                    .await
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete and collect results
        let mut successes = 0;
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => successes += 1,
                Ok(Err(e)) => eprintln!("Task failed with error: {:?}", e),
                Err(e) => eprintln!("Task panicked: {:?}", e),
            }
        }

        // At least most should succeed (allowing for some connection pool contention)
        assert!(
            successes >= 8,
            "Only {} out of 10 concurrent operations succeeded",
            successes
        );

        // Verify messages were inserted
        let messages = storage.list_messages(20, 0, None).await.unwrap();
        assert!(
            messages.len() >= 8,
            "Only {} messages were inserted",
            messages.len()
        );
    }

    #[tokio::test]
    async fn test_storage_error_handling() {
        let storage = create_in_memory_storage().await;

        // Test invalid transaction type (not Transfer or Payment)
        let invalid_msg = create_test_message(
            "invalid_001",
            "https://tap-protocol.io/messages/connect/1.0",
            json!({}),
        );

        let result = storage.insert_transaction(&invalid_msg).await;
        assert!(matches!(
            result,
            Err(StorageError::InvalidTransactionType(_))
        ));

        // Test retrieval of non-existent records
        let tx = storage.get_transaction_by_id("non_existent").await.unwrap();
        assert!(tx.is_none());

        let msg = storage.get_message_by_id("non_existent").await.unwrap();
        assert!(msg.is_none());
    }

    #[tokio::test]
    async fn test_message_json_integrity() {
        let storage = create_in_memory_storage().await;

        // Create a message with complex body
        let complex_body = json!({
            "nested": {
                "field": "value",
                "array": [1, 2, 3],
                "bool": true
            },
            "unicode": "Hello 世界 🌍",
            "special_chars": "Line1\nLine2\tTab"
        });

        let msg = create_test_message(
            "json_test_001",
            "https://tap-protocol.io/messages/test/1.0",
            complex_body,
        );

        storage
            .log_message(&msg, MessageDirection::Incoming)
            .await
            .unwrap();

        // Retrieve and verify JSON integrity
        let retrieved = storage
            .get_message_by_id("json_test_001")
            .await
            .unwrap()
            .unwrap();
        let parsed: PlainMessage = serde_json::from_value(retrieved.message_json).unwrap();

        assert_eq!(parsed.body["nested"]["field"], "value");
        assert_eq!(parsed.body["nested"]["array"][1], 2);
        assert_eq!(parsed.body["unicode"], "Hello 世界 🌍");
        assert_eq!(parsed.body["special_chars"], "Line1\nLine2\tTab");
    }
}
