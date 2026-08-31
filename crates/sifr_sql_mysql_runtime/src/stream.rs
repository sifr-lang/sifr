use crate::connection::{
    ExecutionOptions, MysqlConnection, MysqlNativeConnection, MysqlRow, prepare_statement,
    statement_key, validate_request,
};
use crate::control::{ControlHandle, run_controlled};
use crate::error::{map_mysql_error, provider_error};
use crate::{MysqlProfile, MysqlTransaction, codec::decode_row};
use futures_core::Stream;
use mysql_async::prelude::{Query, WithParams};
use mysql_async::{BinaryProtocol, Params, ResultSetStream, Row};
use sifr_sql_runtime::{
    ExecutionMode, ExecutionRequest, PoolDiscardGuard, ResourceLimitKind, ResourceUsage, SqlError,
    SqlErrorKind,
};
use std::future::{Ready, poll_fn, ready};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

type NativeRowStream = ResultSetStream<'static, 'static, 'static, Row, BinaryProtocol>;

/// A consumer-driven MySQL row stream.
///
/// `mysql_async` owns the raw connection for the lifetime of its stream. The
/// pool therefore keeps the slot occupied and discards that connection when
/// the stream ends. No checked-out connection crosses a task boundary.
pub struct MysqlRowStream {
    stream: Option<Pin<Box<NativeRowStream>>>,
    discard_guard: Option<PoolDiscardGuard<MysqlNativeConnection>>,
    control: ControlHandle,
    profile: Arc<MysqlProfile>,
    options: ExecutionOptions,
    usage: ResourceUsage,
    maximum_rows: u64,
    exhausted: bool,
}

impl MysqlRowStream {
    pub(crate) async fn open(
        mut connection: MysqlConnection,
        request: ExecutionRequest<MysqlProfile>,
        options: ExecutionOptions,
    ) -> Result<Self, SqlError> {
        if request.mode != ExecutionMode::Stream {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        validate_request(&request, &connection.profile)?;
        let maximum_rows = request
            .cardinality
            .maximum
            .unwrap_or(connection.profile.limits.max_collected_rows)
            .min(connection.profile.limits.max_collected_rows);
        let key = statement_key(&request, connection.native()?.server_version)?;
        let parameters = crate::codec::encode_parameters(request.parameters)?;
        let statement = {
            let native = connection.native_mut()?;
            prepare_statement(native, key, &request.statement).await?.0
        };
        let profile = Arc::clone(&connection.profile);
        let control = connection.control()?;
        let (native, discard_guard) = connection.detach_for_stream()?;
        let operation = async {
            statement
                .with(Params::Positional(parameters))
                .run(native.conn)
                .await
                .map_err(|error| map_mysql_error(&error))?
                .stream_and_drop::<Row>()
                .await
                .map_err(|error| map_mysql_error(&error))?
                .ok_or_else(|| SqlError::new(SqlErrorKind::Cardinality))
        };
        let stream = run_controlled(&control, &profile, &options, operation).await?;
        Ok(Self {
            stream: Some(Box::pin(stream)),
            discard_guard: Some(discard_guard),
            control,
            profile,
            options,
            usage: ResourceUsage::default(),
            maximum_rows,
            exhausted: false,
        })
    }

    pub async fn next(&mut self) -> Result<Option<MysqlRow>, SqlError> {
        if self.exhausted {
            return Ok(None);
        }
        let stream = self.stream.as_mut().ok_or_else(provider_error)?;
        let operation = async { Ok(poll_fn(|context| stream.as_mut().poll_next(context)).await) };
        let next =
            match run_controlled(&self.control, &self.profile, &self.options, operation).await {
                Ok(next) => next,
                Err(error) => return Err(self.fail(error)),
            };
        let Some(row) = next else {
            self.exhausted = true;
            self.stream.take();
            self.discard_guard.take();
            return Ok(None);
        };
        let row = match row {
            Ok(row) => row,
            Err(error) => return Err(self.fail(map_mysql_error(&error))),
        };
        let (values, decoded_bytes) = match decode_row(&row, self.profile.limits) {
            Ok(row) => row,
            Err(error) => return Err(self.fail(error)),
        };
        if let Err(error) = self.usage.account_row(decoded_bytes, self.profile.limits) {
            return Err(self.fail(error));
        }
        if self.usage.collected_rows() > self.maximum_rows {
            return Err(self.fail(SqlError::resource_limit(ResourceLimitKind::CollectedRows)));
        }
        Ok(Some(MysqlRow {
            values,
            decoded_bytes,
        }))
    }

    pub fn aclose(mut self) -> Ready<Result<(), SqlError>> {
        self.exhausted = true;
        self.stream.take();
        self.discard_guard.take();
        ready(Ok(()))
    }

    fn fail(&mut self, error: SqlError) -> SqlError {
        self.exhausted = true;
        self.stream.take();
        self.discard_guard.take();
        error
    }
}

impl Drop for MysqlRowStream {
    fn drop(&mut self) {
        self.stream.take();
        self.discard_guard.take();
    }
}

type BorrowedNativeRowStream<'transaction> =
    ResultSetStream<'transaction, 'transaction, 'static, Row, BinaryProtocol>;

/// A transaction stream keeps the transaction connection mutably borrowed.
/// Dropping an incomplete stream poisons the transaction.
pub struct MysqlTransactionRowStream<'transaction> {
    stream: Option<Pin<Box<BorrowedNativeRowStream<'transaction>>>>,
    poison: Arc<AtomicBool>,
    control: ControlHandle,
    profile: Arc<MysqlProfile>,
    options: ExecutionOptions,
    usage: ResourceUsage,
    maximum_rows: u64,
    exhausted: bool,
}

