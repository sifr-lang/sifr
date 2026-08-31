use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SqliteMigrationAction {
    Statement {
        sql: String,
    },
    RebuildTable {
        table: String,
        temporary_table: String,
        create_temporary_sql: String,
        target_columns: Vec<String>,
        source_expressions: Vec<String>,
        recreate_sql: Vec<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteMigrationPlan {
    pub schema_fingerprint_before: String,
    pub schema_fingerprint_after: String,
    pub actions: Vec<SqliteMigrationAction>,
}

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

pub fn validate_sqlite_migration_plan(
    plan: &SqliteMigrationPlan,
) -> Result<(), SqliteMigrationPlanError> {
    if !fingerprint(&plan.schema_fingerprint_before)
        || !fingerprint(&plan.schema_fingerprint_after)
        || plan.actions.is_empty()
    {
        return Err(error(
            "SQLite migration plan has invalid fingerprints or no actions",
        ));
    }
    let mut rebuilt = BTreeSet::new();
    for action in &plan.actions {
        match action {
            SqliteMigrationAction::Statement { sql } => {
                let normalized = sql.trim().to_ascii_uppercase();
                if normalized.is_empty()
                    || normalized.starts_with("ATTACH")
                    || normalized.starts_with("DETACH")
                    || normalized.contains("PRAGMA WRITABLE_SCHEMA")
                    || normalized.contains("VACUUM INTO")
                {
                    return Err(error("SQLite migration statement changes an unowned scope"));
                }
            }
            SqliteMigrationAction::RebuildTable {
                table,
                temporary_table,
                create_temporary_sql,
                target_columns,
                source_expressions,
                recreate_sql,
            } => {
                if !identifier(table)
                    || !temporary_table.starts_with("__sifr_rebuild_")
                    || !identifier(temporary_table)
                    || !rebuilt.insert(table)
                    || target_columns.is_empty()
                    || target_columns.len() != source_expressions.len()
                    || target_columns.iter().any(|column| !identifier(column))
                    || source_expressions
                        .iter()
                        .any(|expression| expression.trim().is_empty())
                    || !create_temporary_sql
                        .trim_start()
                        .to_ascii_uppercase()
                        .starts_with("CREATE TABLE")
                    || recreate_sql.iter().any(|sql| sql.trim().is_empty())
                {
                    return Err(error("SQLite table-rebuild action is invalid or ambiguous"));
                }
            }
        }
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
