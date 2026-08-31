use super::engine::{error, report, safe_call, store, validate_ledger, validate_runtime_identity};
use super::{
    InProgressMigrationRecord, MigrationDirection, MigrationEngine, MigrationExecutionError,
    MigrationExecutionErrorKind, MigrationExecutionEvent, MigrationExecutionNode,
    MigrationExecutionPath, MigrationExecutionPlan, MigrationExecutionReport,
    MigrationExecutionStatus, MigrationExecutionStep, MigrationLedgerSnapshot, MigrationRuntime,
};
use std::collections::BTreeMap;

impl MigrationEngine {
    pub fn rollback_last<R: MigrationRuntime>(
        &self,
        plan: &MigrationExecutionPlan,
        runtime: &mut R,
    ) -> Result<MigrationExecutionReport, MigrationExecutionError> {
        if self.limits.maximum_backfill_batches == 0 {
            return Err(error(
                MigrationExecutionErrorKind::Backfill,
                "migration rollback requires a positive backfill batch limit",
            ));
        }
        let lock = safe_call(MigrationExecutionErrorKind::Lock, || {
            runtime.acquire_lock(plan)
        })?;
        let identity = lock.identity.clone();
        let mut events = vec![MigrationExecutionEvent::LockAcquired {
            identity: identity.clone(),
        }];
        let result = self.rollback_locked(plan, runtime, &mut events);
        let release = safe_call(MigrationExecutionErrorKind::Lock, || {
            runtime.release_lock(lock)
        });
        match (result, release) {
            (Ok(mut report), Ok(())) => {
                report
                    .events
                    .push(MigrationExecutionEvent::LockReleased { identity });
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

    fn rollback_locked<R: MigrationRuntime>(
        &self,
        plan: &MigrationExecutionPlan,
        runtime: &mut R,
        events: &mut Vec<MigrationExecutionEvent>,
    ) -> Result<MigrationExecutionReport, MigrationExecutionError> {
        let mut ledger = safe_call(MigrationExecutionErrorKind::Ledger, || {
            runtime.load_ledger()
        })?;
        let identity = safe_call(MigrationExecutionErrorKind::ProviderMismatch, || {
            runtime.identity()
        })?;
        validate_runtime_identity(plan, &identity)?;
        validate_ledger(plan, &ledger, &identity)?;
        let observed = safe_call(MigrationExecutionErrorKind::SchemaDrift, || {
            runtime.inspect_schema_fingerprint()
        })?;
        if observed != ledger.schema_fingerprint {
            return Err(error(
                MigrationExecutionErrorKind::SchemaDrift,
                "migration ledger fingerprint differs from the live schema",
            ));
        }
        let migration_id = match &ledger.in_progress {
            Some(progress) if progress.direction == MigrationDirection::Rollback => {
                progress.migration.clone()
            }
            Some(_) => {
                return Err(error(
                    MigrationExecutionErrorKind::AmbiguousRecovery,
                    "forward migration progress must finish before rollback",
                ));
            }
            None if ledger.heads.len() == 1 => ledger.heads.first().cloned().ok_or_else(|| {
                error(MigrationExecutionErrorKind::HeadMismatch, "head is absent")
            })?,
            None => {
                return Err(error(
                    MigrationExecutionErrorKind::HeadMismatch,
                    "rollback requires one current migration head",
                ));
            }
        };
        let migration = plan.migrations.get(&migration_id).ok_or_else(|| {
            error(
                MigrationExecutionErrorKind::ForwardOnly,
                "the current head is a baseline and cannot be rolled back",
            )
        })?;
        let applied = ledger.applied.get(&migration_id).cloned().ok_or_else(|| {
            error(
                MigrationExecutionErrorKind::ChecksumDrift,
                "the current migration head has no applied record",
            )
        })?;
        let path = migration.paths.get(&applied.path_parent).ok_or_else(|| {
            error(
                MigrationExecutionErrorKind::ChecksumDrift,
                "the applied migration path is absent from the plan",
            )
        })?;
        let steps = path.rollback.as_deref().ok_or_else(|| {
            error(
                MigrationExecutionErrorKind::ForwardOnly,
                "the current migration is forward-only",
            )
        })?;
        let execution =
            self.execute_rollback_path(runtime, &mut ledger, migration, path, steps, events);
        match execution {
            Ok(true) => Ok(report(MigrationExecutionStatus::Paused, ledger, events)),
            Ok(false) => Ok(report(MigrationExecutionStatus::Complete, ledger, events)),
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
                Err(primary)
            }
        }
    }

    fn execute_rollback_path<R: MigrationRuntime>(
        &self,
        runtime: &mut R,
        ledger: &mut MigrationLedgerSnapshot,
        migration: &MigrationExecutionNode,
        path: &MigrationExecutionPath,
        steps: &[MigrationExecutionStep],
        events: &mut Vec<MigrationExecutionEvent>,
    ) -> Result<bool, MigrationExecutionError> {
        let applied = ledger.applied.get(&migration.id).cloned().ok_or_else(|| {
            error(
                MigrationExecutionErrorKind::ChecksumDrift,
                "record is absent",
            )
        })?;
        let mut progress = if let Some(progress) = ledger.in_progress.take() {
            validate_rollback_progress(migration, path, steps, progress)?
        } else {
            events.push(MigrationExecutionEvent::MigrationStarted {
                direction: MigrationDirection::Rollback,
                migration: migration.id.clone(),
                parent: path.parent.clone(),
                input_fingerprint: path.output_fingerprint.clone(),
            });
            InProgressMigrationRecord {
                direction: MigrationDirection::Rollback,
                migration: migration.id.clone(),
                parent: path.parent.clone(),
                migration_checksum: migration.checksum.clone(),
                completed_steps: BTreeMap::new(),
                current_fingerprint: path.output_fingerprint.clone(),
                recovery_point: None,
                backfill_progress: None,
                transaction_open: false,
                duration_millis: 0,
            }
        };
        ledger.in_progress = Some(progress.clone());
        store(runtime, ledger)?;
        for step in steps {
            if let Some(checksum) = progress.completed_steps.get(&step.id) {
                if checksum != &step.checksum {
                    return Err(error(
                        MigrationExecutionErrorKind::ChecksumDrift,
                        "completed rollback step checksum changed",
                    )
                    .at_step(&migration.id, &step.id));
                }
                continue;
            }
            if progress.current_fingerprint != step.input_fingerprint {
                return Err(error(
                    MigrationExecutionErrorKind::AmbiguousRecovery,
                    "rollback state does not match the next checked step",
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
        if progress.current_fingerprint != path.input_fingerprint {
            return Err(error(
                MigrationExecutionErrorKind::SchemaDrift,
                "completed rollback path has the wrong output fingerprint",
            ));
        }
        ledger.heads = applied.prior_heads;
        ledger.applied.remove(&migration.id);
        ledger.in_progress = None;
        ledger
            .schema_fingerprint
            .clone_from(&path.input_fingerprint);
        store(runtime, ledger)?;
        events.push(MigrationExecutionEvent::MigrationCompleted {
            direction: MigrationDirection::Rollback,
            migration: migration.id.clone(),
            fingerprint: path.input_fingerprint.clone(),
            duration_millis: progress.duration_millis,
            heads: ledger.heads.clone(),
        });
        Ok(false)
    }
}

fn validate_rollback_progress(
    migration: &MigrationExecutionNode,
    path: &MigrationExecutionPath,
    steps: &[MigrationExecutionStep],
    progress: InProgressMigrationRecord,
) -> Result<InProgressMigrationRecord, MigrationExecutionError> {
    if progress.direction != MigrationDirection::Rollback
        || progress.migration != migration.id
        || progress.parent != path.parent
        || progress.migration_checksum != migration.checksum
        || progress.transaction_open
    {
        return Err(error(
            MigrationExecutionErrorKind::AmbiguousRecovery,
            "rollback progress does not match the checked reverse path",
        ));
    }
    let known = steps
        .iter()
        .map(|step| (&step.id, &step.checksum))
        .collect::<BTreeMap<_, _>>();
    if progress
        .completed_steps
        .iter()
        .any(|(id, checksum)| known.get(id) != Some(&checksum))
    {
        return Err(error(
            MigrationExecutionErrorKind::ChecksumDrift,
            "rollback progress contains a changed or unknown step",
        ));
    }
    let mut gap = false;
    let mut expected_fingerprint = path.output_fingerprint.clone();
    for step in steps {
        let completed = progress.completed_steps.contains_key(&step.id);
        if completed && gap {
            return Err(error(
                MigrationExecutionErrorKind::AmbiguousRecovery,
                "completed rollback steps are not a contiguous prefix",
            ));
        }
        if completed {
            if expected_fingerprint != step.input_fingerprint {
                return Err(error(
                    MigrationExecutionErrorKind::AmbiguousRecovery,
                    "completed rollback step has a discontinuous input fingerprint",
                ));
            }
            expected_fingerprint.clone_from(&step.output_fingerprint);
        } else {
            gap = true;
        }
    }
    if progress.current_fingerprint != expected_fingerprint {
        return Err(error(
            MigrationExecutionErrorKind::AmbiguousRecovery,
            "rollback progress fingerprint does not match its completed prefix",
        ));
    }
    Ok(progress)
}
