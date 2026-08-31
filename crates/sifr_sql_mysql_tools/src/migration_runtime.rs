use crate::{lower_hex, pull_live_catalog_from_connection};
use mysql_async::{Conn, Opts, prelude::Queryable};
use sha2::{Digest, Sha256};
use sifr_sql_contract::{DialectIdentity, ProviderIdentity, schema_fingerprint};
use sifr_sql_runtime::{
    MigrationExecutionPlan, MigrationExecutionStepKind, MigrationId, MigrationLedgerSnapshot,
    MigrationLock, MigrationRuntime, MigrationRuntimeIdentity, MigrationStepRequest,
    MigrationStepResult,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;
use tokio::runtime::{Builder, Runtime};

const LEDGER_TABLE: &str = "sifr_migration_ledger";

pub struct MysqlMigrationRuntime {
    runtime: Runtime,
    connection: Conn,
    provider: ProviderIdentity,
    dialect: DialectIdentity,
    identity: MigrationRuntimeIdentity,
    ledger_identity: String,
    lock_name: String,
    transaction_open: bool,
}

impl MysqlMigrationRuntime {
    pub fn import_baseline(
        &mut self,
        plan: &MigrationExecutionPlan,
        baseline: &MigrationId,
    ) -> Result<MigrationLedgerSnapshot, String> {
        let lock = self.acquire_lock(plan)?;
        let result = (|| {
            if self.ledger_payload()?.is_some() {
                return Err("MySQL migration baseline is already imported".to_string());
            }
            let ledger = imported_ledger(plan, baseline, &self.identity)
                .ok_or_else(|| "migration import baseline is absent from the plan".to_string())?;
            if self.inspect_schema_fingerprint()? != ledger.schema_fingerprint {
                return Err("migration import baseline differs from the live schema".to_string());
            }
            self.store_ledger(&ledger)?;
            Ok(ledger)
        })();
        let release = self.release_lock(lock);
        match (result, release) {
            (Ok(ledger), Ok(())) => Ok(ledger),
            (Err(primary), _) => Err(primary),
            (Ok(_), Err(error)) => Err(error),
        }
    }

    fn ledger_payload(&mut self) -> Result<Option<String>, String> {
        let identity = self.ledger_identity.clone();
        self.runtime.block_on(async {
            self.connection
                .exec_first::<String, _, _>(
                    format!("SELECT payload FROM `{LEDGER_TABLE}` WHERE identity = ?"),
                    (identity,),
                )
                .await
                .map_err(|_| "cannot load MySQL migration ledger".to_string())
        })
    }
}

pub fn connect_migration_runtime(
    connection_url: &str,
    provider: ProviderIdentity,
    dialect: DialectIdentity,
    ledger_identity: impl Into<String>,
) -> Result<MysqlMigrationRuntime, String> {
    let ledger_identity = ledger_identity.into();
    if ledger_identity.is_empty() || dialect.family != "mysql" {
        return Err("MySQL migration profile identity is invalid".to_string());
    }
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "cannot initialize MySQL migration I/O".to_string())?;
    let opts = Opts::from_url(connection_url)
        .map_err(|_| "MySQL migration connection configuration is invalid".to_string())?;
    let mut connection = runtime
        .block_on(Conn::new(opts))
        .map_err(|_| "cannot connect to MySQL migrations".to_string())?;
    let metadata: Option<(String, String)> = runtime
        .block_on(async { connection.query_first("SELECT VERSION(), DATABASE()").await })
        .map_err(|_| "cannot read MySQL migration target identity".to_string())?;
    let Some((version, database)) = metadata else {
        return Err("MySQL migration target identity is incomplete".to_string());
    };
    let numeric_version = version
        .split('-')
        .next()
        .ok_or_else(|| "MySQL migration server version is invalid".to_string())?;
    let mut version_parts = numeric_version.split('.');
    let major = version_parts.next();
    let minor = version_parts.next();
    let patch = version_parts.next();
    let observed_series = major
        .zip(minor)
        .map(|(major, minor)| format!("{major}.{minor}"))
        .ok_or_else(|| "MySQL migration server version is invalid".to_string())?;
    let server_version = major
        .zip(minor)
        .zip(patch)
        .map(|((major, minor), patch)| format!("{major}.{minor}.{patch}"))
        .ok_or_else(|| "MySQL migration server version is invalid".to_string())?;
    if observed_series != dialect.server_version {
        return Err("MySQL migration server does not match the profile".to_string());
    }
    let capabilities = dialect
        .features
        .iter()
        .chain(&dialect.modes)
        .cloned()
        .collect();
    let lock_name = migration_lock_name(&format!("{database}:{ledger_identity}"));
    Ok(MysqlMigrationRuntime {
        runtime,
        connection,
        provider,
        dialect,
        identity: MigrationRuntimeIdentity {
            family: "mysql".to_string(),
            server_version,
            capabilities,
        },
        ledger_identity,
        lock_name,
        transaction_open: false,
    })
}

