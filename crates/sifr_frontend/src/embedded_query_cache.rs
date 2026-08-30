use crate::CacheKeyFingerprint;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

pub const DEFAULT_EMBEDDED_QUERY_CACHE_CAPACITY_ENTRIES: usize = 4096;

#[derive(Clone, Debug)]
struct EmbeddedCacheEntry<T> {
    value: Arc<T>,
}

#[derive(Clone, Debug)]
pub struct EmbeddedCacheInsert<T> {
    pub value: Arc<T>,
    pub evicted: Vec<CacheKeyFingerprint>,
}

/// Bounded process-local cache for validated embedded analysis. The cache owns
/// no semantic fallback: misses and evictions must be recomputed by the same
/// provider operation that produced the original value.
pub struct EmbeddedQueryCache<T> {
    capacity: usize,
    entries: BTreeMap<CacheKeyFingerprint, EmbeddedCacheEntry<T>>,
    recency: VecDeque<CacheKeyFingerprint>,
    pinned: BTreeSet<CacheKeyFingerprint>,
}

impl<T> EmbeddedQueryCache<T> {
    #[must_use]
    pub fn open_default() -> Self {
        Self {
            capacity: DEFAULT_EMBEDDED_QUERY_CACHE_CAPACITY_ENTRIES,
            entries: BTreeMap::new(),
            recency: VecDeque::new(),
            pinned: BTreeSet::new(),
        }
    }

    pub fn new(capacity: usize) -> Result<Self, &'static str> {
        if capacity == 0 {
            return Err("embedded query cache capacity must be positive");
        }
        Ok(Self {
            capacity,
            entries: BTreeMap::new(),
            recency: VecDeque::new(),
            pinned: BTreeSet::new(),
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

    pub fn get(&mut self, key: &CacheKeyFingerprint) -> Option<Arc<T>> {
        let value = self.entries.get(key)?.value.clone();
        self.touch(key);
        Some(value)
    }

    pub fn insert(
        &mut self,
        key: &CacheKeyFingerprint,
        value: T,
    ) -> Result<EmbeddedCacheInsert<T>, &'static str> {
        let value = Arc::new(value);
        self.entries.insert(
            key.clone(),
            EmbeddedCacheEntry {
                value: value.clone(),
            },
        );
        self.touch(key);
        let evicted = self.evict()?;
        Ok(EmbeddedCacheInsert { value, evicted })
    }

    pub fn pin(&mut self, key: &CacheKeyFingerprint) -> Result<(), &'static str> {
        if !self.entries.contains_key(key) {
            return Err("cannot pin a missing embedded query cache entry");
        }
        self.pinned.insert(key.clone());
        Ok(())
    }

    pub fn unpin(&mut self, key: &CacheKeyFingerprint) {
        self.pinned.remove(key);
    }

    pub fn remove(&mut self, key: &CacheKeyFingerprint) -> Option<Arc<T>> {
        self.recency.retain(|candidate| candidate != key);
        self.pinned.remove(key);
        self.entries.remove(key).map(|entry| entry.value)
    }

    fn touch(&mut self, key: &CacheKeyFingerprint) {
        self.recency.retain(|candidate| candidate != key);
        self.recency.push_back(key.clone());
    }

    fn evict(&mut self) -> Result<Vec<CacheKeyFingerprint>, &'static str> {
        let mut evicted = Vec::new();
        while self.entries.len() > self.capacity {
            let Some(index) = self
                .recency
                .iter()
                .position(|candidate| !self.pinned.contains(candidate))
            else {
                return Err("pinned embedded query cache entries exceed capacity");
            };
            let Some(victim) = self.recency.remove(index) else {
                return Err("embedded query cache recency index is inconsistent");
            };
            self.entries.remove(&victim);
            evicted.push(victim);
        }
        Ok(evicted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(value: &str) -> CacheKeyFingerprint {
        CacheKeyFingerprint(value.to_string())
    }

    #[test]
    fn pinned_lru_eviction_changes_only_reuse() {
        let mut cache = EmbeddedQueryCache::new(2).expect("capacity");
        cache.insert(&key("a"), "plan-a").expect("insert");
        cache.insert(&key("b"), "plan-b").expect("insert");
        cache.pin(&key("a")).expect("pin");
        let inserted = cache.insert(&key("c"), "plan-c").expect("insert");
        assert_eq!(inserted.evicted, vec![key("b")]);
        assert_eq!(cache.get(&key("a")).as_deref(), Some(&"plan-a"));
        assert!(cache.get(&key("b")).is_none());
        assert_eq!(cache.get(&key("c")).as_deref(), Some(&"plan-c"));
    }
}
