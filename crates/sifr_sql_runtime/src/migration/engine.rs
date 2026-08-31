use super::{
    AppliedMigrationRecord, InProgressMigrationRecord, MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
    MigrationDirection, MigrationExecutionError, MigrationExecutionErrorKind,
    MigrationExecutionEvent, MigrationExecutionLimits, MigrationExecutionNode,
    MigrationExecutionPath, MigrationExecutionPlan, MigrationExecutionReport,
    MigrationExecutionStatus, MigrationExecutionStep, MigrationExecutionStepKind,
    MigrationLedgerSnapshot, MigrationReplayPolicy, MigrationRuntime, MigrationRuntimeIdentity,
    MigrationStepRequest, MigrationStepResult, MigrationTransactionBoundary,
};
use semver::Version;
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

pub struct MigrationEngine {
    pub(super) limits: MigrationExecutionLimits,
}

impl MigrationEngine {
    #[must_use]
    pub fn new(limits: MigrationExecutionLimits) -> Self {
        Self { limits }
    }

    pub fn execute<R: MigrationRuntime>(
        &self,
        graph: &MigrationExecutionPlan,
        runtime: &mut R,
    ) -> Result<MigrationExecutionReport, MigrationExecutionError> {
        if self.limits.maximum_backfill_batches == 0 {
            return Err(error(
                MigrationExecutionErrorKind::Backfill,
                "migration execution requires a positive backfill batch limit",
            ));
        }
        let lock = safe_call(MigrationExecutionErrorKind::Lock, || {
            runtime.acquire_lock(graph)
        })?;
        let lock_identity = lock.identity.clone();
        let mut events = vec![MigrationExecutionEvent::LockAcquired {
            identity: lock_identity.clone(),
        }];
        let result = self.execute_locked(graph, runtime, &mut events);
        let release = safe_call(MigrationExecutionErrorKind::Lock, || {
            runtime.release_lock(lock)
        });
        match (result, release) {
            (Ok(mut report), Ok(())) => {
                report.events.push(MigrationExecutionEvent::LockReleased {
                    identity: lock_identity,
                });
                Ok(report)
            }
            (Err(mut primary), Err(_)) => {
                primary.lock_release_error = true;
                Err(primary)
            }
            (Err(primary), Ok(())) => Err(primary),
            (Ok(_), Err(mut release_error)) => {
                release_error.lock_release_error = true;
                Err(release_error)
            }
        }
    }

    fn execute_locked<R: MigrationRuntime>(
        &self,
        graph: &MigrationExecutionPlan,
        runtime: &mut R,
        events: &mut Vec<MigrationExecutionEvent>,
    ) -> Result<MigrationExecutionReport, MigrationExecutionError> {
        let mut ledger = safe_call(MigrationExecutionErrorKind::Ledger, || {
            runtime.load_ledger()
        })?;
        let identity = safe_call(MigrationExecutionErrorKind::ProviderMismatch, || {
            runtime.identity()
        })?;
        validate_runtime_identity(graph, &identity)?;
        validate_ledger(graph, &ledger, &identity)?;
        let observed = safe_call(MigrationExecutionErrorKind::SchemaDrift, || {
            runtime.inspect_schema_fingerprint()
        })?;
        if observed != ledger.schema_fingerprint {
            return Err(error(
                MigrationExecutionErrorKind::SchemaDrift,
                "migration ledger fingerprint differs from the live schema",
            ));
        }
        loop {
            if ledger.heads.len() == 1 && ledger.heads.contains(&graph.head) {
                if ledger.schema_fingerprint != graph.target_fingerprint {
                    return Err(error(
                        MigrationExecutionErrorKind::SchemaDrift,
                        "migration head is applied but the target fingerprint is absent",
                    ));
                }
                return Ok(report(MigrationExecutionStatus::Complete, ledger, events));
            }
            let (migration, path) = select_path(graph, &ledger)?;
            let execution = self.execute_migration(runtime, &mut ledger, migration, path, events);
            let paused = match execution {
                Ok(paused) => paused,
                Err(mut primary) => {
                    if ledger
                        .in_progress
                        .as_ref()
                        .is_some_and(|progress| progress.transaction_open)
                        && safe_call(MigrationExecutionErrorKind::Transaction, || {
                            runtime.rollback_transaction()
                        })
                        .is_err()
                    {
                        primary.rollback_error = true;
                    }
                    return Err(primary);
                }
            };
            if paused {
                return Ok(report(MigrationExecutionStatus::Paused, ledger, events));
            }
        }
    }

