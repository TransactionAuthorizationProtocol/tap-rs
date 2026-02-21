//! Integration tests for the external decision system.

use serde_json::json;
use std::sync::Arc;
use tap_node::storage::{DecisionStatus, DecisionType, Storage};

/// Helper to create an in-memory storage for testing
async fn test_storage() -> Arc<Storage> {
    Arc::new(Storage::new_in_memory().await.unwrap())
}

#[tokio::test]
async fn test_decision_flow_insert_and_resolve() {
    let storage = test_storage().await;

    let context = json!({
        "transaction_state": "Received",
        "pending_agents": ["did:key:z6MkAgent1"],
        "transaction": {"type": "transfer", "asset": "eip155:1/slip44:60", "amount": "100"}
    });

    // Insert a decision
    let id = storage
        .insert_decision(
            "txn-int-001",
            "did:key:z6MkAgent1",
            DecisionType::AuthorizationRequired,
            &context,
        )
        .await
        .unwrap();

    assert!(id > 0);

    // Verify it's pending
    let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
    assert_eq!(entry.status, DecisionStatus::Pending);

    // Mark as delivered (simulating send to external process)
    storage
        .update_decision_status(id, DecisionStatus::Delivered, None, None)
        .await
        .unwrap();

    let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
    assert_eq!(entry.status, DecisionStatus::Delivered);
    assert!(entry.delivered_at.is_some());

    // Resolve (simulating external process response)
    let detail = json!({"settlement_address": "eip155:1:0xABC123"});
    storage
        .update_decision_status(
            id,
            DecisionStatus::Resolved,
            Some("authorize"),
            Some(&detail),
        )
        .await
        .unwrap();

    let entry = storage.get_decision_by_id(id).await.unwrap().unwrap();
    assert_eq!(entry.status, DecisionStatus::Resolved);
    assert_eq!(entry.resolution.as_deref(), Some("authorize"));
    assert!(entry.resolved_at.is_some());
    assert_eq!(
        entry.resolution_detail.unwrap()["settlement_address"],
        "eip155:1:0xABC123"
    );
}

#[tokio::test]
async fn test_decision_expiration_on_terminal_state() {
    let storage = test_storage().await;

    let context = json!({"info": "test"});

    // Insert two decisions for the same transaction
    let id1 = storage
        .insert_decision(
            "txn-int-002",
            "did:key:z6MkAgent1",
            DecisionType::AuthorizationRequired,
            &context,
        )
        .await
        .unwrap();

    let id2 = storage
        .insert_decision(
            "txn-int-002",
            "did:key:z6MkAgent2",
            DecisionType::AuthorizationRequired,
            &context,
        )
        .await
        .unwrap();

    // Mark one as delivered
    storage
        .update_decision_status(id2, DecisionStatus::Delivered, None, None)
        .await
        .unwrap();

    // Simulate transaction rejection — expire all pending/delivered
    let expired = storage
        .expire_decisions_for_transaction("txn-int-002")
        .await
        .unwrap();
    assert_eq!(expired, 2);

    // Both should be expired
    let e1 = storage.get_decision_by_id(id1).await.unwrap().unwrap();
    assert_eq!(e1.status, DecisionStatus::Expired);

    let e2 = storage.get_decision_by_id(id2).await.unwrap().unwrap();
    assert_eq!(e2.status, DecisionStatus::Expired);
}

#[tokio::test]
async fn test_decision_replay_after_restart() {
    let storage = test_storage().await;

    let context = json!({"transaction": {"type": "transfer"}});

    // Simulate decisions accumulated while process was down
    let id1 = storage
        .insert_decision(
            "txn-int-003",
            "did:key:z6MkAgent1",
            DecisionType::AuthorizationRequired,
            &context,
        )
        .await
        .unwrap();

    let id2 = storage
        .insert_decision(
            "txn-int-004",
            "did:key:z6MkAgent1",
            DecisionType::SettlementRequired,
            &context,
        )
        .await
        .unwrap();

    // One was previously delivered but not resolved
    storage
        .update_decision_status(id1, DecisionStatus::Delivered, None, None)
        .await
        .unwrap();

    // On restart, list all unresolved decisions for replay
    let pending = storage
        .list_decisions(
            Some("did:key:z6MkAgent1"),
            Some(DecisionStatus::Pending),
            None,
            1000,
        )
        .await
        .unwrap();

    let delivered = storage
        .list_decisions(
            Some("did:key:z6MkAgent1"),
            Some(DecisionStatus::Delivered),
            None,
            1000,
        )
        .await
        .unwrap();

    // Should find 1 pending and 1 delivered for replay
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, id2);

    assert_eq!(delivered.len(), 1);
    assert_eq!(delivered[0].id, id1);

    // Total unresolved = 2
    let total_unresolved = pending.len() + delivered.len();
    assert_eq!(total_unresolved, 2);
}

#[tokio::test]
async fn test_decision_expiration_does_not_affect_resolved() {
    let storage = test_storage().await;

    let context = json!({"info": "test"});

    let id1 = storage
        .insert_decision(
            "txn-int-005",
            "did:key:z6MkAgent1",
            DecisionType::AuthorizationRequired,
            &context,
        )
        .await
        .unwrap();

    let id2 = storage
        .insert_decision(
            "txn-int-005",
            "did:key:z6MkAgent2",
            DecisionType::AuthorizationRequired,
            &context,
        )
        .await
        .unwrap();

    // Resolve id1 first
    storage
        .update_decision_status(id1, DecisionStatus::Resolved, Some("authorize"), None)
        .await
        .unwrap();

    // Then transaction reaches terminal state
    let expired = storage
        .expire_decisions_for_transaction("txn-int-005")
        .await
        .unwrap();
    assert_eq!(expired, 1); // Only id2 should be expired

    let e1 = storage.get_decision_by_id(id1).await.unwrap().unwrap();
    assert_eq!(e1.status, DecisionStatus::Resolved);

    let e2 = storage.get_decision_by_id(id2).await.unwrap().unwrap();
    assert_eq!(e2.status, DecisionStatus::Expired);
}

#[tokio::test]
async fn test_decision_since_id_for_catchup() {
    let storage = test_storage().await;

    let context = json!({"info": "test"});

    // Insert 5 decisions
    let mut ids = vec![];
    for i in 0..5 {
        let id = storage
            .insert_decision(
                &format!("txn-int-10{}", i),
                "did:key:z6MkAgent1",
                DecisionType::AuthorizationRequired,
                &context,
            )
            .await
            .unwrap();
        ids.push(id);
    }

    // Simulate: external process has seen up to ids[2]
    let last_seen = ids[2];

    // Query for decisions since the last seen ID
    let new_decisions = storage
        .list_decisions(Some("did:key:z6MkAgent1"), None, Some(last_seen), 1000)
        .await
        .unwrap();

    assert_eq!(new_decisions.len(), 2);
    assert_eq!(new_decisions[0].id, ids[3]);
    assert_eq!(new_decisions[1].id, ids[4]);
}
