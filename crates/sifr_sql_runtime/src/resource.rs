use crate::{ResourceLimitKind, SqlError, SqlErrorKind, SqlErrorMetadata};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeLimits {
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub statement_timeout: Duration,
    pub cleanup_timeout: Duration,
    pub max_decoded_row_bytes: u64,
    pub max_collected_rows: u64,
    pub statement_cache_capacity: u32,
    pub max_parameters: u32,
}

impl RuntimeLimits {
    pub fn validate(self) -> Result<Self, SqlError> {
        if self.max_connections == 0
            || self.acquire_timeout.is_zero()
            || self.statement_timeout.is_zero()
            || self.cleanup_timeout.is_zero()
            || self.max_decoded_row_bytes == 0
            || self.max_collected_rows == 0
            || self.statement_cache_capacity == 0
            || self.max_parameters == 0
        {
            return Err(SqlError::new(SqlErrorKind::Configuration));
        }
        Ok(self)
    }
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        Self {
            max_connections: 10,
            acquire_timeout: Duration::from_secs(30),
            statement_timeout: Duration::from_secs(30),
            cleanup_timeout: Duration::from_secs(5),
            max_decoded_row_bytes: 16 * 1024 * 1024,
            max_collected_rows: 10_000,
            statement_cache_capacity: 100,
            max_parameters: 65_535,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceUsage {
    largest_decoded_row_bytes: u64,
    collected_rows: u64,
    parameters: u32,
}

impl ResourceUsage {
    #[must_use]
    pub const fn largest_decoded_row_bytes(self) -> u64 {
        self.largest_decoded_row_bytes
    }

    #[must_use]
    pub const fn collected_rows(self) -> u64 {
        self.collected_rows
    }

    #[must_use]
    pub const fn parameters(self) -> u32 {
        self.parameters
    }

    pub fn account_parameters(
        &mut self,
        count: u32,
        limits: RuntimeLimits,
    ) -> Result<(), SqlError> {
        if count > limits.max_parameters {
            return Err(limit_error(ResourceLimitKind::Parameters));
        }
        self.parameters = count;
        Ok(())
    }

    pub fn account_row(
        &mut self,
        decoded_bytes: u64,
        limits: RuntimeLimits,
    ) -> Result<(), SqlError> {
        let Some(total_rows) = self.collected_rows.checked_add(1) else {
            return Err(limit_error(ResourceLimitKind::CollectedRows));
        };
        if decoded_bytes > limits.max_decoded_row_bytes {
            return Err(limit_error(ResourceLimitKind::DecodedRowBytes));
        }
        if total_rows > limits.max_collected_rows {
            return Err(limit_error(ResourceLimitKind::CollectedRows));
        }
        self.largest_decoded_row_bytes = self.largest_decoded_row_bytes.max(decoded_bytes);
        self.collected_rows = total_rows;
        Ok(())
    }
}

fn limit_error(limit: ResourceLimitKind) -> SqlError {
    SqlError::with_metadata(
        SqlErrorKind::ResourceLimit,
        SqlErrorMetadata {
            resource_limit: Some(limit),
            ..SqlErrorMetadata::default()
        },
    )
    .unwrap_or_else(|error| error)
}
