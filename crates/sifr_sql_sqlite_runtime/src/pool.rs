use crate::config::{SqliteEvidence, SqliteProfile};
use crate::stream::SqliteRowStream;
use crate::transaction::SqliteTransaction;
use crate::worker::{SqliteExecutionMetadata, SqliteRow, WorkerHandle};
use sifr_sql_runtime::{
    CancellationCarrier, CardinalityViolation, ExecutionMode, ExecutionRequest, ExecutionResult,
    PoolCoordinator, PoolLease, PoolStatistics, ResourceLimitKind, ResourceUsage, SqlError,
    SqlErrorKind, Unverified, VerificationEvidence, Verified, verify_schema,
};
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

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
    pub(crate) fn deadline(&self, profile: &SqliteProfile) -> Result<Duration, SqlError> {
        let timeout = self.timeout.unwrap_or(profile.limits().statement_timeout);
        if timeout.is_zero() || timeout > profile.limits().statement_timeout {
            return Err(SqlError::new(SqlErrorKind::Configuration));
        }
        Ok(timeout)
    }
}

struct PoolShared {
    coordinator: PoolCoordinator<WorkerHandle>,
    profile: Arc<SqliteProfile>,
}

pub struct SqlitePool<S> {
    shared: Arc<PoolShared>,
    evidence: Option<Arc<VerificationEvidence>>,
    state: PhantomData<fn() -> S>,
}

impl<S> Clone for SqlitePool<S> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
            evidence: self.evidence.as_ref().map(Arc::clone),
            state: PhantomData,
        }
    }
}

pub fn open_pool(profile: SqliteProfile) -> Result<SqlitePool<Unverified>, SqlError> {
    let profile = Arc::new(profile);
    let coordinator = PoolCoordinator::new(profile.limits())?;
    Ok(SqlitePool {
        shared: Arc::new(PoolShared {
            coordinator,
            profile,
        }),
        evidence: None,
        state: PhantomData,
    })
}

pub async fn connect(profile: SqliteProfile) -> Result<SqlitePool<Verified>, SqlError> {
    open_pool(profile)?.verify_schema().await
}

impl SqlitePool<Unverified> {
    pub async fn verify_schema(self) -> Result<SqlitePool<Verified>, SqlError> {
        let profile = Arc::clone(&self.shared.profile);
        let observed = match &profile.evidence {
            SqliteEvidence::SignedManifest { manifest, verifier } => {
                SqliteEvidence::verify_manifest(verifier, manifest)
            }
            evidence => {
                let lease = self
                    .shared
                    .coordinator
                    .acquire(|| async { WorkerHandle::open(&profile).await })
                    .await?;
                let observed = lease
                    .resource()?
                    .observe(evidence.clone(), profile.limits().statement_timeout)
                    .await;
                let timeout = profile.limits().cleanup_timeout;
                let release = lease
                    .release(
                        move |worker| Box::pin(async move { worker.reset(timeout).await }),
                        None,
                        "sqlite-verification-worker",
                    )
                    .await;
                match (observed, release) {
                    (Ok(observed), Ok(())) => Ok(observed),
                    (Ok(_), Err(cleanup)) => Err(cleanup),
                    (Err(primary), Ok(())) => Err(primary),
                    (Err(mut primary), Err(cleanup)) => {
                        primary.extend_secondary(cleanup.secondary().iter().cloned());
                        Err(primary)
                    }
                }
            }
        }?;
        verify_schema(profile.strictness, &profile.expected_schema, &observed)?;
        let evidence = VerificationEvidence::with_observation(
            profile.profile_fingerprint.clone(),
            profile.expected_schema.fingerprint().to_string(),
            observed.fingerprint().to_string(),
        )?;
        Ok(SqlitePool {
            shared: self.shared,
            evidence: Some(Arc::new(evidence)),
            state: PhantomData,
        })
    }
}

impl SqlitePool<Verified> {
    pub async fn acquire(&self) -> Result<SqliteConnection, SqlError> {
        let profile = Arc::clone(&self.shared.profile);
        let open_profile = Arc::clone(&profile);
        let lease = self
            .shared
            .coordinator
            .acquire(move || async move { WorkerHandle::open(&open_profile).await })
            .await?;
        let evidence = self
            .evidence
            .as_ref()
            .map(Arc::clone)
            .ok_or_else(provider_error)?;
        Ok(SqliteConnection {
            lease: Some(lease),
            profile,
            evidence,
        })
    }

