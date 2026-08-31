use serde::{Deserialize, Serialize};
use sifr_sql_runtime::{
    MIGRATION_EXECUTION_PLAN_FORMAT_VERSION, MigrationDirection, MigrationExecutionPlan,
    MigrationExecutionStep, MigrationExecutionStepKind, MigrationId, MigrationTransactionBoundary,
    MigrationTransactionRequirement,
};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MysqlMigrationPlanError {
    pub message: String,
}

impl fmt::Display for MysqlMigrationPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MysqlMigrationPlanError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MysqlMigrationActionKind {
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
pub struct MysqlMigrationAction {
    pub migration: MigrationId,
    pub parent: MigrationId,
    pub direction: MigrationDirection,
    pub step: MigrationId,
    pub step_checksum: String,
    pub action: MysqlMigrationActionKind,
    pub transactional: bool,
    pub recovery_point: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlMigrationOperatorPlan {
    pub provider_family: String,
    pub forward_only: Vec<MigrationId>,
    pub reversible: Vec<MigrationId>,
    pub actions: Vec<MysqlMigrationAction>,
}

pub fn validate_mysql_migration_plan(
    plan: &MigrationExecutionPlan,
) -> Result<MysqlMigrationOperatorPlan, MysqlMigrationPlanError> {
    validate_plan_shape(plan)?;
    let mut forward_only = Vec::new();
    let mut reversible = Vec::new();
    let mut actions = Vec::new();
    for id in &plan.topological_order {
        let node = plan
            .migrations
            .get(id)
            .ok_or_else(|| plan_error("MySQL migration order contains an unknown node"))?;
        if node.paths.values().all(|path| path.rollback.is_some()) {
            reversible.push(id.clone());
        } else {
            forward_only.push(id.clone());
        }
        for (parent, path) in &node.paths {
            validate_sequence(
                node.transaction_requirement,
                id,
                parent,
                MigrationDirection::Forward,
                &path.input_fingerprint,
                &path.output_fingerprint,
                &path.steps,
                &mut actions,
            )?;
            if let Some(rollback) = &path.rollback {
                validate_sequence(
                    node.transaction_requirement,
                    id,
                    parent,
                    MigrationDirection::Rollback,
                    &path.output_fingerprint,
                    &path.input_fingerprint,
                    rollback,
                    &mut actions,
                )?;
            }
        }
    }
    Ok(MysqlMigrationOperatorPlan {
        provider_family: plan.provider_family.clone(),
        forward_only,
        reversible,
        actions,
    })
}

fn validate_plan_shape(plan: &MigrationExecutionPlan) -> Result<(), MysqlMigrationPlanError> {
    if plan.format_version != MIGRATION_EXECUTION_PLAN_FORMAT_VERSION
        || plan.provider_family != "mysql"
        || !valid_fingerprint(&plan.target_fingerprint)
        || plan.baseline_fingerprints.is_empty()
        || plan
            .baseline_fingerprints
            .iter()
            .any(|(id, fingerprint)| !valid_id(id) || !valid_fingerprint(fingerprint))
    {
        return Err(plan_error("MySQL migration plan metadata is invalid"));
    }
    let order = plan
        .topological_order
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let declared = plan.migrations.keys().cloned().collect::<BTreeSet<_>>();
    if order != declared || order.len() != plan.topological_order.len() {
        return Err(plan_error(
            "MySQL migration order is incomplete or duplicated",
        ));
    }
    if plan.migrations.is_empty() {
        return if plan.baseline_fingerprints.get(&plan.head) == Some(&plan.target_fingerprint) {
            Ok(())
        } else {
            Err(plan_error("an empty MySQL plan must target one baseline"))
        };
    }
    if !plan.migrations.contains_key(&plan.head) {
        return Err(plan_error("MySQL migration head is absent from the plan"));
    }
    let mut known = plan.baseline_fingerprints.clone();
    let mut referenced = BTreeSet::new();
    for id in &plan.topological_order {
        let node = plan
            .migrations
            .get(id)
            .ok_or_else(|| plan_error("MySQL migration order contains an unknown node"))?;
        if node.id != *id
            || !valid_id(id)
            || !valid_fingerprint(&node.checksum)
            || node.author.trim().is_empty()
            || node.created_at.trim().is_empty()
            || node.provider.family != "mysql"
            || node.parents.is_empty()
            || node.paths.keys().collect::<BTreeSet<_>>()
                != node.parents.iter().collect::<BTreeSet<_>>()
        {
            return Err(plan_error(format!(
                "MySQL migration '{id}' metadata is invalid"
            )));
        }
        let mut outputs = BTreeSet::new();
        for (parent, path) in &node.paths {
            if known.get(parent) != Some(&path.input_fingerprint)
                || path.parent != *parent
                || path.steps.is_empty()
                || !valid_fingerprint(&path.output_fingerprint)
            {
                return Err(plan_error(format!(
                    "MySQL migration '{id}' path is invalid"
                )));
            }
            if plan.migrations.contains_key(parent) {
                referenced.insert(parent.clone());
            }
            outputs.insert(path.output_fingerprint.clone());
        }
        if outputs.len() != 1 {
            return Err(plan_error(format!(
                "MySQL migration '{id}' paths have different outputs"
            )));
        }
        if let Some(output) = outputs.first() {
            known.insert(id.clone(), output.clone());
        }
    }
    let terminals = plan
        .migrations
        .keys()
        .filter(|id| !referenced.contains(*id))
        .cloned()
        .collect::<BTreeSet<_>>();
    if terminals != BTreeSet::from([plan.head.clone()])
        || known.get(&plan.head) != Some(&plan.target_fingerprint)
    {
        return Err(plan_error("MySQL migration plan has no exact target head"));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_sequence(
    requirement: MigrationTransactionRequirement,
    migration: &MigrationId,
    parent: &MigrationId,
    direction: MigrationDirection,
    input: &str,
    output: &str,
    steps: &[MigrationExecutionStep],
    actions: &mut Vec<MysqlMigrationAction>,
) -> Result<(), MysqlMigrationPlanError> {
    let mut fingerprint = input;
    let mut identities = BTreeSet::new();
    let mut transaction_open = false;
    let mut transaction_count = 0_u32;
    let mut recovery_point = None;
    let mut contains_ddl = false;
    for step in steps {
        if !valid_id(&step.id)
            || !identities.insert(step.id.clone())
            || !valid_fingerprint(&step.checksum)
            || step.input_fingerprint != fingerprint
            || !valid_fingerprint(&step.output_fingerprint)
        {
            return Err(plan_error("MySQL migration step sequence is invalid"));
        }
        fingerprint = &step.output_fingerprint;
        let action = match &step.kind {
            MigrationExecutionStepKind::Ddl { .. } => {
                contains_ddl = true;
                if transaction_open || recovery_point.is_none() {
                    return Err(plan_error(
                        "MySQL DDL requires an explicit recovery point outside a transaction",
                    ));
                }
                MysqlMigrationActionKind::Ddl
            }
            MigrationExecutionStepKind::SqlData { .. } => MysqlMigrationActionKind::SqlData,
            MigrationExecutionStepKind::SifrData { .. } => MysqlMigrationActionKind::SifrData,
            MigrationExecutionStepKind::Assertion { .. } => MysqlMigrationActionKind::Assertion,
            MigrationExecutionStepKind::Backfill { .. } => MysqlMigrationActionKind::Backfill,
            MigrationExecutionStepKind::RecoveryPoint { name } => {
                if name.is_empty() || name.len() > 128 {
                    return Err(plan_error("MySQL recovery point name is invalid"));
                }
                recovery_point = Some(name.clone());
                MysqlMigrationActionKind::RecoveryPoint
            }
            MigrationExecutionStepKind::Transaction {
                boundary: MigrationTransactionBoundary::Begin,
            } => {
                if transaction_open || requirement == MigrationTransactionRequirement::Forbidden {
                    return Err(plan_error("invalid MySQL transaction begin boundary"));
                }
                transaction_open = true;
                transaction_count = transaction_count.saturating_add(1);
                MysqlMigrationActionKind::TransactionBegin
            }
            MigrationExecutionStepKind::Transaction {
                boundary: MigrationTransactionBoundary::Commit,
            } => {
                if !transaction_open {
                    return Err(plan_error("invalid MySQL transaction commit boundary"));
                }
                transaction_open = false;
                MysqlMigrationActionKind::TransactionCommit
            }
        };
        actions.push(MysqlMigrationAction {
            migration: migration.clone(),
            parent: parent.clone(),
            direction,
            step: step.id.clone(),
            step_checksum: step.checksum.clone(),
            action,
            transactional: transaction_open,
            recovery_point: recovery_point.clone(),
        });
    }
    if transaction_open || fingerprint != output {
        return Err(plan_error(
            "MySQL migration path does not close at its declared output",
        ));
    }
    if requirement == MigrationTransactionRequirement::Required
        && (contains_ddl || transaction_count != 1)
    {
        return Err(plan_error(
            "transaction-required MySQL paths need one transaction and cannot contain DDL",
        ));
    }
    Ok(())
}

fn valid_id(value: &MigrationId) -> bool {
    let value = value.as_str();
    !value.is_empty()
        && value.len() <= 128
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

fn plan_error(message: impl Into<String>) -> MysqlMigrationPlanError {
    MysqlMigrationPlanError {
        message: message.into(),
    }
}