    fn execute_migration<R: MigrationRuntime>(
        &self,
        runtime: &mut R,
        ledger: &mut MigrationLedgerSnapshot,
        migration: &MigrationExecutionNode,
        path: &MigrationExecutionPath,
        events: &mut Vec<MigrationExecutionEvent>,
    ) -> Result<bool, MigrationExecutionError> {
        let mut progress = if let Some(progress) = ledger.in_progress.take() {
            validate_progress(migration, path, progress)?
        } else {
            events.push(MigrationExecutionEvent::MigrationStarted {
                direction: MigrationDirection::Forward,
                migration: migration.id.clone(),
                parent: path.parent.clone(),
                input_fingerprint: path.input_fingerprint.clone(),
            });
            InProgressMigrationRecord {
                direction: MigrationDirection::Forward,
                migration: migration.id.clone(),
                parent: path.parent.clone(),
                migration_checksum: migration.checksum.clone(),
                completed_steps: BTreeMap::new(),
                current_fingerprint: path.input_fingerprint.clone(),
                recovery_point: None,
                backfill_progress: None,
                transaction_open: false,
                duration_millis: 0,
            }
        };
        ledger.in_progress = Some(progress.clone());
        store(runtime, ledger)?;

        for step in &path.steps {
            if let Some(checksum) = progress.completed_steps.get(&step.id) {
                if checksum != &step.checksum {
                    return Err(error(
                        MigrationExecutionErrorKind::ChecksumDrift,
                        "completed migration step checksum changed",
                    )
                    .at_step(&migration.id, &step.id));
                }
                continue;
            }
            if progress.current_fingerprint != step.input_fingerprint {
                return Err(error(
                    MigrationExecutionErrorKind::AmbiguousRecovery,
                    "in-progress migration state does not match the next checked step",
                )
                .at_step(&migration.id, &step.id));
            }
            events.push(MigrationExecutionEvent::StepStarted {
                migration: migration.id.clone(),
                step: step.id.clone(),
                state: step.input_state.clone(),
                checksum: step.checksum.clone(),
            });
            if self.execute_step(runtime, ledger, migration, step, &mut progress, events)? {
                ledger.in_progress = Some(progress);
                store(runtime, ledger)?;
                return Ok(true);
            }
            progress
                .completed_steps
                .insert(step.id.clone(), step.checksum.clone());
            progress
                .current_fingerprint
                .clone_from(&step.output_fingerprint);
            progress.backfill_progress = None;
            ledger.schema_fingerprint = progress.current_fingerprint.clone();
            ledger.in_progress = Some(progress.clone());
            store(runtime, ledger)?;
        }
        if progress.current_fingerprint != path.output_fingerprint {
            return Err(error(
                MigrationExecutionErrorKind::SchemaDrift,
                "completed migration path has the wrong output fingerprint",
            ));
        }
        let prior_heads = ledger.heads.clone();
        for parent in &migration.parents {
            ledger.heads.remove(parent);
        }
        ledger.heads.insert(migration.id.clone());
        ledger.applied.insert(
            migration.id.clone(),
            AppliedMigrationRecord {
                migration: migration.id.clone(),
                checksum: migration.checksum.clone(),
                path_parent: path.parent.clone(),
                prior_heads,
                output_fingerprint: path.output_fingerprint.clone(),
                duration_millis: progress.duration_millis,
            },
        );
        ledger.in_progress = None;
        ledger
            .schema_fingerprint
            .clone_from(&path.output_fingerprint);
        store(runtime, ledger)?;
        events.push(MigrationExecutionEvent::MigrationCompleted {
            direction: MigrationDirection::Forward,
            migration: migration.id.clone(),
            fingerprint: path.output_fingerprint.clone(),
            duration_millis: progress.duration_millis,
            heads: ledger.heads.clone(),
        });
        Ok(false)
    }

