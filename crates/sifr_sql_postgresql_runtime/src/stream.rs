use crate::config::PostgresProfile;
use crate::connection::{
    ExecutionOptions, PostgresConnection, PostgresRow, decode_row, prepare_statement,
};
use crate::control::{ControlHandle, run_controlled};
use crate::error::{map_postgres_error, provider_error};
use crate::execute::{cache_key, parameter_references, parameters_for};
use crate::transaction::PostgresTransaction;
use futures_core::Stream;
use sifr_sql_runtime::{
    ExecutionMode, ExecutionRequest, ResourceLimitKind, ResourceUsage, SqlError, SqlErrorKind,
};
use std::future::poll_fn;
use std::pin::Pin;
use std::sync::Arc;
use tokio_postgres::RowStream;

/// A pool-created stream owns its connection until `aclose` completes.
pub struct PostgresRowStream {
    connection: Option<PostgresConnection>,
    stream: Option<Pin<Box<RowStream>>>,
    options: ExecutionOptions,
    usage: ResourceUsage,
    maximum_rows: u64,
    exhausted: bool,
}

impl PostgresRowStream {
    pub(crate) async fn open(
        mut connection: PostgresConnection,
        request: ExecutionRequest<PostgresProfile>,
        options: ExecutionOptions,
    ) -> Result<Self, Box<(PostgresConnection, SqlError)>> {
        if request.mode != ExecutionMode::Stream {
            return Err(Box::new((
                connection,
                SqlError::new(SqlErrorKind::Cardinality),
            )));
        }
        if let Err(error) = connection.validate_request(&request) {
            return Err(Box::new((connection, error)));
        }
        let maximum_rows = request
            .cardinality
            .maximum
            .unwrap_or(connection.profile.limits.max_collected_rows)
            .min(connection.profile.limits.max_collected_rows);
        let profile = Arc::clone(&connection.profile);
        let control = match connection.native() {
            Ok(native) => ControlHandle::new(native, &profile),
            Err(error) => return Err(Box::new((connection, error))),
        };
        let operation = open_native_stream(&mut connection, request);
        let stream = match run_controlled(&control, &profile, &options, operation).await {
            Ok(stream) => stream,
            Err(error) => return Err(Box::new((connection, error))),
        };
        Ok(Self {
            connection: Some(connection),
            stream: Some(Box::pin(stream)),
            options,
            usage: ResourceUsage::default(),
            maximum_rows,
            exhausted: false,
        })
    }

    pub async fn next(&mut self) -> Result<Option<PostgresRow>, SqlError> {
        if self.exhausted {
            return Ok(None);
        }
        let connection = self.connection.as_ref().ok_or_else(provider_error)?;
        let profile = Arc::clone(&connection.profile);
        let control = ControlHandle::new(connection.native()?, &profile);
        let stream = self.stream.as_mut().ok_or_else(provider_error)?;
        let operation = async { Ok(poll_fn(|context| stream.as_mut().poll_next(context)).await) };
        let next = run_controlled(&control, &profile, &self.options, operation).await?;
        let Some(row) = next else {
            self.exhausted = true;
            self.stream.take();
            return Ok(None);
        };
        let row = row.map_err(|error| map_postgres_error(&error))?;
        let row = decode_row(&row, profile.limits)?;
        self.usage
            .account_row(row.decoded_bytes(), profile.limits)?;
        if self.usage.collected_rows() > self.maximum_rows {
            if let Some(connection) = &self.connection {
                connection
                    .native()?
                    .poisoned
                    .store(true, std::sync::atomic::Ordering::Release);
            }
            return Err(SqlError::resource_limit(ResourceLimitKind::CollectedRows));
        }
        Ok(Some(row))
    }

    pub async fn aclose(mut self) -> Result<(), SqlError> {
        self.stream.take();
        let connection = self.connection.take().ok_or_else(provider_error)?;
        connection.release(self.options.cancellation.as_ref()).await
    }
}

impl Drop for PostgresRowStream {
    fn drop(&mut self) {
        self.stream.take();
        if let Some(connection) = self.connection.take() {
            connection.discard();
        }
    }
}

/// A transaction stream keeps the transaction mutably borrowed until close.
pub struct PostgresTransactionRowStream<'transaction> {
    transaction: &'transaction mut PostgresTransaction,
    stream: Option<Pin<Box<RowStream>>>,
    options: ExecutionOptions,
    usage: ResourceUsage,
    maximum_rows: u64,
    exhausted: bool,
}

impl<'transaction> PostgresTransactionRowStream<'transaction> {
    pub(crate) async fn open(
        transaction: &'transaction mut PostgresTransaction,
        request: ExecutionRequest<PostgresProfile>,
        options: ExecutionOptions,
    ) -> Result<Self, SqlError> {
        transaction.ensure_live()?;
        if request.mode != ExecutionMode::Stream {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        transaction.connection()?.validate_request(&request)?;
        let maximum_rows = request
            .cardinality
            .maximum
            .unwrap_or(transaction.profile()?.limits.max_collected_rows)
            .min(transaction.profile()?.limits.max_collected_rows);
        let profile = Arc::clone(transaction.profile()?);
        let control = ControlHandle::new(transaction.connection()?.native()?, &profile);
        let operation = open_native_stream(transaction.connection_mut()?, request);
        let stream = run_controlled(&control, &profile, &options, operation).await?;
        Ok(Self {
            transaction,
            stream: Some(Box::pin(stream)),
            options,
            usage: ResourceUsage::default(),
            maximum_rows,
            exhausted: false,
        })
    }

    pub async fn next(&mut self) -> Result<Option<PostgresRow>, SqlError> {
        if self.exhausted {
            return Ok(None);
        }
        let profile = Arc::clone(self.transaction.profile()?);
        let control = ControlHandle::new(self.transaction.connection()?.native()?, &profile);
        let stream = self.stream.as_mut().ok_or_else(provider_error)?;
        let operation = async { Ok(poll_fn(|context| stream.as_mut().poll_next(context)).await) };
        let next = match run_controlled(&control, &profile, &self.options, operation).await {
            Ok(next) => next,
            Err(error) => {
                self.transaction.poison();
                return Err(error);
            }
        };
        let Some(row) = next else {
            self.exhausted = true;
            self.stream.take();
            return Ok(None);
        };
        let row = row.map_err(|error| map_postgres_error(&error))?;
        let row = decode_row(&row, profile.limits)?;
        self.usage
            .account_row(row.decoded_bytes(), profile.limits)?;
        if self.usage.collected_rows() > self.maximum_rows {
            self.transaction.poison();
            return Err(SqlError::resource_limit(ResourceLimitKind::CollectedRows));
        }
        Ok(Some(row))
    }

    pub fn close(mut self) {
        self.stream.take();
        self.exhausted = true;
    }
}

impl Drop for PostgresTransactionRowStream<'_> {
    fn drop(&mut self) {
        if !self.exhausted && self.stream.take().is_some() {
            self.transaction.poison();
        }
    }
}

async fn open_native_stream(
    connection: &mut PostgresConnection,
    request: ExecutionRequest<PostgresProfile>,
) -> Result<RowStream, SqlError> {
    let limits = connection.profile.limits;
    let native = connection.native_mut()?;
    let key = cache_key(&request, native.server_version)?;
    let (statement, _) = prepare_statement(native, key, &request.statement).await?;
    let parameters = parameters_for(&statement, request.parameters.into_values(), limits)?;
    let references = parameter_references(&parameters);
    native
        .client
        .query_raw(&statement, references)
        .await
        .map_err(|error| map_postgres_error(&error))
}
