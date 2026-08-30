use crate::config::PostgresProfile;
use crate::connection::{ExecutionOptions, PostgresConnection, PostgresMetadata, PostgresRow};
use crate::control::{ControlHandle, run_controlled};
use crate::error::provider_error;
use crate::stream::PostgresTransactionRowStream;
use sifr_sql_runtime::{
    AsyncCleanupEvidence, ExecutionRequest, ExecutionResult, IsolationLevel, RetryClassification,
    SqlError, SqlErrorKind, TransactionMachine, TransactionState, isolation_name,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct TransactionOptions {
    pub isolation: IsolationLevel,
    pub read_only: bool,
    pub statement_timeout: Option<Duration>,
    pub cancellation: Option<sifr_sql_runtime::CancellationCarrier>,
}

impl std::fmt::Debug for TransactionOptions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TransactionOptions")
            .field("isolation", &self.isolation)
            .field("read_only", &self.read_only)
            .field("statement_timeout", &self.statement_timeout)
            .field(
                "cancellation",
                &self.cancellation.as_ref().map(|_| "<carrier>"),
            )
            .finish()
    }
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            isolation: IsolationLevel::ReadCommitted,
            read_only: false,
            statement_timeout: None,
            cancellation: None,
        }
    }
}

/// A task-local consuming transaction handle.
///
/// Commit and rollback consume the handle. Dropping a live handle discards the
/// connection, which causes PostgreSQL to roll back the open transaction.
pub struct PostgresTransaction {
    connection: Option<PostgresConnection>,
    machine: TransactionMachine,
    cleanup_options: ExecutionOptions,
}

impl PostgresTransaction {
    pub async fn begin(
        mut connection: PostgresConnection,
        options: TransactionOptions,
    ) -> Result<Self, SqlError> {
        let profile = Arc::clone(&connection.profile);
        let timeout = options
            .statement_timeout
            .unwrap_or(profile.limits.statement_timeout);
        if timeout.is_zero() || timeout > profile.limits.statement_timeout {
            connection.discard();
            return Err(SqlError::new(SqlErrorKind::Configuration));
        }
        let read_mode = if options.read_only {
            "READ ONLY"
        } else {
            "READ WRITE"
        };
        let milliseconds = timeout.as_millis();
        let statement = format!(
            "BEGIN ISOLATION LEVEL {} {read_mode}; SET LOCAL statement_timeout = '{milliseconds}ms'",
            isolation_name(options.isolation)
        );
        let execution = ExecutionOptions {
            timeout: Some(timeout),
            cancellation: options.cancellation,
        };
        if let Err(error) = run_control_sql(&mut connection, &statement, &execution).await {
            connection.discard();
            return Err(error);
        }
        Ok(Self {
            connection: Some(connection),
            machine: TransactionMachine::new(),
            cleanup_options: execution,
        })
    }

    #[must_use]
    pub const fn state(&self) -> TransactionState {
        self.machine.state()
    }

    pub(crate) fn ensure_live(&self) -> Result<(), SqlError> {
        self.machine.ensure_live()
    }

    pub(crate) fn profile(&self) -> Result<&Arc<PostgresProfile>, SqlError> {
        Ok(&self.connection()?.profile)
    }

    pub(crate) fn connection(&self) -> Result<&PostgresConnection, SqlError> {
        self.connection.as_ref().ok_or_else(provider_error)
    }

    pub(crate) fn connection_mut(&mut self) -> Result<&mut PostgresConnection, SqlError> {
        self.connection.as_mut().ok_or_else(provider_error)
    }

    pub(crate) fn poison(&mut self) {
        self.machine.poison();
        if let Some(connection) = &self.connection
            && let Ok(native) = connection.native()
        {
            native
                .poisoned
                .store(true, std::sync::atomic::Ordering::Release);
        }
    }

    pub async fn execute(
        &mut self,
        request: ExecutionRequest<PostgresProfile>,
        options: ExecutionOptions,
    ) -> Result<ExecutionResult<PostgresMetadata>, SqlError> {
        self.ensure_live()?;
        let result = self.connection_mut()?.execute(request, options).await;
        self.poison_after_connection_failure(&result);
        result
    }

    pub async fn fetch_one(
        &mut self,
        request: ExecutionRequest<PostgresProfile>,
        options: ExecutionOptions,
    ) -> Result<PostgresRow, SqlError> {
        self.ensure_live()?;
        let result = self.connection_mut()?.fetch_one(request, options).await;
        self.poison_after_connection_failure(&result);
        result
    }

    pub async fn fetch_optional(
        &mut self,
        request: ExecutionRequest<PostgresProfile>,
        options: ExecutionOptions,
    ) -> Result<Option<PostgresRow>, SqlError> {
        self.ensure_live()?;
        let result = self
            .connection_mut()?
            .fetch_optional(request, options)
            .await;
        self.poison_after_connection_failure(&result);
        result
    }

