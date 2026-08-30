use crate::codec::{RawPostgresValue, decode_value};
use crate::config::{PostgresProfile, PostgresTls};
use crate::error::{map_postgres_error, provider_error};
use sifr_sql_runtime::{
    CancellationCarrier, ExecutionResult, OwnedSqlValue, PoolLease, ProviderLeaseToken,
    SessionSnapshot, SqlError, SqlErrorKind, StatementCache, StatementCacheKey,
    VerificationEvidence,
};
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio_postgres::{CancelToken, Client, NoTls, Statement};
use tokio_postgres_rustls::MakeRustlsConnect;

static NEXT_LEASE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Default)]
pub struct ExecutionOptions {
    pub timeout: Option<Duration>,
    pub cancellation: Option<CancellationCarrier>,
}

impl fmt::Debug for ExecutionOptions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ExecutionOptions")
            .field("timeout", &self.timeout)
            .field(
                "cancellation",
                &self.cancellation.as_ref().map(|_| "<carrier>"),
            )
            .finish()
    }
}

impl ExecutionOptions {
    pub(crate) fn deadline(&self, profile: &PostgresProfile) -> Result<Duration, SqlError> {
        let timeout = self.timeout.unwrap_or(profile.limits.statement_timeout);
        if timeout.is_zero() || timeout > profile.limits.statement_timeout {
            return Err(SqlError::new(SqlErrorKind::Configuration));
        }
        Ok(timeout)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostgresMetadata {
    pub statement_cache_hit: bool,
    pub server_version: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PostgresRow {
    pub(crate) values: Vec<OwnedSqlValue>,
    decoded_bytes: u64,
}

impl PostgresRow {
    #[must_use]
    pub fn values(&self) -> &[OwnedSqlValue] {
        &self.values
    }

    #[must_use]
    pub const fn decoded_bytes(&self) -> u64 {
        self.decoded_bytes
    }
}

pub(crate) struct PostgresNativeConnection {
    pub(crate) client: Client,
    driver_task: JoinHandle<()>,
    pub(crate) cancel_token: CancelToken,
    pub(crate) statements: StatementCache<Statement>,
    pub(crate) server_version: u32,
    pub(crate) poisoned: Arc<AtomicBool>,
    pub(crate) lease: ProviderLeaseToken,
}

impl Drop for PostgresNativeConnection {
    fn drop(&mut self) {
        self.driver_task.abort();
    }
}

pub struct PostgresConnection {
    pub(crate) lease: Option<PoolLease<PostgresNativeConnection>>,
    pub(crate) profile: Arc<PostgresProfile>,
    pub(crate) evidence: Arc<VerificationEvidence>,
}

impl PostgresConnection {
    pub(crate) fn new(
        lease: PoolLease<PostgresNativeConnection>,
        profile: Arc<PostgresProfile>,
        evidence: Arc<VerificationEvidence>,
    ) -> Self {
        Self {
            lease: Some(lease),
            profile,
            evidence,
        }
    }

    pub async fn begin(
        self,
        options: crate::transaction::TransactionOptions,
    ) -> Result<crate::transaction::PostgresTransaction, SqlError> {
        crate::transaction::PostgresTransaction::begin(self, options).await
    }

    pub(crate) fn native(&self) -> Result<&PostgresNativeConnection, SqlError> {
        self.lease.as_ref().ok_or_else(provider_error)?.resource()
    }

    pub(crate) fn native_mut(&mut self) -> Result<&mut PostgresNativeConnection, SqlError> {
        self.lease
            .as_mut()
            .ok_or_else(provider_error)?
            .resource_mut()
    }

    #[must_use]
    pub fn lease_id(&self) -> &str {
        self.native().map_or("", |native| native.lease.as_str())
    }

    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.native()
            .map_or(true, |native| native.poisoned.load(Ordering::Acquire))
    }

    pub async fn release(
        mut self,
        cancellation: Option<&CancellationCarrier>,
    ) -> Result<(), SqlError> {
        let Some(lease) = self.lease.take() else {
            return Err(provider_error());
        };
        if lease.resource()?.poisoned.load(Ordering::Acquire) {
            lease.discard();
            return Err(SqlError::new(SqlErrorKind::Connection));
        }
        let profile = Arc::clone(&self.profile);
        lease
            .release(
                move |native| Box::pin(reset_native(native, Arc::clone(&profile))),
                cancellation,
                "postgresql-connection",
            )
            .await
    }

    pub fn discard(mut self) {
        if let Some(lease) = self.lease.take() {
            lease.discard();
        }
    }

    #[must_use]
    pub fn schema_fingerprint(&self) -> &str {
        self.evidence.schema_fingerprint()
    }

    #[must_use]
    pub fn observed_schema_fingerprint(&self) -> &str {
        self.evidence.observed_schema_fingerprint()
    }
}

impl fmt::Debug for PostgresConnection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PostgresConnection")
            .field("lease", &self.lease_id())
            .field("poisoned", &self.is_poisoned())
            .finish_non_exhaustive()
    }
}

