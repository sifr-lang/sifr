use crate::connection::{ExecutionOptions, MysqlConnection, MysqlMetadata, MysqlRow};
use crate::error::{map_mysql_error, provider_error};
use crate::stream::MysqlTransactionRowStream;
use mysql_async::prelude::Queryable;
use sifr_sql_runtime::{
    ExecutionRequest, ExecutionResult, RetryClassification, SqlError, SqlErrorKind,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::time::Duration;

pub struct MysqlTransaction {
    connection: Option<MysqlConnection>,
    finished: bool,
}

impl MysqlTransaction {
    pub(crate) async fn begin(mut connection: MysqlConnection) -> Result<Self, SqlError> {
        connection
            .native_mut()?
            .conn
            .query_drop("START TRANSACTION")
            .await
            .map_err(|error| map_mysql_error(&error))?;
        Ok(Self {
            connection: Some(connection),
            finished: false,
        })
    }

    pub async fn execute(
        &mut self,
        request: ExecutionRequest<crate::MysqlProfile>,
        options: ExecutionOptions,
    ) -> Result<ExecutionResult<MysqlMetadata>, SqlError> {
        self.ensure_live()?;
        self.connection_mut()?.execute(request, options).await
    }

    pub async fn fetch_all(
        &mut self,
        request: ExecutionRequest<crate::MysqlProfile>,
        options: ExecutionOptions,
    ) -> Result<Vec<MysqlRow>, SqlError> {
        self.ensure_live()?;
        self.connection_mut()?.fetch_all(request, options).await
    }

    pub async fn stream(
        &mut self,
        request: ExecutionRequest<crate::MysqlProfile>,
        options: ExecutionOptions,
    ) -> Result<MysqlTransactionRowStream<'_>, SqlError> {
        self.ensure_live()?;
        MysqlTransactionRowStream::open(self, request, options).await
    }

    pub async fn savepoint<'a>(&'a mut self, name: &str) -> Result<MysqlSavepoint<'a>, SqlError> {
        self.ensure_live()?;
        let quoted = quote_savepoint(name)?;
        self.connection
            .as_mut()
            .ok_or_else(provider_error)?
            .native_mut()?
            .conn
            .query_drop(format!("SAVEPOINT {quoted}"))
            .await
            .map_err(|error| map_mysql_error(&error))?;
        Ok(MysqlSavepoint {
            transaction: self,
            name: quoted,
            finished: false,
        })
    }

    pub async fn commit(mut self) -> Result<(), SqlError> {
        self.ensure_live()?;
        let mut connection = self.connection.take().ok_or_else(provider_error)?;
        connection
            .native_mut()?
            .conn
            .query_drop("COMMIT")
            .await
            .map_err(|error| map_mysql_error(&error))?;
        self.finished = true;
        connection.release(None).await
    }

    pub async fn rollback(mut self) -> Result<(), SqlError> {
        self.ensure_live()?;
        let mut connection = self.connection.take().ok_or_else(provider_error)?;
        connection
            .native_mut()?
            .conn
            .query_drop("ROLLBACK")
            .await
            .map_err(|error| map_mysql_error(&error))?;
        self.finished = true;
        connection.release(None).await
    }

    pub(crate) fn ensure_live(&self) -> Result<(), SqlError> {
        let connection = self.connection.as_ref().ok_or_else(provider_error)?;
        if self.finished || connection.is_poisoned() {
            Err(SqlError::new(SqlErrorKind::Connection))
        } else {
            Ok(())
        }
    }

    pub(crate) fn connection_mut(&mut self) -> Result<&mut MysqlConnection, SqlError> {
        self.connection.as_mut().ok_or_else(provider_error)
    }
}

impl Drop for MysqlTransaction {
    fn drop(&mut self) {
        if !self.finished {
            if let Some(connection) = self.connection.take() {
                connection.discard();
            }
        }
    }
}

pub struct MysqlSavepoint<'a> {
    transaction: &'a mut MysqlTransaction,
    name: String,
    finished: bool,
}

impl MysqlSavepoint<'_> {
    pub async fn release(mut self) -> Result<(), SqlError> {
        let statement = format!("RELEASE SAVEPOINT {}", self.name);
        self.connection()?
            .query_drop(statement)
            .await
            .map_err(|error| map_mysql_error(&error))?;
        self.finished = true;
        Ok(())
    }

    pub async fn rollback(mut self) -> Result<(), SqlError> {
        let statement = format!("ROLLBACK TO SAVEPOINT {}", self.name);
        self.connection()?
            .query_drop(statement)
            .await
            .map_err(|error| map_mysql_error(&error))?;
        self.finished = true;
        Ok(())
    }

    fn connection(&mut self) -> Result<&mut mysql_async::Conn, SqlError> {
        Ok(&mut self
            .transaction
            .connection
            .as_mut()
            .ok_or_else(provider_error)?
            .native_mut()?
            .conn)
    }
}

impl Drop for MysqlSavepoint<'_> {
    fn drop(&mut self) {
        if !self.finished
            && let Ok(connection) = self.transaction.connection_mut()
            && let Ok(native) = connection.native_mut()
        {
            native.poisoned.store(true, Ordering::Release);
        }
    }
}

fn quote_savepoint(value: &str) -> Result<String, SqlError> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(provider_error());
    }
    Ok(format!("`{value}`"))
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

/// Implemented only by compiler-generated wrappers for checked retry-safe code.
#[doc(hidden)]
pub trait RetrySafeCallback<T>: Clone {
    fn call<'transaction>(
        &'transaction self,
        transaction: &'transaction mut MysqlTransaction,
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
    AcquireFuture: Future<Output = Result<MysqlConnection, SqlError>>,
{
    let mut attempt = 1_u8;
    let mut backoff = policy.initial_backoff;
    loop {
        let mut transaction = MysqlTransaction::begin(acquire().await?).await?;
        let outcome = callback.clone().call(&mut transaction).await;
        let result = match outcome {
            Ok(value) => transaction.commit().await.map(|()| value),
            Err(error) => transaction
                .rollback()
                .await
                .map_or(Err(error.clone()), |()| Err(error)),
        };
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