    pub async fn fetch_all(
        &mut self,
        request: ExecutionRequest<PostgresProfile>,
        options: ExecutionOptions,
    ) -> Result<Vec<PostgresRow>, SqlError> {
        self.ensure_live()?;
        let result = self.connection_mut()?.fetch_all(request, options).await;
        self.poison_after_connection_failure(&result);
        result
    }

    pub async fn stream(
        &mut self,
        request: ExecutionRequest<PostgresProfile>,
        options: ExecutionOptions,
    ) -> Result<PostgresTransactionRowStream<'_>, SqlError> {
        PostgresTransactionRowStream::open(self, request, options).await
    }

    pub async fn savepoint(&mut self) -> Result<PostgresSavepoint<'_>, SqlError> {
        self.ensure_live()?;
        let depth = self.machine.push_savepoint()?;
        let name = format!("sifr_savepoint_{depth}");
        let options = self.cleanup_options.clone();
        if let Err(error) = run_control_sql(
            self.connection_mut()?,
            &format!("SAVEPOINT {name}"),
            &options,
        )
        .await
        {
            self.poison();
            return Err(error);
        }
        Ok(PostgresSavepoint {
            transaction: self,
            name,
            completed: false,
        })
    }

    pub async fn commit(mut self) -> Result<(), SqlError> {
        self.ensure_live()?;
        if self.machine.savepoint_depth() != 0 {
            return Err(SqlError::new(SqlErrorKind::TransactionControl));
        }
        let options = self.cleanup_options.clone();
        let result = run_control_sql(self.connection_mut()?, "COMMIT", &options).await;
        match result {
            Ok(()) => {
                self.machine.committed()?;
                self.release_connection().await
            }
            Err(error) => {
                self.poison();
                self.discard_connection();
                Err(error)
            }
        }
    }

    pub async fn rollback(mut self) -> Result<(), SqlError> {
        self.rollback_inner().await
    }

    pub async fn finish_context<T>(mut self, outcome: Result<T, SqlError>) -> Result<T, SqlError> {
        match outcome {
            Ok(value) => {
                self.commit().await?;
                Ok(value)
            }
            Err(mut primary) => match self.rollback_inner().await {
                Ok(()) => Err(primary),
                Err(cleanup) => {
                    if cleanup.secondary().is_empty() {
                        primary.extend_secondary([AsyncCleanupEvidence::cleanup_failed(
                            cleanup.to_string(),
                            "postgresql-transaction-context".to_string(),
                            "postgresql-transaction".to_string(),
                            "rollback".to_string(),
                            self.cleanup_options
                                .timeout
                                .unwrap_or(Duration::from_secs(1)),
                        )]);
                    } else {
                        primary.extend_secondary(cleanup.secondary().iter().cloned());
                    }
                    Err(primary)
                }
            },
        }
    }

    async fn rollback_inner(&mut self) -> Result<(), SqlError> {
        let was_poisoned = self.machine.state() == TransactionState::Poisoned;
        if !was_poisoned {
            self.ensure_live()?;
        }
        let options = self.cleanup_options.clone();
        let result = run_cleanup_sql(self.connection_mut()?, "ROLLBACK", &options).await;
        match result {
            Ok(()) => {
                if was_poisoned {
                    self.discard_connection();
                    Ok(())
                } else {
                    self.machine.rolled_back()?;
                    self.release_connection().await
                }
            }
            Err(error) => {
                self.poison();
                self.discard_connection();
                Err(error)
            }
        }
    }

    async fn release_connection(&mut self) -> Result<(), SqlError> {
        let connection = self.connection.take().ok_or_else(provider_error)?;
        connection
            .release(self.cleanup_options.cancellation.as_ref())
            .await
    }

    fn discard_connection(&mut self) {
        if let Some(connection) = self.connection.take() {
            connection.discard();
        }
    }

    fn poison_after_connection_failure<T>(&mut self, result: &Result<T, SqlError>) {
        if result.is_err() {
            self.poison();
        }
    }
}

impl Drop for PostgresTransaction {
    fn drop(&mut self) {
        if self.connection.is_some() {
            self.machine.dropped();
            self.discard_connection();
        }
    }
}

pub struct PostgresSavepoint<'transaction> {
    transaction: &'transaction mut PostgresTransaction,
    name: String,
    completed: bool,
}

impl PostgresSavepoint<'_> {
    pub async fn release(mut self) -> Result<(), SqlError> {
        let options = self.transaction.cleanup_options.clone();
        run_control_sql(
            self.transaction.connection_mut()?,
            &format!("RELEASE SAVEPOINT {}", self.name),
            &options,
        )
        .await?;
        self.transaction.machine.pop_savepoint()?;
        self.completed = true;
        Ok(())
    }

    pub async fn rollback(mut self) -> Result<(), SqlError> {
        let options = self.transaction.cleanup_options.clone();
        run_control_sql(
            self.transaction.connection_mut()?,
            &format!(
                "ROLLBACK TO SAVEPOINT {}; RELEASE SAVEPOINT {}",
                self.name, self.name
            ),
            &options,
        )
        .await?;
        self.transaction.machine.pop_savepoint()?;
        self.completed = true;
        Ok(())
    }
}