impl MigrationRuntime for MysqlMigrationRuntime {
    fn identity(&mut self) -> Result<MigrationRuntimeIdentity, String> {
        Ok(self.identity.clone())
    }

    fn acquire_lock(&mut self, plan: &MigrationExecutionPlan) -> Result<MigrationLock, String> {
        crate::validate_mysql_migration_plan(plan)
            .map_err(|_| "MySQL migration execution plan is invalid".to_string())?;
        let acquired: Option<u8> = self
            .runtime
            .block_on(async {
                self.connection
                    .exec_first("SELECT GET_LOCK(?, 0)", (&self.lock_name,))
                    .await
            })
            .map_err(|_| "cannot acquire MySQL migration lock".to_string())?;
        if acquired != Some(1) {
            return Err("another MySQL migration is running".to_string());
        }
        let setup = self.runtime.block_on(async {
            self.connection
                .query_drop(format!(
                    "CREATE TABLE IF NOT EXISTS `{LEDGER_TABLE}` (\
                     identity VARCHAR(191) PRIMARY KEY, payload JSON NOT NULL)"
                ))
                .await
        });
        if setup.is_err() {
            let _release: Result<Option<u8>, _> = self.runtime.block_on(async {
                self.connection
                    .exec_first("SELECT RELEASE_LOCK(?)", (&self.lock_name,))
                    .await
            });
            return Err("cannot initialize MySQL migration ledger".to_string());
        }
        Ok(MigrationLock {
            identity: self.lock_name.clone(),
        })
    }

    fn release_lock(&mut self, lock: MigrationLock) -> Result<(), String> {
        if lock.identity != self.lock_name {
            return Err("MySQL migration lock identity is invalid".to_string());
        }
        let released: Option<u8> = self
            .runtime
            .block_on(async {
                self.connection
                    .exec_first("SELECT RELEASE_LOCK(?)", (&self.lock_name,))
                    .await
            })
            .map_err(|_| "cannot release MySQL migration lock".to_string())?;
        if released == Some(1) {
            Ok(())
        } else {
            Err("MySQL migration lock was not held".to_string())
        }
    }

    fn load_ledger(&mut self) -> Result<MigrationLedgerSnapshot, String> {
        self.ledger_payload()?
            .map(|payload| {
                serde_json::from_str(&payload)
                    .map_err(|_| "MySQL migration ledger is invalid".to_string())
            })
            .transpose()?
            .ok_or_else(|| "MySQL migration ledger has no imported baseline".to_string())
    }

    fn store_ledger(&mut self, ledger: &MigrationLedgerSnapshot) -> Result<(), String> {
        let payload = serde_json::to_string(ledger)
            .map_err(|_| "cannot serialize MySQL migration ledger".to_string())?;
        let identity = self.ledger_identity.clone();
        self.runtime.block_on(async {
            self.connection
                .exec_drop(
                    format!(
                        "INSERT INTO `{LEDGER_TABLE}`(identity, payload) VALUES (?, ?) \
                         ON DUPLICATE KEY UPDATE payload = VALUES(payload)"
                    ),
                    (identity, payload),
                )
                .await
                .map_err(|_| "cannot store MySQL migration ledger".to_string())
        })
    }