pub(crate) async fn connect_native(
    profile: Arc<PostgresProfile>,
) -> Result<PostgresNativeConnection, SqlError> {
    let (client, driver_task) = match &profile.tls {
        PostgresTls::Disabled => {
            let (client, connection) = profile
                .config
                .connect(NoTls)
                .await
                .map_err(|error| map_postgres_error(&error))?;
            let task = tokio::spawn(async move {
                let _result = connection.await;
            });
            (client, task)
        }
        PostgresTls::Rustls(config) => {
            let connector = MakeRustlsConnect::new(config.as_ref().clone());
            let (client, connection) = profile
                .config
                .connect(connector)
                .await
                .map_err(|error| map_postgres_error(&error))?;
            let task = tokio::spawn(async move {
                let _result = connection.await;
            });
            (client, task)
        }
    };
    let version = client
        .query_one("SHOW server_version_num", &[])
        .await
        .map_err(|error| map_postgres_error(&error))?
        .try_get::<_, String>(0)
        .map_err(|error| map_postgres_error(&error))?
        .parse::<u32>()
        .map_err(|_| provider_error())?;
    apply_session(&client, &profile).await?;
    verify_session(&client, &profile).await?;
    let lease_number = NEXT_LEASE.fetch_add(1, Ordering::Relaxed);
    Ok(PostgresNativeConnection {
        cancel_token: client.cancel_token(),
        statements: StatementCache::new(profile.limits.statement_cache_capacity)?,
        client,
        driver_task,
        server_version: version,
        poisoned: Arc::new(AtomicBool::new(false)),
        lease: ProviderLeaseToken::new(format!("postgresql-{lease_number}"))?,
    })
}

pub(crate) async fn reset_native(
    native: &mut PostgresNativeConnection,
    profile: Arc<PostgresProfile>,
) -> Result<(), SqlError> {
    if native.poisoned.load(Ordering::Acquire) {
        return Err(SqlError::new(SqlErrorKind::Connection));
    }
    native
        .client
        .batch_execute("RESET ALL; UNLISTEN *; SELECT pg_advisory_unlock_all();")
        .await
        .map_err(|error| map_postgres_error(&error))?;
    apply_session(&native.client, &profile).await?;
    verify_session(&native.client, &profile).await
}

pub(crate) async fn prepare_acquired_native(
    native: &mut PostgresNativeConnection,
    profile: &PostgresProfile,
) -> Result<(), SqlError> {
    if native.poisoned.load(Ordering::Acquire) {
        return Err(SqlError::new(SqlErrorKind::Connection));
    }
    apply_session(&native.client, profile).await?;
    verify_session(&native.client, profile).await
}

pub(crate) async fn prepare_statement(
    native: &mut PostgresNativeConnection,
    key: StatementCacheKey,
    statement: &str,
) -> Result<(Statement, bool), SqlError> {
    if let Some(prepared) = native.statements.get(&key).cloned() {
        return Ok((prepared, true));
    }
    let prepared = native
        .client
        .prepare(statement)
        .await
        .map_err(|error| map_postgres_error(&error))?;
    native.statements.insert(&key, prepared.clone());
    Ok((prepared, false))
}