    pub async fn execute(
        &self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<ExecutionResult<SqliteExecutionMetadata>, SqlError> {
        let cancellation = options.cancellation.clone();
        let mut connection = self.acquire().await?;
        let result = connection.execute(request, options).await;
        finish_connection(connection, result, cancellation.as_ref()).await
    }

    pub async fn fetch_one(
        &self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<SqliteRow, SqlError> {
        let cancellation = options.cancellation.clone();
        let mut connection = self.acquire().await?;
        let result = connection.fetch_one(request, options).await;
        finish_connection(connection, result, cancellation.as_ref()).await
    }

    pub async fn fetch_optional(
        &self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<Option<SqliteRow>, SqlError> {
        let cancellation = options.cancellation.clone();
        let mut connection = self.acquire().await?;
        let result = connection.fetch_optional(request, options).await;
        finish_connection(connection, result, cancellation.as_ref()).await
    }

    pub async fn fetch_all(
        &self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<Vec<SqliteRow>, SqlError> {
        let cancellation = options.cancellation.clone();
        let mut connection = self.acquire().await?;
        let result = connection.fetch_all(request, options).await;
        finish_connection(connection, result, cancellation.as_ref()).await
    }

    pub async fn transaction(&self) -> Result<SqliteTransaction, SqlError> {
        self.acquire().await?.begin().await
    }

    pub async fn stream(
        &self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<SqliteRowStream, SqlError> {
        SqliteRowStream::open(self.acquire().await?, request, options).await
    }
}

impl<S> SqlitePool<S> {
    #[must_use]
    pub fn statistics(&self) -> PoolStatistics {
        self.shared.coordinator.statistics()
    }

    pub fn close(&self) {
        self.shared.coordinator.close();
    }
}

pub struct SqliteConnection {
    pub(crate) lease: Option<PoolLease<WorkerHandle>>,
    pub(crate) profile: Arc<SqliteProfile>,
    evidence: Arc<VerificationEvidence>,
}

impl SqliteConnection {
    pub(crate) fn validate_request(
        &self,
        request: &ExecutionRequest<SqliteProfile>,
    ) -> Result<(), SqlError> {
        validate_request(request, &self.profile, &self.evidence)
    }

    pub async fn execute(
        &mut self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<ExecutionResult<SqliteExecutionMetadata>, SqlError> {
        if request.mode != ExecutionMode::Execute {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        validate_request(&request, &self.profile, &self.evidence)?;
        let timeout = options.deadline(&self.profile)?;
        let parameters = take_parameters(request.parameters, self.profile.limits())?;
        let result = self
            .worker()?
            .execute(
                request.statement.to_string(),
                parameters,
                timeout,
                options.cancellation.as_ref(),
            )
            .await;
        self.discard_if_poisoned();
        result.map(|metadata| ExecutionResult {
            rows_affected: Some(metadata.changes),
            metadata,
        })
    }

    pub async fn fetch_one(
        &mut self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<SqliteRow, SqlError> {
        if request.mode != ExecutionMode::FetchOne {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        let mut rows = self
            .fetch_bounded(request, options, 2, true)
            .await?
            .into_iter();
        let Some(row) = rows.next() else {
            return Err(SqlError::cardinality(
                CardinalityViolation::ExpectedExactlyOneFoundZero,
            ));
        };
        if rows.next().is_some() {
            return Err(SqlError::cardinality(
                CardinalityViolation::ExpectedExactlyOneFoundMany,
            ));
        }
        Ok(row)
    }

    pub async fn fetch_optional(
        &mut self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<Option<SqliteRow>, SqlError> {
        if request.mode != ExecutionMode::FetchOptional {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        let mut rows = self
            .fetch_bounded(request, options, 2, true)
            .await?
            .into_iter();
        let first = rows.next();
        if rows.next().is_some() {
            return Err(SqlError::cardinality(
                CardinalityViolation::ExpectedAtMostOneFoundMany,
            ));
        }
        Ok(first)
    }

    pub async fn fetch_all(
        &mut self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<Vec<SqliteRow>, SqlError> {
        let ExecutionMode::FetchAll { maximum_rows } = request.mode else {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        };
        self.fetch_bounded(request, options, maximum_rows, false)
            .await
    }

    async fn fetch_bounded(
        &mut self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
        maximum_rows: u64,
        stop_at_limit: bool,
    ) -> Result<Vec<SqliteRow>, SqlError> {
        validate_request(&request, &self.profile, &self.evidence)?;
        let maximum_rows = maximum_rows.min(self.profile.limits().max_collected_rows);
        if maximum_rows == 0 {
            return Err(SqlError::resource_limit(ResourceLimitKind::CollectedRows));
        }
        let timeout = options.deadline(&self.profile)?;
        let parameters = take_parameters(request.parameters, self.profile.limits())?;
        let mut limits = self.profile.limits();
        limits.max_collected_rows = maximum_rows;
        let result = self
            .worker()?
            .fetch(
                request.statement.to_string(),
                parameters,
                limits,
                stop_at_limit,
                timeout,
                options.cancellation.as_ref(),
            )
            .await;
        self.discard_if_poisoned();
        result
    }

    pub async fn begin(self) -> Result<SqliteTransaction, SqlError> {
        self.worker()?
            .control("BEGIN IMMEDIATE", self.profile.limits().statement_timeout)
            .await?;
        Ok(SqliteTransaction::new(self))
    }

    pub async fn release(
        mut self,
        cancellation: Option<&CancellationCarrier>,
    ) -> Result<(), SqlError> {
        let Some(lease) = self.lease.take() else {
            return Err(SqlError::new(SqlErrorKind::Connection));
        };
        let timeout = self.profile.limits().cleanup_timeout;
        lease
            .release(
                move |worker| Box::pin(async move { worker.reset(timeout).await }),
                cancellation,
                "sqlite-worker",
            )
            .await
    }

    pub fn discard(mut self) {
        self.discard_lease();
    }

    #[must_use]
    pub fn schema_fingerprint(&self) -> &str {
        self.evidence.schema_fingerprint()
    }

    #[must_use]
    pub fn profile(&self) -> Arc<SqliteProfile> {
        Arc::clone(&self.profile)
    }

    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.lease
            .as_ref()
            .and_then(|lease| lease.resource().ok())
            .is_none_or(WorkerHandle::is_poisoned)
    }

    pub(crate) fn worker(&self) -> Result<&WorkerHandle, SqlError> {
        self.lease
            .as_ref()
            .ok_or_else(|| SqlError::new(SqlErrorKind::Connection))?
            .resource()
    }

    fn discard_if_poisoned(&mut self) {
        if self.is_poisoned() {
            self.discard_lease();
        }
    }

    fn discard_lease(&mut self) {
        if let Some(lease) = self.lease.take() {
            lease.discard();
        }
    }
}

impl fmt::Debug for SqliteConnection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SqliteConnection")
            .field("path", &"<redacted>")
            .field("poisoned", &self.is_poisoned())
            .finish_non_exhaustive()
    }
}

impl Drop for SqliteConnection {
    fn drop(&mut self) {
        self.discard_lease();
    }
}

async fn finish_connection<T>(
    connection: SqliteConnection,
    result: Result<T, SqlError>,
    cancellation: Option<&CancellationCarrier>,
) -> Result<T, SqlError> {
    if connection.is_poisoned() {
        connection.discard();
        return result.and_then(|_| Err(SqlError::new(SqlErrorKind::Connection)));
    }
    match result {
        Ok(value) => {
            connection.release(cancellation).await?;
            Ok(value)
        }
        Err(mut primary) => {
            if let Err(cleanup) = connection.release(cancellation).await {
                primary.extend_secondary(cleanup.secondary().iter().cloned());
            }
            Err(primary)
        }
    }
}

fn validate_request(
    request: &ExecutionRequest<SqliteProfile>,
    profile: &Arc<SqliteProfile>,
    evidence: &VerificationEvidence,
) -> Result<(), SqlError> {
    request.validate()?;
    if !Arc::ptr_eq(&request.profile, profile)
        || request.metadata.schema_fingerprint != evidence.schema_fingerprint()
    {
        return Err(SqlError::new(SqlErrorKind::SchemaContract));
    }
    Ok(())
}

pub(crate) fn take_parameters(
    parameters: sifr_sql_runtime::BoundParameters,
    limits: sifr_sql_runtime::RuntimeLimits,
) -> Result<Vec<sifr_sql_runtime::OwnedSqlValue>, SqlError> {
    let values = parameters.into_values();
    let mut usage = ResourceUsage::default();
    usage.account_parameters(u32::try_from(values.len()).unwrap_or(u32::MAX), limits)?;
    Ok(values
        .into_iter()
        .map(|parameter| parameter.value)
        .collect())
}

fn provider_error() -> SqlError {
    SqlError::new(SqlErrorKind::Provider)
}
