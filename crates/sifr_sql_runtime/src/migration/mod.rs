mod engine;

pub use engine::MigrationEngine;
use serde::{Deserialize, Serialize};
use sifr_sql_contract::{
    CompiledMigrationGraph, CompiledMigrationStep, MigrationNodeId, MigrationStateIdentity,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppliedMigrationRecord {
    pub migration: MigrationNodeId,
    pub checksum: String,
    pub output_fingerprint: String,
    pub duration_millis: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InProgressMigrationRecord {
    pub migration: MigrationNodeId,
    pub parent: MigrationNodeId,
    pub migration_checksum: String,
    pub completed_steps: BTreeMap<MigrationNodeId, String>,
    pub current_fingerprint: String,
    pub recovery_point: Option<String>,
    pub backfill_progress: Option<String>,
    pub transaction_open: bool,
    pub duration_millis: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationLedgerSnapshot {
    pub provider_family: String,
    pub heads: BTreeSet<MigrationNodeId>,
    pub schema_fingerprint: String,
    pub applied: BTreeMap<MigrationNodeId, AppliedMigrationRecord>,
    pub in_progress: Option<InProgressMigrationRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationLock {
    pub identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationStepRequest<'a> {
    pub migration: &'a MigrationNodeId,
    pub step: &'a CompiledMigrationStep,
    pub backfill_progress: Option<&'a str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MigrationStepResult {
    Completed {
        schema_fingerprint: String,
        duration_millis: u64,
    },
    Assertion {
        rows: u64,
        valid: Option<bool>,
        schema_fingerprint: String,
        duration_millis: u64,
    },
    BackfillBatch {
        processed_rows: u64,
        progress: Option<String>,
        complete: bool,
        schema_fingerprint: String,
        duration_millis: u64,
    },
}

pub trait MigrationRuntime {
    fn acquire_lock(&mut self, graph: &CompiledMigrationGraph) -> Result<MigrationLock, String>;
    fn release_lock(&mut self, lock: MigrationLock) -> Result<(), String>;
    fn load_ledger(&mut self) -> Result<MigrationLedgerSnapshot, String>;
    fn store_ledger(&mut self, ledger: &MigrationLedgerSnapshot) -> Result<(), String>;
    fn begin_transaction(&mut self) -> Result<(), String>;
    fn commit_transaction(&mut self) -> Result<(), String>;
    fn rollback_transaction(&mut self) -> Result<(), String>;
    fn execute_step(
        &mut self,
        request: MigrationStepRequest<'_>,
    ) -> Result<MigrationStepResult, String>;
    fn inspect_schema_fingerprint(&mut self) -> Result<String, String>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MigrationExecutionLimits {
    pub maximum_backfill_batches: u64,
}

impl Default for MigrationExecutionLimits {
    fn default() -> Self {
        Self {
            maximum_backfill_batches: 1_000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationExecutionStatus {
    Complete,
    Paused,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MigrationExecutionEvent {
    LockAcquired {
        identity: String,
    },
    MigrationStarted {
        migration: MigrationNodeId,
        parent: MigrationNodeId,
        input_fingerprint: String,
    },
    StepStarted {
        migration: MigrationNodeId,
        step: MigrationNodeId,
        state: MigrationStateIdentity,
        checksum: String,
    },
    BackfillProgress {
        migration: MigrationNodeId,
        step: MigrationNodeId,
        progress: String,
        processed_rows: u64,
    },
    RecoveryPoint {
        migration: MigrationNodeId,
        step: MigrationNodeId,
        name: String,
        fingerprint: String,
    },
    StepCompleted {
        migration: MigrationNodeId,
        step: MigrationNodeId,
        state: MigrationStateIdentity,
        fingerprint: String,
        duration_millis: u64,
    },
    MigrationCompleted {
        migration: MigrationNodeId,
        fingerprint: String,
        duration_millis: u64,
        heads: BTreeSet<MigrationNodeId>,
    },
    Paused {
        migration: MigrationNodeId,
        step: MigrationNodeId,
        recovery_point: Option<String>,
    },
    LockReleased {
        identity: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationExecutionReport {
    pub status: MigrationExecutionStatus,
    pub heads: BTreeSet<MigrationNodeId>,
    pub schema_fingerprint: String,
    pub events: Vec<MigrationExecutionEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MigrationExecutionErrorKind {
    Lock,
    Ledger,
    ProviderMismatch,
    HeadMismatch,
    SchemaDrift,
    ChecksumDrift,
    AmbiguousRecovery,
    AmbiguousPath,
    Transaction,
    Step,
    AssertionFalse,
    AssertionZeroRows,
    AssertionMultipleRows,
    Backfill,
    RuntimePanic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationExecutionError {
    pub kind: MigrationExecutionErrorKind,
    pub migration: Option<MigrationNodeId>,
    pub step: Option<MigrationNodeId>,
    pub recovery_point: Option<String>,
    pub rollback_error: bool,
    pub lock_release_error: bool,
    pub message: String,
}

impl MigrationExecutionError {
    pub(crate) fn new(kind: MigrationExecutionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            migration: None,
            step: None,
            recovery_point: None,
            rollback_error: false,
            lock_release_error: false,
            message: message.into(),
        }
    }

    pub(crate) fn at_step(mut self, migration: &MigrationNodeId, step: &MigrationNodeId) -> Self {
        self.migration = Some(migration.clone());
        self.step = Some(step.clone());
        self
    }
}

impl fmt::Display for MigrationExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MigrationExecutionError {}
