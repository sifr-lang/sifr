#![allow(clippy::expect_used)]

use sifr_sql_runtime::{
    BoundParameters, CancellationCarrier, ExecutionMetadata, ExecutionMode, ExecutionRequest,
    OwnedParameter, OwnedSqlValue, RuntimeCardinality, RuntimeCodecIdentity, RuntimeEffect,
    RuntimeEffectContract, RuntimeLimits, SchemaDependencySlice, SchemaProperty, SchemaStrictness,
};
use sifr_sql_sqlite_runtime::{
    ExecutionOptions, SqliteEvidence, SqliteProfile, VerificationProbe, open_pool,
};
use std::sync::Arc;
use std::time::Duration;

fn profile(path: &std::path::Path) -> SqliteProfile {
    profile_with_attached(path, std::collections::BTreeMap::new())
}

fn profile_with_attached(
    path: &std::path::Path,
    attached_files: std::collections::BTreeMap<String, std::path::PathBuf>,
) -> SqliteProfile {
    let expected_schema = SchemaDependencySlice::new(
        "b".repeat(64),
        [SchemaProperty::new("main.schema", Some("ready".to_string())).expect("property")],
    )
    .expect("schema");
    SqliteProfile::new(
        path,
        "a".repeat(64),
        expected_schema,
        SqliteEvidence::Introspection {
            fingerprint_statement: format!("SELECT '{}'", "b".repeat(64)),
            probes: vec![VerificationProbe::new("main.schema", "SELECT 'ready'").expect("probe")],
        },
        SchemaStrictness::Exact,
        attached_files,
        vec!["json".to_string()],
        (3, 53, 2),
        RuntimeLimits {
            statement_timeout: Duration::from_secs(2),
            acquire_timeout: Duration::from_secs(2),
            cleanup_timeout: Duration::from_secs(2),
            max_connections: 2,
            max_collected_rows: 100,
            max_decoded_row_bytes: 1_024,
            statement_cache_capacity: 8,
            max_parameters: 8,
        },
        250,
    )
    .expect("profile")
}