impl<'transaction> MysqlTransactionRowStream<'transaction> {
    pub(crate) async fn open(
        transaction: &'transaction mut MysqlTransaction,
        request: ExecutionRequest<MysqlProfile>,
        options: ExecutionOptions,
    ) -> Result<Self, SqlError> {
        transaction.ensure_live()?;
        if request.mode != ExecutionMode::Stream {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        let connection = transaction.connection_mut()?;
        validate_request(&request, &connection.profile)?;
        let maximum_rows = request
            .cardinality
            .maximum
            .unwrap_or(connection.profile.limits.max_collected_rows)
            .min(connection.profile.limits.max_collected_rows);
        let key = statement_key(&request, connection.native()?.server_version)?;
        let parameters = crate::codec::encode_parameters(request.parameters)?;
        let profile = Arc::clone(&connection.profile);
        let control = connection.control()?;
        let poison = Arc::clone(&connection.native()?.poisoned);
        let statement = {
            let native = connection.native_mut()?;
            prepare_statement(native, key, &request.statement).await?.0
        };
        let native = connection.native_mut()?;
        let operation = async {
            statement
                .with(Params::Positional(parameters))
                .run(&mut native.conn)
                .await
                .map_err(|error| map_mysql_error(&error))?
                .stream_and_drop::<Row>()
                .await
                .map_err(|error| map_mysql_error(&error))?
                .ok_or_else(|| SqlError::new(SqlErrorKind::Cardinality))
        };
        let stream = run_controlled(&control, &profile, &options, operation).await?;
        Ok(Self {
            stream: Some(Box::pin(stream)),
            poison,
            control,
            profile,
            options,
            usage: ResourceUsage::default(),
            maximum_rows,
            exhausted: false,
        })
    }

    pub async fn next(&mut self) -> Result<Option<MysqlRow>, SqlError> {
        if self.exhausted {
            return Ok(None);
        }
        let stream = self.stream.as_mut().ok_or_else(provider_error)?;
        let operation = async { Ok(poll_fn(|context| stream.as_mut().poll_next(context)).await) };
        let next =
            match run_controlled(&self.control, &self.profile, &self.options, operation).await {
                Ok(next) => next,
                Err(error) => return Err(self.fail(error)),
            };
        let Some(row) = next else {
            self.exhausted = true;
            self.stream.take();
            return Ok(None);
        };
        let row = match row {
            Ok(row) => row,
            Err(error) => return Err(self.fail(map_mysql_error(&error))),
        };
        let (values, decoded_bytes) = match decode_row(&row, self.profile.limits) {
            Ok(row) => row,
            Err(error) => return Err(self.fail(error)),
        };
        if let Err(error) = self.usage.account_row(decoded_bytes, self.profile.limits) {
            return Err(self.fail(error));
        }
        if self.usage.collected_rows() > self.maximum_rows {
            return Err(self.fail(SqlError::resource_limit(ResourceLimitKind::CollectedRows)));
        }
        Ok(Some(MysqlRow {
            values,
            decoded_bytes,
        }))
    }

    pub async fn aclose(mut self) -> Result<(), SqlError> {
        while self.next().await?.is_some() {}
        Ok(())
    }

    fn fail(&mut self, error: SqlError) -> SqlError {
        self.poison.store(true, Ordering::Release);
        self.exhausted = true;
        self.stream.take();
        error
    }
}

impl Drop for MysqlTransactionRowStream<'_> {
    fn drop(&mut self) {
        if !self.exhausted && self.stream.take().is_some() {
            self.poison.store(true, Ordering::Release);
        }
    }
}
