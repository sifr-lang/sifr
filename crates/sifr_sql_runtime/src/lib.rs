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
mod provider;
mod query;
mod resource;

pub use codec::{RuntimeCodec, catch_codec_boundary};
pub use error::{
    ConstraintKind, ResourceLimitKind, RetryClassification, SafeSqlIdentifier, SqlError,
    SqlErrorKind, SqlErrorMetadata, SqlState,
};
pub use future::ProviderFuture;
pub use handles::{
    Connection, OwnedRowStream, Pool, ProviderLeaseToken, RowStream, Transaction, Unverified,
    VerificationEvidence, Verified,
};
pub use parameter::{
    BoundParameters, OwnedParameter, OwnedSqlValue, ParameterError, RuntimeCodecIdentity,
};
pub use provider::{
    CancellationReason, ExecutionMetadata, ExecutionMode, ExecutionRequest, ExecutionResult,
    ProviderRuntime, ResetReason, RuntimeCardinality, RuntimeEffect, RuntimeEffectContract,
};
pub use query::{BoundQuery, EncodeParameters, OrderedParameterEncoder, QueryTemplate};
pub use resource::{ResourceUsage, RuntimeLimits};
pub use sifr_runtime::cancellation::CancellationCarrier;
