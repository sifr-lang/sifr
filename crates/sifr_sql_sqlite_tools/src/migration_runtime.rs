use crate::{lower_hex, pull_live_catalog_from_connection, validate_sqlite_execution_plan};
use rusqlite::{Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use sifr_sql_contract::{DialectIdentity, ProviderIdentity, schema_fingerprint};
use sifr_sql_runtime::{
    MigrationExecutionPlan, MigrationExecutionStepKind, MigrationId, MigrationLedgerSnapshot,
    MigrationLock, MigrationRuntime, MigrationRuntimeIdentity, MigrationStepRequest,
    MigrationStepResult,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Instant;

const LEDGER_TABLE: &str = "sifr_migration_ledger";
const STEP_SAVEPOINT: &str = "sifr_migration_step";

pub struct SqliteMigrationRuntime {
    connection: Connection,
    provider: ProviderIdentity,
    dialect: DialectIdentity,
    identity: MigrationRuntimeIdentity,
    ledger_identity: String,
    lock_identity: String,
    lock_held: bool,
    transaction_open: bool,
    failed: bool,
}

pub fn connect_migration_runtime(
    path: &Path,
    provider: ProviderIdentity,
    dialect: DialectIdentity,
    ledger_identity: impl Into<String>,
) -> Result<SqliteMigrationRuntime, String> {
    let ledger_identity = ledger_identity.into();
    if ledger_identity.is_empty()
        || dialect.family != "sqlite"
        || dialect.server_version != "3.53.2"
    {
        return Err("SQLite migration profile identity is invalid".to_string());
    }
    let connection =
        Connection::open(path).map_err(|_| "cannot open SQLite migration database".to_string())?;
    connection
        .busy_timeout(std::time::Duration::from_millis(250))
        .map_err(|_| "cannot configure SQLite migration lock timeout".to_string())?;
    connection
        .pragma_update(None, "trusted_schema", false)
        .map_err(|_| "cannot secure SQLite migrations".to_string())?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(|_| "cannot enable SQLite foreign keys".to_string())?;
    if rusqlite::version_number() != 3_053_002 {
        return Err("SQLite migration library does not match the profile".to_string());
    }
    let capabilities = dialect
        .features
        .iter()
        .chain(&dialect.modes)
        .cloned()
        .collect();
    let lock_identity = migration_lock_name(path, &ledger_identity);
    Ok(SqliteMigrationRuntime {
        connection,
        provider,
        dialect,
        identity: MigrationRuntimeIdentity {
            family: "sqlite".to_string(),
            server_version: rusqlite::version().to_string(),
            capabilities,
        },
        ledger_identity,
        lock_identity,
        lock_held: false,
        transaction_open: false,
        failed: false,
    })
}

impl SqliteMigrationRuntime {
    pub fn import_baseline(
        &mut self,
        plan: &MigrationExecutionPlan,
        baseline: &MigrationId,
    ) -> Result<MigrationLedgerSnapshot, String> {
        let lock = self.acquire_lock(plan)?;
        let result = (|| {
            if self.ledger_payload()?.is_some() {
                return Err("SQLite migration baseline is already imported".to_string());
            }
            let ledger = imported_ledger(plan, baseline, &self.identity)
                .ok_or_else(|| "migration import baseline is absent from the plan".to_string())?;
            if self.inspect_schema_fingerprint()? != ledger.schema_fingerprint {
                return Err("migration import baseline differs from the live schema".to_string());
            }
            self.store_ledger(&ledger)?;
            Ok(ledger)
        })();
        if result.is_err() {
            self.failed = true;
        }
        let release = self.release_lock(lock);
        match (result, release) {
            (Ok(ledger), Ok(())) => Ok(ledger),
            (Err(primary), _) => Err(primary),
            (Ok(_), Err(error)) => Err(error),
        }
    }

    fn ledger_payload(&self) -> Result<Option<String>, String> {
        self.connection
            .query_row(
                &format!("SELECT payload FROM {LEDGER_TABLE} WHERE identity = ?1"),
                [&self.ledger_identity],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "cannot load SQLite migration ledger".to_string())
    }

    fn execute_sql(&mut self, statement: &str, message: &str) -> Result<(), String> {
        if self.connection.execute_batch(statement).is_err() {
            self.failed = true;
            return Err(message.to_string());
        }
        Ok(())
    }
}

impl MigrationRuntime for SqliteMigrationRuntime {
    fn identity(&mut self) -> Result<MigrationRuntimeIdentity, String> {
        Ok(self.identity.clone())
    }

    fn acquire_lock(&mut self, plan: &MigrationExecutionPlan) -> Result<MigrationLock, String> {
        validate_sqlite_execution_plan(plan)
            .map_err(|_| "SQLite migration execution plan is invalid".to_string())?;
        if self.lock_held {
            return Err("SQLite migration lock is already held".to_string());
        }
        self.connection
            .pragma_update(None, "foreign_keys", false)
            .map_err(|_| "cannot suspend SQLite foreign keys".to_string())?;
        if self.connection.execute_batch("BEGIN IMMEDIATE").is_err() {
            let _restore = self.connection.pragma_update(None, "foreign_keys", true);
            return Err("another SQLite migration is running".to_string());
        }
        self.lock_held = true;
        self.failed = false;
        if self
            .connection
            .execute_batch(&format!(
                "CREATE TABLE IF NOT EXISTS {LEDGER_TABLE} (\
                 identity TEXT PRIMARY KEY NOT NULL, payload TEXT NOT NULL)"
            ))
            .is_err()
        {
            self.failed = true;
            let _release = self.release_lock(MigrationLock {
                identity: self.lock_identity.clone(),
            });
            return Err("cannot initialize SQLite migration ledger".to_string());
        }
        Ok(MigrationLock {
            identity: self.lock_identity.clone(),
        })
    }

    fn release_lock(&mut self, lock: MigrationLock) -> Result<(), String> {
        if !self.lock_held || lock.identity != self.lock_identity {
            return Err("SQLite migration lock identity is invalid".to_string());
        }
        if self.transaction_open {
            let _rollback = self.connection.execute_batch(&format!(
                "ROLLBACK TO {STEP_SAVEPOINT}; RELEASE {STEP_SAVEPOINT}"
            ));
            self.transaction_open = false;
            self.failed = true;
        }
        let mut verification_error = None;
        if !self.failed {
            match self.connection.query_row(
                "SELECT count(*) FROM pragma_foreign_key_check",
                [],
                |row| row.get::<_, i64>(0),
            ) {
                Ok(0) => {}
                Ok(_) => {
                    self.failed = true;
                    verification_error =
                        Some("SQLite migration produced foreign-key violations".to_string());
                }
                Err(_) => {
                    self.failed = true;
                    verification_error = Some("cannot verify SQLite foreign keys".to_string());
                }
            }
        }
        let statement = if self.failed { "ROLLBACK" } else { "COMMIT" };
        let result = self
            .connection
            .execute_batch(statement)
            .map_err(|_| format!("cannot {} SQLite migration", statement.to_ascii_lowercase()));
        self.lock_held = false;
        let restore = self
            .connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(|_| "cannot restore SQLite foreign keys".to_string());
        result.and(restore)?;
        verification_error.map_or(Ok(()), Err)
    }

    fn load_ledger(&mut self) -> Result<MigrationLedgerSnapshot, String> {
        self.ledger_payload()?
            .map(|payload| {
                serde_json::from_str(&payload)
                    .map_err(|_| "SQLite migration ledger is invalid".to_string())
            })
            .transpose()?
            .ok_or_else(|| "SQLite migration ledger has no imported baseline".to_string())
    }

    fn store_ledger(&mut self, ledger: &MigrationLedgerSnapshot) -> Result<(), String> {
        let payload = serde_json::to_string(ledger)
            .map_err(|_| "cannot serialize SQLite migration ledger".to_string())?;
        if self
            .connection
            .execute(
                &format!(
                    "INSERT INTO {LEDGER_TABLE}(identity, payload) VALUES (?1, ?2) \
                     ON CONFLICT(identity) DO UPDATE SET payload = excluded.payload"
                ),
                (&self.ledger_identity, payload),
            )
            .is_err()
        {
            self.failed = true;
            return Err("cannot store SQLite migration ledger".to_string());
        }
        Ok(())
    }

    fn begin_transaction(&mut self) -> Result<(), String> {
        if self.transaction_open {
            return Err("SQLite migration transaction is already open".to_string());
        }
        self.execute_sql(
            &format!("SAVEPOINT {STEP_SAVEPOINT}"),
            "cannot begin SQLite migration transaction",
        )?;
        self.transaction_open = true;
        Ok(())
    }

    fn commit_transaction(&mut self) -> Result<(), String> {
        if !self.transaction_open {
            return Err("SQLite migration transaction is not open".to_string());
        }
        self.execute_sql(
            &format!("RELEASE {STEP_SAVEPOINT}"),
            "cannot commit SQLite migration transaction",
        )?;
        self.transaction_open = false;
        Ok(())
    }

    fn rollback_transaction(&mut self) -> Result<(), String> {
        if !self.transaction_open {
            return Ok(());
        }
        let result = self.execute_sql(
            &format!("ROLLBACK TO {STEP_SAVEPOINT}; RELEASE {STEP_SAVEPOINT}"),
            "cannot roll back SQLite migration transaction",
        );
        self.transaction_open = false;
        result
    }

    fn execute_step(
        &mut self,
        request: MigrationStepRequest<'_>,
    ) -> Result<MigrationStepResult, String> {
        let result = (|| {
            let started = Instant::now();
            let outcome = match &request.step.kind {
                MigrationExecutionStepKind::Ddl { statement } => {
                    self.execute_sql(statement, "SQLite migration DDL failed")?;
                    StepOutcome::Completed
                }
                MigrationExecutionStepKind::SqlData {
                    normalized_statement,
                } => {
                    self.execute_sql(normalized_statement, "SQLite migration data step failed")?;
                    StepOutcome::Completed
                }
                MigrationExecutionStepKind::SifrData { .. } => {
                    self.failed = true;
                    return Err("SQLite migration callback executor is unavailable".to_string());
                }
                MigrationExecutionStepKind::Assertion {
                    normalized_statement,
                } => {
                    let mut statement = self
                        .connection
                        .prepare(normalized_statement)
                        .map_err(|_| "SQLite migration assertion failed".to_string())?;
                    let values = statement
                        .query_map([], |row| row.get::<_, Option<bool>>(0))
                        .and_then(Iterator::collect::<Result<Vec<_>, _>>)
                        .map_err(|_| "SQLite migration assertion failed".to_string())?;
                    if values.len() != 1 || values[0] != Some(true) {
                        self.failed = true;
                    }
                    StepOutcome::Assertion {
                        rows: values.len() as u64,
                        valid: (values.len() == 1).then_some(values[0]).flatten(),
                    }
                }
                MigrationExecutionStepKind::Backfill {
                    normalized_statement,
                    maximum_batch_rows,
                    ..
                } => {
                    let changed = self
                        .connection
                        .execute(normalized_statement, [])
                        .map_err(|_| "SQLite migration backfill failed".to_string())?
                        as u64;
                    if changed > *maximum_batch_rows {
                        self.failed = true;
                    }
                    let prior = request
                        .backfill_progress
                        .and_then(|value| value.parse::<u64>().ok())
                        .unwrap_or(0);
                    StepOutcome::Backfill {
                        processed_rows: changed,
                        progress: prior.checked_add(changed).map(|value| value.to_string()),
                        complete: changed < *maximum_batch_rows,
                    }
                }
                MigrationExecutionStepKind::Transaction { .. }
                | MigrationExecutionStepKind::RecoveryPoint { .. } => {
                    self.failed = true;
                    return Err("SQLite received an internal migration step".to_string());
                }
            };
            let fingerprint = self.inspect_schema_fingerprint()?;
            if fingerprint != request.step.output_fingerprint {
                return Err(
                    "SQLite migration step produced an unexpected schema fingerprint".into(),
                );
            }
            let duration_millis = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
            Ok(match outcome {
                StepOutcome::Completed => MigrationStepResult::Completed {
                    schema_fingerprint: fingerprint,
                    duration_millis,
                },
                StepOutcome::Assertion { rows, valid } => MigrationStepResult::Assertion {
                    rows,
                    valid,
                    schema_fingerprint: fingerprint,
                    duration_millis,
                },
                StepOutcome::Backfill {
                    processed_rows,
                    progress,
                    complete,
                } => MigrationStepResult::BackfillBatch {
                    processed_rows,
                    progress,
                    complete,
                    schema_fingerprint: fingerprint,
                    duration_millis,
                },
            })
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    fn inspect_schema_fingerprint(&mut self) -> Result<String, String> {
        let schema = pull_live_catalog_from_connection(
            &self.connection,
            self.provider.clone(),
            self.dialect.clone(),
        )
        .map_err(|_| "cannot inspect SQLite migration schema".to_string())?;
        schema_fingerprint(&schema)
            .map(|value| value.as_str().to_string())
            .map_err(|_| "cannot fingerprint SQLite migration schema".to_string())
    }
}

enum StepOutcome {
    Completed,
    Assertion {
        rows: u64,
        valid: Option<bool>,
    },
    Backfill {
        processed_rows: u64,
        progress: Option<String>,
        complete: bool,
    },
}

fn migration_lock_name(path: &Path, identity: &str) -> String {
    let digest =
        Sha256::digest(format!("sifr-sqlite-migration:{}:{identity}", path.display()).as_bytes());
    format!("sifr-migration-{}", lower_hex(&digest[..20]))
}

fn imported_ledger(
    plan: &MigrationExecutionPlan,
    baseline: &MigrationId,
    identity: &MigrationRuntimeIdentity,
) -> Option<MigrationLedgerSnapshot> {
    plan.baseline_fingerprints
        .get(baseline)
        .map(|fingerprint| MigrationLedgerSnapshot {
            provider_family: plan.provider_family.clone(),
            provider_server_version: identity.server_version.clone(),
            provider_capabilities: identity.capabilities.clone(),
            heads: BTreeSet::from([baseline.clone()]),
            schema_fingerprint: fingerprint.clone(),
            applied: BTreeMap::new(),
            in_progress: None,
        })
}
