use crate::config::PostgresProfile;
use crate::connection::{
    ExecutionOptions, PostgresConnection, PostgresMetadata, PostgresNativeConnection, PostgresRow,
    connect_native, prepare_acquired_native,
};
use crate::error::provider_error;
use crate::stream::PostgresRowStream;
use crate::transaction::{
    PostgresTransaction, RetryPolicy, RetrySafeCallback, TransactionOptions, run_retry_safe,
};
use crate::verification::observe_schema;
use sifr_sql_runtime::{
    CancellationCarrier, ExecutionRequest, ExecutionResult, PoolCoordinator, PoolStatistics,
    SqlError, Unverified, VerificationEvidence, Verified, verify_schema,
};
use std::marker::PhantomData;
use std::sync::Arc;

struct PoolShared {
    coordinator: PoolCoordinator<PostgresNativeConnection>,
    profile: Arc<PostgresProfile>,
}

pub struct PostgresPool<S> {
    shared: Arc<PoolShared>,
    evidence: Option<Arc<VerificationEvidence>>,
    state: PhantomData<fn() -> S>,
}

impl<S> Clone for PostgresPool<S> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
            evidence: self.evidence.as_ref().map(Arc::clone),
            state: PhantomData,
        }
    }
}

pub fn open_pool(profile: PostgresProfile) -> Result<PostgresPool<Unverified>, SqlError> {
    let profile = Arc::new(profile);
    let coordinator = PoolCoordinator::new(profile.limits)?;
    Ok(PostgresPool {
        shared: Arc::new(PoolShared {
            coordinator,
            profile,
        }),
        evidence: None,
        state: PhantomData,
    })
}

pub async fn connect(profile: PostgresProfile) -> Result<PostgresPool<Verified>, SqlError> {
    open_pool(profile)?.verify_schema().await
}

impl PostgresPool<Unverified> {
    pub async fn verify_schema(self) -> Result<PostgresPool<Verified>, SqlError> {
        let profile = Arc::clone(&self.shared.profile);
        let lease = self
            .shared
            .coordinator
            .acquire(|| connect_native(Arc::clone(&profile)))
            .await?;
        let observed = observe_schema(lease.resource()?, &profile).await;
        let verification = observed.and_then(|observed| {
            verify_schema(profile.strictness, &profile.expected_schema, &observed)?;
            VerificationEvidence::with_observation(
                profile.profile_fingerprint.clone(),
                profile.expected_schema.fingerprint().to_string(),
                observed.fingerprint().to_string(),
            )
        });
        let reset_profile = Arc::clone(&profile);
        let release = lease
            .release(
                move |native| {
                    Box::pin(crate::connection::reset_native(
                        native,
                        Arc::clone(&reset_profile),
                    ))
                },
                None,
                "postgresql-verification-connection",
            )
            .await;
        let evidence = match verification {
            Ok(evidence) => {
                release?;
                evidence
            }
            Err(mut primary) => {
                if let Err(cleanup) = release {
                    primary.extend_secondary(cleanup.secondary().iter().cloned());
                }
                return Err(primary);
            }
        };
        Ok(PostgresPool {
            shared: self.shared,
            evidence: Some(Arc::new(evidence)),
            state: PhantomData,
        })
    }
}

impl PostgresPool<Verified> {
    pub async fn acquire(&self) -> Result<PostgresConnection, SqlError> {
        let profile = Arc::clone(&self.shared.profile);
        let mut lease = self
            .shared
            .coordinator
            .acquire(|| connect_native(Arc::clone(&profile)))
            .await?;
        if let Err(error) = prepare_acquired_native(lease.resource_mut()?, &profile).await {
            lease.discard();
            return Err(error);
        }
        let evidence = self
            .evidence
            .as_ref()
            .map(Arc::clone)
            .ok_or_else(provider_error)?;
        Ok(PostgresConnection::new(lease, profile, evidence))
    }

