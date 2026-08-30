use crate::{OwnedSqlValue, SqlError, SqlErrorKind};
use std::panic::{AssertUnwindSafe, catch_unwind};

pub trait RuntimeCodec: Send + Sync {
    type Value;

    fn encode(&self, value: &Self::Value) -> Result<OwnedSqlValue, SqlError>;

    fn decode(&self, value: &OwnedSqlValue) -> Result<Self::Value, SqlError>;
}

pub fn catch_codec_boundary<T>(
    operation: impl FnOnce() -> Result<T, SqlError>,
) -> Result<T, SqlError> {
    catch_unwind(AssertUnwindSafe(operation))
        .unwrap_or_else(|_| Err(SqlError::new(SqlErrorKind::Provider)))
}
