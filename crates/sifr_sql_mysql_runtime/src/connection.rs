use crate::codec::{decode_row, encode_parameters};
use crate::config::MysqlProfile;
use crate::control::{ControlHandle, run_controlled};
use crate::error::{map_mysql_error, provider_error};
use mysql_async::{Conn, Params, Statement, prelude::Queryable};
use sifr_sql_runtime::{
    CancellationCarrier, ExecutionRequest, ExecutionResult, OwnedSqlValue, PoolDiscardGuard,
    PoolLease, ProviderLeaseToken, ResourceUsage, SqlError, SqlErrorKind, StatementCache,
    StatementCacheKey, VerificationEvidence,
};
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

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
    pub(crate) fn deadline(&self, profile: &MysqlProfile) -> Result<Duration, SqlError> {
        let timeout = self.timeout.unwrap_or(profile.limits.statement_timeout);
        if timeout.is_zero() || timeout > profile.limits.statement_timeout {
            return Err(SqlError::new(SqlErrorKind::Configuration));
        }
        Ok(timeout)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MysqlMetadata {
    pub statement_cache_hit: bool,
    pub server_version: (u16, u16, u16),
    pub last_insert_id: Option<u64>,
    pub warnings: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MysqlRow {
    pub(crate) values: Vec<OwnedSqlValue>,
    pub(crate) decoded_bytes: u64,
}

impl MysqlRow {
    #[must_use]
    pub fn values(&self) -> &[OwnedSqlValue] {
        &self.values
    }

    #[must_use]
    pub const fn decoded_bytes(&self) -> u64 {
        self.decoded_bytes
    }
}

pub(crate) struct MysqlNativeConnection {
    pub(crate) conn: Conn,
    pub(crate) statements: StatementCache<Statement>,
    pub(crate) server_version: (u16, u16, u16),
    pub(crate) connection_id: u32,
    pub(crate) poisoned: Arc<AtomicBool>,
    pub(crate) lease: ProviderLeaseToken,
}

pub struct MysqlConnection {
    pub(crate) lease: Option<PoolLease<MysqlNativeConnection>>,
    pub(crate) profile: Arc<MysqlProfile>,
    pub(crate) evidence: Arc<VerificationEvidence>,
}

impl MysqlConnection {
    pub(crate) fn new(
        lease: PoolLease<MysqlNativeConnection>,
        profile: Arc<MysqlProfile>,
        evidence: Arc<VerificationEvidence>,
    ) -> Self {
        Self {
            lease: Some(lease),
            profile,
            evidence,
        }
    }

    pub async fn begin(self) -> Result<crate::transaction::MysqlTransaction, SqlError> {
        crate::transaction::MysqlTransaction::begin(self).await
    }

    pub async fn execute(
        &mut self,
        request: ExecutionRequest<MysqlProfile>,
        options: ExecutionOptions,
    ) -> Result<ExecutionResult<MysqlMetadata>, SqlError> {
        validate_request(&request, &self.profile)?;
        let key = statement_key(&request, self.native()?.server_version)?;
        let parameters = encode_parameters(request.parameters)?;
        let profile = Arc::clone(&self.profile);
        let control = self.control()?;
        let native = self.native_mut()?;
        let (statement, cache_hit) = prepare_statement(native, key, &request.statement).await?;
        let result = run_controlled(&control, &profile, &options, async {
            native
                .conn
                .exec_drop(statement, Params::Positional(parameters))
                .await
                .map_err(|error| map_mysql_error(&error))?;
            Ok(execution_result(
                native,
                Some(native.conn.affected_rows()),
                cache_hit,
            ))
        })
        .await;
        if control.poison.load(Ordering::Acquire) {
            self.discard_native();
        }
        result
    }

    pub async fn fetch_all(
        &mut self,
        request: ExecutionRequest<MysqlProfile>,
        options: ExecutionOptions,
    ) -> Result<Vec<MysqlRow>, SqlError> {
        validate_request(&request, &self.profile)?;
        let key = statement_key(&request, self.native()?.server_version)?;
        let parameters = encode_parameters(request.parameters)?;
        let profile = Arc::clone(&self.profile);
        let limits = profile.limits;
        let control = self.control()?;
        let native = self.native_mut()?;
        let (statement, _) = prepare_statement(native, key, &request.statement).await?;
        let result = run_controlled(&control, &profile, &options, async {
            let mut query = native
                .conn
                .exec_iter(statement, Params::Positional(parameters))
                .await
                .map_err(|error| map_mysql_error(&error))?;
            let mut rows = Vec::new();
            let mut usage = ResourceUsage::default();
            while let Some(row) = query
                .next()
                .await
                .map_err(|error| map_mysql_error(&error))?
            {
                let (values, decoded_bytes) = decode_row(&row, limits)?;
                usage.account_row(decoded_bytes, limits)?;
                rows.push(MysqlRow {
                    values,
                    decoded_bytes,
                });
            }
            drop(query);
            Ok(rows)
        })
        .await;
        if control.poison.load(Ordering::Acquire) {
            self.discard_native();
        }
        result
    }

    pub(crate) fn detach_for_stream(
        mut self,
    ) -> Result<
        (
            MysqlNativeConnection,
            PoolDiscardGuard<MysqlNativeConnection>,
        ),
        SqlError,
    > {
        self.lease
            .take()
            .ok_or_else(provider_error)?
            .detach_for_discard()
    }

    pub(crate) fn native(&self) -> Result<&MysqlNativeConnection, SqlError> {
        self.lease.as_ref().ok_or_else(provider_error)?.resource()
    }

    pub(crate) fn native_mut(&mut self) -> Result<&mut MysqlNativeConnection, SqlError> {
        self.lease
            .as_mut()
            .ok_or_else(provider_error)?
            .resource_mut()
    }

    pub(crate) fn control(&self) -> Result<ControlHandle, SqlError> {
        let native = self.native()?;
        Ok(ControlHandle::new(
            Arc::clone(&native.poisoned),
            native.connection_id,
            &self.profile,
        ))
    }

    fn discard_native(&mut self) {
        if let Some(lease) = self.lease.take() {
            lease.discard();
        }
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

    #[must_use]
    pub fn schema_fingerprint(&self) -> &str {
        self.evidence.schema_fingerprint()
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
                "mysql-connection",
            )
            .await
    }

    pub fn discard(mut self) {
        self.discard_native();
    }
}

impl fmt::Debug for MysqlConnection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MysqlConnection")
            .field("lease", &self.lease_id())
            .field("poisoned", &self.is_poisoned())
            .finish_non_exhaustive()
    }
}

