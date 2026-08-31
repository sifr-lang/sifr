#![allow(clippy::expect_used)]

use mysql_async::{Opts, OptsBuilder};
use sifr_sql_mysql_runtime::{
    ExecutionOptions, MysqlProfile, MysqlSchemaVerifier, MysqlTlsPolicy, connect,
};
use sifr_sql_runtime::{
    BoundParameters, CancellationCarrier, ExecutionMetadata, ExecutionMode, ExecutionRequest,
    IsolationLevel, PoolingMode, ProviderFuture, RuntimeCardinality, RuntimeEffect,
    RuntimeEffectContract, RuntimeLimits, SchemaDependencySlice, SchemaStrictness, SessionContract,
    SqlErrorKind,
};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

struct StaticVerifier;

impl MysqlSchemaVerifier for StaticVerifier {
    fn observe<'a>(
        &'a self,
        _connection: &'a mut mysql_async::Conn,
        profile: &'a MysqlProfile,
    ) -> ProviderFuture<'a, SchemaDependencySlice> {
        ProviderFuture::new(async move {
            SchemaDependencySlice::new(profile.schema_fingerprint(), std::iter::empty())
        })
    }
}

fn profile(url: &str) -> MysqlProfile {
    let opts = Opts::from_url(url).expect("URL");
    let opts: Opts = OptsBuilder::from_opts(opts).stmt_cache_size(0).into();
    MysqlProfile::new(
        opts.clone(),
        opts,
        "b".repeat(64),
        SchemaDependencySlice::new("a".repeat(64), std::iter::empty()).expect("schema"),
        SchemaStrictness::Exact,
        SessionContract {
            search_path: vec!["app".to_string()],
            time_zone: "+00:00".to_string(),
            role: None,
            default_isolation: IsolationLevel::RepeatableRead,
            read_only: false,
            pooling: PoolingMode::Session,
            requires_session_affinity: false,
        },
        RuntimeLimits {
            max_connections: 2,
            acquire_timeout: Duration::from_secs(5),
            statement_timeout: Duration::from_secs(2),
            cleanup_timeout: Duration::from_secs(2),
            max_decoded_row_bytes: 1024 * 1024,
            max_collected_rows: 100,
            statement_cache_capacity: 4,
            max_parameters: 100,
        },
        MysqlTlsPolicy::DisabledForLocalTest,
        Arc::new(StaticVerifier),
    )
    .expect("profile")
}

fn read_request(
    profile: &MysqlProfile,
    statement: &str,
    mode: ExecutionMode,
) -> ExecutionRequest<MysqlProfile> {
    ExecutionRequest {
        profile: Arc::new(profile.clone()),
        statement: Arc::from(statement),
        parameters: BoundParameters::default(),
        cardinality: RuntimeCardinality::new(0, Some(100)).expect("cardinality"),
        effects: RuntimeEffectContract::new(
            RuntimeEffect::Read,
            vec!["app.users".to_string()],
            Vec::new(),
        )
        .expect("effects"),
        returns_rows: true,
        metadata: ExecutionMetadata {
            normalized_statement_fingerprint: "c".repeat(64),
            parameter_type_fingerprint: "d".repeat(64),
            result_type_fingerprint: "e".repeat(64),
            schema_fingerprint: "a".repeat(64),
        },
        mode,
    }
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires SIFR_MYSQL_TEST_URL"]
async fn live_pool_stream_cache_and_kill_query_are_safe() {
    let url = std::env::var("SIFR_MYSQL_TEST_URL").expect("test URL");
    let profile = profile(&url);
    let pool = connect(profile.clone()).await.expect("verified pool");
    let rows = pool
        .fetch_all(
            read_request(
                &profile,
                "SELECT id, email FROM users ORDER BY id",
                ExecutionMode::FetchAll { maximum_rows: 10 },
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("rows");
    assert!(!rows.is_empty());
    let mut stream = pool
        .stream(
            read_request(
                &profile,
                "SELECT id, email FROM users ORDER BY id",
                ExecutionMode::Stream,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("stream");
    assert!(stream.next().await.expect("row").is_some());
    stream.aclose().await.expect("bounded close");

    let timed_out = pool
        .fetch_all(
            read_request(
                &profile,
                "SELECT SLEEP(1)",
                ExecutionMode::FetchAll { maximum_rows: 1 },
            ),
            ExecutionOptions {
                timeout: Some(Duration::from_millis(20)),
                cancellation: None,
            },
        )
        .await;
    assert!(timed_out.is_err());
    let healthy = pool
        .fetch_all(
            read_request(
                &profile,
                "SELECT id FROM users LIMIT 1",
                ExecutionMode::FetchAll { maximum_rows: 1 },
            ),
            ExecutionOptions::default(),
        )
        .await;
    assert!(healthy.is_ok());

    let cancellation = CancellationCarrier::new();
    let cancellation_request = cancellation.clone();
    let cancelled_operation = pool.fetch_all(
        read_request(
            &profile,
            "SELECT SLEEP(10)",
            ExecutionMode::FetchAll { maximum_rows: 1 },
        ),
        ExecutionOptions {
            timeout: None,
            cancellation: Some(cancellation),
        },
    );
    tokio::pin!(cancelled_operation);
    assert!(
        tokio::time::timeout(Duration::from_millis(100), &mut cancelled_operation)
            .await
            .is_err()
    );
    let _request = cancellation_request.request_cancel();
    let cancelled = tokio::time::timeout(Duration::from_secs(3), &mut cancelled_operation)
        .await
        .expect("carrier cancellation stayed within the cleanup budget");
    assert_eq!(
        cancelled.expect_err("query must be cancelled").kind(),
        SqlErrorKind::Cancelled
    );
    assert!(
        cancellation_request
            .take_async_cleanup_evidence()
            .is_empty()
    );
    assert!(
        pool.fetch_all(
            read_request(
                &profile,
                "SELECT id FROM users LIMIT 1",
                ExecutionMode::FetchAll { maximum_rows: 1 },
            ),
            ExecutionOptions::default(),
        )
        .await
        .is_ok()
    );

    let warm = pool
        .fetch_all(
            read_request(
                &profile,
                "SELECT id FROM users LIMIT 1",
                ExecutionMode::FetchAll { maximum_rows: 1 },
            ),
            ExecutionOptions::default(),
        )
        .await;
    assert!(warm.is_ok());
    let started = Instant::now();
    for _ in 0..100 {
        let measured = pool
            .fetch_all(
                read_request(
                    &profile,
                    "SELECT id FROM users LIMIT 1",
                    ExecutionMode::FetchAll { maximum_rows: 1 },
                ),
                ExecutionOptions::default(),
            )
            .await;
        assert!(measured.is_ok());
    }
    assert!(
        started.elapsed() <= Duration::from_secs(10),
        "warm MySQL runtime batch exceeded the 10-second live budget"
    );
}
