//! One versioned project-history envelope shared by the driver and editor.
//! Callers hold the owning generation and namespace leases throughout reads.
//! This module owns representation/identity bounds, not semantic reuse policy.
mod bundle;
use serde::{Deserialize, Serialize};
use sifr_frontend::persistence::identity;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::{Path, PathBuf},
};

pub const RECORD_LIMIT: u64 = 16 * 1024 * 1024;
pub const MANIFEST_LIMIT: u64 = 1024 * 1024;
pub const SCHEMA: u32 = 3;
pub const PROJECTS: &str = "projects-v3";
/// GC holds this context-level lease exclusively while marking references.
/// Readers hold it shared only until their individual generation lease exists.
pub const READER_ADMISSION: &str = "reader-admission";
// Per-generation publication history. Old generations remain under explicit prune.
pub const RECORD_COUNT: usize = 128;
pub const HISTORY_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub workspace: String,
    pub context: String,
    pub records: Vec<String>,
    pub payloads: BTreeMap<String, Payload>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Payload {
    /// None identifies this bundle's new record; inherited records name their
    /// direct payload owner, never a chain of historical manifest lookups.
    pub owner: Option<String>,
    pub length: u64,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
pub fn key(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn stamp<T: Serialize>(domain: &str, value: &T) -> io::Result<String> {
    identity(domain, value).map_err(io::Error::other)
}
impl Manifest {
    pub fn identity(&self) -> io::Result<String> {
        stamp("project-generation-v3", self)
    }
    pub fn read(path: &Path, workspace: &str, context: &str, id: &str) -> io::Result<Self> {
        if !key(id) {
            return Err(invalid("invalid generation identity"));
        }
        let manifest = bundle::manifest(path)?;
        if manifest.schema != SCHEMA
            || manifest.workspace != workspace
            || manifest.context != context
            || manifest.records.is_empty()
            || manifest.records.len() > RECORD_COUNT
            || manifest.records.iter().collect::<BTreeSet<_>>().len() != manifest.records.len()
            || manifest.payloads.keys().collect::<BTreeSet<_>>()
                != manifest.records.iter().collect::<BTreeSet<_>>()
            || manifest.identity()? != id
        {
            return Err(invalid("incompatible project generation"));
        }
        let mut total_bytes = 0;
        for (record, payload) in &manifest.payloads {
            if !key(record)
                || payload.length > RECORD_LIMIT
                || payload
                    .owner
                    .as_ref()
                    .is_some_and(|owner| !key(owner) || owner == id)
                || (payload.owner.is_none() && manifest.records.last() != Some(record))
            {
                return Err(invalid("invalid project payload reference"));
            }
            total_bytes += payload.length;
            if total_bytes > HISTORY_BYTES {
                return Err(invalid("project history byte limit"));
            }
        }
        Ok(manifest)
    }
    pub fn record_path(&self, generation_path: &Path, record: &str) -> io::Result<PathBuf> {
        let payload = self
            .payloads
            .get(record)
            .ok_or_else(|| invalid("unknown project record"))?;
        match &payload.owner {
            None => Ok(generation_path.to_path_buf()),
            Some(owner) if key(owner) => generation_path
                .parent()
                .map(|parent| parent.join(owner))
                .ok_or_else(|| invalid("missing generation parent")),
            Some(_) => Err(invalid("invalid payload owner")),
        }
    }
    pub fn read_record(&self, generation_path: &Path, record: &str) -> io::Result<Vec<u8>> {
        let path = self.record_path(generation_path, record)?;
        bundle::payload(&path, self.payloads[record].length)
    }
}
/// Write and flush one private envelope before its caller atomically publishes it.
/// The caller supplies a validated publication and owns its writer/staging lease.
pub fn write(path: &Path, manifest: &Manifest, bytes: &[u8]) -> io::Result<()> {
    bundle::write(path, manifest, bytes)
}
