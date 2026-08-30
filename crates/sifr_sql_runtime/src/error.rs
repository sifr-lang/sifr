use sifr_runtime::async_cleanup::AsyncCleanupEvidence;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SqlErrorKind {
    Configuration,
    SchemaContract,
    Connection,
    Authentication,
    Timeout,
    Cancelled,
    Constraint,
    Serialization,
    Deadlock,
    Decode,
    Encode,
    Cardinality,
    ResourceLimit,
    TransactionControl,
    Provider,
    Migration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintKind {
    Unique,
    ForeignKey,
    Check,
    NotNull,
    Exclusion,
    ProviderSpecific,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetryClassification {
    Never,
    RetryTransaction,
    RetryConnection,
    ProviderClassified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceLimitKind {
    Connections,
    AcquireDeadline,
    StatementDeadline,
    CleanupDeadline,
    DecodedRowBytes,
    CollectedRows,
    StatementCache,
    Parameters,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardinalityViolation {
    ExpectedExactlyOneFoundZero,
    ExpectedExactlyOneFoundMany,
    ExpectedAtMostOneFoundMany,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlState(String);

impl SqlState {
    pub fn new(value: &str) -> Result<Self, SqlError> {
        let bytes = value.as_bytes();
        if bytes.len() != 5
            || !bytes
                .iter()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        {
            return Err(SqlError::new(SqlErrorKind::Provider));
        }
        Ok(Self(value.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafeSqlIdentifier(String);

impl SafeSqlIdentifier {
    pub fn new(value: impl Into<String>) -> Result<Self, SqlError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 256
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        {
            return Err(SqlError::new(SqlErrorKind::Provider));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlErrorMetadata {
    pub sql_state: Option<SqlState>,
    pub vendor_code: Option<i64>,
    pub constraint_kind: Option<ConstraintKind>,
    pub constraint_identity: Option<SafeSqlIdentifier>,
    pub table_identity: Option<SafeSqlIdentifier>,
    pub columns: Vec<SafeSqlIdentifier>,
    pub retry: RetryClassification,
    pub resource_limit: Option<ResourceLimitKind>,
    pub cardinality: Option<CardinalityViolation>,
}

impl Default for SqlErrorMetadata {
    fn default() -> Self {
        Self {
            sql_state: None,
            vendor_code: None,
            constraint_kind: None,
            constraint_identity: None,
            table_identity: None,
            columns: Vec::new(),
            retry: RetryClassification::Never,
            resource_limit: None,
            cardinality: None,
        }
    }
}

impl SqlErrorMetadata {
    pub fn validate(&self) -> Result<(), SqlError> {
        if self.constraint_kind.is_none() && self.constraint_identity.is_some() {
            return Err(SqlError::new(SqlErrorKind::Provider));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlError {
    kind: SqlErrorKind,
    metadata: Box<SqlErrorMetadata>,
    secondary: Vec<AsyncCleanupEvidence>,
}

impl SqlError {
    #[must_use]
    pub fn new(kind: SqlErrorKind) -> Self {
        Self {
            kind,
            metadata: Box::default(),
            secondary: Vec::new(),
        }
    }

    pub fn with_metadata(kind: SqlErrorKind, metadata: SqlErrorMetadata) -> Result<Self, Self> {
        metadata.validate()?;
        Ok(Self {
            kind,
            metadata: Box::new(metadata),
            secondary: Vec::new(),
        })
    }

    #[must_use]
    pub const fn kind(&self) -> SqlErrorKind {
        self.kind
    }

    #[must_use]
    pub fn metadata(&self) -> &SqlErrorMetadata {
        self.metadata.as_ref()
    }

    #[must_use]
    pub const fn retry_classification(&self) -> RetryClassification {
        self.metadata.retry
    }

    #[must_use]
    pub fn secondary(&self) -> &[AsyncCleanupEvidence] {
        &self.secondary
    }

    #[must_use]
    pub fn with_secondary(mut self, evidence: AsyncCleanupEvidence) -> Self {
        self.secondary.push(evidence);
        self
    }

    pub fn extend_secondary(&mut self, evidence: impl IntoIterator<Item = AsyncCleanupEvidence>) {
        self.secondary.extend(evidence);
    }

    #[must_use]
    pub fn resource_limit(limit: ResourceLimitKind) -> Self {
        Self::with_metadata(
            SqlErrorKind::ResourceLimit,
            SqlErrorMetadata {
                resource_limit: Some(limit),
                ..SqlErrorMetadata::default()
            },
        )
        .unwrap_or_else(|error| error)
    }

    #[must_use]
    pub fn cardinality(violation: CardinalityViolation) -> Self {
        Self::with_metadata(
            SqlErrorKind::Cardinality,
            SqlErrorMetadata {
                cardinality: Some(violation),
                ..SqlErrorMetadata::default()
            },
        )
        .unwrap_or_else(|error| error)
    }
}

impl fmt::Display for SqlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.kind {
            SqlErrorKind::Configuration => "SQL configuration is invalid",
            SqlErrorKind::SchemaContract => "SQL schema verification failed",
            SqlErrorKind::Connection => "database connection failed",
            SqlErrorKind::Authentication => "database authentication failed",
            SqlErrorKind::Timeout => "database operation timed out",
            SqlErrorKind::Cancelled => "database operation was cancelled",
            SqlErrorKind::Constraint => "database constraint was violated",
            SqlErrorKind::Serialization => "database serialization conflict occurred",
            SqlErrorKind::Deadlock => "database deadlock occurred",
            SqlErrorKind::Decode => "database value decoding failed",
            SqlErrorKind::Encode => "database parameter encoding failed",
            SqlErrorKind::Cardinality => "database result cardinality was violated",
            SqlErrorKind::ResourceLimit => "SQL resource limit was exceeded",
            SqlErrorKind::TransactionControl => "SQL transaction state is invalid",
            SqlErrorKind::Provider => "database provider failed",
            SqlErrorKind::Migration => "database migration failed",
        })
    }
}

impl std::error::Error for SqlError {}
