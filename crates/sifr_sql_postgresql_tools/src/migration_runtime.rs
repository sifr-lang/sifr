use crate::pull_catalog_from_client;
use rustls::ClientConfig;
use rustls_platform_verifier::ConfigVerifierExt;
use sha2::{Digest, Sha256};
use sifr_sql_contract::{DialectIdentity, ProviderIdentity, schema_fingerprint};
use sifr_sql_postgresql::{PostgresDdlExecutionClass, classify_migration_ddl};
use sifr_sql_runtime::{
    MigrationExecutionPlan, MigrationExecutionStepKind, MigrationId, MigrationLedgerSnapshot,
    MigrationLock, MigrationRuntime, MigrationRuntimeIdentity, MigrationStepRequest,
    MigrationStepResult,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;
use tokio_postgres::config::SslMode;
use tokio_postgres::{Client, Config, NoTls};
use tokio_postgres_rustls::MakeRustlsConnect;

const LEDGER_SCHEMA: &str = "sifr_internal";
const LEDGER_TABLE: &str = "migration_ledger";
const LEDGER_OWNER: &str = "sifr:sql-migration-ledger:v1";

pub struct PostgresMigrationRuntime {
    runtime: Runtime,
    client: Client,
    driver_task: JoinHandle<()>,
    provider: ProviderIdentity,
    dialect: DialectIdentity,
    identity: MigrationRuntimeIdentity,
    ledger_identity: String,
    lock_key: i64,
    bootstrap_lock_key: i64,
    transaction_open: bool,
}

impl Drop for PostgresMigrationRuntime {
    fn drop(&mut self) {
        self.driver_task.abort();
    }
}

impl PostgresMigrationRuntime {
    pub fn import_baseline(
        &mut self,
        plan: &MigrationExecutionPlan,
        baseline: &MigrationId,
    ) -> Result<MigrationLedgerSnapshot, String> {
        let lock = self.acquire_lock(plan)?;
        let result = (|| {
            let existing = self.runtime.block_on(async {
                self.client
                    .query_opt(
                        &format!(
                            "SELECT 1 FROM {LEDGER_SCHEMA}.{LEDGER_TABLE} WHERE identity = $1"
                        ),
                        &[&self.ledger_identity],
                    )
                    .await
                    .map_err(|_| "cannot inspect PostgreSQL migration ledger".to_string())
            })?;
            if existing.is_some() {
                return Err("PostgreSQL migration baseline is already imported".to_string());
            }
            let ledger = imported_ledger(plan, baseline, &self.identity)
                .ok_or_else(|| "migration import baseline is absent from the plan".to_string())?;
            let observed = self.inspect_schema_fingerprint()?;
            if observed != ledger.schema_fingerprint {
                return Err("migration import baseline differs from the live schema".to_string());
            }
            self.store_ledger(&ledger)?;
            Ok(ledger)
        })();
        let release = self.release_lock(lock);
        match (result, release) {
            (Ok(ledger), Ok(())) => Ok(ledger),
            (Err(primary), _) => Err(primary),
            (Ok(_), Err(release)) => Err(release),
        }
    }
}

pub fn connect_migration_runtime(
    connection_url: &str,
    provider: ProviderIdentity,
    dialect: DialectIdentity,
    ledger_identity: impl Into<String>,
) -> Result<PostgresMigrationRuntime, String> {
    let ledger_identity = ledger_identity.into();
    if ledger_identity.is_empty() {
        return Err("migration ledger identity is empty".to_string());
    }
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "cannot initialize PostgreSQL migration I/O".to_string())?;
    let config = connection_url
        .parse::<Config>()
        .map_err(|_| "PostgreSQL migration connection configuration is invalid".to_string())?;
    let (client, driver_task) = runtime.block_on(async {
        if config.get_ssl_mode() == SslMode::Disable {
            let (client, connection) = config
                .connect(NoTls)
                .await
                .map_err(|_| "cannot connect to PostgreSQL migrations".to_string())?;
            let task = tokio::spawn(async move {
                let _result = connection.await;
            });
            Ok::<_, String>((client, task))
        } else {
            let tls = ClientConfig::with_platform_verifier()
                .map_err(|_| "cannot initialize PostgreSQL migration TLS".to_string())?;
            let (client, connection) = config
                .connect(MakeRustlsConnect::new(tls))
                .await
                .map_err(|_| "cannot connect to PostgreSQL migrations with TLS".to_string())?;
            let task = tokio::spawn(async move {
                let _result = connection.await;
            });
            Ok((client, task))
        }
    })?;
    let (server_version, database_oid) = runtime.block_on(async {
        let row = client
            .query_one(
                "SELECT current_setting('server_version_num'), oid::text \
                 FROM pg_database WHERE datname = current_database()",
                &[],
            )
            .await
            .map_err(|_| "cannot read PostgreSQL migration target identity".to_string())?;
        let version = row
            .try_get::<_, String>(0)
            .map_err(|_| "PostgreSQL migration server version is invalid".to_string())?;
        let database_oid = row
            .try_get::<_, String>(1)
            .map_err(|_| "PostgreSQL migration database identity is invalid".to_string())?;
        Ok::<_, String>((version, database_oid))
    })?;
    let major = server_version
        .parse::<u32>()
        .ok()
        .map(|value| value / 10_000)
        .filter(|value| *value >= 10)
        .ok_or_else(|| "PostgreSQL migration server version is invalid".to_string())?;
    let expected_major = dialect
        .server_version
        .split('.')
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or_else(|| "PostgreSQL migration profile version is invalid".to_string())?;
    if major != expected_major || dialect.family != "postgresql" {
        return Err("PostgreSQL migration server does not match the profile".to_string());
    }
    let capabilities = dialect
        .features
        .iter()
        .chain(&dialect.modes)
        .cloned()
        .collect();
    Ok(PostgresMigrationRuntime {
        runtime,
        client,
        driver_task,
        provider,
        dialect,
        identity: MigrationRuntimeIdentity {
            family: "postgresql".to_string(),
            server_version: format!("{major}.0.0"),
            capabilities,
        },
        lock_key: advisory_key(&format!("{database_oid}:{ledger_identity}")),
        bootstrap_lock_key: advisory_key(&format!("{database_oid}:ledger-bootstrap")),
        ledger_identity,
        transaction_open: false,
    })
}