impl Drop for PostgresSavepoint<'_> {
    fn drop(&mut self) {
        if !self.completed {
            self.transaction.poison();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetryPolicy {
    max_attempts: u8,
    initial_backoff: Duration,
    maximum_backoff: Duration,
}

impl RetryPolicy {
    pub fn serialization(max_attempts: u8) -> Result<Self, SqlError> {
        Self::new(
            max_attempts,
            Duration::from_millis(10),
            Duration::from_secs(1),
        )
    }

    pub fn new(
        max_attempts: u8,
        initial_backoff: Duration,
        maximum_backoff: Duration,
    ) -> Result<Self, SqlError> {
        if max_attempts == 0
            || max_attempts > 16
            || initial_backoff.is_zero()
            || maximum_backoff.is_zero()
            || initial_backoff > maximum_backoff
            || maximum_backoff > Duration::from_secs(30)
        {
            return Err(SqlError::new(SqlErrorKind::Configuration));
        }
        Ok(Self {
            max_attempts,
            initial_backoff,
            maximum_backoff,
        })
    }
}

/// Implemented only by compiler-generated wrappers for validated `@retry_safe`
/// callbacks. The callback receives a fresh transaction on every invocation.
#[doc(hidden)]
pub trait RetrySafeCallback<T>: Clone {
    fn call<'transaction>(
        &'transaction self,
        transaction: &'transaction mut PostgresTransaction,
    ) -> Pin<Box<dyn Future<Output = Result<T, SqlError>> + 'transaction>>;
}

pub(crate) async fn run_retry_safe<T, C, Acquire, AcquireFuture>(
    callback: C,
    policy: RetryPolicy,
    mut acquire: Acquire,
) -> Result<T, SqlError>
where
    C: RetrySafeCallback<T>,
    Acquire: FnMut() -> AcquireFuture,
    AcquireFuture: Future<Output = Result<PostgresConnection, SqlError>>,
{
    let mut attempt = 1_u8;
    let mut backoff = policy.initial_backoff;
    loop {
        let connection = acquire().await?;
        let mut transaction =
            PostgresTransaction::begin(connection, TransactionOptions::default()).await?;
        let attempt_callback = callback.clone();
        let outcome = attempt_callback.call(&mut transaction).await;
        let result = transaction.finish_context(outcome).await;
        match result {
            Err(error)
                if error.retry_classification() == RetryClassification::RetryTransaction
                    && attempt < policy.max_attempts =>
            {
                tokio::time::sleep(backoff).await;
                attempt = attempt.saturating_add(1);
                backoff = backoff.saturating_mul(2).min(policy.maximum_backoff);
            }
            other => return other,
        }
    }
}

async fn run_control_sql(
    connection: &mut PostgresConnection,
    statement: &str,
    options: &ExecutionOptions,
) -> Result<(), SqlError> {
    let profile = Arc::clone(&connection.profile);
    let control = ControlHandle::new(connection.native()?, &profile);
    let operation = connection.native_mut()?.client.batch_execute(statement);
    run_controlled(&control, &profile, options, async {
        operation
            .await
            .map_err(|error| crate::error::map_postgres_error(&error))
    })
    .await
}

async fn run_cleanup_sql(
    connection: &mut PostgresConnection,
    statement: &str,
    options: &ExecutionOptions,
) -> Result<(), SqlError> {
    let budget = connection.profile.limits.cleanup_timeout;
    let result = tokio::time::timeout(
        budget,
        connection.native_mut()?.client.batch_execute(statement),
    )
    .await;
    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => {
            let mut mapped = crate::error::map_postgres_error(&error);
            let evidence = AsyncCleanupEvidence::cleanup_failed(
                mapped.to_string(),
                "postgresql-transaction-context".to_string(),
                "postgresql-transaction".to_string(),
                "rollback".to_string(),
                budget,
            );
            if let Some(carrier) = &options.cancellation {
                carrier.record_async_cleanup_evidence(evidence.clone());
            }
            mapped.extend_secondary([evidence]);
            Err(mapped)
        }
        Err(_) => {
            let evidence = AsyncCleanupEvidence::cleanup_timed_out(
                "postgresql-transaction-context".to_string(),
                "postgresql-transaction".to_string(),
                "rollback".to_string(),
                budget,
            );
            if let Some(carrier) = &options.cancellation {
                carrier.record_async_cleanup_evidence(evidence.clone());
            }
            Err(sifr_sql_runtime::SqlError::resource_limit(
                sifr_sql_runtime::ResourceLimitKind::CleanupDeadline,
            )
            .with_secondary(evidence))
        }
    }
}