    pub(super) fn execute_step<R: MigrationRuntime>(
        &self,
        runtime: &mut R,
        ledger: &mut MigrationLedgerSnapshot,
        migration: &MigrationExecutionNode,
        step: &MigrationExecutionStep,
        progress: &mut InProgressMigrationRecord,
        events: &mut Vec<MigrationExecutionEvent>,
    ) -> Result<bool, MigrationExecutionError> {
        match &step.kind {
            MigrationExecutionStepKind::Transaction {
                boundary: MigrationTransactionBoundary::Begin,
            } => {
                safe_call(MigrationExecutionErrorKind::Transaction, || {
                    runtime.begin_transaction()
                })?;
                progress.transaction_open = true;
            }
            MigrationExecutionStepKind::Transaction {
                boundary: MigrationTransactionBoundary::Commit,
            } => {
                safe_call(MigrationExecutionErrorKind::Transaction, || {
                    runtime.commit_transaction()
                })?;
                progress.transaction_open = false;
            }
            _ => {}
        }
        if let MigrationExecutionStepKind::RecoveryPoint { name } = &step.kind {
            progress.recovery_point = Some(name.clone());
            ledger.in_progress = Some(progress.clone());
            store(runtime, ledger)?;
            events.push(MigrationExecutionEvent::RecoveryPoint {
                migration: migration.id.clone(),
                step: step.id.clone(),
                name: name.clone(),
                fingerprint: progress.current_fingerprint.clone(),
            });
        }
        if matches!(
            &step.kind,
            MigrationExecutionStepKind::Transaction { .. }
                | MigrationExecutionStepKind::RecoveryPoint { .. }
        ) {
            complete_step_event(migration, step, 0, events);
            return Ok(false);
        }

        let maximum_rows = match &step.kind {
            MigrationExecutionStepKind::Backfill {
                maximum_batch_rows, ..
            } => Some(*maximum_batch_rows),
            _ => None,
        };
        let replay = match &step.kind {
            MigrationExecutionStepKind::Backfill { replay, .. } => Some(replay),
            _ => None,
        };
        let mut batches = 0_u64;
        loop {
            let result = safe_call(MigrationExecutionErrorKind::Step, || {
                runtime.execute_step(MigrationStepRequest {
                    migration: &migration.id,
                    step,
                    backfill_progress: progress.backfill_progress.as_deref(),
                })
            })
            .map_err(|error| error.at_step(&migration.id, &step.id))?;
            let (fingerprint, duration, complete) = match result {
                MigrationStepResult::Completed {
                    schema_fingerprint,
                    duration_millis,
                } => (schema_fingerprint, duration_millis, true),
                MigrationStepResult::Assertion {
                    rows,
                    valid,
                    schema_fingerprint,
                    duration_millis,
                } => {
                    if rows == 0 {
                        return Err(error(
                            MigrationExecutionErrorKind::AssertionZeroRows,
                            "migration assertion returned zero rows",
                        )
                        .at_step(&migration.id, &step.id));
                    }
                    if rows > 1 {
                        return Err(error(
                            MigrationExecutionErrorKind::AssertionMultipleRows,
                            "migration assertion returned multiple rows",
                        )
                        .at_step(&migration.id, &step.id));
                    }
                    if valid != Some(true) {
                        return Err(error(
                            MigrationExecutionErrorKind::AssertionFalse,
                            "migration assertion returned false or null",
                        )
                        .at_step(&migration.id, &step.id));
                    }
                    (schema_fingerprint, duration_millis, true)
                }
                MigrationStepResult::BackfillBatch {
                    processed_rows,
                    progress: next_progress,
                    complete,
                    schema_fingerprint,
                    duration_millis,
                } => {
                    let bound = maximum_rows.ok_or_else(|| {
                        error(
                            MigrationExecutionErrorKind::Backfill,
                            "non-backfill step returned backfill progress",
                        )
                    })?;
                    if processed_rows > bound {
                        return Err(error(
                            MigrationExecutionErrorKind::Backfill,
                            "backfill batch exceeded its checked row bound",
                        )
                        .at_step(&migration.id, &step.id));
                    }
                    if !complete {
                        let MigrationReplayPolicy::Idempotent { .. } = replay.ok_or_else(|| {
                            error(
                                MigrationExecutionErrorKind::Backfill,
                                "backfill replay policy is missing",
                            )
                        })?
                        else {
                            return Err(error(
                                MigrationExecutionErrorKind::Backfill,
                                "non-idempotent backfill cannot persist resumable progress",
                            )
                            .at_step(&migration.id, &step.id));
                        };
                        let next =
                            next_progress
                                .filter(|value| !value.is_empty())
                                .ok_or_else(|| {
                                    error(
                                        MigrationExecutionErrorKind::Backfill,
                                        "incomplete backfill batch did not return progress",
                                    )
                                })?;
                        if progress.backfill_progress.as_deref() == Some(next.as_str()) {
                            return Err(error(
                                MigrationExecutionErrorKind::Backfill,
                                "backfill progress did not advance",
                            )
                            .at_step(&migration.id, &step.id));
                        }
                        progress.backfill_progress = Some(next.clone());
                        progress.duration_millis =
                            checked_duration(progress.duration_millis, duration_millis)?;
                        ledger.in_progress = Some(progress.clone());
                        store(runtime, ledger)?;
                        events.push(MigrationExecutionEvent::BackfillProgress {
                            migration: migration.id.clone(),
                            step: step.id.clone(),
                            progress: next,
                            processed_rows,
                        });
                    }
                    (schema_fingerprint, duration_millis, complete)
                }
            };
            if fingerprint != step.output_fingerprint {
                return Err(error(
                    MigrationExecutionErrorKind::SchemaDrift,
                    "migration step produced an unexpected schema fingerprint",
                )
                .at_step(&migration.id, &step.id));
            }
            if complete {
                progress.duration_millis = checked_duration(progress.duration_millis, duration)?;
                complete_step_event(migration, step, duration, events);
                return Ok(false);
            }
            batches = batches.saturating_add(1);
            if batches >= self.limits.maximum_backfill_batches {
                if progress.transaction_open {
                    return Err(error(
                        MigrationExecutionErrorKind::Backfill,
                        "a resumable backfill cannot pause inside a transaction",
                    )
                    .at_step(&migration.id, &step.id));
                }
                events.push(MigrationExecutionEvent::Paused {
                    migration: migration.id.clone(),
                    step: step.id.clone(),
                    recovery_point: progress.recovery_point.clone(),
                });
                return Ok(true);
            }
        }
    }
}