impl Drop for MysqlConnection {
    fn drop(&mut self) {
        self.discard_native();
    }
}

pub(crate) async fn connect_native(
    profile: Arc<MysqlProfile>,
) -> Result<MysqlNativeConnection, SqlError> {
    let mut conn = Conn::new(profile.opts.clone())
        .await
        .map_err(|error| map_mysql_error(&error))?;
    let server_version = conn.server_version();
    let connection_id = conn.id();
    apply_session(&mut conn, &profile).await?;
    verify_session(&mut conn, &profile).await?;
    let lease_number = NEXT_LEASE.fetch_add(1, Ordering::Relaxed);
    Ok(MysqlNativeConnection {
        conn,
        statements: StatementCache::new(profile.limits.statement_cache_capacity)?,
        server_version,
        connection_id,
        poisoned: Arc::new(AtomicBool::new(false)),
        lease: ProviderLeaseToken::new(format!("mysql-{lease_number}"))?,
    })
}

pub(crate) async fn prepare_acquired_native(
    native: &mut MysqlNativeConnection,
    profile: &MysqlProfile,
) -> Result<(), SqlError> {
    if native.poisoned.load(Ordering::Acquire) || native.conn.is_disconnected() {
        return Err(SqlError::new(SqlErrorKind::Connection));
    }
    apply_session(&mut native.conn, profile).await?;
    verify_session(&mut native.conn, profile).await
}

pub(crate) async fn reset_native(
    native: &mut MysqlNativeConnection,
    profile: Arc<MysqlProfile>,
) -> Result<(), SqlError> {
    if native.poisoned.load(Ordering::Acquire) {
        return Err(SqlError::new(SqlErrorKind::Connection));
    }
    let reset = native
        .conn
        .reset()
        .await
        .map_err(|error| map_mysql_error(&error))?;
    if !reset {
        return Err(SqlError::new(SqlErrorKind::Connection));
    }
    native.statements.clear();
    apply_session(&mut native.conn, &profile).await?;
    verify_session(&mut native.conn, &profile).await
}

