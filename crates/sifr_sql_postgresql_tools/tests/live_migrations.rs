#![allow(clippy::expect_used)]

use semver::Version;
use sifr_sql_contract::{DialectIdentity, ProviderIdentity, SchemaIr, schema_fingerprint};
use sifr_sql_postgresql_tools::{
    connect_migration_runtime, pull_catalog_from_client, validate_postgres_migration_plan,
};
use sifr_sql_runtime::{
    MIGRATION_EXECUTION_PLAN_FORMAT_VERSION, MigrationEngine, MigrationExecutionErrorKind,
    MigrationExecutionLimits, MigrationExecutionNode, MigrationExecutionPath,
    MigrationExecutionPlan, MigrationExecutionStatus, MigrationExecutionStep,
    MigrationExecutionStepKind, MigrationId, MigrationReplayPolicy, MigrationRuntime,
    MigrationRuntimeConstraint, MigrationStateId, MigrationTransactionBoundary,
    MigrationTransactionRequirement,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Barrier};
use std::thread;
use tokio::runtime::Builder;
use tokio_postgres::NoTls;

const TABLE: &str = "sifr_migration_accounts";

#[test]
#[ignore = "requires SIFR_POSTGRESQL_MIGRATION_TEST_URL"]
fn live_postgresql_migration_contract() {
    let url = std::env::var("SIFR_POSTGRESQL_MIGRATION_TEST_URL")
        .expect("live harness must set the PostgreSQL URL");
    let major = std::env::var("SIFR_POSTGRESQL_MIGRATION_TEST_MAJOR")
        .expect("live harness must set the PostgreSQL major")
        .parse::<u16>()
        .expect("PostgreSQL major must be numeric");
    let fingerprints = probe_fingerprints(&url, major);
    let plan = qualification_plan(&fingerprints);
    let operator = validate_postgres_migration_plan(&plan).expect("operator plan");
    assert_eq!(operator.actions.len(), 26);
    assert_eq!(
        operator.forward_only,
        vec![id("left"), id("right"), id("upgrade")]
    );
    assert_eq!(
        operator.reversible,
        vec![id("create"), id("merge"), id("finish")]
    );

    verify_false_import(&url, major, &plan);
    verify_concurrent_profile_bootstrap(&url, major, &plan);
    verify_failed_transaction(&url, major, &fingerprints);
    verify_head_mismatch(&url, major, &plan);
    verify_schema_drift(&url, major, &plan);
    verify_lock_and_apply_resume_rollback(&url, major, &plan, &fingerprints);
    verify_checksum_drift(&url, major, &plan);
}

fn verify_concurrent_profile_bootstrap(url: &str, major: u16, plan: &MigrationExecutionPlan) {
    let barrier = Arc::new(Barrier::new(2));
    thread::scope(|scope| {
        let handles = ["qualification-bootstrap-a", "qualification-bootstrap-b"].map(|identity| {
            let barrier = Arc::clone(&barrier);
            scope.spawn(move || {
                let mut runtime = migration_runtime(url, major, identity);
                barrier.wait();
                let lock = MigrationRuntime::acquire_lock(&mut runtime, plan)
                    .expect("cross-profile bootstrap lock");
                MigrationRuntime::release_lock(&mut runtime, lock)
                    .expect("cross-profile bootstrap release");
            })
        });
        for handle in handles {
            handle.join().expect("bootstrap worker");
        }
    });
}

#[derive(Clone)]
struct Fingerprints {
    baseline: String,
    table: String,
    index: String,
    final_schema: String,
}

