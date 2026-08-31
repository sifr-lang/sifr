//! MySQL runtime bridge for Sifr's schema-first SQL platform.
//!
//! This crate owns a bounded pool of raw `mysql_async::Conn` values. It never
//! constructs or exposes `mysql_async::Pool`. Cancellation uses a separate raw
//! control connection, sends `KILL QUERY`, and permanently discards the target.

mod codec;
mod config;
mod connection;
mod control;
mod error;
mod pool;
mod stream;
mod transaction;

pub use config::{MysqlProfile, MysqlSchemaVerifier, MysqlTlsPolicy};
pub use connection::{ExecutionOptions, MysqlConnection, MysqlMetadata, MysqlRow};
pub use pool::{MysqlPool, connect, open_pool};
pub use stream::{MysqlRowStream, MysqlTransactionRowStream};
pub use transaction::{MysqlSavepoint, MysqlTransaction, RetryPolicy, RetrySafeCallback};

pub use sifr_sql_runtime::{Unverified, Verified};
