#![allow(clippy::expect_used)]

use sifr_sql_runtime::{
    AppliedMigrationRecord, InProgressMigrationRecord, MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
    MigrationDirection, MigrationEngine, MigrationExecutionErrorKind, MigrationExecutionLimits,
    MigrationExecutionNode, MigrationExecutionPath, MigrationExecutionPlan,
    MigrationExecutionStatus, MigrationExecutionStep, MigrationExecutionStepKind, MigrationId,
    MigrationLedgerSnapshot, MigrationLock, MigrationReplayPolicy, MigrationRuntime,
    MigrationRuntimeConstraint, MigrationRuntimeIdentity, MigrationStateId, MigrationStepRequest,
    MigrationStepResult, MigrationTransactionBoundary, MigrationTransactionRequirement,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

fn id(value: &str) -> MigrationId {
    MigrationId::new(value)
}

fn step(
    value: &str,
    input: &str,
    output: &str,
    kind: MigrationExecutionStepKind,
) -> MigrationExecutionStep {
    MigrationExecutionStep {
        id: id(value),
        input_state: MigrationStateId::new(format!("state.{value}.input")),
        output_state: MigrationStateId::new(format!("state.{value}.output")),
        input_fingerprint: input.to_string(),
        output_fingerprint: output.to_string(),
        checksum: value.bytes().cycle().take(64).map(char::from).collect(),
        kind,
    }
}

fn node(
    identity: MigrationId,
    parents: BTreeSet<MigrationId>,
    checksum: String,
    paths: BTreeMap<MigrationId, MigrationExecutionPath>,
) -> MigrationExecutionNode {
    MigrationExecutionNode {
        id: identity,
        parents,
        provider: MigrationRuntimeConstraint {
            family: "postgresql".to_string(),
            minimum_server_version: Some("13.0.0".to_string()),
            required_capabilities: BTreeSet::new(),
        },
        transaction_requirement: MigrationTransactionRequirement::Optional,
        checksum,
        paths,
        author: "test".to_string(),
        created_at: "2026-08-31T00:00:00Z".to_string(),
    }
}

fn graph(with_backfill: bool) -> MigrationExecutionPlan {
    let before = "a".repeat(64);
    let after = "b".repeat(64);
    let baseline = id("baseline");
    let migration_id = id("m1");
    let mut steps = vec![step(
        "ddl",
        &before,
        &after,
        MigrationExecutionStepKind::Ddl {
            statement: "ALTER".to_string(),
        },
    )];
    if with_backfill {
        steps.push(step(
            "recovery",
            &after,
            &after,
            MigrationExecutionStepKind::RecoveryPoint {
                name: "after-ddl".to_string(),
            },
        ));
        steps.push(step(
            "backfill",
            &after,
            &after,
            MigrationExecutionStepKind::Backfill {
                normalized_statement: "UPDATE".to_string(),
                maximum_batch_rows: 10,
                replay: MigrationReplayPolicy::Idempotent {
                    progress_keys: vec![],
                },
            },
        ));
    }
    steps.push(step(
        "assert",
        &after,
        &after,
        MigrationExecutionStepKind::Assertion {
            normalized_statement: "SELECT true AS valid".to_string(),
        },
    ));
    let path = MigrationExecutionPath {
        parent: baseline.clone(),
        input_fingerprint: before.clone(),
        output_fingerprint: after.clone(),
        steps,
        rollback: None,
    };
    MigrationExecutionPlan {
        format_version: MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
        provider_family: "postgresql".to_string(),
        target_fingerprint: after.clone(),
        head: migration_id.clone(),
        topological_order: vec![migration_id.clone()],
        baseline_fingerprints: BTreeMap::from([(baseline.clone(), before.clone())]),
        migrations: BTreeMap::from([(
            migration_id.clone(),
            node(
                migration_id,
                BTreeSet::from([baseline.clone()]),
                "c".repeat(64),
                BTreeMap::from([(baseline, path)]),
            ),
        )]),
    }
}

fn transactional_graph() -> MigrationExecutionPlan {
    let mut graph = graph(false);
    let migration = graph
        .migrations
        .get_mut(&id("m1"))
        .expect("test migration should exist");
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
            MigrationExecutionStepKind::Transaction {
                boundary: MigrationTransactionBoundary::Begin,
            },
        ),
    );
    path.steps.push(step(
        "commit",
        &after,
        &after,
        MigrationExecutionStepKind::Transaction {
            boundary: MigrationTransactionBoundary::Commit,
        },
    ));
    graph
}