fn probe_fingerprints(url: &str, major: u16) -> Fingerprints {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("probe runtime");
    runtime.block_on(async {
        let (client, connection) = tokio_postgres::connect(url, NoTls)
            .await
            .expect("probe connection");
        let driver = tokio::spawn(async move {
            let _result = connection.await;
        });
        let baseline = fingerprint(
            &pull_catalog_from_client(&client, provider(), dialect(major), true)
                .await
                .expect("baseline catalog"),
        );
        client
            .batch_execute(&format!(
                "CREATE TABLE {TABLE} (id bigint PRIMARY KEY, processed boolean NOT NULL DEFAULT false)"
            ))
            .await
            .expect("probe table");
        let table = fingerprint(
            &pull_catalog_from_client(&client, provider(), dialect(major), true)
                .await
                .expect("table catalog"),
        );
        client
            .batch_execute(&format!(
                "CREATE INDEX CONCURRENTLY {TABLE}_processed_idx ON {TABLE}(processed)"
            ))
            .await
            .expect("probe index");
        let index = fingerprint(
            &pull_catalog_from_client(&client, provider(), dialect(major), true)
                .await
                .expect("index catalog"),
        );
        client
            .batch_execute(&format!("ALTER TABLE {TABLE} ADD COLUMN note text"))
            .await
            .expect("probe final schema");
        let final_schema = fingerprint(
            &pull_catalog_from_client(&client, provider(), dialect(major), true)
                .await
                .expect("final catalog"),
        );
        client
            .batch_execute(&format!("DROP TABLE {TABLE}"))
            .await
            .expect("probe cleanup");
        let restored = fingerprint(
            &pull_catalog_from_client(&client, provider(), dialect(major), true)
                .await
                .expect("restored catalog"),
        );
        assert_eq!(restored, baseline);
        driver.abort();
        Fingerprints {
            baseline,
            table,
            index,
            final_schema,
        }
    })
}

