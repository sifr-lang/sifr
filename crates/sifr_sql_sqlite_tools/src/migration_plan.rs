use sifr_sql_runtime::{
    MIGRATION_EXECUTION_PLAN_FORMAT_VERSION, MigrationExecutionPlan, MigrationExecutionStep,
    MigrationExecutionStepKind, MigrationId, MigrationTransactionBoundary,
};
use sifr_sql_sqlite::{SUPPORTED_SQLITE_SERIES, SqliteParser, Token, tokenize};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqliteMigrationPlanError {
    pub message: String,
}

impl fmt::Display for SqliteMigrationPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SqliteMigrationPlanError {}

pub fn validate_sqlite_execution_plan(
    plan: &MigrationExecutionPlan,
) -> Result<(), SqliteMigrationPlanError> {
    if plan.format_version != MIGRATION_EXECUTION_PLAN_FORMAT_VERSION
        || plan.provider_family != "sqlite"
        || !fingerprint(&plan.target_fingerprint)
        || plan.baseline_fingerprints.is_empty()
        || plan
            .baseline_fingerprints
            .iter()
            .any(|(id, value)| !migration_id(id) || !fingerprint(value))
    {
        return Err(error("SQLite migration graph metadata is invalid"));
    }
    let ordered = plan
        .topological_order
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if ordered.len() != plan.topological_order.len()
        || ordered != plan.migrations.keys().cloned().collect()
    {
        return Err(error("SQLite migration graph order is not closed"));
    }
    if plan.migrations.is_empty() {
        return (plan.baseline_fingerprints.get(&plan.head) == Some(&plan.target_fingerprint))
            .then_some(())
            .ok_or_else(|| error("empty SQLite migration graph has no exact target baseline"));
    }
    let mut known = plan.baseline_fingerprints.clone();
    let mut referenced = BTreeSet::new();
    for id in &plan.topological_order {
        let node = plan
            .migrations
            .get(id)
            .ok_or_else(|| error("SQLite migration order contains an unknown node"))?;
        if node.id != *id
            || !migration_id(id)
            || !fingerprint(&node.checksum)
            || node.provider.family != "sqlite"
            || node.parents.is_empty()
            || node.paths.keys().collect::<BTreeSet<_>>()
                != node.parents.iter().collect::<BTreeSet<_>>()
        {
            return Err(error(format!(
                "SQLite migration '{id}' metadata is invalid"
            )));
        }
        let mut outputs = BTreeSet::new();
        for (parent, path) in &node.paths {
            if path.parent != *parent
                || known.get(parent) != Some(&path.input_fingerprint)
                || path.steps.is_empty()
                || !fingerprint(&path.output_fingerprint)
            {
                return Err(error(format!("SQLite migration '{id}' path is invalid")));
            }
            validate_step_sequence(
                &path.input_fingerprint,
                &path.output_fingerprint,
                &path.steps,
            )?;
            if let Some(rollback) = &path.rollback {
                validate_step_sequence(
                    &path.output_fingerprint,
                    &path.input_fingerprint,
                    rollback,
                )?;
            }
            if plan.migrations.contains_key(parent) {
                referenced.insert(parent.clone());
            }
            outputs.insert(path.output_fingerprint.clone());
        }
        if outputs.len() != 1 {
            return Err(error(format!(
                "SQLite migration '{id}' paths have different outputs"
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
        return Err(error("SQLite migration graph has no exact target head"));
    }
    Ok(())
}

fn validate_step_sequence(
    input: &str,
    output: &str,
    steps: &[MigrationExecutionStep],
) -> Result<(), SqliteMigrationPlanError> {
    let mut current = input;
    let mut identities = BTreeSet::new();
    let mut transaction_open = false;
    for step in steps {
        if !migration_id(&step.id)
            || !identities.insert(step.id.clone())
            || !fingerprint(&step.checksum)
            || step.input_fingerprint != current
            || !fingerprint(&step.output_fingerprint)
        {
            return Err(error("SQLite migration step sequence is invalid"));
        }
        current = &step.output_fingerprint;
        match &step.kind {
            MigrationExecutionStepKind::Ddl { statement } => {
                validate_owned_statement(statement)?;
            }
            MigrationExecutionStepKind::SqlData {
                statement,
                normalized_statement,
            }
            | MigrationExecutionStepKind::Assertion {
                statement,
                normalized_statement,
            }
            | MigrationExecutionStepKind::Backfill {
                statement,
                normalized_statement,
                ..
            } => {
                if statement.trim().is_empty() || normalized_statement.trim().is_empty() {
                    return Err(error("SQLite migration SQL step is empty"));
                }
                validate_owned_statement(statement)?;
            }
            MigrationExecutionStepKind::SifrData { callback } => {
                if callback.trim().is_empty() {
                    return Err(error("SQLite migration callback identity is empty"));
                }
            }
            MigrationExecutionStepKind::RecoveryPoint { name } => {
                if !identifier(name) {
                    return Err(error("SQLite migration recovery point is invalid"));
                }
            }
            MigrationExecutionStepKind::Transaction {
                boundary: MigrationTransactionBoundary::Begin,
            } => {
                if transaction_open {
                    return Err(error("SQLite migration transaction is nested"));
                }
                transaction_open = true;
            }
            MigrationExecutionStepKind::Transaction {
                boundary: MigrationTransactionBoundary::Commit,
            } => {
                if !transaction_open {
                    return Err(error("SQLite migration transaction commit has no begin"));
                }
                transaction_open = false;
            }
        }
    }
    if transaction_open || current != output {
        return Err(error(
            "SQLite migration step sequence does not reach its declared output",
        ));
    }
    Ok(())
}

fn validate_owned_statement(sql: &str) -> Result<(), SqliteMigrationPlanError> {
    let parser = SqliteParser::new(SUPPORTED_SQLITE_SERIES[0], BTreeSet::<String>::new())
        .map_err(|failure| error(failure.to_string()))?;
    let statements = parser
        .parse(sql)
        .map_err(|failure| error(failure.to_string()))?;
    let words = tokenize(sql)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|failure| error(failure.to_string()))?
        .into_iter()
        .filter_map(|(_, token, _)| match token {
            Token::Identifier(value) | Token::QuotedIdentifier(value) => {
                Some(value.to_ascii_uppercase())
            }
            Token::Keyword(keyword) => Some(keyword.text().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if statements.len() != 1
        || words
            .iter()
            .any(|word| matches!(word.as_str(), "ATTACH" | "DETACH" | "WRITABLE_SCHEMA"))
        || words.windows(2).any(|pair| pair == ["VACUUM", "INTO"])
    {
        return Err(error("SQLite migration statement changes an unowned scope"));
    }
    Ok(())
}

fn identifier(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
        && value.len() <= 128
}

fn migration_id(value: &MigrationId) -> bool {
    let value = value.as_str();
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn error(message: impl Into<String>) -> SqliteMigrationPlanError {
    SqliteMigrationPlanError {
        message: message.into(),
    }
}
