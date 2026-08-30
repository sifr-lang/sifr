//! Driver-free SQL runtime contracts shared by provider packages.
//!
//! Provider crates implement [`ProviderRuntime`]. This crate owns the common
//! handle states, owned parameters, redacted errors, and panic-safe future
//! boundary. It does not parse SQL or link a database driver.

mod codec;
mod error;
mod future;
mod handles;
mod parameter;
mod pool;
mod provider;
mod query;
mod resource;
mod session;
mod statement_cache;
mod transaction;
mod verification;

pub use codec::{RuntimeCodec, catch_codec_boundary};
pub use error::{
    CardinalityViolation, ConstraintKind, ResourceLimitKind, RetryClassification,
    SafeSqlIdentifier, SqlError, SqlErrorKind, SqlErrorMetadata, SqlState,
};
pub use future::ProviderFuture;
pub use handles::{
    Connection, OwnedRowStream, Pool, ProviderLeaseToken, RowStream, Transaction, Unverified,
    VerificationEvidence, Verified,
};
pub use parameter::{
    BoundParameters, OwnedParameter, OwnedSqlValue, ParameterError, RuntimeCodecIdentity,
};
pub use pool::{PoolCoordinator, PoolLease, PoolStatistics};
pub use provider::{
    CancellationReason, ExecutionMetadata, ExecutionMode, ExecutionRequest, ExecutionResult,
    ProviderRuntime, ResetReason, RuntimeCardinality, RuntimeEffect, RuntimeEffectContract,
};
pub use query::{BoundQuery, EncodeParameters, OrderedParameterEncoder, QueryTemplate};
pub use resource::{ResourceUsage, RuntimeLimits};
pub use session::{IsolationLevel, PoolingMode, SessionContract, SessionSnapshot, isolation_name};
pub use sifr_runtime::async_cleanup::AsyncCleanupEvidence;
pub use sifr_runtime::cancellation::CancellationCarrier;
pub use statement_cache::{StatementCache, StatementCacheKey};
pub use transaction::{TransactionMachine, TransactionState};
pub use verification::{
    SchemaDependencySlice, SchemaEvidenceMode, SchemaProperty, SchemaStrictness, verify_schema,
};
