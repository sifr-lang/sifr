//! Immutable project generations. Lock order: writer, then generation lease.
//! Readers take only a shared generation lease; GC takes writer then exclusive
//! generation leases. Lock inodes are permanent and never deleted.
use crate::cache_storage as storage;
use serde::{Deserialize, Serialize};
use sifr_frontend::persistence::{CompletedCheck, identity};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
const RECORD_LIMIT: u64 = 16 * 1024 * 1024;
const MANIFEST_LIMIT: u64 = 1024 * 1024;
// Per-generation publication history. Old generations remain under explicit prune.
pub(super) const RECORD_COUNT: usize = 128;
const HISTORY_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    schema: u32,
    workspace: String,
    context: String,
    pub(super) records: Vec<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Hint {
    schema: u32,
    workspace: String,
    context: String,
    generation: String,
}

pub(super) struct Store {
    pub(super) root: PathBuf,
    workspace_root: PathBuf,
    workspace: String,
    context: String,
}
pub(super) struct Generation {
    pub(super) path: PathBuf,
    pub(super) manifest: Manifest,
    _lease: File,
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn stamp<T: Serialize>(domain: &str, value: &T) -> io::Result<String> {
    identity(domain, value).map_err(io::Error::other)
}
fn key(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn read(root: &Path, name: &str, limit: u64) -> io::Result<Vec<u8>> {
    storage::payload(root, Path::new(name))?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(root.join(name))?;
    if !file.metadata()?.is_file() || file.metadata()?.len() > limit {
        return Err(invalid("project record limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(invalid("project record grew past limit"));
    }
    Ok(bytes)
}
fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    #[cfg(test)]
    if std::env::var_os("SIFR_DX13_STORAGE_FULL").is_some()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(key)
    {
        file.write_all(&bytes[..bytes.len().min(8)])?;
        return Err(io::Error::from_raw_os_error(libc::ENOSPC));
    }
    file.write_all(bytes)?;
    file.sync_all()
}
fn token() -> String {
    format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}
fn cancelled(cancel: &AtomicBool) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "cancelled project publication",
        ))
    } else {
        Ok(())
    }
}
impl Store {
    pub(super) fn open(cache: &Path, workspace_root: &Path, context: &str) -> io::Result<Self> {
        if !key(context) {
            return Err(invalid("invalid context identity"));
        }
        storage::directory(cache)?;
        storage::directory(&cache.join("projects"))?;
        let workspace_root = workspace_root.canonicalize()?;
        let workspace = stamp("project-workspace-v1", &workspace_root)?;
        storage::directory(&cache.join("projects").join(&workspace))?;
        let root = cache.join("projects").join(&workspace).join(context);
        storage::directory(&root)?;
        storage::directory(&root.join("generations"))?;
        Ok(Self {
            root,
            workspace_root,
            workspace,
            context: context.into(),
        })
    }
    pub(super) fn latest(&self) -> Option<Generation> {
        // The workspace hint is optional. An absent/read-only workspace still
        // discovers the generation through the owner-scoped user cache pointer.
        let hint = read(&self.workspace_root, ".sifrbuildinfo", 4096)
            .ok()
            .filter(|bytes| bytes.len() <= 4096)
            .and_then(|bytes| serde_json::from_slice::<Hint>(&bytes).ok());
        let pointer = read(&self.root, "latest", 4096)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Hint>(&bytes).ok());
        for hint in pointer.into_iter().chain(hint) {
            if hint.schema == 1
                && hint.workspace == self.workspace
                && hint.context == self.context
                && key(&hint.generation)
            {
                if let Ok(generation) = self.generation(&hint.generation) {
                    return Some(generation);
                }
            }
        }
        None
    }
    fn generation(&self, id: &str) -> io::Result<Generation> {
        if !key(id) {
            return Err(invalid("invalid generation identity"));
        }
        let parent = self.root.join("generations");
        let lease = storage::process_entry_lock(&parent, id)?;
        lease.try_lock_shared().map_err(io::Error::from)?;
        let path = parent.join(id);
        let bytes = read(&path, "manifest.json", MANIFEST_LIMIT)?;
        let manifest: Manifest = serde_json::from_slice(&bytes)?;
        if manifest.schema != 2
            || manifest.workspace != self.workspace
            || manifest.context != self.context
            || manifest.records.len() > RECORD_COUNT
            || manifest.records.iter().collect::<BTreeSet<_>>().len() != manifest.records.len()
            || stamp("project-generation-v1", &manifest)? != id
        {
            return Err(invalid("incompatible project generation"));
        }
        let mut total_bytes = 0;
        for record in &manifest.records {
            if !key(record) {
                return Err(invalid("invalid record reference"));
            }
            let bytes = read(&path, record, RECORD_LIMIT)?;
            if stamp("project-record-v1", &bytes)? != *record {
                return Err(invalid("corrupt project record"));
            }
            total_bytes += bytes.len() as u64;
            if total_bytes > HISTORY_BYTES {
                return Err(invalid("project history byte limit"));
            }
            let record: CompletedCheck = serde_json::from_slice(&bytes)?;
            if record.schema != 1
                || record
                    .result
                    .inputs
                    .semantic_inputs
                    .identity()
                    .ok()
                    .as_deref()
                    != Some(&self.context)
                || record.diagnostics().is_err()
            {
                return Err(invalid("incomplete project record"));
            }
        }
        Ok(Generation {
            path,
            manifest,
            _lease: lease,
        })
    }
    pub(super) fn publish(
        &self,
        record: &CompletedCheck,
        cancel: &AtomicBool,
    ) -> io::Result<String> {
        cancelled(cancel)?;
        let writer = storage::process_entry_lock(&self.root, "writer")?;
        writer.try_lock().map_err(io::Error::from)?;
        let previous = self.latest();
        let bytes = serde_json::to_vec(record)?;
        if bytes.len() as u64 > RECORD_LIMIT {
            return Err(invalid("project result exceeds limit"));
        }
        let record_id = stamp("project-record-v1", &bytes)?;
        let mut records = previous
            .as_ref()
            .map(|old| old.manifest.records.clone())
            .unwrap_or_default();
        // Refresh publication order, then retain a deterministic recent suffix.
        records.retain(|id| id != &record_id);
        records.push(record_id.clone());
        let mut retained_bytes = bytes.len() as u64;
        let mut retained = vec![record_id.clone()];
        if let Some(previous) = &previous {
            for id in records[..records.len() - 1].iter().rev() {
                let size = fs::metadata(previous.path.join(id))?.len();
                if retained.len() == RECORD_COUNT || retained_bytes + size > HISTORY_BYTES {
                    break;
                }
                retained_bytes += size;
                retained.push(id.clone());
            }
        }
        retained.reverse();
        records = retained;
        let manifest = Manifest {
            schema: 2,
            workspace: self.workspace.clone(),
            context: self.context.clone(),
            records,
        };
        let generation_id = stamp("project-generation-v1", &manifest)?;
        let parent = self.root.join("generations");
        let destination = parent.join(&generation_id);
        if destination.exists() {
            // Never trust a directory-exists winner; validate all references.
            let _winner = self.generation(&generation_id)?;
            self.point(&generation_id)?;
            return Ok(generation_id);
        }
        let stage_name = format!("{}.stage-{}", generation_id, token());
        let stage_lease = storage::process_entry_lock(&parent, &stage_name)?;
        stage_lease.try_lock().map_err(io::Error::from)?;
        let stage = parent.join(&stage_name);
        storage::directory(&stage)?;
        let result = (|| {
            if let Some(previous) = &previous {
                for inherited in &manifest.records {
                    if inherited == &record_id {
                        continue;
                    }
                    cancelled(cancel)?;
                    // A link is immutable. Never open an inherited destination
                    // for truncation, including when the new result is identical.
                    let source = previous.path.join(inherited);
                    let target = stage.join(inherited);
                    if fs::hard_link(&source, &target).is_err() {
                        write_new(&target, &read(&previous.path, inherited, RECORD_LIMIT)?)?;
                    }
                }
            }
            if !stage.join(&record_id).exists() {
                write_new(&stage.join(&record_id), &bytes)?;
            }
            write_new(
                &stage.join("manifest.json"),
                &serde_json::to_vec(&manifest)?,
            )?;
            File::open(&stage)?.sync_all()?;
            #[cfg(test)]
            super::tests::pause("before-rename");
            cancelled(cancel)?;
            fs::rename(&stage, &destination)?;
            #[cfg(test)]
            super::tests::pause("after-rename");
            File::open(&parent)?.sync_all()?;
            let _validated = self.generation(&generation_id)?;
            self.point(&generation_id)?;
            Ok(generation_id.clone())
        })();
        if stage.exists() {
            let _ = fs::remove_dir_all(stage);
        }
        result
    }
    fn point(&self, generation: &str) -> io::Result<()> {
        let hint = serde_json::to_vec(&Hint {
            schema: 1,
            workspace: self.workspace.clone(),
            context: self.context.clone(),
            generation: generation.into(),
        })?;
        let scratch = self.root.join(format!("latest.stage-{}", token()));
        write_new(&scratch, &hint)?;
        fs::rename(&scratch, self.root.join("latest"))?;
        File::open(&self.root)?.sync_all()?;
        // Hint failures never invalidate the complete user-cache generation.
        let scratch = self
            .workspace_root
            .join(format!(".sifrbuildinfo.stage-{}", token()));
        if write_new(&scratch, &hint).is_ok() {
            let _ = fs::rename(&scratch, self.workspace_root.join(".sifrbuildinfo"));
            let _ = fs::remove_file(&scratch);
        }
        Ok(())
    }
    /// Pressure-driven project GC. Current generation inherits all retained
    /// records, so inactive predecessors can go without losing unused results.
    pub(super) fn prune(&self, pressure: bool, dry_run: bool) -> io::Result<usize> {
        if !pressure {
            return Ok(0);
        }
        let writer = storage::process_entry_lock(&self.root, "writer")?;
        writer.try_lock().map_err(io::Error::from)?;
        let latest = self.latest();
        let parent = self.root.join("generations");
        let mut removed = 0;
        for entry in fs::read_dir(&parent)? {
            let path = entry?.path();
            if latest.as_ref().is_some_and(|latest| latest.path == path) {
                continue;
            }
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !(key(name) || name.split_once(".stage-").is_some_and(|(id, _)| key(id))) {
                continue;
            }
            storage::check_owned(&path)?;
            let lease = storage::process_entry_lock(&parent, name)?;
            if lease.try_lock().is_err() {
                continue;
            }
            if !dry_run {
                fs::remove_dir_all(&path)?;
            }
            removed += 1;
        }
        Ok(removed)
    }
}
impl Generation {
    pub(super) fn records(&self) -> impl Iterator<Item = io::Result<CompletedCheck>> + '_ {
        self.manifest.records.iter().rev().map(|record| {
            let bytes = read(&self.path, record, RECORD_LIMIT)?;
            if stamp("project-record-v1", &bytes)? != *record {
                return Err(invalid("changed immutable record"));
            }
            serde_json::from_slice(&bytes).map_err(io::Error::other)
        })
    }
}
