#![allow(clippy::expect_used)]

use mysql_async::{Conn, Opts, prelude::Queryable};
use semver::Version;
use sifr_sql_contract::{DialectIdentity, ProviderIdentity, schema_fingerprint};
use sifr_sql_mysql_tools::{
    connect_migration_runtime, pull_live_catalog, validate_mysql_migration_plan,
};
use sifr_sql_runtime::{
    MIGRATION_EXECUTION_PLAN_FORMAT_VERSION, MigrationEngine, MigrationExecutionErrorKind,
    MigrationExecutionLimits, MigrationExecutionNode, MigrationExecutionPath,
    MigrationExecutionPlan, MigrationExecutionStatus, MigrationExecutionStep,
    MigrationExecutionStepKind, MigrationId, MigrationRuntime, MigrationRuntimeConstraint,
    MigrationStateId, MigrationTransactionRequirement,
};
use std::collections::{BTreeMap, BTreeSet};

const TABLE: &str = "sifr_migration_accounts";
const LEDGER_IDENTITY: &str = "mysql-live-qualification";

#[test]
#[ignore = "requires SIFR_MYSQL_TEST_URL"]
fn live_mysql_migration_contract() {
    let url = std::env::var("SIFR_MYSQL_TEST_URL").expect("URL");
    let series = std::env::var("SIFR_MYSQL_TEST_SERIES").expect("series");
    cleanup(&url);
    let dialect = live_dialect(&url, &series);
    let baseline = live_fingerprint(&url, &dialect);
    execute_sql(
        &url,
        &format!("CREATE TABLE `{TABLE}` (id BIGINT UNSIGNED PRIMARY KEY)"),
    );
    let with_table = live_fingerprint(&url, &dialect);
    execute_sql(&url, &format!("DROP TABLE `{TABLE}`"));
    assert_eq!(live_fingerprint(&url, &dialect), baseline);

    let plan = qualification_plan(&series, &baseline, &with_table);
    validate_mysql_migration_plan(&plan).expect("checked MySQL plan");
    let mut runtime = migration_runtime(&url, dialect.clone());
    runtime
        .import_baseline(&plan, &id("baseline"))
        .expect("truthful baseline import");

    let lock = runtime.acquire_lock(&plan).expect("first lock");
    let mut contender = migration_runtime(&url, dialect.clone());
    assert!(contender.acquire_lock(&plan).is_err());
    runtime.release_lock(lock).expect("release lock");
    drop(contender);

    let engine = MigrationEngine::new(MigrationExecutionLimits::default());
    let interrupted = engine
        .execute(&plan, &mut runtime)
        .expect_err("duplicate row failure");
    assert_eq!(interrupted.kind, MigrationExecutionErrorKind::Step);
    execute_sql(&url, &format!("DELETE FROM `{TABLE}` WHERE id = 1"));
    let resumed = engine
        .execute(&plan, &mut runtime)
        .expect("resume same plan");
    assert_eq!(resumed.status, MigrationExecutionStatus::Complete);
    assert_eq!(resumed.heads, BTreeSet::from([id("head")]));

    let mut drifted = plan.clone();
    drifted
        .migrations
        .get_mut(&id("head"))
        .expect("head")
        .checksum = "f".repeat(64);
    let drift = engine
        .execute(&drifted, &mut runtime)
        .expect_err("checksum drift");
    assert_eq!(drift.kind, MigrationExecutionErrorKind::ChecksumDrift);

    let rolled_back = engine
        .rollback_last(&plan, &mut runtime)
        .expect("explicit rollback");
    assert_eq!(rolled_back.status, MigrationExecutionStatus::Complete);
    assert_eq!(rolled_back.heads, BTreeSet::from([id("baseline")]));
    assert_eq!(live_fingerprint(&url, &dialect), baseline);
    drop(runtime);
    cleanup(&url);
}