    pub async fn execute(
        &self,
        request: ExecutionRequest<PostgresProfile>,
        options: ExecutionOptions,
    ) -> Result<ExecutionResult<PostgresMetadata>, SqlError> {
        let cancellation = options.cancellation.clone();
        let mut connection = self.acquire().await?;
        let result = connection.execute(request, options).await;
        finish_connection(connection, result, cancellation.as_ref()).await
    }

    pub async fn fetch_one(
        &self,
        request: ExecutionRequest<PostgresProfile>,
        options: ExecutionOptions,
    ) -> Result<PostgresRow, SqlError> {
        let cancellation = options.cancellation.clone();
        let mut connection = self.acquire().await?;
        let result = connection.fetch_one(request, options).await;
        finish_connection(connection, result, cancellation.as_ref()).await
    }

    pub async fn fetch_optional(
        &self,
        request: ExecutionRequest<PostgresProfile>,
        options: ExecutionOptions,
    ) -> Result<Option<PostgresRow>, SqlError> {
        let cancellation = options.cancellation.clone();
        let mut connection = self.acquire().await?;
        let result = connection.fetch_optional(request, options).await;
        finish_connection(connection, result, cancellation.as_ref()).await
    }

    pub async fn fetch_all(
        &self,
        request: ExecutionRequest<PostgresProfile>,
        options: ExecutionOptions,
    ) -> Result<Vec<PostgresRow>, SqlError> {
        let cancellation = options.cancellation.clone();
        let mut connection = self.acquire().await?;
        let result = connection.fetch_all(request, options).await;
        finish_connection(connection, result, cancellation.as_ref()).await
    }

    pub async fn stream(
        &self,
        request: ExecutionRequest<PostgresProfile>,
        options: ExecutionOptions,
    ) -> Result<PostgresRowStream, SqlError> {
        let cancellation = options.cancellation.clone();
        let connection = self.acquire().await?;
        match PostgresRowStream::open(connection, request, options).await {
            Ok(stream) => Ok(stream),
            Err(failure) => {
                let (connection, error) = *failure;
                finish_connection::<()>(connection, Err(error), cancellation.as_ref()).await?;
                Err(provider_error())
            }
        }
    }

    pub async fn transaction(
        &self,
        options: TransactionOptions,
    ) -> Result<PostgresTransaction, SqlError> {
        PostgresTransaction::begin(self.acquire().await?, options).await
    }

    pub async fn run_transaction<T, C>(
        &self,
        callback: C,
        retry: RetryPolicy,
    ) -> Result<T, SqlError>
    where
        C: RetrySafeCallback<T>,
    {
        run_retry_safe(callback, retry, || self.acquire()).await
    }

    #[must_use]
    pub fn profile(&self) -> &Arc<PostgresProfile> {
        &self.shared.profile
    }

    #[must_use]
    pub fn schema_fingerprint(&self) -> &str {
        self.evidence
            .as_ref()
            .map_or("", |evidence| evidence.schema_fingerprint())
    }
}

impl<S> PostgresPool<S> {
    #[must_use]
    pub fn statistics(&self) -> PoolStatistics {
        self.shared.coordinator.statistics()
    }

    pub fn close(&self) {
        self.shared.coordinator.close();
    }
}

async fn finish_connection<T>(
    connection: PostgresConnection,
    result: Result<T, SqlError>,
    cancellation: Option<&CancellationCarrier>,
) -> Result<T, SqlError> {
    if connection.is_poisoned() {
        connection.discard();
        return result.and_then(|_| Err(SqlError::new(sifr_sql_runtime::SqlErrorKind::Cancelled)));
    }
    let release = connection.release(cancellation).await;
    match (result, release) {
        (Ok(value), Ok(())) => Ok(value),
        (Ok(_), Err(cleanup)) => Err(cleanup),
        (Err(primary), Ok(())) => Err(primary),
        (Err(mut primary), Err(cleanup)) => {
            primary.extend_secondary(cleanup.secondary().iter().cloned());
            Err(primary)
        }
    }
}