fn qualification_plan(fingerprints: &Fingerprints) -> MigrationExecutionPlan {
    let baseline = id("baseline");
    let create = id("create");
    let left = id("left");
    let right = id("right");
    let merge = id("merge");
    let upgrade = id("upgrade");
    let finish = id("finish");
    let create_path = path(
        &baseline,
        &fingerprints.baseline,
        &fingerprints.table,
        vec![
            transaction_step("create.begin", &fingerprints.baseline, true),
            step(
                "create.table",
                &fingerprints.baseline,
                &fingerprints.table,
                MigrationExecutionStepKind::Ddl {
                    statement: format!(
                        "CREATE TABLE {TABLE} (id bigint PRIMARY KEY, processed boolean NOT NULL DEFAULT false)"
                    ),
                },
            ),
            step(
                "create.rows",
                &fingerprints.table,
                &fingerprints.table,
                MigrationExecutionStepKind::SqlData {
                    normalized_statement: format!("INSERT INTO {TABLE}(id) VALUES (1), (2), (3)"),
                },
            ),
            transaction_step("create.commit", &fingerprints.table, false),
        ],
        Some(vec![
            transaction_step("create.undo.begin", &fingerprints.table, true),
            step(
                "create.undo.table",
                &fingerprints.table,
                &fingerprints.baseline,
                MigrationExecutionStepKind::Ddl {
                    statement: format!("DROP TABLE {TABLE}"),
                },
            ),
            transaction_step("create.undo.commit", &fingerprints.baseline, false),
        ]),
    );
    let branch_path = |name: &str| {
        path(
            &create,
            &fingerprints.table,
            &fingerprints.table,
            vec![recovery_step(&format!("{name}.ready"), &fingerprints.table)],
            None,
        )
    };
    let merge_path = |parent: &MigrationId| {
        path(
            parent,
            &fingerprints.table,
            &fingerprints.index,
            vec![
                recovery_step("merge.before_index", &fingerprints.table),
                step(
                    "merge.index",
                    &fingerprints.table,
                    &fingerprints.index,
                    MigrationExecutionStepKind::Ddl {
                        statement: format!(
                            "CREATE INDEX CONCURRENTLY {TABLE}_processed_idx ON {TABLE}(processed)"
                        ),
                    },
                ),
            ],
            Some(vec![
                recovery_step("merge.undo.before_index", &fingerprints.index),
                step(
                    "merge.undo.index",
                    &fingerprints.index,
                    &fingerprints.table,
                    MigrationExecutionStepKind::Ddl {
                        statement: format!("DROP INDEX CONCURRENTLY {TABLE}_processed_idx"),
                    },
                ),
            ]),
        )
    };
    let upgrade_path = path(
        &merge,
        &fingerprints.index,
        &fingerprints.index,
        vec![
            recovery_step("upgrade.before_backfill", &fingerprints.index),
            step(
                "upgrade.backfill",
                &fingerprints.index,
                &fingerprints.index,
                MigrationExecutionStepKind::Backfill {
                    normalized_statement: format!(
                        "UPDATE {TABLE} SET processed = true WHERE id IN (SELECT id FROM {TABLE} WHERE NOT processed ORDER BY id LIMIT 2)"
                    ),
                    maximum_batch_rows: 2,
                    replay: MigrationReplayPolicy::Idempotent {
                        progress_keys: vec!["id".to_string()],
                    },
                },
            ),
            step(
                "upgrade.assert",
                &fingerprints.index,
                &fingerprints.index,
                MigrationExecutionStepKind::Assertion {
                    normalized_statement: format!(
                        "SELECT count(*) = 3 FROM {TABLE} WHERE processed"
                    ),
                },
            ),
        ],
        None,
    );
    let finish_path = path(
        &upgrade,
        &fingerprints.index,
        &fingerprints.final_schema,
        vec![
            transaction_step("finish.begin", &fingerprints.index, true),
            step(
                "finish.column",
                &fingerprints.index,
                &fingerprints.final_schema,
                MigrationExecutionStepKind::Ddl {
                    statement: format!("ALTER TABLE {TABLE} ADD COLUMN note text"),
                },
            ),
            transaction_step("finish.commit", &fingerprints.final_schema, false),
        ],
        Some(vec![
            transaction_step("finish.undo.begin", &fingerprints.final_schema, true),
            step(
                "finish.undo.column",
                &fingerprints.final_schema,
                &fingerprints.index,
                MigrationExecutionStepKind::Ddl {
                    statement: format!("ALTER TABLE {TABLE} DROP COLUMN note"),
                },
            ),
            transaction_step("finish.undo.commit", &fingerprints.index, false),
        ]),
    );
    MigrationExecutionPlan {
        format_version: MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
        provider_family: "postgresql".to_string(),
        target_fingerprint: fingerprints.final_schema.clone(),
        head: finish.clone(),
        topological_order: vec![
            create.clone(),
            left.clone(),
            right.clone(),
            merge.clone(),
            upgrade.clone(),
            finish.clone(),
        ],
        baseline_fingerprints: BTreeMap::from([(baseline.clone(), fingerprints.baseline.clone())]),
        migrations: BTreeMap::from([
            (
                create.clone(),
                node(
                    &create,
                    BTreeSet::from([baseline.clone()]),
                    MigrationTransactionRequirement::Required,
                    BTreeMap::from([(baseline, create_path)]),
                ),
            ),
            (
                left.clone(),
                node(
                    &left,
                    BTreeSet::from([create.clone()]),
                    MigrationTransactionRequirement::Optional,
                    BTreeMap::from([(create.clone(), branch_path("left"))]),
                ),
            ),
            (
                right.clone(),
                node(
                    &right,
                    BTreeSet::from([create.clone()]),
                    MigrationTransactionRequirement::Optional,
                    BTreeMap::from([(create.clone(), branch_path("right"))]),
                ),
            ),
            (
                merge.clone(),
                node(
                    &merge,
                    BTreeSet::from([left.clone(), right.clone()]),
                    MigrationTransactionRequirement::Forbidden,
                    BTreeMap::from([
                        (left.clone(), merge_path(&left)),
                        (right.clone(), merge_path(&right)),
                    ]),
                ),
            ),
            (
                upgrade.clone(),
                node(
                    &upgrade,
                    BTreeSet::from([merge.clone()]),
                    MigrationTransactionRequirement::Optional,
                    BTreeMap::from([(merge, upgrade_path)]),
                ),
            ),
            (
                finish.clone(),
                node(
                    &finish,
                    BTreeSet::from([upgrade.clone()]),
                    MigrationTransactionRequirement::Required,
                    BTreeMap::from([(upgrade, finish_path)]),
                ),
            ),
        ]),
    }
}

