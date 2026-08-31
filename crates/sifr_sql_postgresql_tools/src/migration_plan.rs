use serde::{Deserialize, Serialize};
use sifr_sql_postgresql::{PostgresDdlExecutionClass, classify_migration_ddl};
use sifr_sql_runtime::{
    MIGRATION_EXECUTION_PLAN_FORMAT_VERSION, MigrationDirection, MigrationExecutionPlan,
    MigrationExecutionStep, MigrationExecutionStepKind, MigrationId, MigrationTransactionBoundary,
    MigrationTransactionRequirement,
};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostgresMigrationPlanError {
    pub message: String,
}

impl fmt::Display for PostgresMigrationPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PostgresMigrationPlanError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostgresMigrationActionKind {
    Ddl,
    SqlData,
    SifrData,
    Assertion,
    Backfill,
    TransactionBegin,
    TransactionCommit,
    RecoveryPoint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresMigrationAction {
    pub migration: MigrationId,
    pub parent: MigrationId,
    pub direction: MigrationDirection,
    pub step: MigrationId,
    pub step_checksum: String,
    pub action: PostgresMigrationActionKind,
    pub transactional: bool,
    pub recovery_point: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresMigrationOperatorPlan {
    pub provider_family: String,
    pub forward_only: Vec<MigrationId>,
    pub reversible: Vec<MigrationId>,
    pub actions: Vec<PostgresMigrationAction>,
}

pub fn validate_postgres_migration_plan(
    plan: &MigrationExecutionPlan,
) -> Result<PostgresMigrationOperatorPlan, PostgresMigrationPlanError> {
    validate_plan_shape(plan)?;
    if plan.provider_family != "postgresql" {
        return Err(plan_error(
            "PostgreSQL migration execution requires a PostgreSQL plan",
        ));
    }
    let mut forward_only = Vec::new();
    let mut reversible = Vec::new();
    let mut actions = Vec::new();
    for migration_id in &plan.topological_order {
        let migration = plan.migrations.get(migration_id).ok_or_else(|| {
            plan_error("PostgreSQL migration topological order contains an unknown node")
        })?;
        let is_reversible = migration.paths.values().all(|path| path.rollback.is_some());
        if is_reversible {
            reversible.push(migration.id.clone());
        } else {
            forward_only.push(migration.id.clone());
        }
        for path in migration.paths.values() {
            validate_steps(
                migration.transaction_requirement,
                &migration.id,
                &path.parent,
                MigrationDirection::Forward,
                &path.steps,
                &mut actions,
            )?;
            if let Some(rollback) = &path.rollback {
                validate_steps(
                    migration.transaction_requirement,
                    &migration.id,
                    &path.parent,
                    MigrationDirection::Rollback,
                    rollback,
                    &mut actions,
                )?;
            }
        }
    }
    Ok(PostgresMigrationOperatorPlan {
        provider_family: plan.provider_family.clone(),
        forward_only,
        reversible,
        actions,
    })
}

fn validate_plan_shape(plan: &MigrationExecutionPlan) -> Result<(), PostgresMigrationPlanError> {
    if plan.format_version != MIGRATION_EXECUTION_PLAN_FORMAT_VERSION
        || !valid_id(&plan.head)
        || !valid_fingerprint(&plan.target_fingerprint)
        || plan.baseline_fingerprints.is_empty()
        || plan
            .baseline_fingerprints
            .iter()
            .any(|(id, fingerprint)| !valid_id(id) || !valid_fingerprint(fingerprint))
        || plan
            .baseline_fingerprints
            .keys()
            .any(|id| plan.migrations.contains_key(id))
    {
        return Err(plan_error("PostgreSQL migration plan metadata is invalid"));
    }
    let ordered = plan
        .topological_order
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let declared = plan.migrations.keys().cloned().collect::<BTreeSet<_>>();
    if ordered.len() != plan.topological_order.len() || ordered != declared {
        return Err(plan_error(
            "PostgreSQL migration topological order is incomplete or duplicated",
        ));
    }
    if plan.migrations.is_empty() {
        return if plan
            .baseline_fingerprints
            .get(&plan.head)
            .is_some_and(|fingerprint| fingerprint == &plan.target_fingerprint)
        {
            Ok(())
        } else {
            Err(plan_error(
                "an empty PostgreSQL migration plan must target one baseline",
            ))
        };
    }
    if !plan.migrations.contains_key(&plan.head) {
        return Err(plan_error(
            "PostgreSQL migration head is absent from the plan",
        ));
    }
    let mut known_fingerprints = plan.baseline_fingerprints.clone();
    let mut referenced_migrations = BTreeSet::new();
    for migration_id in &plan.topological_order {
        let migration = plan.migrations.get(migration_id).ok_or_else(|| {
            plan_error("PostgreSQL migration topological order contains an unknown node")
        })?;
        if migration.id != *migration_id
            || !valid_id(migration_id)
            || !valid_fingerprint(&migration.checksum)
            || migration.author.trim().is_empty()
            || migration.created_at.trim().is_empty()
            || migration.provider.family != plan.provider_family
            || migration
                .provider
                .required_capabilities
                .iter()
                .any(|capability| capability.trim().is_empty())
            || migration.parents.is_empty()
            || migration.paths.keys().collect::<BTreeSet<_>>()
                != migration.parents.iter().collect::<BTreeSet<_>>()
            || migration
                .parents
                .iter()
                .any(|parent| !known_fingerprints.contains_key(parent))
        {
            return Err(plan_error(format!(
                "PostgreSQL migration '{migration_id}' metadata is invalid"
            )));
        }
        for parent in &migration.parents {
            if plan.migrations.contains_key(parent) {
                referenced_migrations.insert(parent.clone());
            }
        }
        let mut outputs = BTreeSet::new();
        for (parent, path) in &migration.paths {
            if known_fingerprints.get(parent) != Some(&path.input_fingerprint) {
                return Err(plan_error(format!(
                    "PostgreSQL migration '{migration_id}' parent fingerprint is invalid"
                )));
            }
            validate_path(parent, path)?;
            outputs.insert(path.output_fingerprint.clone());
        }
        let output = outputs.first().cloned().ok_or_else(|| {
            plan_error(format!(
                "PostgreSQL migration '{migration_id}' has no output fingerprint"
            ))
        })?;
        if outputs.len() != 1 {
            return Err(plan_error(format!(
                "PostgreSQL migration '{migration_id}' paths have different outputs"
            )));
        }
        known_fingerprints.insert(migration_id.clone(), output);
    }
    let terminals = plan
        .migrations
        .keys()
        .filter(|id| !referenced_migrations.contains(*id))
        .cloned()
        .collect::<BTreeSet<_>>();
    if terminals != BTreeSet::from([plan.head.clone()])
        || plan.migrations.get(&plan.head).is_some_and(|head| {
            head.paths
                .values()
                .any(|path| path.output_fingerprint != plan.target_fingerprint)
        })
    {
        return Err(plan_error(
            "PostgreSQL migration plan does not have one exact target head",
        ));
    }
    Ok(())
}

fn validate_path(
    parent: &MigrationId,
    path: &sifr_sql_runtime::MigrationExecutionPath,
) -> Result<(), PostgresMigrationPlanError> {
    if path.parent != *parent
        || !valid_fingerprint(&path.input_fingerprint)
        || !valid_fingerprint(&path.output_fingerprint)
        || path.steps.is_empty()
    {
        return Err(plan_error("PostgreSQL migration path metadata is invalid"));
    }
    validate_step_sequence(
        &path.input_fingerprint,
        &path.output_fingerprint,
        &path.steps,
    )?;
    if let Some(rollback) = &path.rollback {
        if rollback.is_empty() {
            return Err(plan_error("PostgreSQL reverse path is empty"));
        }
        validate_step_sequence(&path.output_fingerprint, &path.input_fingerprint, rollback)?;
    }
    Ok(())
}

fn validate_step_sequence(
    input: &str,
    output: &str,
    steps: &[MigrationExecutionStep],
) -> Result<(), PostgresMigrationPlanError> {
    let mut fingerprint = input;
    let mut identities = BTreeSet::new();
    for step in steps {
        if !valid_id(&step.id)
            || !identities.insert(step.id.clone())
            || !valid_state_id(step.input_state.as_str())
            || !valid_state_id(step.output_state.as_str())
            || !valid_fingerprint(&step.checksum)
            || step.input_fingerprint != fingerprint
            || !valid_fingerprint(&step.output_fingerprint)
        {
            return Err(plan_error("PostgreSQL migration step sequence is invalid"));
        }
        fingerprint = &step.output_fingerprint;
    }
    if fingerprint != output {
        return Err(plan_error(
            "PostgreSQL migration path output fingerprint is invalid",
        ));
    }
    Ok(())
}

fn valid_id(value: &MigrationId) -> bool {
    valid_id_value(value.as_str(), 128)
}

fn valid_state_id(value: &str) -> bool {
    valid_id_value(value, 512)
}

fn valid_id_value(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn validate_steps(
    requirement: MigrationTransactionRequirement,
    migration: &MigrationId,
    parent: &MigrationId,
    direction: MigrationDirection,
    steps: &[MigrationExecutionStep],
    actions: &mut Vec<PostgresMigrationAction>,
) -> Result<(), PostgresMigrationPlanError> {
    let mut transaction_open = false;
    let mut transaction_count = 0_u32;
    let mut recovery_point = None::<String>;
    for step in steps {
        let (action, transactional) = match &step.kind {
            MigrationExecutionStepKind::Ddl { statement } => {
                match classify_migration_ddl(statement) {
                    PostgresDdlExecutionClass::Transactional => {
                        (PostgresMigrationActionKind::Ddl, transaction_open)
                    }
                    PostgresDdlExecutionClass::RequiresAutocommit { .. } => {
                        if transaction_open || recovery_point.is_none() {
                            return Err(plan_error(
                                "non-transactional PostgreSQL DDL requires an explicit recovery point outside a transaction",
                            ));
                        }
                        (PostgresMigrationActionKind::Ddl, false)
                    }
                }
            }
            MigrationExecutionStepKind::SqlData { .. } => {
                (PostgresMigrationActionKind::SqlData, transaction_open)
            }
            MigrationExecutionStepKind::SifrData { .. } => {
                (PostgresMigrationActionKind::SifrData, transaction_open)
            }
            MigrationExecutionStepKind::Assertion { .. } => {
                (PostgresMigrationActionKind::Assertion, transaction_open)
            }
            MigrationExecutionStepKind::Backfill { .. } => {
                (PostgresMigrationActionKind::Backfill, transaction_open)
            }
            MigrationExecutionStepKind::Transaction {
                boundary: MigrationTransactionBoundary::Begin,
            } => {
                if transaction_open || requirement == MigrationTransactionRequirement::Forbidden {
                    return Err(plan_error("invalid PostgreSQL transaction begin boundary"));
                }
                transaction_open = true;
                transaction_count = transaction_count.saturating_add(1);
                (PostgresMigrationActionKind::TransactionBegin, true)
            }
            MigrationExecutionStepKind::Transaction {
                boundary: MigrationTransactionBoundary::Commit,
            } => {
                if !transaction_open {
                    return Err(plan_error("invalid PostgreSQL transaction commit boundary"));
                }
                transaction_open = false;
                (PostgresMigrationActionKind::TransactionCommit, true)
            }
            MigrationExecutionStepKind::RecoveryPoint { name } => {
                recovery_point = Some(name.clone());
                (PostgresMigrationActionKind::RecoveryPoint, transaction_open)
            }
        };
        actions.push(PostgresMigrationAction {
            migration: migration.clone(),
            parent: parent.clone(),
            direction,
            step: step.id.clone(),
            step_checksum: step.checksum.clone(),
            action,
            transactional,
            recovery_point: recovery_point.clone(),
        });
    }
    if transaction_open {
        return Err(plan_error(
            "PostgreSQL migration path leaves a transaction open",
        ));
    }
    let completely_enclosed = matches!(
        steps.first().map(|step| &step.kind),
        Some(MigrationExecutionStepKind::Transaction {
            boundary: MigrationTransactionBoundary::Begin,
        })
    ) && matches!(
        steps.last().map(|step| &step.kind),
        Some(MigrationExecutionStepKind::Transaction {
            boundary: MigrationTransactionBoundary::Commit,
        })
    );
    if requirement == MigrationTransactionRequirement::Required
        && (transaction_count != 1 || !completely_enclosed)
    {
        return Err(plan_error(
            "transaction-required PostgreSQL paths need one complete outer boundary",
        ));
    }
    Ok(())
}

fn plan_error(message: impl Into<String>) -> PostgresMigrationPlanError {
    PostgresMigrationPlanError {
        message: message.into(),
    }
}
