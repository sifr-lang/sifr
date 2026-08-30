use crate::codec::PostgresParameter;
use crate::connection::{
    ExecutionOptions, PostgresConnection, PostgresMetadata, PostgresNativeConnection, PostgresRow,
    decode_row, execution_result, prepare_statement,
};
use crate::control::{ControlHandle, run_controlled};
use crate::error::{map_postgres_error, provider_error};
use futures_core::Stream;
use postgres_types::ToSql;
use sifr_sql_runtime::{
    CardinalityViolation, ExecutionMode, ExecutionRequest, ExecutionResult, ResourceLimitKind,
    ResourceUsage, SqlError, SqlErrorKind, StatementCacheKey,
};
use std::future::poll_fn;
use std::sync::Arc;
use tokio_postgres::{RowStream, Statement};

impl PostgresConnection {
    pub async fn execute(
        &mut self,
        request: ExecutionRequest<PostgresProfileMarker>,
        options: ExecutionOptions,
    ) -> Result<ExecutionResult<PostgresMetadata>, SqlError> {
        if request.mode != ExecutionMode::Execute {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        self.validate_request(&request)?;
        let profile = Arc::clone(&self.profile);
        let control = ControlHandle::new(self.native()?, &profile);
        let operation = execute_native(self.native_mut()?, request);
        let result = run_controlled(&control, &profile, &options, operation).await;
        self.finish_operation(result)
    }

    pub async fn fetch_one(
        &mut self,
        request: ExecutionRequest<PostgresProfileMarker>,
        options: ExecutionOptions,
    ) -> Result<PostgresRow, SqlError> {
        if request.mode != ExecutionMode::FetchOne {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        let rows = self
            .fetch_bounded(
                request,
                options,
                1,
                OverflowPolicy::Cardinality(CardinalityViolation::ExpectedExactlyOneFoundMany),
            )
            .await?;
        let mut rows = rows.into_iter();
        let Some(row) = rows.next() else {
            return Err(SqlError::cardinality(
                CardinalityViolation::ExpectedExactlyOneFoundZero,
            ));
        };
        if rows.next().is_some() {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        Ok(row)
    }

    pub async fn fetch_optional(
        &mut self,
        request: ExecutionRequest<PostgresProfileMarker>,
        options: ExecutionOptions,
    ) -> Result<Option<PostgresRow>, SqlError> {
        if request.mode != ExecutionMode::FetchOptional {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        let rows = self
            .fetch_bounded(
                request,
                options,
                1,
                OverflowPolicy::Cardinality(CardinalityViolation::ExpectedAtMostOneFoundMany),
            )
            .await?;
        let mut rows = rows.into_iter();
        let first = rows.next();
        if rows.next().is_some() {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        Ok(first)
    }

    pub async fn fetch_all(
        &mut self,
        request: ExecutionRequest<PostgresProfileMarker>,
        options: ExecutionOptions,
    ) -> Result<Vec<PostgresRow>, SqlError> {
        let ExecutionMode::FetchAll { maximum_rows } = request.mode else {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        };
        self.fetch_bounded(
            request,
            options,
            maximum_rows,
            OverflowPolicy::CollectedRows,
        )
        .await
    }

    pub async fn warm(
        &mut self,
        request: &ExecutionRequest<PostgresProfileMarker>,
        options: ExecutionOptions,
    ) -> Result<PostgresMetadata, SqlError> {
        self.validate_request(request)?;
        let profile = Arc::clone(&self.profile);
        let control = ControlHandle::new(self.native()?, &profile);
        let statement = Arc::clone(&request.statement);
        let key = cache_key(request, self.native()?.server_version)?;
        let operation = async {
            let native = self.native_mut()?;
            let (_, cache_hit) = prepare_statement(native, key, &statement).await?;
            Ok(PostgresMetadata {
                statement_cache_hit: cache_hit,
                server_version: native.server_version,
            })
        };
        run_controlled(&control, &profile, &options, operation).await
    }

    async fn fetch_bounded(
        &mut self,
        request: ExecutionRequest<PostgresProfileMarker>,
        options: ExecutionOptions,
        maximum_rows: u64,
        overflow: OverflowPolicy,
    ) -> Result<Vec<PostgresRow>, SqlError> {
        self.validate_request(&request)?;
        let ceiling = maximum_rows.min(self.profile.limits.max_collected_rows);
        if ceiling == 0 {
            return Err(SqlError::resource_limit(ResourceLimitKind::CollectedRows));
        }
        let profile = Arc::clone(&self.profile);
        let limits = profile.limits;
        let control = ControlHandle::new(self.native()?, &profile);
        let operation = fetch_native(self.native_mut()?, request, ceiling, limits, overflow);
        let result = run_controlled(&control, &profile, &options, operation).await;
        self.finish_operation(result)
    }

    pub(crate) fn validate_request(
        &self,
        request: &ExecutionRequest<PostgresProfileMarker>,
    ) -> Result<(), SqlError> {
        request.validate()?;
        if request.metadata.schema_fingerprint != self.schema_fingerprint()
            || request.profile.schema_fingerprint() != self.schema_fingerprint()
        {
            return Err(SqlError::new(SqlErrorKind::SchemaContract));
        }
        Ok(())
    }

    pub(crate) fn finish_operation<T>(&self, result: Result<T, SqlError>) -> Result<T, SqlError> {
        if self.is_poisoned() && result.is_ok() {
            Err(SqlError::new(SqlErrorKind::Cancelled))
        } else {
            result
        }
    }
}

pub(crate) type PostgresProfileMarker = crate::config::PostgresProfile;

async fn execute_native(
    native: &mut PostgresNativeConnection,
    request: ExecutionRequest<PostgresProfileMarker>,
) -> Result<ExecutionResult<PostgresMetadata>, SqlError> {
    let key = cache_key(&request, native.server_version)?;
    let (statement, cache_hit) = prepare_statement(native, key, &request.statement).await?;
    let parameters = parameters_for(
        &statement,
        request.parameters.into_values(),
        request.profile.limits,
    )?;
    let references = parameter_references(&parameters);
    let rows = native
        .client
        .execute(&statement, &references)
        .await
        .map_err(|error| map_postgres_error(&error))?;
    Ok(execution_result(
        Some(rows),
        cache_hit,
        native.server_version,
    ))
}

async fn fetch_native(
    native: &mut PostgresNativeConnection,
    request: ExecutionRequest<PostgresProfileMarker>,
    maximum_rows: u64,
    limits: sifr_sql_runtime::RuntimeLimits,
    overflow: OverflowPolicy,
) -> Result<Vec<PostgresRow>, SqlError> {
    let key = cache_key(&request, native.server_version)?;
    let (statement, _) = prepare_statement(native, key, &request.statement).await?;
    let parameters = parameters_for(&statement, request.parameters.into_values(), limits)?;
    let references = parameter_references(&parameters);
    let stream = native
        .client
        .query_raw(&statement, references)
        .await
        .map_err(|error| map_postgres_error(&error))?;
    collect_rows(stream, maximum_rows, limits, overflow).await
}

#[derive(Clone, Copy)]
pub(crate) enum OverflowPolicy {
    Cardinality(CardinalityViolation),
    CollectedRows,
}

pub(crate) async fn collect_rows(
    stream: RowStream,
    maximum_rows: u64,
    limits: sifr_sql_runtime::RuntimeLimits,
    overflow: OverflowPolicy,
) -> Result<Vec<PostgresRow>, SqlError> {
    let mut stream = Box::pin(stream);
    let mut rows = Vec::new();
    let mut usage = ResourceUsage::default();
    loop {
        let next = poll_fn(|context| stream.as_mut().poll_next(context)).await;
        let Some(next) = next else {
            return Ok(rows);
        };
        let row = next.map_err(|error| map_postgres_error(&error))?;
        let row = decode_row(&row, limits)?;
        usage.account_row(row.decoded_bytes(), limits)?;
        if usage.collected_rows() > maximum_rows {
            return Err(match overflow {
                OverflowPolicy::Cardinality(violation) => SqlError::cardinality(violation),
                OverflowPolicy::CollectedRows => {
                    SqlError::resource_limit(ResourceLimitKind::CollectedRows)
                }
            });
        }
        rows.push(row);
    }
}

pub(crate) fn parameters_for(
    statement: &Statement,
    values: Vec<sifr_sql_runtime::OwnedParameter>,
    limits: sifr_sql_runtime::RuntimeLimits,
) -> Result<Vec<PostgresParameter>, SqlError> {
    if statement.params().len() != values.len() {
        return Err(SqlError::new(SqlErrorKind::Encode));
    }
    let mut usage = ResourceUsage::default();
    usage.account_parameters(
        u32::try_from(values.len()).map_err(|_| provider_error())?,
        limits,
    )?;
    Ok(values
        .into_iter()
        .map(|parameter| PostgresParameter(parameter.value))
        .collect())
}

pub(crate) fn parameter_references(parameters: &[PostgresParameter]) -> Vec<&(dyn ToSql + Sync)> {
    parameters
        .iter()
        .map(|parameter| parameter as &(dyn ToSql + Sync))
        .collect()
}

pub(crate) fn cache_key(
    request: &ExecutionRequest<PostgresProfileMarker>,
    server_version: u32,
) -> Result<StatementCacheKey, SqlError> {
    StatementCacheKey {
        normalized_statement_fingerprint: request.metadata.normalized_statement_fingerprint.clone(),
        parameter_type_fingerprint: request.metadata.parameter_type_fingerprint.clone(),
        result_type_fingerprint: request.metadata.result_type_fingerprint.clone(),
        provider_version: server_version.to_string(),
        schema_fingerprint: request.metadata.schema_fingerprint.clone(),
    }
    .validate()
}

impl PostgresRow {
    pub fn into_scalar(mut self) -> Result<sifr_sql_runtime::OwnedSqlValue, SqlError> {
        if self.values.len() != 1 {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        self.values.pop().ok_or_else(provider_error)
    }
}