fn verify_false_import(url: &str, major: u16, plan: &MigrationExecutionPlan) {
    let invalid = MigrationExecutionPlan {
        format_version: MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
        provider_family: plan.provider_family.clone(),
        target_fingerprint: "f".repeat(64),
        head: id("baseline"),
        topological_order: Vec::new(),
        baseline_fingerprints: BTreeMap::from([(id("baseline"), "f".repeat(64))]),
        migrations: BTreeMap::new(),
    };
    let mut runtime = migration_runtime(url, major, "qualification-false-import");
    assert!(runtime.import_baseline(&invalid, &id("baseline")).is_err());
    assert!(MigrationRuntime::load_ledger(&mut runtime).is_err());
}

fn verify_failed_transaction(url: &str, major: u16, fingerprints: &Fingerprints) {
    let baseline = id("baseline");
    let failure = id("failure");
    let failure_path = path(
        &baseline,
        &fingerprints.baseline,
        &fingerprints.table,
        vec![
            transaction_step("failure.begin", &fingerprints.baseline, true),
            step(
                "failure.table",
                &fingerprints.baseline,
                &fingerprints.table,
                MigrationExecutionStepKind::Ddl {
                    statement: format!(
                        "CREATE TABLE {TABLE} (id bigint PRIMARY KEY, processed boolean NOT NULL DEFAULT false)"
                    ),
                },
            ),
            step(
                "failure.assert",
                &fingerprints.table,
                &fingerprints.table,
                MigrationExecutionStepKind::Assertion {
                    normalized_statement: "SELECT false".to_string(),
                },
            ),
            transaction_step("failure.commit", &fingerprints.table, false),
        ],
        None,
    );
    let plan = MigrationExecutionPlan {
        format_version: MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
        provider_family: "postgresql".to_string(),
        target_fingerprint: fingerprints.table.clone(),
        head: failure.clone(),
        topological_order: vec![failure.clone()],
        baseline_fingerprints: BTreeMap::from([(baseline.clone(), fingerprints.baseline.clone())]),
        migrations: BTreeMap::from([(
            failure.clone(),
            node(
                &failure,
                BTreeSet::from([baseline.clone()]),
                MigrationTransactionRequirement::Required,
                BTreeMap::from([(baseline, failure_path)]),
            ),
        )]),
    };
    let mut runtime = migration_runtime(url, major, "qualification-failure");
    runtime
        .import_baseline(&plan, &id("baseline"))
        .expect("failure baseline import");
    let error = MigrationEngine::new(MigrationExecutionLimits::default())
        .execute(&plan, &mut runtime)
        .expect_err("false assertion must fail");
    assert_eq!(error.kind, MigrationExecutionErrorKind::AssertionFalse);
    assert_eq!(
        MigrationRuntime::inspect_schema_fingerprint(&mut runtime).expect("post-failure schema"),
        fingerprints.baseline
    );
}