pub(crate) async fn prepare_statement(
    native: &mut MysqlNativeConnection,
    key: StatementCacheKey,
    statement: &str,
) -> Result<(Statement, bool), SqlError> {
    if let Some(prepared) = native.statements.get(&key).cloned() {
        return Ok((prepared, true));
    }
    let prepared = native
        .conn
        .prep(statement)
        .await
        .map_err(|error| map_mysql_error(&error))?;
    native.statements.insert(&key, prepared.clone());
    Ok((prepared, false))
}

async fn apply_session(conn: &mut Conn, profile: &MysqlProfile) -> Result<(), SqlError> {
    let database = profile
        .session
        .search_path
        .first()
        .ok_or_else(provider_error)?;
    conn.query_drop(format!("USE {}", quote_identifier(database)))
        .await
        .map_err(|error| map_mysql_error(&error))?;
    conn.exec_drop(
        "SET SESSION time_zone = ?",
        Params::Positional(vec![mysql_async::Value::Bytes(
            profile.session.time_zone.as_bytes().to_vec(),
        )]),
    )
    .await
    .map_err(|error| map_mysql_error(&error))?;
    let isolation =
        sifr_sql_runtime::isolation_name(profile.session.default_isolation).to_ascii_uppercase();
    conn.query_drop(format!(
        "SET SESSION TRANSACTION ISOLATION LEVEL {isolation}"
    ))
    .await
    .map_err(|error| map_mysql_error(&error))?;
    conn.query_drop(format!(
        "SET SESSION transaction_read_only = {}",
        u8::from(profile.session.read_only)
    ))
    .await
    .map_err(|error| map_mysql_error(&error))?;
    if let Some(role) = &profile.session.role {
        conn.query_drop(format!("SET ROLE {}", quote_identifier(role)))
            .await
            .map_err(|error| map_mysql_error(&error))?;
    }
    Ok(())
}

async fn verify_session(conn: &mut Conn, profile: &MysqlProfile) -> Result<(), SqlError> {
    let row: Option<(String, String, u8)> = conn
        .query_first("SELECT DATABASE(), @@session.time_zone, @@session.transaction_read_only")
        .await
        .map_err(|error| map_mysql_error(&error))?;
    let Some((database, time_zone, read_only)) = row else {
        return Err(SqlError::new(SqlErrorKind::SchemaContract));
    };
    if Some(database.as_str()) == profile.session.search_path.first().map(String::as_str)
        && time_zone == profile.session.time_zone
        && (read_only != 0) == profile.session.read_only
    {
        Ok(())
    } else {
        Err(SqlError::new(SqlErrorKind::SchemaContract))
    }
}

pub(crate) fn validate_request(
    request: &ExecutionRequest<MysqlProfile>,
    profile: &MysqlProfile,
) -> Result<(), SqlError> {
    request.validate()?;
    let parameter_count = u32::try_from(request.parameters.as_slice().len())
        .map_err(|_| SqlError::new(SqlErrorKind::ResourceLimit))?;
    if parameter_count > profile.limits.max_parameters
        || request.profile.profile_fingerprint != profile.profile_fingerprint
        || request.metadata.schema_fingerprint != profile.expected_schema.fingerprint()
    {
        return Err(SqlError::new(SqlErrorKind::SchemaContract));
    }
    Ok(())
}

pub(crate) fn statement_key(
    request: &ExecutionRequest<MysqlProfile>,
    server_version: (u16, u16, u16),
) -> Result<StatementCacheKey, SqlError> {
    StatementCacheKey {
        normalized_statement_fingerprint: request.metadata.normalized_statement_fingerprint.clone(),
        parameter_type_fingerprint: request.metadata.parameter_type_fingerprint.clone(),
        result_type_fingerprint: request.metadata.result_type_fingerprint.clone(),
        provider_version: format!(
            "{}.{}.{}",
            server_version.0, server_version.1, server_version.2
        ),
        schema_fingerprint: request.metadata.schema_fingerprint.clone(),
    }
    .validate()
}

fn execution_result(
    native: &MysqlNativeConnection,
    rows_affected: Option<u64>,
    cache_hit: bool,
) -> ExecutionResult<MysqlMetadata> {
    ExecutionResult {
        rows_affected,
        metadata: MysqlMetadata {
            statement_cache_hit: cache_hit,
            server_version: native.server_version,
            last_insert_id: native.conn.last_insert_id(),
            warnings: native.conn.get_warnings(),
        },
    }
}

fn quote_identifier(value: &str) -> String {
    format!("`{}`", value.replace('`', "``"))
}
