use crate::config::MysqlProfile;
use crate::connection::{
    ExecutionOptions, MysqlConnection, MysqlMetadata, MysqlNativeConnection, MysqlRow,
    connect_native, prepare_acquired_native,
};
use crate::error::provider_error;
use crate::stream::MysqlRowStream;
use crate::transaction::{MysqlTransaction, RetryPolicy, RetrySafeCallback, run_retry_safe};
use sifr_sql_runtime::{
    CancellationCarrier, ExecutionRequest, ExecutionResult, PoolCoordinator, PoolStatistics,
    SqlError, Unverified, VerificationEvidence, Verified, verify_schema,
};
use std::marker::PhantomData;
use std::sync::Arc;

struct PoolShared {
    coordinator: PoolCoordinator<MysqlNativeConnection>,
    profile: Arc<MysqlProfile>,
}

pub struct MysqlPool<S> {
    shared: Arc<PoolShared>,
    evidence: Option<Arc<VerificationEvidence>>,
    state: PhantomData<fn() -> S>,
}

impl<S> Clone for MysqlPool<S> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
            evidence: self.evidence.as_ref().map(Arc::clone),
            state: PhantomData,
        }
    }
}

pub fn open_pool(profile: MysqlProfile) -> Result<MysqlPool<Unverified>, SqlError> {
    let profile = Arc::new(profile);
    let coordinator = PoolCoordinator::new(profile.limits)?;
    Ok(MysqlPool {
        shared: Arc::new(PoolShared {
            coordinator,
            profile,
        }),
        evidence: None,
        state: PhantomData,
    })
}

pub async fn connect(profile: MysqlProfile) -> Result<MysqlPool<Verified>, SqlError> {
    open_pool(profile)?.verify_schema().await
}

impl MysqlPool<Unverified> {
    pub async fn verify_schema(self) -> Result<MysqlPool<Verified>, SqlError> {
        let profile = Arc::clone(&self.shared.profile);
        let mut lease = self
            .shared
            .coordinator
            .acquire(|| connect_native(Arc::clone(&profile)))
            .await?;
        let observed = profile
            .verifier
            .observe(&mut lease.resource_mut()?.conn, &profile)
            .await;
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
                "mysql-verification-connection",
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
        Ok(MysqlPool {
            shared: self.shared,
            evidence: Some(Arc::new(evidence)),
            state: PhantomData,
        })
    }
}

impl MysqlPool<Verified> {
    pub async fn acquire(&self) -> Result<MysqlConnection, SqlError> {
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
        Ok(MysqlConnection::new(lease, profile, evidence))
    }

    pub async fn execute(
        &self,
        request: ExecutionRequest<MysqlProfile>,
        options: ExecutionOptions,
    ) -> Result<ExecutionResult<MysqlMetadata>, SqlError> {
        let cancellation = options.cancellation.clone();
        let mut connection = self.acquire().await?;
        let result = connection.execute(request, options).await;
        finish_connection(connection, result, cancellation.as_ref()).await
    }

    pub async fn fetch_all(
        &self,
        request: ExecutionRequest<MysqlProfile>,
        options: ExecutionOptions,
    ) -> Result<Vec<MysqlRow>, SqlError> {
        let cancellation = options.cancellation.clone();
        let mut connection = self.acquire().await?;
        let result = connection.fetch_all(request, options).await;
        finish_connection(connection, result, cancellation.as_ref()).await
    }

    pub async fn stream(
        &self,
        request: ExecutionRequest<MysqlProfile>,
        options: ExecutionOptions,
    ) -> Result<MysqlRowStream, SqlError> {
        MysqlRowStream::open(self.acquire().await?, request, options).await
    }

    pub async fn transaction(&self) -> Result<MysqlTransaction, SqlError> {
        MysqlTransaction::begin(self.acquire().await?).await
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
    pub fn statistics(&self) -> PoolStatistics {
        self.shared.coordinator.statistics()
    }

    pub fn close(&self) {
        self.shared.coordinator.close();
    }
}

async fn finish_connection<T>(
    connection: MysqlConnection,
    result: Result<T, SqlError>,
    cancellation: Option<&CancellationCarrier>,
) -> Result<T, SqlError> {
    if connection.is_poisoned() {
        connection.discard();
        return match result {
            Ok(_) => Err(SqlError::new(sifr_sql_runtime::SqlErrorKind::Connection)),
            Err(error) => Err(error),
        };
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