fn verify_lock_and_apply_resume_rollback(
    url: &str,
    major: u16,
    plan: &MigrationExecutionPlan,
    fingerprints: &Fingerprints,
) {
    let mut runtime = migration_runtime(url, major, "qualification-main");
    let imported = runtime
        .import_baseline(plan, &id("baseline"))
        .expect("truthful baseline import");
    assert!(imported.applied.is_empty());
    assert_eq!(imported.heads, BTreeSet::from([id("baseline")]));
    let lock = MigrationRuntime::acquire_lock(&mut runtime, plan).expect("first lock");
    let mut contender = migration_runtime(url, major, "qualification-main");
    assert!(MigrationRuntime::acquire_lock(&mut contender, plan).is_err());
    MigrationRuntime::release_lock(&mut runtime, lock).expect("release first lock");

    let paused = MigrationEngine::new(MigrationExecutionLimits {
        maximum_backfill_batches: 1,
    })
    .execute(plan, &mut runtime)
    .expect("migration must pause after one backfill batch");
    assert_eq!(paused.status, MigrationExecutionStatus::Paused);
    let progress = MigrationRuntime::load_ledger(&mut runtime)
        .expect("paused ledger")
        .in_progress
        .expect("paused progress");
    assert_eq!(progress.migration, id("upgrade"));
    assert_eq!(progress.backfill_progress.as_deref(), Some("2"));

    let complete = MigrationEngine::new(MigrationExecutionLimits::default())
        .execute(plan, &mut runtime)
        .expect("migration must resume");
    assert_eq!(complete.status, MigrationExecutionStatus::Complete);
    assert_eq!(complete.heads, BTreeSet::from([id("finish")]));
    assert_eq!(complete.schema_fingerprint, fingerprints.final_schema);
    let ledger = MigrationRuntime::load_ledger(&mut runtime).expect("complete ledger");
    assert!(ledger.applied.contains_key(&id("left")));
    assert!(ledger.applied.contains_key(&id("right")));
    assert!(ledger.applied.contains_key(&id("merge")));

    let rollback = MigrationEngine::new(MigrationExecutionLimits::default())
        .rollback_last(plan, &mut runtime)
        .expect("declared rollback");
    assert_eq!(rollback.heads, BTreeSet::from([id("upgrade")]));
    assert_eq!(rollback.schema_fingerprint, fingerprints.index);
    let forward_only = MigrationEngine::new(MigrationExecutionLimits::default())
        .rollback_last(plan, &mut runtime)
        .expect_err("forward-only migration must not synthesize rollback");
    assert_eq!(forward_only.kind, MigrationExecutionErrorKind::ForwardOnly);
}

fn verify_checksum_drift(url: &str, major: u16, plan: &MigrationExecutionPlan) {
    let mut runtime = migration_runtime(url, major, "qualification-main");
    let mut changed = plan.clone();
    changed
        .migrations
        .get_mut(&id("upgrade"))
        .expect("upgrade node")
        .checksum = "0".repeat(64);
    let error = MigrationEngine::new(MigrationExecutionLimits::default())
        .execute(&changed, &mut runtime)
        .expect_err("changed checksum must fail");
    assert_eq!(error.kind, MigrationExecutionErrorKind::ChecksumDrift);
}

fn verify_head_mismatch(url: &str, major: u16, plan: &MigrationExecutionPlan) {
    let mut runtime = migration_runtime(url, major, "qualification-head-mismatch");
    runtime
        .import_baseline(plan, &id("baseline"))
        .expect("head baseline import");
    let mut ledger = MigrationRuntime::load_ledger(&mut runtime).expect("head ledger");
    ledger.heads = BTreeSet::from([id("unknown")]);
    MigrationRuntime::store_ledger(&mut runtime, &ledger).expect("store changed head");
    let error = MigrationEngine::new(MigrationExecutionLimits::default())
        .execute(plan, &mut runtime)
        .expect_err("unknown head must fail");
    assert_eq!(error.kind, MigrationExecutionErrorKind::HeadMismatch);
}

fn verify_schema_drift(url: &str, major: u16, plan: &MigrationExecutionPlan) {
    let mut runtime = migration_runtime(url, major, "qualification-schema-drift");
    runtime
        .import_baseline(plan, &id("baseline"))
        .expect("drift baseline import");
    execute_sql(url, "CREATE TABLE sifr_migration_drift (id bigint)");
    let error = MigrationEngine::new(MigrationExecutionLimits::default())
        .execute(plan, &mut runtime)
        .expect_err("live schema drift must fail");
    assert_eq!(error.kind, MigrationExecutionErrorKind::SchemaDrift);
    execute_sql(url, "DROP TABLE sifr_migration_drift");
}