impl MigrationRuntime for PostgresMigrationRuntime {
    fn identity(&mut self) -> Result<MigrationRuntimeIdentity, String> {
        Ok(self.identity.clone())
    }

    fn acquire_lock(&mut self, plan: &MigrationExecutionPlan) -> Result<MigrationLock, String> {
        crate::validate_postgres_migration_plan(plan)
            .map_err(|_| "PostgreSQL migration execution plan is invalid".to_string())?;
        let acquired = self.runtime.block_on(async {
            self.client
                .query_one("SELECT pg_try_advisory_lock($1::bigint)", &[&self.lock_key])
                .await
                .map_err(|_| "cannot acquire PostgreSQL migration lock".to_string())?
                .try_get::<_, bool>(0)
                .map_err(|_| "PostgreSQL migration lock result is invalid".to_string())
        })?;
        if !acquired {
            return Err("another PostgreSQL migration is running".to_string());
        }
        let bootstrap = self.runtime.block_on(async {
            self.client
                .query_one(
                    "SELECT pg_advisory_lock($1::bigint)",
                    &[&self.bootstrap_lock_key],
                )
                .await
                .map(|_| ())
                .map_err(|_| "cannot acquire PostgreSQL ledger bootstrap lock".to_string())
        });
        if let Err(failure) = bootstrap {
            let _release = self.runtime.block_on(async {
                self.client
                    .query_one("SELECT pg_advisory_unlock($1::bigint)", &[&self.lock_key])
                    .await
            });
            return Err(failure);
        }
        let setup = self.runtime.block_on(async {
            let existing = self
                .client
                .query_opt(
                    "SELECT obj_description(oid, 'pg_namespace') FROM pg_namespace WHERE nspname = 'sifr_internal'",
                    &[],
                )
                .await
                .map_err(|_| "cannot inspect PostgreSQL migration ledger namespace".to_string())?;
            if let Some(row) = existing {
                let owner = row
                    .try_get::<_, Option<String>>(0)
                    .map_err(|_| "PostgreSQL migration ledger namespace owner is invalid".to_string())?;
                if owner.as_deref() != Some(LEDGER_OWNER) {
                    return Err("PostgreSQL namespace 'sifr_internal' is reserved and is not owned by Sifr".to_string());
                }
            }
            self.client
                .batch_execute(&format!(
                    "CREATE SCHEMA IF NOT EXISTS {LEDGER_SCHEMA};\
                     COMMENT ON SCHEMA {LEDGER_SCHEMA} IS '{LEDGER_OWNER}';\
                     CREATE TABLE IF NOT EXISTS {LEDGER_SCHEMA}.{LEDGER_TABLE} (\
                       identity text PRIMARY KEY, payload jsonb NOT NULL\
                     );"
                ))
                .await
                .map_err(|_| "cannot initialize PostgreSQL migration ledger".to_string())
        });
        let bootstrap_release = self.runtime.block_on(async {
            self.client
                .query_one(
                    "SELECT pg_advisory_unlock($1::bigint)",
                    &[&self.bootstrap_lock_key],
                )
                .await
                .map_err(|_| "cannot release PostgreSQL ledger bootstrap lock".to_string())?
                .try_get::<_, bool>(0)
                .map_err(|_| "PostgreSQL ledger bootstrap unlock result is invalid".to_string())
                .and_then(|released| {
                    released
                        .then_some(())
                        .ok_or_else(|| "PostgreSQL ledger bootstrap lock was not held".to_string())
                })
        });
        if let Some(failure) = setup.err().or_else(|| bootstrap_release.err()) {
            let _release = self.runtime.block_on(async {
                self.client
                    .query_one("SELECT pg_advisory_unlock($1::bigint)", &[&self.lock_key])
                    .await
            });
            return Err(failure);
        }
        Ok(MigrationLock {
            identity: format!("postgresql-advisory:{:016x}", self.lock_key.cast_unsigned()),
        })
    }

