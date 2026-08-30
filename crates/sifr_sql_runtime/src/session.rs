use crate::{SqlError, SqlErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoolingMode {
    Session,
    Transaction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionContract {
    pub search_path: Vec<String>,
    pub time_zone: String,
    pub role: Option<String>,
    pub default_isolation: IsolationLevel,
    pub read_only: bool,
    pub pooling: PoolingMode,
    pub requires_session_affinity: bool,
}

impl SessionContract {
    pub fn validate(self) -> Result<Self, SqlError> {
        if self.search_path.is_empty()
            || self
                .search_path
                .iter()
                .any(|value| !valid_identifier(value))
            || self.time_zone.is_empty()
            || self.time_zone.len() > 128
            || self.time_zone.chars().any(char::is_control)
            || self
                .role
                .as_ref()
                .is_some_and(|role| !valid_identifier(role))
            || (self.pooling == PoolingMode::Transaction && self.requires_session_affinity)
        {
            return Err(SqlError::new(SqlErrorKind::Configuration));
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionSnapshot {
    pub search_path: String,
    pub time_zone: String,
    pub role: String,
    pub default_isolation: String,
    pub read_only: bool,
}

impl SessionSnapshot {
    pub fn matches(&self, contract: &SessionContract) -> bool {
        let expected_path = contract.search_path.join(", ");
        self.search_path == expected_path
            && self.time_zone == contract.time_zone
            && contract
                .role
                .as_ref()
                .is_none_or(|expected_role| self.role == *expected_role)
            && self.default_isolation == isolation_name(contract.default_isolation)
            && self.read_only == contract.read_only
    }
}

#[must_use]
pub const fn isolation_name(isolation: IsolationLevel) -> &'static str {
    match isolation {
        IsolationLevel::ReadCommitted => "read committed",
        IsolationLevel::RepeatableRead => "repeatable read",
        IsolationLevel::Serializable => "serializable",
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
}