fn qualification_plan(
    series: &str,
    baseline_fingerprint: &str,
    table_fingerprint: &str,
) -> MigrationExecutionPlan {
    let baseline = id("baseline");
    let head = id("head");
    let forward = vec![
        step(
            "before-create",
            baseline_fingerprint,
            baseline_fingerprint,
            MigrationExecutionStepKind::RecoveryPoint {
                name: "before-create".to_string(),
            },
            'a',
        ),
        step(
            "create-table",
            baseline_fingerprint,
            table_fingerprint,
            MigrationExecutionStepKind::Ddl {
                statement: format!("CREATE TABLE `{TABLE}` (id BIGINT UNSIGNED PRIMARY KEY)"),
            },
            'b',
        ),
        step(
            "insert-first",
            table_fingerprint,
            table_fingerprint,
            MigrationExecutionStepKind::SqlData {
                statement: format!("INSERT INTO `{TABLE}` VALUES (1)"),
                normalized_statement: format!("INSERT INTO `{TABLE}` VALUES (1)"),
            },
            'c',
        ),
        step(
            "insert-duplicate",
            table_fingerprint,
            table_fingerprint,
            MigrationExecutionStepKind::SqlData {
                statement: format!("INSERT INTO `{TABLE}` VALUES (1)"),
                normalized_statement: format!("INSERT INTO `{TABLE}` VALUES (1)"),
            },
            'd',
        ),
    ];
    let rollback = vec![
        step(
            "before-drop",
            table_fingerprint,
            table_fingerprint,
            MigrationExecutionStepKind::RecoveryPoint {
                name: "before-drop".to_string(),
            },
            'e',
        ),
        step(
            "drop-table",
            table_fingerprint,
            baseline_fingerprint,
            MigrationExecutionStepKind::Ddl {
                statement: format!("DROP TABLE `{TABLE}`"),
            },
            'f',
        ),
    ];
    MigrationExecutionPlan {
        format_version: MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
        provider_family: "mysql".to_string(),
        target_fingerprint: table_fingerprint.to_string(),
        head: head.clone(),
        topological_order: vec![head.clone()],
        baseline_fingerprints: BTreeMap::from([(
            baseline.clone(),
            baseline_fingerprint.to_string(),
        )]),
        migrations: BTreeMap::from([(
            head.clone(),
            MigrationExecutionNode {
                id: head,
                parents: BTreeSet::from([baseline.clone()]),
                provider: MigrationRuntimeConstraint {
                    family: "mysql".to_string(),
                    minimum_server_version: Some(format!("{series}.0")),
                    required_capabilities: BTreeSet::new(),
                },
                transaction_requirement: MigrationTransactionRequirement::Optional,
                checksum: "9".repeat(64),
                paths: BTreeMap::from([(
                    baseline.clone(),
                    MigrationExecutionPath {
                        parent: baseline,
                        input_fingerprint: baseline_fingerprint.to_string(),
                        output_fingerprint: table_fingerprint.to_string(),
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

fn step(
    name: &str,
    input: &str,
    output: &str,
    kind: MigrationExecutionStepKind,
    checksum: char,
) -> MigrationExecutionStep {
    MigrationExecutionStep {
        id: id(name),
        input_state: MigrationStateId::new(format!("{name}-input")),
        output_state: MigrationStateId::new(format!("{name}-output")),
        input_fingerprint: input.to_string(),
        output_fingerprint: output.to_string(),
        checksum: checksum.to_string().repeat(64),
        kind,
    }
}

fn migration_runtime(
    url: &str,
    dialect: DialectIdentity,
) -> sifr_sql_mysql_tools::MysqlMigrationRuntime {
    connect_migration_runtime(url, provider(), dialect, LEDGER_IDENTITY).expect("migration runtime")
}

fn live_dialect(url: &str, series: &str) -> DialectIdentity {
    let (modes, character_set, collation): (String, String, String) = query_first(
        url,
        "SELECT @@session.sql_mode, @@character_set_database, @@collation_database",
    );
    DialectIdentity {
        family: "mysql".to_string(),
        server_version: series.to_string(),
        modes: modes
            .split(',')
            .filter(|mode| !mode.is_empty())
            .map(str::to_string)
            .chain([
                format!("character-set:{character_set}"),
                format!("collation:{collation}"),
            ])
            .collect(),
        features: BTreeSet::new(),
    }
}

fn live_fingerprint(url: &str, dialect: &DialectIdentity) -> String {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("catalog runtime");
    let schema = runtime
        .block_on(pull_live_catalog(url, provider(), dialect.clone()))
        .expect("live catalog");
    schema_fingerprint(&schema)
        .expect("schema fingerprint")
        .as_str()
        .to_string()
}

fn execute_sql(url: &str, statement: &str) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("SQL runtime");
    runtime.block_on(async {
        let mut connection = Conn::new(Opts::from_url(url).expect("opts"))
            .await
            .expect("connection");
        connection.query_drop(statement).await.expect("statement");
        connection.disconnect().await.expect("disconnect");
    });
}

fn query_first<T>(url: &str, statement: &str) -> T
where
    T: mysql_async::prelude::FromRow + Send + 'static,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("query runtime");
    runtime.block_on(async {
        let mut connection = Conn::new(Opts::from_url(url).expect("opts"))
            .await
            .expect("connection");
        let value = connection
            .query_first(statement)
            .await
            .expect("query")
            .expect("row");
        connection.disconnect().await.expect("disconnect");
        value
    })
}

fn cleanup(url: &str) {
    execute_sql(url, &format!("DROP TABLE IF EXISTS `{TABLE}`"));
    execute_sql(
        url,
        "CREATE TABLE IF NOT EXISTS `sifr_migration_ledger` (identity VARCHAR(191) PRIMARY KEY, payload JSON NOT NULL)",
    );
    execute_sql(
        url,
        &format!("DELETE FROM `sifr_migration_ledger` WHERE identity = '{LEDGER_IDENTITY}'"),
    );
}

fn provider() -> ProviderIdentity {
    ProviderIdentity {
        package_id: "sifr-sql-mysql@0.0.0#live-migrations".to_string(),
        package_version: Version::new(0, 0, 0),
        package_source: "workspace".to_string(),
        package_graph_digest: "a".repeat(64),
        compiler_components: BTreeMap::from([("mysql".to_string(), "b".repeat(64))]),
    }
}

fn id(value: &str) -> MigrationId {
    MigrationId::new(value)
}
