#![allow(clippy::expect_used)]

use sha2::{Digest, Sha256};
use sifr_sql_postgresql_runtime::{
    ExecutionOptions, ManifestVerifier, PostgresEvidence, PostgresPool, PostgresProfile,
    PostgresTls, RetryPolicy, RetrySafeCallback, SignedSchemaManifest, TransactionOptions,
    VerificationProbe, connect,
};
use sifr_sql_runtime::{
    BoundParameters, ConstraintKind, ExecutionMetadata, ExecutionMode, ExecutionRequest,
    IsolationLevel, OwnedParameter, OwnedSqlValue, PoolingMode, RetryClassification,
    RuntimeCardinality, RuntimeCodecIdentity, RuntimeEffect, RuntimeEffectContract, RuntimeLimits,
    SchemaDependencySlice, SchemaProperty, SchemaStrictness, SessionContract, SqlError,
    SqlErrorKind, SqlErrorMetadata,
};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

fn profile(url: &str) -> PostgresProfile {
    let fingerprint = "a".repeat(64);
    PostgresProfile::new(
        url,
        "b".repeat(64),
        SchemaDependencySlice::new(
            fingerprint,
            [
                SchemaProperty::new("runtime-probe", Some("present".to_string()))
                    .expect("valid property"),
            ],
        )
        .expect("valid schema"),
        PostgresEvidence::Introspection {
            fingerprint_statement: "SELECT repeat('a', 64)".to_string(),
            probes: vec![
                VerificationProbe::new("runtime-probe", "SELECT 'present'").expect("valid probe"),
            ],
        },
        SchemaStrictness::Exact,
        SessionContract {
            search_path: vec!["public".to_string()],
            time_zone: "UTC".to_string(),
            role: None,
            default_isolation: IsolationLevel::ReadCommitted,
            read_only: false,
            pooling: PoolingMode::Session,
            requires_session_affinity: false,
        },
        RuntimeLimits {
            max_connections: 3,
            acquire_timeout: Duration::from_secs(5),
            statement_timeout: Duration::from_secs(2),
            cleanup_timeout: Duration::from_secs(2),
            max_decoded_row_bytes: 1024 * 1024,
            max_collected_rows: 100,
            statement_cache_capacity: 2,
            max_parameters: 100,
        },
        PostgresTls::Disabled,
    )
    .expect("valid live profile")
}

fn compatible_profile(url: &str) -> PostgresProfile {
    let limits = profile(url).limits();
    PostgresProfile::new(
        url,
        "f".repeat(64),
        SchemaDependencySlice::new(
            "a".repeat(64),
            [
                SchemaProperty::new("runtime-probe", Some("present".to_string()))
                    .expect("valid property"),
            ],
        )
        .expect("valid expected schema"),
        PostgresEvidence::Introspection {
            fingerprint_statement: "SELECT repeat('b', 64)".to_string(),
            probes: vec![
                VerificationProbe::new("runtime-probe", "SELECT 'present'").expect("valid probe"),
            ],
        },
        SchemaStrictness::Compatible,
        SessionContract {
            search_path: vec!["public".to_string()],
            time_zone: "UTC".to_string(),
            role: None,
            default_isolation: IsolationLevel::ReadCommitted,
            read_only: false,
            pooling: PoolingMode::Session,
            requires_session_affinity: false,
        },
        limits,
        PostgresTls::Disabled,
    )
    .expect("valid compatible profile")
}

#[derive(Clone)]
struct StaticManifestVerifier {
    schema: SchemaDependencySlice,
}

impl ManifestVerifier for StaticManifestVerifier {
    fn verify(&self, _manifest: &SignedSchemaManifest) -> Result<SchemaDependencySlice, SqlError> {
        Ok(self.schema.clone())
    }
}

