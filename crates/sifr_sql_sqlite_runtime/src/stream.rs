use crate::config::SqliteProfile;
use crate::pool::{ExecutionOptions, SqliteConnection, take_parameters};
use crate::transaction::SqliteTransaction;
use crate::worker::{SqliteRow, WorkerRowStream};
use sifr_sql_runtime::{
    ExecutionMode, ExecutionRequest, ResourceLimitKind, SqlError, SqlErrorKind,
};

/// A consumer-driven SQLite stream backed by a one-row worker channel.
pub struct SqliteRowStream {
    connection: Option<SqliteConnection>,
    worker: Option<WorkerRowStream>,
    exhausted: bool,
}

impl SqliteRowStream {
    pub(crate) async fn open(
        connection: SqliteConnection,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<Self, SqlError> {
        if request.mode != ExecutionMode::Stream {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        connection.validate_request(&request)?;
        let maximum_rows = request
            .cardinality
            .maximum
            .unwrap_or(connection.profile.limits().max_collected_rows)
            .min(connection.profile.limits().max_collected_rows);
        if maximum_rows == 0 {
            return Err(SqlError::resource_limit(ResourceLimitKind::CollectedRows));
        }
        let timeout = options.deadline(&connection.profile)?;
        let parameters = take_parameters(request.parameters, connection.profile.limits())?;
        let mut limits = connection.profile.limits();
        limits.max_collected_rows = maximum_rows;
        let worker = connection
            .worker()?
            .stream(
                request.statement.to_string(),
                parameters,
                limits,
                timeout,
                options.cancellation.as_ref(),
            )
            .await?;
        Ok(Self {
            connection: Some(connection),
            worker: Some(worker),
            exhausted: false,
        })
    }

    pub async fn next(&mut self) -> Result<Option<SqliteRow>, SqlError> {
        if self.exhausted {
            return Ok(None);
        }
        let result = self
            .worker
            .as_mut()
            .ok_or_else(provider_error)?
            .next()
            .await;
        match result {
            Ok(Some(row)) => Ok(Some(row)),
            Ok(None) => {
                self.exhausted = true;
                self.worker.take();
                self.release_connection().await?;
                Ok(None)
            }
            Err(mut primary) => {
                self.exhausted = true;
                self.worker.take();
                if let Err(cleanup) = self.finish_failed().await {
                    primary.extend_secondary(cleanup.secondary().iter().cloned());
                }
                Err(primary)
            }
        }
    }

    pub async fn aclose(mut self) -> Result<(), SqlError> {
        let close = match self.worker.as_mut() {
            Some(worker) => worker.close().await,
            None => Ok(()),
        };
        self.worker.take();
        self.exhausted = true;
        match close {
            Ok(()) => self.release_connection().await,
            Err(mut primary) => {
                if let Err(cleanup) = self.finish_failed().await {
                    primary.extend_secondary(cleanup.secondary().iter().cloned());
                }
                Err(primary)
            }
        }
    }

    async fn release_connection(&mut self) -> Result<(), SqlError> {
        self.connection
            .take()
            .ok_or_else(provider_error)?
            .release(None)
            .await
    }

    async fn finish_failed(&mut self) -> Result<(), SqlError> {
        let connection = self.connection.take().ok_or_else(provider_error)?;
        if connection.is_poisoned() {
            connection.discard();
            Ok(())
        } else {
            connection.release(None).await
        }
    }
}

impl Drop for SqliteRowStream {
    fn drop(&mut self) {
        self.worker.take();
        if let Some(connection) = self.connection.take() {
            connection.discard();
        }
    }
}

fn provider_error() -> SqlError {
    SqlError::new(SqlErrorKind::Provider)
}

/// A SQLite stream that keeps its transaction mutably borrowed until close.
pub struct SqliteTransactionRowStream<'transaction> {
    transaction: &'transaction mut SqliteTransaction,
    worker: Option<WorkerRowStream>,
    exhausted: bool,
}

impl<'transaction> SqliteTransactionRowStream<'transaction> {
    pub(crate) async fn open(
        transaction: &'transaction mut SqliteTransaction,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<Self, SqlError> {
        if request.mode != ExecutionMode::Stream {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        let connection = transaction.connection_mut()?;
        connection.validate_request(&request)?;
        let maximum_rows = request
            .cardinality
            .maximum
            .unwrap_or(connection.profile.limits().max_collected_rows)
            .min(connection.profile.limits().max_collected_rows);
        if maximum_rows == 0 {
            return Err(SqlError::resource_limit(ResourceLimitKind::CollectedRows));
        }
        let timeout = options.deadline(&connection.profile)?;
        let parameters = take_parameters(request.parameters, connection.profile.limits())?;
        let mut limits = connection.profile.limits();
        limits.max_collected_rows = maximum_rows;
        let worker = connection
            .worker()?
            .stream(
                request.statement.to_string(),
                parameters,
                limits,
                timeout,
                options.cancellation.as_ref(),
            )
            .await?;
        Ok(Self {
            transaction,
            worker: Some(worker),
            exhausted: false,
        })
    }

    pub async fn next(&mut self) -> Result<Option<SqliteRow>, SqlError> {
        if self.exhausted {
            return Ok(None);
        }
        match self
            .worker
            .as_mut()
            .ok_or_else(provider_error)?
            .next()
            .await
        {
            Ok(Some(row)) => Ok(Some(row)),
            Ok(None) => {
                self.exhausted = true;
                self.worker.take();
                Ok(None)
            }
            Err(error) => {
                self.exhausted = true;
                self.worker.take();
                Err(error)
            }
        }
    }

    pub async fn aclose(mut self) -> Result<(), SqlError> {
        if let Some(worker) = self.worker.as_mut() {
            worker.close().await?;
        }
        self.worker.take();
        self.exhausted = true;
        Ok(())
    }

    #[must_use]
    pub fn transaction_is_poisoned(&mut self) -> bool {
        match self.transaction.connection_mut() {
            Ok(connection) => connection.is_poisoned(),
            Err(_) => true,
        }
    }
}

impl Drop for SqliteTransactionRowStream<'_> {
    fn drop(&mut self) {
        if !self.exhausted {
            if let Some(worker) = self.worker.as_mut() {
                worker.abort();
            }
        }
        self.worker.take();
    }
}