#[tokio::test(flavor = "current_thread")]
async fn attached_schema_is_opened_and_queryable_on_every_worker() {
    let directory = tempfile::tempdir().expect("directory");
    let attached = directory.path().join("analytics.sqlite3");
    rusqlite::Connection::open(&attached)
        .expect("attached database")
        .execute_batch("CREATE TABLE events(id INTEGER PRIMARY KEY, value TEXT); INSERT INTO events VALUES (1, 'attached')")
        .expect("attached schema");
    let selected = profile_with_attached(
        &directory.path().join("main.sqlite3"),
        std::collections::BTreeMap::from([("analytics".to_string(), attached)]),
    );
    let pool = open_pool(selected)
        .expect("pool")
        .verify_schema()
        .await
        .expect("verification");
    let selected = pool.acquire().await.expect("worker").profile();
    let rows = pool
        .fetch_all(
            request(
                selected,
                "SELECT value FROM analytics.events WHERE id = 1",
                vec![],
                true,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("attached query");
    assert_eq!(
        rows[0].values(),
        &[OwnedSqlValue::Text("attached".to_string())]
    );
}

fn request(
    profile: Arc<SqliteProfile>,
    sql: &str,
    parameters: Vec<OwnedSqlValue>,
    returns_rows: bool,
) -> ExecutionRequest<SqliteProfile> {
    let bound = parameters
        .into_iter()
        .enumerate()
        .map(|(slot, value)| OwnedParameter {
            slot: u32::try_from(slot).expect("slot"),
            codec: RuntimeCodecIdentity::new("sqlite.dynamic.v1").expect("codec"),
            value,
        })
        .collect();
    ExecutionRequest {
        profile,
        statement: Arc::from(sql),
        parameters: BoundParameters::new(bound).expect("parameters"),
        cardinality: RuntimeCardinality::new(0, if returns_rows { None } else { Some(0) })
            .expect("cardinality"),
        effects: RuntimeEffectContract::new(
            if returns_rows {
                RuntimeEffect::Read
            } else {
                RuntimeEffect::Write
            },
            vec![],
            if returns_rows {
                vec![]
            } else {
                vec!["main.items".to_string()]
            },
        )
        .expect("effects"),
        returns_rows,
        metadata: ExecutionMetadata {
            normalized_statement_fingerprint: "c".repeat(64),
            parameter_type_fingerprint: "d".repeat(64),
            result_type_fingerprint: "e".repeat(64),
            schema_fingerprint: "b".repeat(64),
        },
        mode: if returns_rows {
            ExecutionMode::FetchAll { maximum_rows: 100 }
        } else {
            ExecutionMode::Execute
        },
    }
}

#[tokio::test(flavor = "current_thread")]
async fn dedicated_workers_execute_decode_reset_and_reuse() {
    let directory = tempfile::tempdir().expect("directory");
    let pool = open_pool(profile(&directory.path().join("runtime.sqlite3")))
        .expect("pool")
        .verify_schema()
        .await
        .expect("verification");
    let mut connection = pool.acquire().await.expect("connection");
    let selected_profile = connection.profile();
    let uncached_create = connection
        .execute(
            request(
                Arc::clone(&selected_profile),
                "CREATE TABLE items(id INTEGER PRIMARY KEY, value TEXT NOT NULL) STRICT",
                vec![],
                false,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("create");
    connection
        .execute(
            request(
                Arc::clone(&selected_profile),
                "INSERT INTO items(value) VALUES (?)",
                vec![OwnedSqlValue::Text("hello".to_string())],
                false,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("insert");
    assert!(!uncached_create.metadata.statement_cache_hit);
    let first_cached_insert = connection
        .execute(
            request(
                Arc::clone(&selected_profile),
                "INSERT INTO items(value) VALUES (?)",
                vec![OwnedSqlValue::Text("first cached".to_string())],
                false,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("first cached insert");
    let second_cached_insert = connection
        .execute(
            request(
                Arc::clone(&selected_profile),
                "INSERT INTO items(value) VALUES (?)",
                vec![OwnedSqlValue::Text("second cached".to_string())],
                false,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("second cached insert");
    assert!(first_cached_insert.metadata.statement_cache_hit);
    assert!(second_cached_insert.metadata.statement_cache_hit);
    let rows = connection
        .fetch_all(
            request(
                selected_profile,
                "SELECT id, value FROM items",
                vec![],
                true,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("fetch");
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[0].values()[1],
        OwnedSqlValue::Text("hello".to_string())
    );
    connection.release(None).await.expect("release");
    assert_eq!(pool.statistics().idle, 1);
}

#[tokio::test(flavor = "current_thread")]
async fn one_row_fetches_report_cardinality_before_collection_limits() {
    let directory = tempfile::tempdir().expect("directory");
    let pool = open_pool(profile(&directory.path().join("cardinality.sqlite3")))
        .expect("pool")
        .verify_schema()
        .await
        .expect("verification");
    let mut connection = pool.acquire().await.expect("connection");
    let selected = connection.profile();
    connection
        .execute(
            request(
                Arc::clone(&selected),
                "CREATE TABLE items(id INTEGER PRIMARY KEY) STRICT",
                vec![],
                false,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("create");
    connection
        .execute(
            request(
                Arc::clone(&selected),
                "INSERT INTO items VALUES (1), (2), (3)",
                vec![],
                false,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("insert");
    let mut one = request(
        Arc::clone(&selected),
        "SELECT id FROM items ORDER BY id",
        vec![],
        true,
    );
    one.mode = ExecutionMode::FetchOne;
    let error = connection
        .fetch_one(one, ExecutionOptions::default())
        .await
        .expect_err("three rows violate fetch_one");
    assert_eq!(error.kind(), sifr_sql_runtime::SqlErrorKind::Cardinality);
    let mut optional = request(selected, "SELECT id FROM items ORDER BY id", vec![], true);
    optional.mode = ExecutionMode::FetchOptional;
    let error = connection
        .fetch_optional(optional, ExecutionOptions::default())
        .await
        .expect_err("three rows violate fetch_optional");
    assert_eq!(error.kind(), sifr_sql_runtime::SqlErrorKind::Cardinality);
}

#[tokio::test(flavor = "current_thread")]
async fn warmed_statement_cache_meets_the_local_execution_budget() {
    let directory = tempfile::tempdir().expect("directory");
    let pool = open_pool(profile(&directory.path().join("performance.sqlite3")))
        .expect("pool")
        .verify_schema()
        .await
        .expect("verification");
    let mut connection = pool.acquire().await.expect("connection");
    let selected = connection.profile();
    let sql = "SELECT ? + 1";
    for _ in 0..20 {
        connection
            .fetch_all(
                request(
                    Arc::clone(&selected),
                    sql,
                    vec![OwnedSqlValue::Signed(1)],
                    true,
                ),
                ExecutionOptions::default(),
            )
            .await
            .expect("warmup");
    }
    let started = std::time::Instant::now();
    for _ in 0..200 {
        connection
            .fetch_all(
                request(
                    Arc::clone(&selected),
                    sql,
                    vec![OwnedSqlValue::Signed(1)],
                    true,
                ),
                ExecutionOptions::default(),
            )
            .await
            .expect("hot execution");
    }
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "200 warmed executions exceeded the two-second qualification budget"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn interrupt_cancellation_retires_the_worker_with_a_bound() {
    let directory = tempfile::tempdir().expect("directory");
    let pool = open_pool(profile(&directory.path().join("cancel.sqlite3")))
        .expect("pool")
        .verify_schema()
        .await
        .expect("verification");
    let mut connection = pool.acquire().await.expect("connection");
    let carrier = CancellationCarrier::new();
    let request_carrier = carrier.clone();
    let thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        let _ = request_carrier.request_cancel();
    });
    let profile = connection.profile();
    let result = connection
        .fetch_all(
            request(
                profile,
                "WITH RECURSIVE count(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM count WHERE x < 100000000) SELECT sum(x) FROM count",
                vec![],
                true,
            ),
            ExecutionOptions {
                timeout: Some(Duration::from_secs(1)),
                cancellation: Some(carrier),
            },
        )
        .await;
    thread.join().expect("cancellation thread");
    assert!(result.is_err());
    assert!(connection.is_poisoned());
    assert_eq!(pool.statistics().idle, 0);
}

#[tokio::test(flavor = "current_thread")]
async fn malformed_database_bytes_are_structured_errors_not_panics() {
    let directory = tempfile::tempdir().expect("directory");
    let path = directory.path().join("corrupt.sqlite3");
    std::fs::write(&path, b"not a sqlite database").expect("corrupt fixture");
    let pool = open_pool(profile(&path)).expect("pool");
    let outcome = pool.verify_schema().await;
    assert!(outcome.is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn damaged_page_in_a_valid_database_is_a_structured_error_not_a_panic() {
    use std::io::{Seek as _, SeekFrom, Write as _};

    let directory = tempfile::tempdir().expect("directory");
    let path = directory.path().join("page-corrupt.sqlite3");
    {
        let connection = rusqlite::Connection::open(&path).expect("valid database");
        connection
            .execute_batch(
                "PRAGMA page_size=4096; CREATE TABLE damage(value TEXT NOT NULL); \
                 WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x < 2000) \
                 INSERT INTO damage SELECT printf('%01000d', x) FROM n;",
            )
            .expect("populate multiple pages");
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("open valid database for page damage");
    file.seek(SeekFrom::Start(4096 + 128)).expect("seek page");
    file.write_all(&[0xff; 512]).expect("damage one page");
    drop(file);

    let pool = open_pool(profile(&path))
        .expect("pool")
        .verify_schema()
        .await
        .expect("header and profile verification");
    let mut connection = pool.acquire().await.expect("connection");
    let selected_profile = connection.profile();
    let result = connection
        .fetch_all(
            request(
                selected_profile,
                "SELECT sum(length(value)) FROM damage",
                vec![],
                true,
            ),
            ExecutionOptions::default(),
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn verified_pool_streams_with_one_row_backpressure_and_releases_on_exhaustion() {
    let directory = tempfile::tempdir().expect("directory");
    let pool = open_pool(profile(&directory.path().join("stream.sqlite3")))
        .expect("pool")
        .verify_schema()
        .await
        .expect("verification");
    let mut connection = pool.acquire().await.expect("connection");
    let selected_profile = connection.profile();
    connection
        .execute(
            request(
                Arc::clone(&selected_profile),
                "CREATE TABLE items(id INTEGER PRIMARY KEY, value TEXT NOT NULL) STRICT",
                vec![],
                false,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("create");
    connection
        .execute(
            request(
                Arc::clone(&selected_profile),
                "INSERT INTO items(value) VALUES ('one'), ('two'), ('three')",
                vec![],
                false,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("insert");
    connection.release(None).await.expect("release");

    let mut stream_request = request(
        selected_profile,
        "SELECT id, value FROM items ORDER BY id",
        vec![],
        true,
    );
    stream_request.mode = ExecutionMode::Stream;
    stream_request.cardinality = RuntimeCardinality::new(0, Some(3)).expect("cardinality");
    let mut stream = pool
        .stream(stream_request, ExecutionOptions::default())
        .await
        .expect("stream");
    let mut values = Vec::new();
    while let Some(row) = stream.next().await.expect("next") {
        values.push(row.values()[1].clone());
    }
    assert_eq!(
        values,
        vec![
            OwnedSqlValue::Text("one".to_string()),
            OwnedSqlValue::Text("two".to_string()),
            OwnedSqlValue::Text("three".to_string()),
        ]
    );
    assert_eq!(pool.statistics().idle, 1);
}

#[tokio::test(flavor = "current_thread")]
async fn write_lock_timeout_is_structured_and_rollback_recovers_the_pool() {
    let directory = tempfile::tempdir().expect("directory");
    let pool = open_pool(profile(&directory.path().join("locking.sqlite3")))
        .expect("pool")
        .verify_schema()
        .await
        .expect("verification");
    let mut setup = pool.acquire().await.expect("setup connection");
    let selected_profile = setup.profile();
    setup
        .execute(
            request(
                Arc::clone(&selected_profile),
                "CREATE TABLE items(id INTEGER PRIMARY KEY, value TEXT NOT NULL) STRICT",
                vec![],
                false,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect("create");
    setup.release(None).await.expect("release setup");

    let transaction = pool.transaction().await.expect("transaction");
    let mut contender = pool.acquire().await.expect("contender");
    let locked = contender
        .execute(
            request(
                Arc::clone(&selected_profile),
                "INSERT INTO items(value) VALUES ('blocked')",
                vec![],
                false,
            ),
            ExecutionOptions::default(),
        )
        .await
        .expect_err("write lock must fail within the busy timeout");
    assert_eq!(locked.kind(), sifr_sql_runtime::SqlErrorKind::Timeout);
    contender.release(None).await.expect("release contender");
    transaction.rollback().await.expect("rollback");

    pool.execute(
        request(
            selected_profile,
            "INSERT INTO items(value) VALUES ('recovered')",
            vec![],
            false,
        ),
        ExecutionOptions::default(),
    )
    .await
    .expect("recovered write");
}
