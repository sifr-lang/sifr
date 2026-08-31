#![allow(clippy::expect_used)]

use rusqlite::Connection;
use semver::Version;
use sifr_sql_contract::{DialectIdentity, ProviderIdentity, schema_fingerprint};
use sifr_sql_runtime::{
    MIGRATION_EXECUTION_PLAN_FORMAT_VERSION, MigrationEngine, MigrationExecutionErrorKind,
    MigrationExecutionLimits, MigrationExecutionNode, MigrationExecutionPath,
    MigrationExecutionPlan, MigrationExecutionStatus, MigrationExecutionStep,
    MigrationExecutionStepKind, MigrationId, MigrationRuntime, MigrationRuntimeConstraint,
    MigrationStateId, MigrationTransactionRequirement,
};
use sifr_sql_sqlite_tools::{
    connect_migration_runtime, pull_live_catalog_from_path, validate_sqlite_execution_plan,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[test]
fn common_engine_rebuilds_locks_detects_drift_and_rolls_back_explicitly() {
    let directory = tempfile::tempdir().expect("directory");
    let planning = directory.path().join("planning.sqlite3");
    let database = directory.path().join("migration.sqlite3");
    create_baseline(&planning);
    create_baseline(&database);
    let baseline = fingerprint(&planning);

    let connection = Connection::open(&planning).expect("planning database");
    connection
        .execute_batch("PRAGMA foreign_keys=OFF; BEGIN IMMEDIATE")
        .expect("planning transaction");
    let forward_sql = [
        "CREATE TABLE __sifr_rebuild_items(id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES parents(id), value TEXT NOT NULL, note TEXT) STRICT",
        "INSERT INTO __sifr_rebuild_items(id, parent_id, value, note) SELECT id, parent_id, value, 'new' FROM items",
        "DROP TABLE items",
        "ALTER TABLE __sifr_rebuild_items RENAME TO items",
        "CREATE INDEX items_value ON items(value)",
    ];
    let forward_fingerprints = execute_and_fingerprint(&connection, &forward_sql);
    let target = forward_fingerprints.last().expect("target").clone();
    let rollback_sql = [
        "CREATE TABLE __sifr_rebuild_items(id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES parents(id), value TEXT) STRICT",
        "INSERT INTO __sifr_rebuild_items(id, parent_id, value) SELECT id, parent_id, value FROM items",
        "DROP TABLE items",
        "ALTER TABLE __sifr_rebuild_items RENAME TO items",
    ];
    let rollback_fingerprints = execute_and_fingerprint(&connection, &rollback_sql);
    assert_eq!(rollback_fingerprints.last(), Some(&baseline));
    connection
        .execute_batch("ROLLBACK")
        .expect("planning rollback");
    drop(connection);

    let plan = rebuild_plan(
        &baseline,
        &target,
        &forward_sql,
        &forward_fingerprints,
        &rollback_sql,
        &rollback_fingerprints,
    );
    validate_sqlite_execution_plan(&plan).expect("checked common graph");
    let mut runtime = make_runtime(&database, "sqlite-rebuild");
    runtime
        .import_baseline(&plan, &id("baseline"))
        .expect("truthful baseline import");

    let lock = runtime.acquire_lock(&plan).expect("first writer lock");
    let mut contender = make_runtime(&database, "sqlite-rebuild");
    assert!(contender.acquire_lock(&plan).is_err());
    runtime.release_lock(lock).expect("release writer lock");

    let engine = MigrationEngine::new(MigrationExecutionLimits::default());
    let report = engine.execute(&plan, &mut runtime).expect("apply rebuild");
    assert_eq!(report.status, MigrationExecutionStatus::Complete);
    assert_eq!(report.schema_fingerprint, target);
    let connection = Connection::open(&database).expect("reopen");
    let row: (String, String) = connection
        .query_row("SELECT value, note FROM items WHERE id=7", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .expect("preserved row");
    assert_eq!(row, ("kept".to_string(), "new".to_string()));
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("foreign keys"),
        0
    );
    drop(connection);

    Connection::open(&database)
        .expect("drift connection")
        .execute_batch("CREATE INDEX unowned_drift ON items(id)")
        .expect("introduce drift");
    let drift = engine
        .execute(&plan, &mut runtime)
        .expect_err("live schema drift");
    assert_eq!(drift.kind, MigrationExecutionErrorKind::SchemaDrift);
    Connection::open(&database)
        .expect("drift repair connection")
        .execute_batch("DROP INDEX unowned_drift")
        .expect("remove drift");

    let mut changed = plan.clone();
    changed
        .migrations
        .get_mut(&id("rebuild"))
        .expect("migration")
        .checksum = "f".repeat(64);
    let checksum = engine
        .execute(&changed, &mut runtime)
        .expect_err("applied checksum drift");
    assert_eq!(checksum.kind, MigrationExecutionErrorKind::ChecksumDrift);

    let rollback = engine
        .rollback_last(&plan, &mut runtime)
        .expect("explicit rollback");
    assert_eq!(rollback.heads, BTreeSet::from([id("baseline")]));
    assert_eq!(fingerprint(&database), baseline);
}

#[test]
fn migration_validation_rejects_multi_statement_and_unowned_scope_forms() {
    for sql in [
        "CREATE TABLE owned(id INTEGER); ATTACH 'other.db' AS other",
        "PRAGMA main . writable_schema = ON",
        "VACUUM main INTO 'copy.db'",
        "DETACH DATABASE other",
    ] {
        let baseline = "a".repeat(64);
        let target = "b".repeat(64);
        let forward_fingerprints = [target.clone()];
        let rollback_sql = ["DROP TABLE owned"];
        let rollback_fingerprints = [baseline.clone()];
        let plan = rebuild_plan(
            &baseline,
            &target,
            &[sql],
            &forward_fingerprints,
            &rollback_sql,
            &rollback_fingerprints,
        );
        assert!(validate_sqlite_execution_plan(&plan).is_err(), "{sql}");
    }
    let baseline = "a".repeat(64);
    let target = "b".repeat(64);
    let allowed = rebuild_plan(
        &baseline,
        &target,
        &["CREATE TABLE owned(value TEXT DEFAULT 'ATTACH')"],
        std::slice::from_ref(&target),
        &["DROP TABLE owned"],
        std::slice::from_ref(&baseline),
    );
    validate_sqlite_execution_plan(&allowed).expect("string contents are not migration syntax");
}

#[test]
fn failed_atomic_run_restores_the_ledger_baseline_and_can_retry() {
    let directory = tempfile::tempdir().expect("directory");
    let planning = directory.path().join("retry-planning.sqlite3");
    let database = directory.path().join("retry.sqlite3");
    for path in [&planning, &database] {
        Connection::open(path)
            .expect("open retry database")
            .execute_batch(
                "CREATE TABLE retry_keys(id INTEGER PRIMARY KEY) STRICT;
                 INSERT INTO retry_keys VALUES (1);",
            )
            .expect("retry baseline");
    }
    let baseline = fingerprint(&planning);
    Connection::open(&planning)
        .expect("planning")
        .execute_batch("CREATE INDEX retry_keys_index ON retry_keys(id)")
        .expect("target index");
    let target = fingerprint(&planning);
    let forward_sql = ["CREATE INDEX retry_keys_index ON retry_keys(id)"];
    let forward_fingerprints = [target.clone()];
    let rollback_sql = ["DROP INDEX retry_keys_index"];
    let rollback_fingerprints = [baseline.clone()];
    let mut plan = rebuild_plan(
        &baseline,
        &target,
        &forward_sql,
        &forward_fingerprints,
        &rollback_sql,
        &rollback_fingerprints,
    );
    let path = plan
        .migrations
        .get_mut(&id("rebuild"))
        .expect("migration")
        .paths
        .get_mut(&id("baseline"))
        .expect("path");
    path.steps.push(MigrationExecutionStep {
        id: id("insert-retry-key"),
        input_state: MigrationStateId::new("target-before-data"),
        output_state: MigrationStateId::new("target-after-data"),
        input_fingerprint: target.clone(),
        output_fingerprint: target.clone(),
        checksum: "e".repeat(64),
        kind: MigrationExecutionStepKind::SqlData {
            normalized_statement: "INSERT INTO retry_keys VALUES (1)".to_string(),
        },
    });
    validate_sqlite_execution_plan(&plan).expect("retry graph");
    let mut runtime = make_runtime(&database, "sqlite-retry");
    runtime
        .import_baseline(&plan, &id("baseline"))
        .expect("retry baseline import");
    let engine = MigrationEngine::new(MigrationExecutionLimits::default());
    let interrupted = engine
        .execute(&plan, &mut runtime)
        .expect_err("duplicate data step");
    assert_eq!(interrupted.kind, MigrationExecutionErrorKind::Step);
    assert_eq!(fingerprint(&database), baseline);

    Connection::open(&database)
        .expect("repair")
        .execute("DELETE FROM retry_keys WHERE id=1", [])
        .expect("operator repair");
    let resumed = engine.execute(&plan, &mut runtime).expect("safe retry");
    assert_eq!(resumed.status, MigrationExecutionStatus::Complete);
    assert_eq!(resumed.schema_fingerprint, target);
}

fn create_baseline(path: &Path) {
    Connection::open(path)
        .expect("open")
        .execute_batch(
            "PRAGMA foreign_keys=ON;
             CREATE TABLE parents(id INTEGER PRIMARY KEY) STRICT;
             CREATE TABLE items(id INTEGER PRIMARY KEY, parent_id INTEGER REFERENCES parents(id), value TEXT) STRICT;
             INSERT INTO parents VALUES (1);
             INSERT INTO items VALUES (7, 1, 'kept');",
        )
        .expect("baseline");
}

fn execute_and_fingerprint(connection: &Connection, statements: &[&str]) -> Vec<String> {
    statements
        .iter()
        .map(|statement| {
            connection.execute_batch(statement).expect("planning step");
            let schema = sifr_sql_sqlite_tools::pull_live_catalog_from_connection(
                connection,
                provider(),
                dialect(),
            )
            .expect("planning catalog");
            schema_fingerprint(&schema)
                .expect("planning fingerprint")
                .as_str()
                .to_string()
        })
        .collect()
}

fn rebuild_plan(
    baseline: &str,
    target: &str,
    forward_sql: &[&str],
    forward_fingerprints: &[String],
    rollback_sql: &[&str],
    rollback_fingerprints: &[String],
) -> MigrationExecutionPlan {
    let forward = steps(baseline, forward_sql, forward_fingerprints, "forward");
    let rollback = steps(target, rollback_sql, rollback_fingerprints, "rollback");
    let baseline_id = id("baseline");
    let head = id("rebuild");
    MigrationExecutionPlan {
        format_version: MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
        provider_family: "sqlite".to_string(),
        target_fingerprint: target.to_string(),
        head: head.clone(),
        topological_order: vec![head.clone()],
        baseline_fingerprints: BTreeMap::from([(baseline_id.clone(), baseline.to_string())]),
        migrations: BTreeMap::from([(
            head.clone(),
            MigrationExecutionNode {
                id: head,
                parents: BTreeSet::from([baseline_id.clone()]),
                provider: MigrationRuntimeConstraint {
                    family: "sqlite".to_string(),
                    minimum_server_version: Some("3.53.2".to_string()),
                    required_capabilities: BTreeSet::new(),
                },
                transaction_requirement: MigrationTransactionRequirement::Optional,
                checksum: "a".repeat(64),
                paths: BTreeMap::from([(
                    baseline_id.clone(),
                    MigrationExecutionPath {
                        parent: baseline_id,
                        input_fingerprint: baseline.to_string(),
                        output_fingerprint: target.to_string(),
                        steps: forward,
                        rollback: Some(rollback),
                    },
                )]),
                author: "qualification".to_string(),
                created_at: "2026-08-31T00:00:00Z".to_string(),
            },
        )]),
    }
}

fn steps(
    input: &str,
    statements: &[&str],
    fingerprints: &[String],
    prefix: &str,
) -> Vec<MigrationExecutionStep> {
    let mut prior = input.to_string();
    statements
        .iter()
        .zip(fingerprints)
        .enumerate()
        .map(|(index, (statement, output))| {
            let step = MigrationExecutionStep {
                id: id(&format!("{prefix}-{index}")),
                input_state: MigrationStateId::new(format!("{prefix}-state-{index}")),
                output_state: MigrationStateId::new(format!("{prefix}-state-{}", index + 1)),
                input_fingerprint: prior.clone(),
                output_fingerprint: output.clone(),
                checksum: format!("{:064x}", index + 1),
                kind: if statement.starts_with("INSERT") {
                    MigrationExecutionStepKind::SqlData {
                        normalized_statement: (*statement).to_string(),
                    }
                } else {
                    MigrationExecutionStepKind::Ddl {
                        statement: (*statement).to_string(),
                    }
                },
            };
            prior.clone_from(output);
            step
        })
        .collect()
}

fn make_runtime(path: &Path, ledger: &str) -> sifr_sql_sqlite_tools::SqliteMigrationRuntime {
    connect_migration_runtime(path, provider(), dialect(), ledger).expect("runtime")
}

fn fingerprint(path: &Path) -> String {
    let schema = pull_live_catalog_from_path(path, provider(), dialect()).expect("catalog");
    schema_fingerprint(&schema)
        .expect("fingerprint")
        .as_str()
        .to_string()
}

fn provider() -> ProviderIdentity {
    ProviderIdentity {
        package_id: "sifr-sql-sqlite".to_string(),
        package_version: Version::new(0, 0, 0),
        package_source: "workspace".to_string(),
        package_graph_digest: "a".repeat(64),
        compiler_components: BTreeMap::from([("sqlite".to_string(), "b".repeat(64))]),
    }
}

fn dialect() -> DialectIdentity {
    DialectIdentity {
        family: "sqlite".to_string(),
        server_version: "3.53.2".to_string(),
        modes: BTreeSet::new(),
        features: BTreeSet::new(),
    }
}

fn id(value: &str) -> MigrationId {
    MigrationId::new(value)
}