    fn begin_transaction(&mut self) -> Result<(), String> {
        if self.transaction_open {
            return Err("MySQL migration transaction is already open".to_string());
        }
        self.runtime.block_on(async {
            self.connection
                .query_drop("START TRANSACTION")
                .await
                .map_err(|_| "cannot begin MySQL migration transaction".to_string())
        })?;
        self.transaction_open = true;
        Ok(())
    }

    fn commit_transaction(&mut self) -> Result<(), String> {
        if !self.transaction_open {
            return Err("MySQL migration transaction is not open".to_string());
        }
        self.runtime.block_on(async {
            self.connection
                .query_drop("COMMIT")
                .await
                .map_err(|_| "cannot commit MySQL migration transaction".to_string())
        })?;
        self.transaction_open = false;
        Ok(())
    }

    fn rollback_transaction(&mut self) -> Result<(), String> {
        if !self.transaction_open {
            return Ok(());
        }
        self.runtime.block_on(async {
            self.connection
                .query_drop("ROLLBACK")
                .await
                .map_err(|_| "cannot roll back MySQL migration transaction".to_string())
        })?;
        self.transaction_open = false;
        Ok(())
    }

    fn execute_step(
        &mut self,
        request: MigrationStepRequest<'_>,
    ) -> Result<MigrationStepResult, String> {
        let started = Instant::now();
        let outcome = match &request.step.kind {
            MigrationExecutionStepKind::Ddl { statement } => {
                if self.transaction_open {
                    return Err("MySQL DDL cannot run inside a migration transaction".to_string());
                }
                self.run_drop(statement, "MySQL migration DDL failed")?;
                StepOutcome::Completed
            }
            MigrationExecutionStepKind::SqlData {
                normalized_statement,
            } => {
                self.run_drop(normalized_statement, "MySQL migration data step failed")?;
                StepOutcome::Completed
            }
            MigrationExecutionStepKind::SifrData { .. } => {
                return Err("MySQL migration callback executor is unavailable".to_string());
            }
            MigrationExecutionStepKind::Assertion {
                normalized_statement,
            } => {
                let rows: Vec<(Option<bool>,)> = self
                    .runtime
                    .block_on(async { self.connection.query(normalized_statement).await })
                    .map_err(|_| "MySQL migration assertion failed".to_string())?;
                StepOutcome::Assertion {
                    rows: rows.len() as u64,
                    valid: (rows.len() == 1).then(|| rows[0].0).flatten(),
                }
            }
            MigrationExecutionStepKind::Backfill {
                normalized_statement,
                maximum_batch_rows,
                ..
            } => {
                self.run_drop(normalized_statement, "MySQL migration backfill failed")?;
                let processed_rows = self.connection.affected_rows();
                let prior = request
                    .backfill_progress
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or(0);
                StepOutcome::Backfill {
                    processed_rows,
                    progress: prior
                        .checked_add(processed_rows)
                        .map(|value| value.to_string()),
                    complete: processed_rows < *maximum_batch_rows,
                }
            }
            MigrationExecutionStepKind::Transaction { .. }
            | MigrationExecutionStepKind::RecoveryPoint { .. } => {
                return Err("MySQL received an internal migration step".to_string());
            }
        };
        let fingerprint = self.inspect_schema_fingerprint()?;
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
    }

    fn inspect_schema_fingerprint(&mut self) -> Result<String, String> {
        let schema = self
            .runtime
            .block_on(pull_live_catalog_from_connection(
                &mut self.connection,
                self.provider.clone(),
                self.dialect.clone(),
            ))
            .map_err(|_| "cannot inspect MySQL migration schema".to_string())?;
        schema_fingerprint(&schema)
            .map(|value| value.as_str().to_string())
            .map_err(|_| "cannot fingerprint MySQL migration schema".to_string())
    }
}

impl MysqlMigrationRuntime {
    fn run_drop(&mut self, statement: &str, message: &str) -> Result<(), String> {
        self.runtime.block_on(async {
            self.connection
                .query_drop(statement)
                .await
                .map_err(|_| message.to_string())
        })
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

fn migration_lock_name(identity: &str) -> String {
    let digest = Sha256::digest(format!("sifr-mysql-migration:{identity}").as_bytes());
    let suffix = lower_hex(&digest[..20]);
    format!("sifr-migration-{suffix}")
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
