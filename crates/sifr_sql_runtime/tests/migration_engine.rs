#![allow(clippy::expect_used)]

use sifr_sql_contract::{
    CompiledMigration, CompiledMigrationGraph, CompiledMigrationPath, CompiledMigrationStep,
    CompiledStepKind, MigrationNodeId, MigrationProviderConstraint, MigrationStateIdentity,
    ReplayPolicy, TransactionBoundary, TransactionRequirement,
};
use sifr_sql_runtime::{
    AppliedMigrationRecord, MigrationEngine, MigrationExecutionErrorKind, MigrationExecutionLimits,
    MigrationExecutionStatus, MigrationLedgerSnapshot, MigrationLock, MigrationRuntime,
    MigrationStepRequest, MigrationStepResult,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

fn id(value: &str) -> MigrationNodeId {
    MigrationNodeId::new(value).expect("test migration identity should be valid")
}

fn step(value: &str, input: &str, output: &str, kind: CompiledStepKind) -> CompiledMigrationStep {
    CompiledMigrationStep {
        id: id(value),
        input_state: MigrationStateIdentity::new(format!("state.{value}.input")),
        output_state: MigrationStateIdentity::new(format!("state.{value}.output")),
        input_fingerprint: input.to_string(),
        output_fingerprint: output.to_string(),
        checksum: value.bytes().cycle().take(64).map(char::from).collect(),
        referenced_objects: BTreeSet::new(),
        affected_objects: BTreeSet::new(),
        kind,
    }
}

fn graph(with_backfill: bool) -> CompiledMigrationGraph {
    let before = "a".repeat(64);
    let after = "b".repeat(64);
    let baseline = id("baseline");
    let migration_id = id("m1");
    let mut steps = vec![step(
        "ddl",
        &before,
        &after,
        CompiledStepKind::ReflectedDdl {
            statement: "ALTER".to_string(),
        },
    )];
    if with_backfill {
        steps.push(step(
            "recovery",
            &after,
            &after,
            CompiledStepKind::RecoveryPoint {
                name: "after-ddl".to_string(),
            },
        ));
        steps.push(step(
            "backfill",
            &after,
            &after,
            CompiledStepKind::Backfill {
                normalized_statement: "UPDATE".to_string(),
                maximum_batch_rows: 10,
                replay: ReplayPolicy::Idempotent {
                    progress_key: vec![],
                },
            },
        ));
    }
    steps.push(step(
        "assert",
        &after,
        &after,
        CompiledStepKind::Assertion {
            normalized_statement: "SELECT true AS valid".to_string(),
        },
    ));
    let path = CompiledMigrationPath {
        parent: baseline.clone(),
        input_fingerprint: before.clone(),
        output_fingerprint: after.clone(),
        steps,
        rollback: None,
    };
    CompiledMigrationGraph {
        format_version: 1,
        provider_family: "postgresql".to_string(),
        target_fingerprint: after,
        head: migration_id.clone(),
        topological_order: vec![migration_id.clone()],
        baseline_fingerprints: BTreeMap::from([(baseline.clone(), before)]),
        migrations: BTreeMap::from([(
            migration_id.clone(),
            CompiledMigration {
                id: migration_id,
                parents: BTreeSet::from([baseline.clone()]),
                provider: MigrationProviderConstraint {
                    family: "postgresql".to_string(),
                    minimum_server_version: None,
                    required_capabilities: BTreeSet::new(),
                },
                transaction_requirement: TransactionRequirement::Optional,
                checksum: "c".repeat(64),
                paths: BTreeMap::from([(baseline, path)]),
                author: "test".to_string(),
                created_at: "2026-08-31T00:00:00Z".to_string(),
            },
        )]),
        impacts: Vec::new(),
    }
}

fn transactional_graph() -> CompiledMigrationGraph {
    let mut graph = graph(false);
    let migration = graph
        .migrations
        .get_mut(&id("m1"))
        .expect("test migration should exist");
    migration.transaction_requirement = TransactionRequirement::Required;
    let path = migration
        .paths
        .get_mut(&id("baseline"))
        .expect("test path should exist");
    let before = path.input_fingerprint.clone();
    let after = path.output_fingerprint.clone();
    path.steps.insert(
        0,
        step(
            "begin",
            &before,
            &before,
            CompiledStepKind::Transaction {
                boundary: TransactionBoundary::Begin,
            },
        ),
    );
    path.steps.push(step(
        "commit",
        &after,
        &after,
        CompiledStepKind::Transaction {
            boundary: TransactionBoundary::Commit,
        },
    ));
    graph
}

struct FakeRuntime {
    ledger: MigrationLedgerSnapshot,
    results: VecDeque<MigrationStepResult>,
    stored: Vec<MigrationLedgerSnapshot>,
    locked: bool,
    panic_on_execute: bool,
    transaction_open: bool,
    rollback_count: u64,
}

impl FakeRuntime {
    fn new(graph: &CompiledMigrationGraph, results: Vec<MigrationStepResult>) -> Self {
        let (baseline, fingerprint) = graph
            .baseline_fingerprints
            .first_key_value()
            .expect("test graph should have a baseline");
        Self {
            ledger: MigrationLedgerSnapshot {
                provider_family: graph.provider_family.clone(),
                heads: BTreeSet::from([baseline.clone()]),
                schema_fingerprint: fingerprint.clone(),
                applied: BTreeMap::new(),
                in_progress: None,
            },
            results: results.into(),
            stored: Vec::new(),
            locked: false,
            panic_on_execute: false,
            transaction_open: false,
            rollback_count: 0,
        }
    }
}

impl MigrationRuntime for FakeRuntime {
    fn acquire_lock(&mut self, _graph: &CompiledMigrationGraph) -> Result<MigrationLock, String> {
        if self.locked {
            return Err("already locked".to_string());
        }
        self.locked = true;
        Ok(MigrationLock {
            identity: "test-lock".to_string(),
        })
    }

    fn release_lock(&mut self, _lock: MigrationLock) -> Result<(), String> {
        self.locked = false;
        Ok(())
    }

    fn load_ledger(&mut self) -> Result<MigrationLedgerSnapshot, String> {
        Ok(self.ledger.clone())
    }

    fn store_ledger(&mut self, ledger: &MigrationLedgerSnapshot) -> Result<(), String> {
        self.ledger = ledger.clone();
        self.stored.push(ledger.clone());
        Ok(())
    }

    fn begin_transaction(&mut self) -> Result<(), String> {
        self.transaction_open = true;
        Ok(())
    }

    fn commit_transaction(&mut self) -> Result<(), String> {
        self.transaction_open = false;
        Ok(())
    }

    fn rollback_transaction(&mut self) -> Result<(), String> {
        self.transaction_open = false;
        self.rollback_count += 1;
        Ok(())
    }

    fn execute_step(
        &mut self,
        _request: MigrationStepRequest<'_>,
    ) -> Result<MigrationStepResult, String> {
        assert!(!self.panic_on_execute, "provider panic");
        self.results
            .pop_front()
            .ok_or_else(|| "missing test result".to_string())
    }

    fn inspect_schema_fingerprint(&mut self) -> Result<String, String> {
        Ok(self.ledger.schema_fingerprint.clone())
    }
}

fn complete(after: &str) -> MigrationStepResult {
    MigrationStepResult::Completed {
        schema_fingerprint: after.to_string(),
        duration_millis: 2,
    }
}

fn assertion(after: &str, rows: u64, valid: Option<bool>) -> MigrationStepResult {
    MigrationStepResult::Assertion {
        rows,
        valid,
        schema_fingerprint: after.to_string(),
        duration_millis: 1,
    }
}

#[test]
fn engine_records_lock_steps_heads_checksums_and_fingerprints() {
    let graph = graph(false);
    let after = graph.target_fingerprint.clone();
    let mut runtime = FakeRuntime::new(
        &graph,
        vec![complete(&after), assertion(&after, 1, Some(true))],
    );
    let report = MigrationEngine::new(MigrationExecutionLimits::default())
        .execute(&graph, &mut runtime)
        .expect("migration should complete");
    assert_eq!(report.status, MigrationExecutionStatus::Complete);
    assert_eq!(report.heads, BTreeSet::from([id("m1")]));
    assert_eq!(report.schema_fingerprint, after);
    assert!(!runtime.locked);
    assert!(runtime.ledger.in_progress.is_none());
    assert_eq!(runtime.ledger.applied[&id("m1")].checksum, "c".repeat(64));
    assert!(report.events.len() >= 8);
}

#[test]
fn assertion_failures_are_distinct() {
    let graph = graph(false);
    let after = graph.target_fingerprint.clone();
    for (rows, valid, expected) in [
        (
            0,
            Some(true),
            MigrationExecutionErrorKind::AssertionZeroRows,
        ),
        (
            2,
            Some(true),
            MigrationExecutionErrorKind::AssertionMultipleRows,
        ),
        (1, Some(false), MigrationExecutionErrorKind::AssertionFalse),
        (1, None, MigrationExecutionErrorKind::AssertionFalse),
    ] {
        let mut runtime = FakeRuntime::new(
            &graph,
            vec![complete(&after), assertion(&after, rows, valid)],
        );
        let error = MigrationEngine::new(MigrationExecutionLimits::default())
            .execute(&graph, &mut runtime)
            .expect_err("invalid assertion outcome must fail");
        assert_eq!(error.kind, expected);
        assert!(!runtime.locked);
    }
}

#[test]
fn bounded_backfill_pauses_and_resumes_from_persisted_progress() {
    let graph = graph(true);
    let after = graph.target_fingerprint.clone();
    let mut runtime = FakeRuntime::new(
        &graph,
        vec![
            complete(&after),
            MigrationStepResult::BackfillBatch {
                processed_rows: 10,
                progress: Some("10".to_string()),
                complete: false,
                schema_fingerprint: after.clone(),
                duration_millis: 3,
            },
        ],
    );
    let paused = MigrationEngine::new(MigrationExecutionLimits {
        maximum_backfill_batches: 1,
    })
    .execute(&graph, &mut runtime)
    .expect("bounded execution should pause cleanly");
    assert_eq!(paused.status, MigrationExecutionStatus::Paused);
    assert_eq!(
        runtime
            .ledger
            .in_progress
            .as_ref()
            .and_then(|record| record.backfill_progress.as_deref()),
        Some("10")
    );

    runtime.results.extend([
        MigrationStepResult::BackfillBatch {
            processed_rows: 4,
            progress: None,
            complete: true,
            schema_fingerprint: after.clone(),
            duration_millis: 2,
        },
        assertion(&after, 1, Some(true)),
    ]);
    let complete = MigrationEngine::new(MigrationExecutionLimits::default())
        .execute(&graph, &mut runtime)
        .expect("resumed migration should complete");
    assert_eq!(complete.status, MigrationExecutionStatus::Complete);
    assert!(runtime.ledger.in_progress.is_none());
}

#[test]
fn checksum_drift_schema_drift_and_provider_panics_fail_closed() {
    let graph = graph(false);
    let after = graph.target_fingerprint.clone();
    let mut runtime = FakeRuntime::new(&graph, Vec::new());
    runtime.ledger.applied.insert(
        id("m1"),
        AppliedMigrationRecord {
            migration: id("m1"),
            checksum: "d".repeat(64),
            output_fingerprint: after.clone(),
            duration_millis: 1,
        },
    );
    assert_eq!(
        MigrationEngine::new(MigrationExecutionLimits::default())
            .execute(&graph, &mut runtime)
            .expect_err("checksum drift must fail")
            .kind,
        MigrationExecutionErrorKind::ChecksumDrift
    );

    let mut runtime = FakeRuntime::new(&graph, vec![complete(&after)]);
    runtime.panic_on_execute = true;
    assert_eq!(
        MigrationEngine::new(MigrationExecutionLimits::default())
            .execute(&graph, &mut runtime)
            .expect_err("provider panic must be contained")
            .kind,
        MigrationExecutionErrorKind::RuntimePanic
    );
    assert!(!runtime.locked);
}

#[test]
fn failed_transaction_rolls_back_before_lock_release() {
    let graph = transactional_graph();
    let after = graph.target_fingerprint.clone();
    let mut runtime = FakeRuntime::new(
        &graph,
        vec![complete(&after), assertion(&after, 1, Some(false))],
    );
    let error = MigrationEngine::new(MigrationExecutionLimits::default())
        .execute(&graph, &mut runtime)
        .expect_err("failed transaction must stop");
    assert_eq!(error.kind, MigrationExecutionErrorKind::AssertionFalse);
    assert_eq!(runtime.rollback_count, 1);
    assert!(!runtime.transaction_open);
    assert!(!runtime.locked);
}