fn branching_graph() -> MigrationExecutionPlan {
    let before = "a".repeat(64);
    let after = "b".repeat(64);
    let baseline = id("baseline");
    let left = id("left");
    let right = id("right");
    let merge = id("merge");
    let branch_path = |migration: &str| MigrationExecutionPath {
        parent: baseline.clone(),
        input_fingerprint: before.clone(),
        output_fingerprint: before.clone(),
        steps: vec![step(
            &format!("{migration}-point"),
            &before,
            &before,
            MigrationExecutionStepKind::RecoveryPoint {
                name: format!("{migration}-complete"),
            },
        )],
        rollback: None,
    };
    let merge_step = step(
        "merge-ddl",
        &before,
        &after,
        MigrationExecutionStepKind::Ddl {
            statement: "ALTER".to_string(),
        },
    );
    let merge_path = |parent: MigrationId| MigrationExecutionPath {
        parent,
        input_fingerprint: before.clone(),
        output_fingerprint: after.clone(),
        steps: vec![merge_step.clone()],
        rollback: None,
    };
    MigrationExecutionPlan {
        format_version: MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
        provider_family: "postgresql".to_string(),
        target_fingerprint: after.clone(),
        head: merge.clone(),
        topological_order: vec![left.clone(), right.clone(), merge.clone()],
        baseline_fingerprints: BTreeMap::from([(baseline.clone(), before.clone())]),
        migrations: BTreeMap::from([
            (
                left.clone(),
                node(
                    left.clone(),
                    BTreeSet::from([baseline.clone()]),
                    "c".repeat(64),
                    BTreeMap::from([(baseline.clone(), branch_path("left"))]),
                ),
            ),
            (
                right.clone(),
                node(
                    right.clone(),
                    BTreeSet::from([baseline.clone()]),
                    "d".repeat(64),
                    BTreeMap::from([(baseline.clone(), branch_path("right"))]),
                ),
            ),
            (
                merge.clone(),
                node(
                    merge,
                    BTreeSet::from([left.clone(), right.clone()]),
                    "e".repeat(64),
                    BTreeMap::from([
                        (left.clone(), merge_path(left)),
                        (right.clone(), merge_path(right)),
                    ]),
                ),
            ),
        ]),
    }
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
    fn new(graph: &MigrationExecutionPlan, results: Vec<MigrationStepResult>) -> Self {
        let (baseline, fingerprint) = graph
            .baseline_fingerprints
            .first_key_value()
            .expect("test graph should have a baseline");
        Self {
            ledger: MigrationLedgerSnapshot {
                provider_family: graph.provider_family.clone(),
                provider_server_version: "18.0.0".to_string(),
                provider_capabilities: BTreeSet::new(),
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
    fn identity(&mut self) -> Result<MigrationRuntimeIdentity, String> {
        Ok(MigrationRuntimeIdentity {
            family: "postgresql".to_string(),
            server_version: "18.0.0".to_string(),
            capabilities: BTreeSet::new(),
        })
    }

    fn acquire_lock(&mut self, _plan: &MigrationExecutionPlan) -> Result<MigrationLock, String> {
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
            path_parent: id("baseline"),
            prior_heads: BTreeSet::from([id("baseline")]),
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

#[test]
fn branching_plan_uses_stable_topological_order_and_reaches_merge_head() {
    let graph = branching_graph();
    let after = graph.target_fingerprint.clone();
    let mut runtime = FakeRuntime::new(&graph, vec![complete(&after)]);
    let report = MigrationEngine::new(MigrationExecutionLimits::default())
        .execute(&graph, &mut runtime)
        .expect("branching migration should complete");
    assert_eq!(report.heads, BTreeSet::from([id("merge")]));
    let started = report
        .events
        .iter()
        .filter_map(|event| match event {
            sifr_sql_runtime::MigrationExecutionEvent::MigrationStarted { migration, .. } => {
                Some(migration.as_str())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(started, vec!["left", "right", "merge"]);
}

#[test]
fn schema_changing_merge_requires_every_parent_in_applied_history() {
    let mut graph = branching_graph();
    let before = "a".repeat(64);
    let branch = "b".repeat(64);
    let target = "c".repeat(64);
    for name in ["left", "right"] {
        let path = graph
            .migrations
            .get_mut(&id(name))
            .and_then(|migration| migration.paths.get_mut(&id("baseline")))
            .expect("branch path");
        path.output_fingerprint = branch.clone();
        path.steps = vec![step(
            &format!("{name}-ddl"),
            &before,
            &branch,
            MigrationExecutionStepKind::Ddl {
                statement: "ALTER".to_string(),
            },
        )];
    }
    let merge = graph
        .migrations
        .get_mut(&id("merge"))
        .expect("merge migration");
    for path in merge.paths.values_mut() {
        path.input_fingerprint = branch.clone();
        path.output_fingerprint = target.clone();
        path.steps = vec![step(
            "merge-ddl",
            &branch,
            &target,
            MigrationExecutionStepKind::Ddl {
                statement: "ALTER".to_string(),
            },
        )];
    }
    graph.target_fingerprint = target;
    let mut runtime = FakeRuntime::new(&graph, vec![complete(&branch)]);
    let error = MigrationEngine::new(MigrationExecutionLimits::default())
        .execute(&graph, &mut runtime)
        .expect_err("incomplete merge must fail");
    assert_eq!(error.kind, MigrationExecutionErrorKind::IncompleteMerge);
}

#[test]
fn explicit_reverse_plan_rolls_back_to_the_recorded_prior_heads() {
    let mut graph = graph(false);
    let before = graph.baseline_fingerprints[&id("baseline")].clone();
    let after = graph.target_fingerprint.clone();
    graph
        .migrations
        .get_mut(&id("m1"))
        .and_then(|migration| migration.paths.get_mut(&id("baseline")))
        .expect("migration path")
        .rollback = Some(vec![step(
        "undo-ddl",
        &after,
        &before,
        MigrationExecutionStepKind::Ddl {
            statement: "DROP".to_string(),
        },
    )]);
    let mut runtime = FakeRuntime::new(
        &graph,
        vec![
            complete(&after),
            assertion(&after, 1, Some(true)),
            complete(&before),
        ],
    );
    let engine = MigrationEngine::new(MigrationExecutionLimits::default());
    engine
        .execute(&graph, &mut runtime)
        .expect("forward migration");
    let report = engine
        .rollback_last(&graph, &mut runtime)
        .expect("explicit rollback");
    assert_eq!(report.heads, BTreeSet::from([id("baseline")]));
    assert_eq!(report.schema_fingerprint, before);
    assert!(runtime.ledger.applied.is_empty());
}

#[test]
fn forward_execution_refuses_pending_rollback_progress_at_the_target_head() {
    let graph = graph(false);
    let after = graph.target_fingerprint.clone();
    let mut runtime = FakeRuntime::new(
        &graph,
        vec![complete(&after), assertion(&after, 1, Some(true))],
    );
    let engine = MigrationEngine::new(MigrationExecutionLimits::default());
    engine
        .execute(&graph, &mut runtime)
        .expect("forward migration");
    runtime.ledger.in_progress = Some(InProgressMigrationRecord {
        direction: MigrationDirection::Rollback,
        migration: id("m1"),
        parent: id("baseline"),
        migration_checksum: "c".repeat(64),
        completed_steps: BTreeMap::new(),
        current_fingerprint: after,
        recovery_point: None,
        backfill_progress: None,
        transaction_open: false,
        duration_millis: 0,
    });
    let error = engine
        .execute(&graph, &mut runtime)
        .expect_err("forward execution must not hide rollback progress");
    assert_eq!(error.kind, MigrationExecutionErrorKind::AmbiguousRecovery);
}