pub(super) fn validate_ledger(
    graph: &MigrationExecutionPlan,
    ledger: &MigrationLedgerSnapshot,
    identity: &MigrationRuntimeIdentity,
) -> Result<(), MigrationExecutionError> {
    if graph.format_version != MIGRATION_EXECUTION_PLAN_FORMAT_VERSION {
        return Err(error(
            MigrationExecutionErrorKind::Ledger,
            "migration execution plan format is not supported",
        ));
    }
    if ledger.provider_family != graph.provider_family
        || ledger.provider_family != identity.family
        || ledger.provider_server_version != identity.server_version
        || ledger.provider_capabilities != identity.capabilities
    {
        return Err(error(
            MigrationExecutionErrorKind::ProviderMismatch,
            "migration ledger provider does not match the compiled graph",
        ));
    }
    if ledger.heads.is_empty() || !valid_fingerprint(&ledger.schema_fingerprint) {
        return Err(error(
            MigrationExecutionErrorKind::HeadMismatch,
            "migration ledger has no valid head or fingerprint",
        ));
    }
    for head in &ledger.heads {
        let expected = graph.baseline_fingerprints.get(head).cloned().or_else(|| {
            graph
                .migrations
                .get(head)
                .and_then(|migration| {
                    let outputs = migration
                        .paths
                        .values()
                        .map(|path| path.output_fingerprint.as_str())
                        .collect::<std::collections::BTreeSet<_>>();
                    (outputs.len() == 1)
                        .then(|| outputs.first().copied())
                        .flatten()
                })
                .map(str::to_string)
        });
        if expected.is_none()
            || (ledger.in_progress.is_none()
                && expected.as_deref() != Some(ledger.schema_fingerprint.as_str()))
        {
            return Err(error(
                MigrationExecutionErrorKind::HeadMismatch,
                format!("migration ledger head '{head}' does not match its schema"),
            ));
        }
    }
    for (id, applied) in &ledger.applied {
        let compiled = graph.migrations.get(id).ok_or_else(|| {
            error(
                MigrationExecutionErrorKind::ChecksumDrift,
                format!("applied migration '{id}' is absent from the graph"),
            )
        })?;
        let path = compiled.paths.get(&applied.path_parent);
        if applied.migration != *id
            || applied.checksum != compiled.checksum
            || path.map(|path| &path.output_fingerprint) != Some(&applied.output_fingerprint)
            || applied.prior_heads.is_empty()
            || applied.prior_heads.iter().any(|head| {
                !graph.baseline_fingerprints.contains_key(head)
                    && !graph.migrations.contains_key(head)
            })
        {
            return Err(error(
                MigrationExecutionErrorKind::ChecksumDrift,
                format!("applied migration '{id}' checksum changed"),
            ));
        }
    }
    Ok(())
}

