use crate::{BoundParameters, ProviderFuture, ProviderLeaseToken, SqlError, SqlErrorKind};
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    Execute,
    FetchOne,
    FetchOptional,
    FetchAll { maximum_rows: u64 },
    Stream,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancellationReason {
    User,
    Deadline,
    ResourceLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetReason {
    ReturnToPool,
    FailedOperation,
    CancelledOperation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeCardinality {
    pub minimum: u64,
    pub maximum: Option<u64>,
}

impl RuntimeCardinality {
    pub fn new(minimum: u64, maximum: Option<u64>) -> Result<Self, SqlError> {
        if maximum.is_some_and(|maximum| minimum > maximum) {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        Ok(Self { minimum, maximum })
    }

    #[must_use]
    pub fn supports(self, mode: ExecutionMode, returns_rows: bool) -> bool {
        match mode {
            ExecutionMode::Execute => !returns_rows,
            ExecutionMode::FetchOne | ExecutionMode::FetchOptional => {
                returns_rows && self.maximum.is_some_and(|maximum| maximum <= 1)
            }
            ExecutionMode::FetchAll { maximum_rows } => returns_rows && maximum_rows > 0,
            ExecutionMode::Stream => returns_rows,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeEffect {
    Read,
    Write,
    ReadWrite,
    SchemaChange,
    SessionChange,
    TransactionControl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeEffectContract {
    pub effect: RuntimeEffect,
    pub referenced_objects: Arc<[String]>,
    pub affected_objects: Arc<[String]>,
}

impl RuntimeEffectContract {
    pub fn new(
        effect: RuntimeEffect,
        referenced_objects: Vec<String>,
        affected_objects: Vec<String>,
    ) -> Result<Self, SqlError> {
        if referenced_objects
            .iter()
            .chain(&affected_objects)
            .any(|value| {
                value.is_empty() || value.len() > 256 || value.chars().any(char::is_control)
            })
        {
            return Err(SqlError::new(SqlErrorKind::Provider));
        }
        let affects_rows = matches!(
            effect,
            RuntimeEffect::Write | RuntimeEffect::ReadWrite | RuntimeEffect::SchemaChange
        );
        if (effect == RuntimeEffect::Read && !affected_objects.is_empty())
            || (affects_rows && affected_objects.is_empty())
            || (matches!(
                effect,
                RuntimeEffect::SessionChange | RuntimeEffect::TransactionControl
            ) && (!referenced_objects.is_empty() || !affected_objects.is_empty()))
        {
            return Err(SqlError::new(SqlErrorKind::Provider));
        }
        Ok(Self {
            effect,
            referenced_objects: referenced_objects.into(),
            affected_objects: affected_objects.into(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionMetadata {
    pub normalized_statement_fingerprint: String,
    pub parameter_type_fingerprint: String,
    pub result_type_fingerprint: String,
    pub schema_fingerprint: String,
}

#[derive(Clone, PartialEq)]
pub struct ExecutionRequest<P> {
    pub profile: Arc<P>,
    pub statement: Arc<str>,
    pub parameters: BoundParameters,
    pub cardinality: RuntimeCardinality,
    pub effects: RuntimeEffectContract,
    pub returns_rows: bool,
    pub metadata: ExecutionMetadata,
    pub mode: ExecutionMode,
}

impl<P> fmt::Debug for ExecutionRequest<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ExecutionRequest")
            .field("statement_len", &self.statement.len())
            .field("parameters", &self.parameters)
            .field("cardinality", &self.cardinality)
            .field("effects", &self.effects)
            .field("returns_rows", &self.returns_rows)
            .field("metadata", &self.metadata)
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

impl<P> ExecutionRequest<P> {
    pub fn validate(&self) -> Result<(), SqlError> {
        if self.statement.trim().is_empty()
            || !valid_fingerprint(&self.metadata.normalized_statement_fingerprint)
            || !valid_fingerprint(&self.metadata.parameter_type_fingerprint)
            || !valid_fingerprint(&self.metadata.result_type_fingerprint)
            || !valid_fingerprint(&self.metadata.schema_fingerprint)
            || !self.cardinality.supports(self.mode, self.returns_rows)
        {
            return Err(SqlError::new(SqlErrorKind::Cardinality));
        }
        if matches!(
            self.effects.effect,
            RuntimeEffect::SchemaChange
                | RuntimeEffect::SessionChange
                | RuntimeEffect::TransactionControl
        ) {
            return Err(SqlError::new(SqlErrorKind::Provider));
        }
        Ok(())
    }
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionResult<M> {
    pub rows_affected: Option<u64>,
    pub metadata: M,
}

/// Provider bridges implement driver calls. Common runtime code owns handle
/// states, error shape, cancellation reasons, and resource accounting.
pub trait ProviderRuntime: Send + Sync + 'static {
    type Profile: Send + Sync + 'static;
    type NativeConnection: Send + 'static;
    type Row: Send + 'static;
    type Metadata: Send + 'static;

    fn acquire<'a>(
        &'a self,
        profile: &'a Self::Profile,
    ) -> ProviderFuture<'a, (ProviderLeaseToken, Self::NativeConnection)>;

    fn execute<'a>(
        &'a self,
        connection: &'a mut Self::NativeConnection,
        request: ExecutionRequest<Self::Profile>,
    ) -> ProviderFuture<'a, ExecutionResult<Self::Metadata>>;

    fn fetch<'a>(
        &'a self,
        connection: &'a mut Self::NativeConnection,
        request: ExecutionRequest<Self::Profile>,
    ) -> ProviderFuture<'a, Vec<Self::Row>>;

    fn cancel<'a>(
        &'a self,
        connection: &'a mut Self::NativeConnection,
        reason: CancellationReason,
    ) -> ProviderFuture<'a, ()>;

    fn reset<'a>(
        &'a self,
        connection: &'a mut Self::NativeConnection,
        reason: ResetReason,
    ) -> ProviderFuture<'a, ()>;

    fn discard(&self, connection: Self::NativeConnection) -> Result<(), SqlError>;
}
