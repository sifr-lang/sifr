//! Bundled SQLite runtime for Sifr's schema-first SQL platform.
//!
//! Each native connection lives on one dedicated worker thread. Async callers
//! communicate through a bounded channel. Cancellation uses SQLite's
//! `InterruptHandle`; a cancelled or timed-out worker is never returned to the pool.

mod config;
mod pool;
mod stream;
mod transaction;
mod worker;

pub use config::{
    ManifestVerifier, SignedSchemaManifest, SqliteEvidence, SqliteProfile, VerificationProbe,
};
pub use pool::{ExecutionOptions, SqliteConnection, SqlitePool, connect, open_pool};
pub use sifr_sql_runtime::{Unverified, Verified};
pub use stream::{SqliteRowStream, SqliteTransactionRowStream};
pub use transaction::{SqliteSavepoint, SqliteTransaction};
pub use worker::{SqliteExecutionMetadata, SqliteRow};

pub const BUNDLED_SQLITE_VERSION: &str = "3.53.2";
pub const BUNDLED_SQLITE_VERSION_NUMBER: i32 = 3_053_002;
