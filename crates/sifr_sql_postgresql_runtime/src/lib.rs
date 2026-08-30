//! PostgreSQL runtime bridge for Sifr's schema-first SQL platform.
//!
//! This crate wraps raw `tokio-postgres` clients. Public values contain no
//! upstream driver types and never render statements, parameters, or URLs.

mod codec;
mod config;
mod connection;
mod control;
mod error;
mod execute;
mod pool;
mod stream;
mod transaction;
mod verification;

pub use config::{
    ManifestVerifier, PostgresEvidence, PostgresProfile, PostgresTls, SignedSchemaManifest,
    VerificationProbe,
};
pub use connection::{ExecutionOptions, PostgresConnection, PostgresMetadata, PostgresRow};
pub use pool::{PostgresPool, connect, open_pool};
pub use stream::{PostgresRowStream, PostgresTransactionRowStream};
pub use transaction::{
    PostgresSavepoint, PostgresTransaction, RetryPolicy, RetrySafeCallback, TransactionOptions,
};

pub use sifr_sql_runtime::{Unverified, Verified};
