use crate::migration_plan::{
    SqliteMigrationAction, SqliteMigrationPlan, validate_sqlite_migration_plan,
};
use rusqlite::Connection;
use std::fmt;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqliteMigrationError {
    pub message: String,
}

impl fmt::Display for SqliteMigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SqliteMigrationError {}

pub struct SqliteMigrationRuntime {
    connection: Connection,
}

pub fn connect_migration_runtime(
    path: &Path,
) -> Result<SqliteMigrationRuntime, SqliteMigrationError> {
    let connection =
        Connection::open(path).map_err(|_| error("cannot open SQLite migration database"))?;
    connection
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|_| error("cannot configure SQLite migration lock timeout"))?;
    connection
        .pragma_update(None, "trusted_schema", false)
        .map_err(|_| error("cannot secure SQLite migrations"))?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(|_| error("cannot enable SQLite foreign keys"))?;
    Ok(SqliteMigrationRuntime { connection })
}

impl SqliteMigrationRuntime {
    pub fn apply(&mut self, plan: &SqliteMigrationPlan) -> Result<(), SqliteMigrationError> {
        validate_sqlite_migration_plan(plan).map_err(|failure| error(failure.message))?;
        let has_rebuild = plan
            .actions
            .iter()
            .any(|action| matches!(action, SqliteMigrationAction::RebuildTable { .. }));
        if has_rebuild {
            self.connection
                .pragma_update(None, "foreign_keys", false)
                .map_err(|_| error("cannot suspend SQLite foreign keys for table rebuild"))?;
        }
        let applied = self.apply_transaction(plan);
        let restore = self
            .connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(|_| error("cannot restore SQLite foreign-key enforcement"));
        applied.and(restore)
    }

    fn apply_transaction(
        &mut self,
        plan: &SqliteMigrationPlan,
    ) -> Result<(), SqliteMigrationError> {
        let transaction = self
            .connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|_| error("cannot acquire the SQLite migration writer lock"))?;
        for action in &plan.actions {
            match action {
                SqliteMigrationAction::Statement { sql } => {
                    transaction
                        .execute_batch(sql)
                        .map_err(|_| error("SQLite migration statement failed"))?;
                }
                SqliteMigrationAction::RebuildTable {
                    table,
                    temporary_table,
                    create_temporary_sql,
                    target_columns,
                    source_expressions,
                    recreate_sql,
                } => {
                    transaction
                        .execute_batch(create_temporary_sql)
                        .map_err(|_| error("cannot create SQLite rebuild table"))?;
                    let targets = target_columns
                        .iter()
                        .map(|column| quote_identifier(column))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let sources = source_expressions.join(", ");
                    transaction
                        .execute_batch(&format!(
                            "INSERT INTO {} ({targets}) SELECT {sources} FROM {}",
                            quote_identifier(temporary_table),
                            quote_identifier(table),
                        ))
                        .map_err(|_| error("cannot copy rows during SQLite table rebuild"))?;
                    transaction
                        .execute_batch(&format!("DROP TABLE {}", quote_identifier(table)))
                        .map_err(|_| error("cannot replace the old SQLite table"))?;
                    transaction
                        .execute_batch(&format!(
                            "ALTER TABLE {} RENAME TO {}",
                            quote_identifier(temporary_table),
                            quote_identifier(table),
                        ))
                        .map_err(|_| error("cannot rename the rebuilt SQLite table"))?;
                    for sql in recreate_sql {
                        transaction.execute_batch(sql).map_err(|_| {
                            error("cannot recreate a SQLite index, trigger, or view")
                        })?;
                    }
                }
            }
        }
        let violations: i64 = transaction
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .map_err(|_| error("cannot verify SQLite foreign keys after migration"))?;
        if violations != 0 {
            return Err(error("SQLite migration produced foreign-key violations"));
        }
        transaction
            .commit()
            .map_err(|_| error("cannot commit SQLite migration"))
    }
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn error(message: impl Into<String>) -> SqliteMigrationError {
    SqliteMigrationError {
        message: message.into(),
    }
}