fn select_path<'a>(
    graph: &'a MigrationExecutionPlan,
    ledger: &MigrationLedgerSnapshot,
) -> Result<(&'a MigrationExecutionNode, &'a MigrationExecutionPath), MigrationExecutionError> {
    if let Some(progress) = &ledger.in_progress {
        let migration = graph.migrations.get(&progress.migration).ok_or_else(|| {
            error(
                MigrationExecutionErrorKind::AmbiguousRecovery,
                "in-progress migration is absent from the compiled graph",
            )
        })?;
        let path = migration.paths.get(&progress.parent).ok_or_else(|| {
            error(
                MigrationExecutionErrorKind::AmbiguousRecovery,
                "in-progress migration parent path is absent",
            )
        })?;
        return Ok((migration, path));
    }
    let mut incomplete_merge = false;
    for id in &graph.topological_order {
        if ledger.applied.contains_key(id) {
            continue;
        }
        let migration = graph.migrations.get(id).ok_or_else(|| {
            error(
                MigrationExecutionErrorKind::HeadMismatch,
                "compiled migration is missing",
            )
        })?;
        for path in migration.paths.values() {
            let parent_is_current = ledger.heads.contains(&path.parent);
            let parent_was_applied = ledger.applied.contains_key(&path.parent);
            let parent_is_baseline = graph.baseline_fingerprints.contains_key(&path.parent);
            if path.input_fingerprint == ledger.schema_fingerprint
                && (parent_is_current || parent_was_applied || parent_is_baseline)
            {
                let all_migration_parents_present = migration.parents.iter().all(|parent| {
                    graph.baseline_fingerprints.contains_key(parent)
                        || ledger.heads.contains(parent)
                        || ledger.applied.contains_key(parent)
                });
                if all_migration_parents_present {
                    return Ok((migration, path));
                }
                incomplete_merge = true;
            }
        }
    }
    if incomplete_merge {
        return Err(error(
            MigrationExecutionErrorKind::IncompleteMerge,
            "migration merge requires every non-baseline parent in applied history",
        ));
    }
    Err(error(
        MigrationExecutionErrorKind::NoMatchingPath,
        "no checked migration path matches the current heads and schema",
    ))
}

fn validate_progress(
    migration: &MigrationExecutionNode,
    path: &MigrationExecutionPath,
    progress: InProgressMigrationRecord,
) -> Result<InProgressMigrationRecord, MigrationExecutionError> {
    if progress.migration != migration.id
        || progress.direction != MigrationDirection::Forward
        || progress.parent != path.parent
        || progress.migration_checksum != migration.checksum
        || !valid_fingerprint(&progress.current_fingerprint)
        || progress.transaction_open
    {
        return Err(error(
            MigrationExecutionErrorKind::AmbiguousRecovery,
            "in-progress migration record does not match the compiled path",
        ));
    }
    let known_steps = path
        .steps
        .iter()
        .map(|step| (&step.id, &step.checksum))
        .collect::<BTreeMap<_, _>>();
    if progress
        .completed_steps
        .iter()
        .any(|(id, checksum)| known_steps.get(id) != Some(&checksum))
    {
        return Err(error(
            MigrationExecutionErrorKind::ChecksumDrift,
            "in-progress migration contains a changed or unknown step",
        ));
    }
    let mut gap = false;
    let mut transaction_open = false;
    for step in &path.steps {
        let completed = progress.completed_steps.contains_key(&step.id);
        if completed && gap {
            return Err(error(
                MigrationExecutionErrorKind::AmbiguousRecovery,
                "in-progress migration completed steps are not a contiguous prefix",
            ));
        }
        if !completed {
            gap = true;
        }
        if completed {
            match &step.kind {
                MigrationExecutionStepKind::Transaction {
                    boundary: MigrationTransactionBoundary::Begin,
                } => transaction_open = true,
                MigrationExecutionStepKind::Transaction {
                    boundary: MigrationTransactionBoundary::Commit,
                } => transaction_open = false,
                _ => {}
            }
        }
    }
    if transaction_open {
        return Err(error(
            MigrationExecutionErrorKind::AmbiguousRecovery,
            "migration cannot resume from inside an uncommitted transaction",
        ));
    }
    Ok(progress)
}