fn execute_sql(url: &str, statement: &str) {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("SQL helper runtime");
    runtime.block_on(async {
        let (client, connection) = tokio_postgres::connect(url, NoTls)
            .await
            .expect("SQL helper connection");
        let driver = tokio::spawn(async move {
            let _result = connection.await;
        });
        client
            .batch_execute(statement)
            .await
            .expect("SQL helper statement");
        driver.abort();
    });
}

fn migration_runtime(
    url: &str,
    major: u16,
    identity: &str,
) -> sifr_sql_postgresql_tools::PostgresMigrationRuntime {
    connect_migration_runtime(url, provider(), dialect(major), identity)
        .expect("PostgreSQL migration runtime")
}

fn fingerprint(schema: &SchemaIr) -> String {
    schema_fingerprint(schema)
        .expect("schema fingerprint")
        .as_str()
        .to_string()
}

fn provider() -> ProviderIdentity {
    ProviderIdentity {
        package_id: "sifr-sql-postgresql@0.0.0#qualification".to_string(),
        package_version: Version::new(0, 0, 0),
        package_source: "path+crates/sifr_sql_postgresql".to_string(),
        package_graph_digest: "qualification-graph".to_string(),
        compiler_components: BTreeMap::from([("schema".to_string(), "a".repeat(64))]),
    }
}

fn dialect(major: u16) -> DialectIdentity {
    DialectIdentity {
        family: "postgresql".to_string(),
        server_version: format!("{major}.0.0"),
        modes: BTreeSet::new(),
        features: BTreeSet::from(["transactional-ddl".to_string()]),
    }
}

fn id(value: &str) -> MigrationId {
    MigrationId::new(value)
}

fn step(
    name: &str,
    input: &str,
    output: &str,
    kind: MigrationExecutionStepKind,
) -> MigrationExecutionStep {
    MigrationExecutionStep {
        id: id(name),
        input_state: MigrationStateId::new(format!("state.{name}.input")),
        output_state: MigrationStateId::new(format!("state.{name}.output")),
        input_fingerprint: input.to_string(),
        output_fingerprint: output.to_string(),
        checksum: checksum(name),
        kind,
    }
}

fn transaction_step(name: &str, fingerprint: &str, begin: bool) -> MigrationExecutionStep {
    step(
        name,
        fingerprint,
        fingerprint,
        MigrationExecutionStepKind::Transaction {
            boundary: if begin {
                MigrationTransactionBoundary::Begin
            } else {
                MigrationTransactionBoundary::Commit
            },
        },
    )
}

fn recovery_step(name: &str, fingerprint: &str) -> MigrationExecutionStep {
    step(
        name,
        fingerprint,
        fingerprint,
        MigrationExecutionStepKind::RecoveryPoint {
            name: name.to_string(),
        },
    )
}

fn path(
    parent: &MigrationId,
    input: &str,
    output: &str,
    steps: Vec<MigrationExecutionStep>,
    rollback: Option<Vec<MigrationExecutionStep>>,
) -> MigrationExecutionPath {
    MigrationExecutionPath {
        parent: parent.clone(),
        input_fingerprint: input.to_string(),
        output_fingerprint: output.to_string(),
        steps,
        rollback,
    }
}

fn node(
    migration: &MigrationId,
    parents: BTreeSet<MigrationId>,
    transaction_requirement: MigrationTransactionRequirement,
    paths: BTreeMap<MigrationId, MigrationExecutionPath>,
) -> MigrationExecutionNode {
    MigrationExecutionNode {
        id: migration.clone(),
        parents,
        provider: MigrationRuntimeConstraint {
            family: "postgresql".to_string(),
            minimum_server_version: Some("13.0.0".to_string()),
            required_capabilities: BTreeSet::from(["transactional-ddl".to_string()]),
        },
        transaction_requirement,
        checksum: checksum(migration.as_str()),
        paths,
        author: "SQL platform qualification".to_string(),
        created_at: "2026-08-31T00:00:00Z".to_string(),
    }
}

fn checksum(value: &str) -> String {
    let bytes = value.as_bytes();
    (0..64)
        .map(|index| char::from(b'a' + (bytes[index % bytes.len()] % 6)))
        .collect()
}
