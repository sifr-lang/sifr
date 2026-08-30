#![allow(clippy::expect_used)]

use sifr_sql_runtime::{
    IsolationLevel, PoolCoordinator, PoolingMode, ResourceLimitKind, RuntimeLimits,
    SchemaDependencySlice, SchemaProperty, SchemaStrictness, SessionContract, SqlError,
    SqlErrorKind, StatementCache, StatementCacheKey, TransactionMachine, TransactionState,
    verify_schema,
};
use std::time::Duration;

fn fingerprint(byte: char) -> String {
    byte.to_string().repeat(64)
}

fn cache_key(byte: char) -> StatementCacheKey {
    StatementCacheKey {
        normalized_statement_fingerprint: fingerprint(byte),
        parameter_type_fingerprint: fingerprint('a'),
        result_type_fingerprint: fingerprint('b'),
        provider_version: "13".to_string(),
        schema_fingerprint: fingerprint('c'),
    }
    .validate()
    .expect("test key is valid")
}

#[test]
fn compatible_verification_checks_values_and_absence_facts() {
    let expected = SchemaDependencySlice::new(
        fingerprint('a'),
        [
            SchemaProperty::new("column:users.id:type", Some("int8".to_string()))
                .expect("valid property"),
            SchemaProperty::new("overload:app.lookup:text", None).expect("valid absence fact"),
        ],
    )
    .expect("valid expected slice");
    let observed = SchemaDependencySlice::new(
        fingerprint('b'),
        [
            SchemaProperty::new("column:users.id:type", Some("int8".to_string()))
                .expect("valid property"),
            SchemaProperty::new("overload:app.lookup:text", None).expect("valid absence fact"),
            SchemaProperty::new("unreferenced:addition", Some("allowed".to_string()))
                .expect("valid property"),
        ],
    )
    .expect("valid observed slice");
    verify_schema(SchemaStrictness::Compatible, &expected, &observed)
        .expect("unreferenced additions are compatible");

    let drifted = SchemaDependencySlice::new(
        fingerprint('b'),
        [
            SchemaProperty::new("column:users.id:type", Some("int8".to_string()))
                .expect("valid property"),
            SchemaProperty::new(
                "overload:app.lookup:text",
                Some("new candidate".to_string()),
            )
            .expect("valid changed fact"),
        ],
    )
    .expect("valid observed slice");
    assert_eq!(
        verify_schema(SchemaStrictness::Compatible, &expected, &drifted)
            .expect_err("changed absence fact must fail")
            .kind(),
        SqlErrorKind::SchemaContract,
    );
    assert!(verify_schema(SchemaStrictness::Exact, &expected, &observed).is_err());
}

#[test]
fn statement_cache_is_bounded_lru_with_complete_identity() {
    let mut cache = StatementCache::new(2).expect("positive capacity");
    let first = cache_key('d');
    let second = cache_key('e');
    let third = cache_key('f');
    cache.insert(&first, 1);
    cache.insert(&second, 2);
    assert_eq!(cache.get(&first), Some(&1));
    cache.insert(&third, 3);
    assert!(cache.get(&second).is_none());
    assert_eq!(cache.get(&first), Some(&1));
    assert_eq!(cache.get(&third), Some(&3));
    assert_eq!(cache.len(), 2);
}

#[test]
fn transaction_machine_covers_every_terminal_transition() {
    let mut committed = TransactionMachine::new();
    committed.committed().expect("live transaction can commit");
    assert_eq!(committed.state(), TransactionState::Committed);
    assert_eq!(
        committed
            .rolled_back()
            .expect_err("terminal handle is not reusable")
            .kind(),
        SqlErrorKind::TransactionControl,
    );

    let mut rolled_back = TransactionMachine::new();
    rolled_back
        .rolled_back()
        .expect("live transaction can roll back");
    assert_eq!(rolled_back.state(), TransactionState::RolledBack);

    let mut poisoned = TransactionMachine::new();
    poisoned.poison();
    assert_eq!(poisoned.state(), TransactionState::Poisoned);
    assert!(poisoned.committed().is_err());

    let mut dropped = TransactionMachine::new();
    dropped.dropped();
    assert_eq!(dropped.state(), TransactionState::Dropped);
    assert!(dropped.rolled_back().is_err());

    let mut savepoint = TransactionMachine::new();
    assert_eq!(savepoint.push_savepoint().expect("savepoint"), 1);
    assert!(savepoint.committed().is_err());
    assert_eq!(savepoint.pop_savepoint().expect("release"), 1);
    savepoint
        .committed()
        .expect("released savepoint permits commit");
}

#[test]
fn transaction_pooling_rejects_session_affinity() {
    let contract = SessionContract {
        search_path: vec!["public".to_string()],
        time_zone: "UTC".to_string(),
        role: None,
        default_isolation: IsolationLevel::ReadCommitted,
        read_only: false,
        pooling: PoolingMode::Transaction,
        requires_session_affinity: true,
    };
    assert_eq!(
        contract
            .validate()
            .expect_err("affinity cannot use transaction pooling")
            .kind(),
        SqlErrorKind::Configuration,
    );
}

#[tokio::test(flavor = "current_thread")]
async fn pool_enforces_connection_and_acquisition_bounds() {
    let limits = RuntimeLimits {
        max_connections: 1,
        acquire_timeout: Duration::from_millis(10),
        ..RuntimeLimits::default()
    };
    let pool = PoolCoordinator::new(limits).expect("valid limits");
    let lease = pool
        .acquire(|| async { Ok::<_, SqlError>(1_u8) })
        .await
        .expect("first lease");
    let Err(error) = pool.acquire(|| async { Ok::<_, SqlError>(2_u8) }).await else {
        panic!("second lease must time out");
    };
    assert_eq!(
        error.metadata().resource_limit,
        Some(ResourceLimitKind::AcquireDeadline)
    );
    lease.discard();
    assert_eq!(pool.statistics().total, 0);
}

#[tokio::test(flavor = "current_thread")]
async fn reset_failure_discards_the_connection_and_keeps_typed_evidence() {
    let pool = PoolCoordinator::new(RuntimeLimits::default()).expect("valid limits");
    let lease = pool
        .acquire(|| async { Ok::<_, SqlError>(1_u8) })
        .await
        .expect("lease");
    let error = lease
        .release(
            |_| Box::pin(async { Err(SqlError::new(SqlErrorKind::Connection)) }),
            None,
            "test-resource",
        )
        .await
        .expect_err("failed reset must fail release");
    assert_eq!(error.kind(), SqlErrorKind::Connection);
    assert_eq!(error.secondary().len(), 1);
    assert_eq!(pool.statistics().total, 0);
}
