use crate::{SqlError, SqlErrorKind};
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatementCacheKey {
    pub normalized_statement_fingerprint: String,
    pub parameter_type_fingerprint: String,
    pub result_type_fingerprint: String,
    pub provider_version: String,
    pub schema_fingerprint: String,
}

impl StatementCacheKey {
    pub fn validate(self) -> Result<Self, SqlError> {
        if !valid_fingerprint(&self.normalized_statement_fingerprint)
            || !valid_fingerprint(&self.parameter_type_fingerprint)
            || !valid_fingerprint(&self.result_type_fingerprint)
            || !valid_fingerprint(&self.schema_fingerprint)
            || self.provider_version.is_empty()
            || self.provider_version.len() > 64
            || self.provider_version.chars().any(char::is_control)
        {
            return Err(SqlError::new(SqlErrorKind::Configuration));
        }
        Ok(self)
    }
}

pub struct StatementCache<V> {
    capacity: usize,
    entries: BTreeMap<StatementCacheKey, V>,
    recency: VecDeque<StatementCacheKey>,
}

impl<V> StatementCache<V> {
    pub fn new(capacity: u32) -> Result<Self, SqlError> {
        let capacity =
            usize::try_from(capacity).map_err(|_| SqlError::new(SqlErrorKind::Configuration))?;
        if capacity == 0 {
            return Err(SqlError::new(SqlErrorKind::Configuration));
        }
        Ok(Self {
            capacity,
            entries: BTreeMap::new(),
            recency: VecDeque::new(),
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&mut self, key: &StatementCacheKey) -> Option<&V> {
        if !self.entries.contains_key(key) {
            return None;
        }
        self.touch(key);
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: &StatementCacheKey, value: V) -> Option<V> {
        let previous = self.entries.insert(key.clone(), value);
        self.touch(key);
        while self.entries.len() > self.capacity {
            let Some(oldest) = self.recency.pop_front() else {
                break;
            };
            if oldest != *key || self.entries.len() > 1 {
                self.entries.remove(&oldest);
            }
        }
        previous
    }

    pub fn invalidate_schema(&mut self, schema_fingerprint: &str) {
        self.entries
            .retain(|key, _| key.schema_fingerprint == schema_fingerprint);
        self.recency
            .retain(|key| key.schema_fingerprint == schema_fingerprint);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.recency.clear();
    }

    fn touch(&mut self, key: &StatementCacheKey) {
        self.recency.retain(|existing| existing != key);
        self.recency.push_back(key.clone());
    }
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