    fn release_lock(&mut self, lock: MigrationLock) -> Result<(), String> {
        let expected = format!("postgresql-advisory:{:016x}", self.lock_key.cast_unsigned());
        if lock.identity != expected {
            return Err("PostgreSQL migration lock identity changed".to_string());
        }
        let released = self.runtime.block_on(async {
            self.client
                .query_one("SELECT pg_advisory_unlock($1::bigint)", &[&self.lock_key])
                .await
                .map_err(|_| "cannot release PostgreSQL migration lock".to_string())?
                .try_get::<_, bool>(0)
                .map_err(|_| "PostgreSQL migration unlock result is invalid".to_string())
        })?;
        if released {
            Ok(())
        } else {
            Err("PostgreSQL migration lock was not held".to_string())
        }
    }

    fn load_ledger(&mut self) -> Result<MigrationLedgerSnapshot, String> {
        let payload = self.runtime.block_on(async {
            self.client
                .query_opt(
                    &format!(
                        "SELECT payload::text FROM {LEDGER_SCHEMA}.{LEDGER_TABLE} WHERE identity = $1"
                    ),
                    &[&self.ledger_identity],
                )
                .await
                .map_err(|_| "cannot load PostgreSQL migration ledger".to_string())?
                .map(|row| {
                    row.try_get::<_, String>(0)
                        .map_err(|_| "PostgreSQL migration ledger is invalid".to_string())
                })
                .transpose()
        })?;
        payload
            .map(|value| {
                serde_json::from_str(&value)
                    .map_err(|_| "PostgreSQL migration ledger is invalid".to_string())
            })
            .transpose()?
            .ok_or_else(|| "PostgreSQL migration ledger has no imported baseline".to_string())
    }

    fn store_ledger(&mut self, ledger: &MigrationLedgerSnapshot) -> Result<(), String> {
        let payload = serde_json::to_string(ledger)
            .map_err(|_| "cannot serialize PostgreSQL migration ledger".to_string())?;
        self.runtime.block_on(async {
            self.client
                .execute(
                    &format!(
                        "INSERT INTO {LEDGER_SCHEMA}.{LEDGER_TABLE}(identity, payload) \
                         VALUES ($1, ($2::text)::jsonb) \
                         ON CONFLICT (identity) DO UPDATE SET payload = EXCLUDED.payload"
                    ),
                    &[&self.ledger_identity, &payload],
                )
                .await
                .map(|_| ())
                .map_err(|_| "cannot store PostgreSQL migration ledger".to_string())
        })
    }