fn alternate_evidence_profile(
    url: &str,
    profile_byte: char,
    evidence: PostgresEvidence,
) -> PostgresProfile {
    PostgresProfile::new(
        url,
        profile_byte.to_string().repeat(64),
        SchemaDependencySlice::new(
            "a".repeat(64),
            [
                SchemaProperty::new("runtime-probe", Some("present".to_string()))
                    .expect("valid property"),
            ],
        )
        .expect("valid schema"),
        evidence,
        SchemaStrictness::Exact,
        SessionContract {
            search_path: vec!["public".to_string()],
            time_zone: "UTC".to_string(),
            role: None,
            default_isolation: IsolationLevel::ReadCommitted,
            read_only: false,
            pooling: PoolingMode::Session,
            requires_session_affinity: false,
        },
        profile(url).limits(),
        PostgresTls::Disabled,
    )
    .expect("valid alternate evidence profile")
}

fn request(
    pool: &PostgresPool<sifr_sql_postgresql_runtime::Verified>,
    statement: &str,
    mode: ExecutionMode,
    cardinality: RuntimeCardinality,
    effect: RuntimeEffect,
) -> ExecutionRequest<PostgresProfile> {
    let mut statement_fingerprint = String::with_capacity(64);
    for byte in Sha256::digest(statement.as_bytes()) {
        let _result = write!(&mut statement_fingerprint, "{byte:02x}");
    }
    let affected_objects = if matches!(effect, RuntimeEffect::Write | RuntimeEffect::ReadWrite) {
        vec!["public.sifr_runtime_probe".to_string()]
    } else {
        Vec::new()
    };
    ExecutionRequest {
        profile: Arc::clone(pool.profile()),
        statement: Arc::from(statement),
        parameters: BoundParameters::default(),
        cardinality,
        effects: RuntimeEffectContract::new(effect, Vec::new(), affected_objects)
            .expect("valid effect"),
        returns_rows: mode != ExecutionMode::Execute,
        metadata: ExecutionMetadata {
            normalized_statement_fingerprint: statement_fingerprint,
            parameter_type_fingerprint: "d".repeat(64),
            result_type_fingerprint: "e".repeat(64),
            schema_fingerprint: "a".repeat(64),
        },
        mode,
    }
}

#[derive(Clone)]
struct RetryOnce {
    calls: Arc<AtomicUsize>,
}

