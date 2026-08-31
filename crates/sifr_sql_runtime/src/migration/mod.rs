mod engine;
mod plan;
mod rollback;

pub use engine::MigrationEngine;
pub use plan::{
    MIGRATION_EXECUTION_PLAN_FORMAT_VERSION, MigrationExecutionNode, MigrationExecutionPath,
    MigrationExecutionPlan, MigrationExecutionStep, MigrationExecutionStepKind, MigrationId,
    MigrationReplayPolicy, MigrationRuntimeConstraint, MigrationStateId,
    MigrationTransactionBoundary, MigrationTransactionRequirement,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppliedMigrationRecord {
    pub migration: MigrationId,
    pub checksum: String,
    pub path_parent: MigrationId,
    pub prior_heads: BTreeSet<MigrationId>,
    pub output_fingerprint: String,
    pub duration_millis: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InProgressMigrationRecord {
    pub direction: MigrationDirection,
    pub migration: MigrationId,
    pub parent: MigrationId,
    pub migration_checksum: String,
    pub completed_steps: BTreeMap<MigrationId, String>,
    pub current_fingerprint: String,
    pub recovery_point: Option<String>,
    pub backfill_progress: Option<String>,
    pub transaction_open: bool,
    pub duration_millis: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationDirection {
    Forward,
    Rollback,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationLedgerSnapshot {
    pub provider_family: String,
    pub provider_server_version: String,
    pub provider_capabilities: BTreeSet<String>,
    pub heads: BTreeSet<MigrationId>,
    pub schema_fingerprint: String,
    pub applied: BTreeMap<MigrationId, AppliedMigrationRecord>,
    pub in_progress: Option<InProgressMigrationRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationLock {
    pub identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationStepRequest<'a> {
    pub migration: &'a MigrationId,
    pub step: &'a MigrationExecutionStep,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationRuntimeIdentity {
    pub family: String,
    pub server_version: String,
    pub capabilities: BTreeSet<String>,
}

pub trait MigrationRuntime {
    fn identity(&mut self) -> Result<MigrationRuntimeIdentity, String>;
    fn acquire_lock(&mut self, plan: &MigrationExecutionPlan) -> Result<MigrationLock, String>;
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
        direction: MigrationDirection,
        migration: MigrationId,
        parent: MigrationId,
        input_fingerprint: String,
    },
    StepStarted {
        migration: MigrationId,
        step: MigrationId,
        state: MigrationStateId,
        checksum: String,
    },
    BackfillProgress {
        migration: MigrationId,
        step: MigrationId,
        progress: String,
        processed_rows: u64,
    },
    RecoveryPoint {
        migration: MigrationId,
        step: MigrationId,
        name: String,
        fingerprint: String,
    },
    StepCompleted {
        migration: MigrationId,
        step: MigrationId,
        state: MigrationStateId,
        fingerprint: String,
        duration_millis: u64,
    },
    MigrationCompleted {
        direction: MigrationDirection,
        migration: MigrationId,
        fingerprint: String,
        duration_millis: u64,
        heads: BTreeSet<MigrationId>,
    },
    Paused {
        migration: MigrationId,
        step: MigrationId,
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
    pub heads: BTreeSet<MigrationId>,
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
    NoMatchingPath,
    IncompleteMerge,
    ForwardOnly,
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
    pub migration: Option<MigrationId>,
    pub step: Option<MigrationId>,
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

    pub(crate) fn at_step(mut self, migration: &MigrationId, step: &MigrationId) -> Self {
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
