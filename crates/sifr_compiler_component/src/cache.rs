use crate::registration::hex_digest;
use crate::{
    ComponentError, ComponentErrorKind, EmbeddedAnalysisRequest, EmbeddedAnalysisResponse,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub const DEFAULT_COMPONENT_CACHE_CAPACITY_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CacheKey(pub String);

impl CacheKey {
    pub fn for_request(request: &EmbeddedAnalysisRequest) -> Result<Self, ComponentError> {
        let bytes = serde_json::to_vec(request).map_err(cache_serialization_error)?;
        Ok(Self(hex_digest(Sha256::digest(bytes).as_slice())))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CacheEntry {
    key: CacheKey,
    response: EmbeddedAnalysisResponse,
}

#[derive(Debug)]
pub struct AnalysisCache {
    root: PathBuf,
    capacity_bytes: u64,
    access_counter: u64,
    access_order: BTreeMap<CacheKey, u64>,
    pinned: BTreeSet<CacheKey>,
}

impl AnalysisCache {
    pub fn open_default(root: impl Into<PathBuf>) -> Result<Self, ComponentError> {
        Self::open(root, DEFAULT_COMPONENT_CACHE_CAPACITY_BYTES)
    }

    pub fn open(root: impl Into<PathBuf>, capacity_bytes: u64) -> Result<Self, ComponentError> {
        if capacity_bytes == 0 {
            return Err(ComponentError::new(
                ComponentErrorKind::Cache,
                "component cache capacity must be positive",
            ));
        }
        let root = root.into();
        fs::create_dir_all(&root).map_err(cache_io_error)?;
        let mut cache = Self {
            root,
            capacity_bytes,
            access_counter: 0,
            access_order: BTreeMap::new(),
            pinned: BTreeSet::new(),
        };
        cache.index_existing_entries()?;
        cache.evict()?;
        Ok(cache)
    }

    pub fn get(
        &mut self,
        key: &CacheKey,
        max_entry_bytes: u64,
    ) -> Result<Option<EmbeddedAnalysisResponse>, ComponentError> {
        let path = self.entry_path(key);
        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(cache_io_error(error)),
        };
        if metadata.len() > max_entry_bytes {
            return Err(ComponentError::new(
                ComponentErrorKind::ResourceLimit,
                "component cache entry exceeds the response byte limit",
            ));
        }
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(cache_io_error(error)),
        };
        let entry: CacheEntry =
            serde_json::from_slice(&bytes).map_err(cache_serialization_error)?;
        if entry.key != *key {
            return Err(ComponentError::new(
                ComponentErrorKind::Cache,
                "component cache entry key does not match its path",
            ));
        }
        self.touch(key);
        Ok(Some(entry.response))
    }

    pub fn put(
        &mut self,
        key: &CacheKey,
        response: &EmbeddedAnalysisResponse,
    ) -> Result<(), ComponentError> {
        let bytes = serde_json::to_vec(&CacheEntry {
            key: key.clone(),
            response: response.clone(),
        })
        .map_err(cache_serialization_error)?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > self.capacity_bytes {
            return Err(ComponentError::new(
                ComponentErrorKind::ResourceLimit,
                "component cache entry exceeds the cache capacity",
            ));
        }
        let final_path = self.entry_path(key);
        let temporary = self.temporary_path(key);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(cache_io_error)?;
        file.write_all(&bytes).map_err(cache_io_error)?;
        file.sync_all().map_err(cache_io_error)?;
        fs::rename(&temporary, &final_path).map_err(cache_io_error)?;
        self.touch(key);
        self.evict()?;
        Ok(())
    }

    pub fn pin(&mut self, key: CacheKey) {
        self.pinned.insert(key);
    }

    pub fn unpin(&mut self, key: &CacheKey) {
        self.pinned.remove(key);
    }

    fn touch(&mut self, key: &CacheKey) {
        self.access_counter = self.access_counter.saturating_add(1);
        self.access_order.insert(key.clone(), self.access_counter);
    }

    fn evict(&mut self) -> Result<(), ComponentError> {
        while self.total_size()? > self.capacity_bytes {
            let victim = self
                .access_order
                .iter()
                .filter(|(key, _)| !self.pinned.contains(*key))
                .min_by_key(|(key, access)| (**access, (*key).clone()))
                .map(|(key, _)| key.clone())
                .ok_or_else(|| {
                    ComponentError::new(
                        ComponentErrorKind::ResourceLimit,
                        "pinned component cache entries exceed the cache capacity",
                    )
                })?;
            match fs::remove_file(self.entry_path(&victim)) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(cache_io_error(error)),
            }
            self.access_order.remove(&victim);
        }
        Ok(())
    }

    fn total_size(&self) -> Result<u64, ComponentError> {
        let mut total = 0_u64;
        for entry in fs::read_dir(&self.root).map_err(cache_io_error)? {
            let entry = entry.map_err(cache_io_error)?;
            if entry
                .path()
                .extension()
                .is_some_and(|value| value == "json")
            {
                total = total.saturating_add(entry.metadata().map_err(cache_io_error)?.len());
            }
        }
        Ok(total)
    }

    fn index_existing_entries(&mut self) -> Result<(), ComponentError> {
        let mut keys = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(cache_io_error)? {
            let path = entry.map_err(cache_io_error)?.path();
            let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            if path.extension().is_some_and(|value| value == "json")
                && stem.len() == 64
                && stem
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                keys.push(CacheKey(stem.to_string()));
            }
        }
        keys.sort();
        for key in keys {
            self.touch(&key);
        }
        Ok(())
    }

    fn entry_path(&self, key: &CacheKey) -> PathBuf {
        self.root.join(format!("{}.json", key.0))
    }

    fn temporary_path(&self, key: &CacheKey) -> PathBuf {
        self.root.join(format!(
            ".{}.{}.{}.tmp",
            key.0,
            std::process::id(),
            self.access_counter
        ))
    }
}

#[allow(clippy::needless_pass_by_value)]
fn cache_io_error(error: std::io::Error) -> ComponentError {
    ComponentError::new(ComponentErrorKind::Cache, error.to_string())
}

#[allow(clippy::needless_pass_by_value)]
fn cache_serialization_error(error: serde_json::Error) -> ComponentError {
    ComponentError::new(ComponentErrorKind::Cache, error.to_string())
}