pub(crate) fn decode_row(
    row: &tokio_postgres::Row,
    limits: sifr_sql_runtime::RuntimeLimits,
) -> Result<PostgresRow, SqlError> {
    let mut values = Vec::with_capacity(row.len());
    let mut total_bytes = 0_u64;
    for (index, column) in row.columns().iter().enumerate() {
        let raw = row
            .try_get::<_, Option<RawPostgresValue>>(index)
            .map_err(|error| map_postgres_error(&error))?;
        let byte_count = raw.as_ref().map_or(0_u64, |value| {
            u64::try_from(value.0.len()).unwrap_or(u64::MAX)
        });
        total_bytes = total_bytes
            .checked_add(byte_count)
            .ok_or_else(|| SqlError::new(SqlErrorKind::ResourceLimit))?;
        if total_bytes > limits.max_decoded_row_bytes {
            return Err(SqlError::resource_limit(
                sifr_sql_runtime::ResourceLimitKind::DecodedRowBytes,
            ));
        }
        values.push(decode_value(column.type_(), raw)?);
    }
    Ok(PostgresRow {
        values,
        decoded_bytes: total_bytes,
    })
}

pub(crate) fn execution_result(
    rows_affected: Option<u64>,
    cache_hit: bool,
    server_version: u32,
) -> ExecutionResult<PostgresMetadata> {
    ExecutionResult {
        rows_affected,
        metadata: PostgresMetadata {
            statement_cache_hit: cache_hit,
            server_version,
        },
    }
}

pub(crate) async fn cancel_query(tls: &PostgresTls, token: &CancelToken) -> Result<(), SqlError> {
    match tls {
        PostgresTls::Disabled => token
            .cancel_query(NoTls)
            .await
            .map_err(|error| map_postgres_error(&error)),
        PostgresTls::Rustls(config) => token
            .cancel_query(MakeRustlsConnect::new(config.as_ref().clone()))
            .await
            .map_err(|error| map_postgres_error(&error)),
    }
}

async fn apply_session(client: &Client, profile: &PostgresProfile) -> Result<(), SqlError> {
    let path = profile.session.search_path.join(", ");
    let isolation = sifr_sql_runtime::isolation_name(profile.session.default_isolation);
    let read_only = if profile.session.read_only {
        "on"
    } else {
        "off"
    };
    for (name, value) in [
        ("search_path", path.as_str()),
        ("TimeZone", profile.session.time_zone.as_str()),
        ("default_transaction_isolation", isolation),
        ("default_transaction_read_only", read_only),
    ] {
        client
            .execute("SELECT set_config($1, $2, false)", &[&name, &value])
            .await
            .map_err(|error| map_postgres_error(&error))?;
    }
    if let Some(role) = &profile.session.role {
        client
            .batch_execute(&format!("SET ROLE {}", quote_identifier(role)))
            .await
            .map_err(|error| map_postgres_error(&error))?;
    } else {
        client
            .batch_execute("RESET ROLE")
            .await
            .map_err(|error| map_postgres_error(&error))?;
    }
    Ok(())
}

async fn verify_session(client: &Client, profile: &PostgresProfile) -> Result<(), SqlError> {
    let row = client
        .query_one(
            "SELECT current_setting('search_path'), current_setting('TimeZone'), current_user, \
             current_setting('default_transaction_isolation'), \
             current_setting('default_transaction_read_only')",
            &[],
        )
        .await
        .map_err(|error| map_postgres_error(&error))?;
    let snapshot = SessionSnapshot {
        search_path: row.try_get(0).map_err(|error| map_postgres_error(&error))?,
        time_zone: row.try_get(1).map_err(|error| map_postgres_error(&error))?,
        role: row.try_get(2).map_err(|error| map_postgres_error(&error))?,
        default_isolation: row.try_get(3).map_err(|error| map_postgres_error(&error))?,
        read_only: row
            .try_get::<_, String>(4)
            .map_err(|error| map_postgres_error(&error))?
            == "on",
    };
    if snapshot.matches(&profile.session) {
        Ok(())
    } else {
        Err(SqlError::new(SqlErrorKind::SchemaContract))
    }
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