    fn begin_transaction(&mut self) -> Result<(), String> {
        if self.transaction_open {
            return Err("PostgreSQL migration transaction is already open".to_string());
        }
        self.runtime.block_on(async {
            self.client
                .batch_execute("BEGIN")
                .await
                .map_err(|_| "cannot begin PostgreSQL migration transaction".to_string())
        })?;
        self.transaction_open = true;
        Ok(())
    }

    fn commit_transaction(&mut self) -> Result<(), String> {
        if !self.transaction_open {
            return Err("PostgreSQL migration transaction is not open".to_string());
        }
        self.runtime.block_on(async {
            self.client
                .batch_execute("COMMIT")
                .await
                .map_err(|_| "cannot commit PostgreSQL migration transaction".to_string())
        })?;
        self.transaction_open = false;
        Ok(())
    }

    fn rollback_transaction(&mut self) -> Result<(), String> {
        if !self.transaction_open {
            return Ok(());
        }
        self.runtime.block_on(async {
            self.client
                .batch_execute("ROLLBACK")
                .await
                .map_err(|_| "cannot roll back PostgreSQL migration transaction".to_string())
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
                if self.transaction_open
                    && matches!(
                        classify_migration_ddl(statement),
                        PostgresDdlExecutionClass::RequiresAutocommit { .. }
                    )
                {
                    return Err(
                        "non-transactional PostgreSQL DDL ran inside a transaction".to_string()
                    );
                }
                self.runtime.block_on(async {
                    self.client
                        .batch_execute(statement)
                        .await
                        .map_err(|_| "PostgreSQL migration DDL failed".to_string())
                })?;
                StepOutcome::Completed
            }
            MigrationExecutionStepKind::SqlData { statement, .. } => {
                self.runtime.block_on(async {
                    self.client
                        .batch_execute(statement)
                        .await
                        .map_err(|_| "PostgreSQL migration data step failed".to_string())
                })?;
                StepOutcome::Completed
            }
            MigrationExecutionStepKind::SifrData { .. } => {
                return Err("PostgreSQL migration callback executor is unavailable".to_string());
            }
            MigrationExecutionStepKind::Assertion { statement, .. } => {
                let rows = self.runtime.block_on(async {
                    self.client
                        .query(statement, &[])
                        .await
                        .map_err(|_| "PostgreSQL migration assertion failed".to_string())
                })?;
                let valid = if rows.len() == 1 {
                    rows[0]
                        .try_get::<_, Option<bool>>(0)
                        .map_err(|_| "PostgreSQL migration assertion is not Boolean".to_string())?
                } else {
                    None
                };
                StepOutcome::Assertion {
                    rows: rows.len() as u64,
                    valid,
                }
            }
            MigrationExecutionStepKind::Backfill {
                statement,
                maximum_batch_rows,
                ..
            } => {
                let processed_rows = self.runtime.block_on(async {
                    self.client
                        .execute(statement, &[])
                        .await
                        .map_err(|_| "PostgreSQL migration backfill failed".to_string())
                })?;
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
                return Err("PostgreSQL received an internal migration step".to_string());
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
            .block_on(pull_catalog_from_client(
                &self.client,
                self.provider.clone(),
                self.dialect.clone(),
                !self.transaction_open,
            ))
            .map_err(|_| "cannot inspect PostgreSQL migration schema".to_string())?;
        schema_fingerprint(&schema)
            .map(|value| value.as_str().to_string())
            .map_err(|_| "cannot fingerprint PostgreSQL migration schema".to_string())
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

fn advisory_key(identity: &str) -> i64 {
    let digest = Sha256::digest(format!("sifr-postgresql-migration:{identity}").as_bytes());
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    i64::from_be_bytes(bytes)
}

fn imported_ledger(
    plan: &MigrationExecutionPlan,
    baseline: &sifr_sql_runtime::MigrationId,
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