impl RetrySafeCallback<usize> for RetryOnce {
    fn call<'transaction>(
        &'transaction self,
        _transaction: &'transaction mut sifr_sql_postgresql_runtime::PostgresTransaction,
    ) -> Pin<Box<dyn Future<Output = Result<usize, SqlError>> + 'transaction>> {
        Box::pin(async move {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            if call == 0 {
                return Err(SqlError::with_metadata(
                    SqlErrorKind::Serialization,
                    SqlErrorMetadata {
                        retry: RetryClassification::RetryTransaction,
                        ..SqlErrorMetadata::default()
                    },
                )
                .expect("retry metadata"));
            }
            Ok(call + 1)
        })
    }
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "the SQL platform live harness supplies PostgreSQL"]
async fn live_postgresql_runtime_contract() {
    let url = std::env::var("SIFR_POSTGRESQL_TEST_URL").expect("live harness must set URL");
    let pool = connect(profile(&url)).await.expect("schema must verify");
    assert_eq!(pool.schema_fingerprint(), "a".repeat(64));

    let compatible = connect(compatible_profile(&url))
        .await
        .expect("compatible dependency slice must verify");
    let compatible_connection = compatible.acquire().await.expect("compatible connection");
    assert_eq!(compatible_connection.schema_fingerprint(), "a".repeat(64));
    assert_eq!(
        compatible_connection.observed_schema_fingerprint(),
        "b".repeat(64)
    );
    compatible_connection
        .release(None)
        .await
        .expect("compatible connection release");
    compatible
        .fetch_one(
            request(
                &compatible,
                "SELECT 1::bigint",
                ExecutionMode::FetchOne,
                RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("compatible verified handle can execute");

    let accepted_schema = SchemaDependencySlice::new(
        "a".repeat(64),
        [
            SchemaProperty::new("runtime-probe", Some("present".to_string()))
                .expect("valid property"),
        ],
    )
    .expect("valid accepted schema");
    let migration = connect(alternate_evidence_profile(
        &url,
        '8',
        PostgresEvidence::MigrationHead {
            head_statement: "SELECT 'head-a'".to_string(),
            accepted_states: BTreeMap::from([("head-a".to_string(), accepted_schema.clone())]),
        },
    ))
    .await
    .expect("migration-head evidence verifies");
    migration.close();
    let signed = connect(alternate_evidence_profile(
        &url,
        '9',
        PostgresEvidence::SignedManifest {
            manifest: SignedSchemaManifest {
                signer: "deployment".to_string(),
                payload: Arc::from([1_u8]),
                signature: Arc::from([2_u8]),
            },
            verifier: Arc::new(StaticManifestVerifier {
                schema: accepted_schema,
            }),
        },
    ))
    .await
    .expect("signed-manifest evidence verifies");
    signed.close();

    let row = pool
        .fetch_one(
            request(
                &pool,
                "SELECT 42::bigint",
                ExecutionMode::FetchOne,
                RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("fetch one");
    assert_eq!(
        row.clone().into_scalar().expect("one field"),
        OwnedSqlValue::Signed(42)
    );

    let mut array_request = request(
        &pool,
        "SELECT cardinality($1::bigint[])::bigint",
        ExecutionMode::FetchOne,
        RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
        RuntimeEffect::Read,
    );
    array_request.parameters = BoundParameters::new(vec![OwnedParameter {
        slot: 0,
        codec: RuntimeCodecIdentity::new("postgresql.int8-array.v1").expect("codec identity"),
        value: OwnedSqlValue::Sequence(vec![
            OwnedSqlValue::Signed(1),
            OwnedSqlValue::Signed(2),
            OwnedSqlValue::Signed(3),
        ]),
    }])
    .expect("array parameter");
    assert_eq!(
        pool.fetch_one(array_request, ExecutionOptions::default())
            .await
            .expect("array codec")
            .into_scalar()
            .expect("one field"),
        OwnedSqlValue::Signed(3)
    );

    let duplicate = pool
        .execute(
            request(
                &pool,
                "INSERT INTO sifr_runtime_probe(id, value) VALUES (1, 0)",
                ExecutionMode::Execute,
                RuntimeCardinality::new(0, Some(1)).expect("cardinality"),
                RuntimeEffect::Write,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect_err("duplicate key must be structured");
    assert_eq!(duplicate.kind(), SqlErrorKind::Constraint);
    assert_eq!(
        duplicate.metadata().constraint_kind,
        Some(ConstraintKind::Unique)
    );

    let optional = pool
        .fetch_optional(
            request(
                &pool,
                "SELECT 1::bigint WHERE false",
                ExecutionMode::FetchOptional,
                RuntimeCardinality::new(0, Some(1)).expect("cardinality"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("optional query");
    assert!(optional.is_none());

    let rows = pool
        .fetch_all(
            request(
                &pool,
                "SELECT value::bigint FROM generate_series(1, 3) AS value",
                ExecutionMode::FetchAll { maximum_rows: 3 },
                RuntimeCardinality::new(3, Some(3)).expect("cardinality"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("bounded collection");
    assert_eq!(rows.len(), 3);

    let bounded = pool
        .fetch_all(
            request(
                &pool,
                "SELECT value::bigint FROM generate_series(1, 2) AS value",
                ExecutionMode::FetchAll { maximum_rows: 1 },
                RuntimeCardinality::new(0, None).expect("cardinality"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect_err("fetch_all must stop before its bound is exceeded");
    assert_eq!(bounded.kind(), SqlErrorKind::ResourceLimit);

    let cardinality = pool
        .fetch_one(
            request(
                &pool,
                "SELECT value::bigint FROM generate_series(1, 2) AS value",
                ExecutionMode::FetchOne,
                RuntimeCardinality::new(0, Some(1)).expect("narrowed adapter contract"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect_err("expect_at_most_one must detect extra rows");
    assert_eq!(cardinality.kind(), SqlErrorKind::Cardinality);

    let first = pool
        .fetch_optional(
            request(
                &pool,
                "SELECT value::bigint FROM generate_series(1, 2) AS value LIMIT 1",
                ExecutionMode::FetchOptional,
                RuntimeCardinality::new(0, Some(1)).expect("first adapter contract"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("first uses provider one-row SQL");
    assert!(first.is_some());

    let execution = pool
        .execute(
            request(
                &pool,
                "UPDATE sifr_runtime_probe SET value = value + 1 WHERE id = 1",
                ExecutionMode::Execute,
                RuntimeCardinality::new(0, Some(1)).expect("cardinality"),
                RuntimeEffect::Write,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("execute update");
    assert_eq!(execution.rows_affected, Some(1));

    let mut connection = pool.acquire().await.expect("connection");
    let warm_request = request(
        &pool,
        "SELECT 7::bigint",
        ExecutionMode::FetchOne,
        RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
        RuntimeEffect::Read,
    );
    assert!(
        !connection
            .warm(&warm_request, ExecutionOptions::default())
            .await
            .expect("first warm")
            .statement_cache_hit
    );
    assert!(
        connection
            .warm(&warm_request, ExecutionOptions::default())
            .await
            .expect("second warm")
            .statement_cache_hit
    );
    for statement in ["SELECT 8::bigint", "SELECT 9::bigint"] {
        let extra = request(
            &pool,
            statement,
            ExecutionMode::FetchOne,
            RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
            RuntimeEffect::Read,
        );
        connection
            .warm(&extra, ExecutionOptions::default())
            .await
            .expect("warm cache entry");
    }
    assert!(
        !connection
            .warm(&warm_request, ExecutionOptions::default())
            .await
            .expect("least-recently-used entry was evicted")
            .statement_cache_hit
    );
    connection.release(None).await.expect("release");

    pool.fetch_one(
        request(
            &pool,
            "SELECT set_config('TimeZone', 'Europe/Stockholm', false)",
            ExecutionMode::FetchOne,
            RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
            RuntimeEffect::Read,
        ),
        ExecutionOptions::default(),
    )
    .await
    .expect("test changes one session value");
    let reset_zone = pool
        .fetch_one(
            request(
                &pool,
                "SELECT current_setting('TimeZone')",
                ExecutionMode::FetchOne,
                RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("next acquisition reapplies session contract")
        .into_scalar()
        .expect("one field");
    assert_eq!(reset_zone, OwnedSqlValue::Text("UTC".to_string()));

    let mut stream = pool
        .stream(
            request(
                &pool,
                "SELECT value::bigint FROM generate_series(1, 3) AS value",
                ExecutionMode::Stream,
                RuntimeCardinality::new(0, Some(3)).expect("cardinality"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("stream");
    assert!(stream.next().await.expect("first row").is_some());
    stream.aclose().await.expect("early close");

    let mut committed = pool
        .transaction(TransactionOptions::default())
        .await
        .expect("transaction");
    committed
        .execute(
            request(
                &pool,
                "UPDATE sifr_runtime_probe SET value = value + 10 WHERE id = 1",
                ExecutionMode::Execute,
                RuntimeCardinality::new(0, Some(1)).expect("cardinality"),
                RuntimeEffect::Write,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("transaction update");
    committed.commit().await.expect("commit");

    let mut rolled_back = pool
        .transaction(TransactionOptions::default())
        .await
        .expect("transaction");
    rolled_back
        .execute(
            request(
                &pool,
                "UPDATE sifr_runtime_probe SET value = value + 100 WHERE id = 1",
                ExecutionMode::Execute,
                RuntimeCardinality::new(0, Some(1)).expect("cardinality"),
                RuntimeEffect::Write,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("transaction update");
    rolled_back.rollback().await.expect("rollback");

    let mut savepoint_tx = pool
        .transaction(TransactionOptions::default())
        .await
        .expect("transaction");
    savepoint_tx
        .savepoint()
        .await
        .expect("savepoint")
        .rollback()
        .await
        .expect("savepoint rollback");
    savepoint_tx.commit().await.expect("commit after savepoint");

    let mut streaming_tx = pool
        .transaction(TransactionOptions::default())
        .await
        .expect("streaming transaction");
    {
        let mut transaction_stream = streaming_tx
            .stream(
                request(
                    &pool,
                    "SELECT value::bigint FROM generate_series(1, 3) AS value",
                    ExecutionMode::Stream,
                    RuntimeCardinality::new(0, Some(3)).expect("cardinality"),
                    RuntimeEffect::Read,
                ),
                ExecutionOptions::default(),
            )
            .await
            .expect("transaction stream");
        assert!(
            transaction_stream
                .next()
                .await
                .expect("transaction stream row")
                .is_some()
        );
        transaction_stream.close();
    }
    streaming_tx
        .commit()
        .await
        .expect("closed stream permits commit");

    let abandoned = pool
        .transaction(TransactionOptions::default())
        .await
        .expect("abnormal-exit transaction");
    drop(abandoned);
    let recovered = pool
        .acquire()
        .await
        .expect("discarded transaction is replaced");
    recovered.release(None).await.expect("replacement release");

    let calls = Arc::new(AtomicUsize::new(0));
    let retried = pool
        .run_transaction(
            RetryOnce {
                calls: Arc::clone(&calls),
            },
            RetryPolicy::serialization(2).expect("retry policy"),
        )
        .await
        .expect("second fresh transaction succeeds");
    assert_eq!(retried, 2);
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    let mut cleanup_tx = pool
        .transaction(TransactionOptions::default())
        .await
        .expect("cleanup evidence transaction");
    let backend = cleanup_tx
        .fetch_one(
            request(
                &pool,
                "SELECT pg_backend_pid()::bigint",
                ExecutionMode::FetchOne,
                RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("backend identity")
        .into_scalar()
        .expect("one backend field");
    let OwnedSqlValue::Signed(backend) = backend else {
        panic!("backend identity must be an integer");
    };
    pool.fetch_one(
        request(
            &pool,
            &format!("SELECT pg_terminate_backend({backend})"),
            ExecutionMode::FetchOne,
            RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
            RuntimeEffect::Read,
        ),
        ExecutionOptions::default(),
    )
    .await
    .expect("separate connection terminates transaction backend");
    let primary = cleanup_tx
        .fetch_one(
            request(
                &pool,
                "SELECT 1::bigint",
                ExecutionMode::FetchOne,
                RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect_err("terminated backend must fail");
    let preserved = cleanup_tx
        .finish_context::<()>(Err(primary.clone()))
        .await
        .expect_err("body error remains primary");
    assert_eq!(preserved.kind(), primary.kind());
    assert!(!preserved.secondary().is_empty());

    let timeout = pool
        .fetch_one(
            request(
                &pool,
                "SELECT 1::bigint FROM pg_sleep(1)",
                ExecutionMode::FetchOne,
                RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions {
                timeout: Some(Duration::from_millis(20)),
                cancellation: None,
            },
        )
        .await
        .expect_err("deadline must cancel and discard");
    assert_eq!(timeout.kind(), SqlErrorKind::Timeout);

    let carrier = sifr_sql_runtime::CancellationCarrier::new();
    let cancel_request = request(
        &pool,
        "SELECT 1::bigint FROM pg_sleep(1)",
        ExecutionMode::FetchOne,
        RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
        RuntimeEffect::Read,
    );
    let cancel_future = pool.fetch_one(
        cancel_request,
        ExecutionOptions {
            timeout: Some(Duration::from_secs(2)),
            cancellation: Some(carrier.clone()),
        },
    );
    tokio::pin!(cancel_future);
    let cancelled = tokio::select! {
        result = &mut cancel_future => result,
        () = tokio::time::sleep(Duration::from_millis(20)) => {
            let _request = carrier.request_cancel();
            cancel_future.await
        }
    }
    .expect_err("explicit cancellation must stop the query");
    assert_eq!(cancelled.kind(), SqlErrorKind::Cancelled);
    for _ in 0..64 {
        pool.fetch_one(
            request(
                &pool,
                "SELECT 1::bigint",
                ExecutionMode::FetchOne,
                RuntimeCardinality::new(1, Some(1)).expect("cardinality"),
                RuntimeEffect::Read,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("bounded repeated execution");
    }
    assert!(pool.statistics().total <= 3);
}