pub(super) fn validate_runtime_identity(
    graph: &MigrationExecutionPlan,
    identity: &MigrationRuntimeIdentity,
) -> Result<(), MigrationExecutionError> {
    if identity.family != graph.provider_family {
        return Err(error(
            MigrationExecutionErrorKind::ProviderMismatch,
            "migration runtime provider family does not match the execution plan",
        ));
    }
    let server_version = Version::parse(&identity.server_version).map_err(|_| {
        error(
            MigrationExecutionErrorKind::ProviderMismatch,
            "migration runtime returned an invalid server version",
        )
    })?;
    for migration in graph.migrations.values() {
        if migration.provider.family != identity.family {
            return Err(error(
                MigrationExecutionErrorKind::ProviderMismatch,
                "migration provider family differs from the runtime",
            ));
        }
        if let Some(minimum) = &migration.provider.minimum_server_version {
            let minimum = Version::parse(minimum).map_err(|_| {
                error(
                    MigrationExecutionErrorKind::ProviderMismatch,
                    "migration plan contains an invalid minimum server version",
                )
            })?;
            if server_version < minimum {
                return Err(error(
                    MigrationExecutionErrorKind::ProviderMismatch,
                    "migration requires a newer server version",
                ));
            }
        }
        if !migration
            .provider
            .required_capabilities
            .is_subset(&identity.capabilities)
        {
            return Err(error(
                MigrationExecutionErrorKind::ProviderMismatch,
                "migration runtime lacks a required provider capability",
            ));
        }
    }
    Ok(())
}

pub(super) fn store<R: MigrationRuntime>(
    runtime: &mut R,
    ledger: &MigrationLedgerSnapshot,
) -> Result<(), MigrationExecutionError> {
    safe_call(MigrationExecutionErrorKind::Ledger, || {
        runtime.store_ledger(ledger)
    })
}

pub(super) fn safe_call<T>(
    kind: MigrationExecutionErrorKind,
    callback: impl FnOnce() -> Result<T, String>,
) -> Result<T, MigrationExecutionError> {
    match catch_unwind(AssertUnwindSafe(callback)) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(_)) => Err(error(kind, "migration runtime operation failed")),
        Err(_) => Err(error(
            MigrationExecutionErrorKind::RuntimePanic,
            "migration runtime panicked at the provider boundary",
        )),
    }
}

fn complete_step_event(
    migration: &MigrationExecutionNode,
    step: &MigrationExecutionStep,
    duration_millis: u64,
    events: &mut Vec<MigrationExecutionEvent>,
) {
    events.push(MigrationExecutionEvent::StepCompleted {
        migration: migration.id.clone(),
        step: step.id.clone(),
        state: step.output_state.clone(),
        fingerprint: step.output_fingerprint.clone(),
        duration_millis,
    });
}

pub(super) fn checked_duration(
    current: u64,
    addition: u64,
) -> Result<u64, MigrationExecutionError> {
    current.checked_add(addition).ok_or_else(|| {
        error(
            MigrationExecutionErrorKind::Ledger,
            "migration duration exceeded its record range",
        )
    })
}

pub(super) fn report(
    status: MigrationExecutionStatus,
    ledger: MigrationLedgerSnapshot,
    events: &mut Vec<MigrationExecutionEvent>,
) -> MigrationExecutionReport {
    MigrationExecutionReport {
        status,
        heads: ledger.heads,
        schema_fingerprint: ledger.schema_fingerprint,
        events: std::mem::take(events),
    }
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

pub(super) fn error(
    kind: MigrationExecutionErrorKind,
    message: impl Into<String>,
) -> MigrationExecutionError {
    MigrationExecutionError::new(kind, message)
}
