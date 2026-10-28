//! Immutable project bundles. Readers take shared admission, acquire their
//! generation lease, then release admission. GC takes writer, pins the latest
//! generation, then holds exclusive admission while probing/reclaiming entries.
//! Namespace leases outlive stores through their readers.
//! Generation and namespace lock inodes remain stable while reachable.
use super::record_validation::RecordValidation;
use crate::cache_storage as storage;
use serde::{Deserialize, Serialize};
use sifr_cache_storage::ProcessLease;
#[cfg(test)]
pub(super) use sifr_compiler_services::project_history::MANIFEST_LIMIT;
pub(super) use sifr_compiler_services::project_history::{
    HISTORY_BYTES, Manifest, PROJECTS, Payload, READER_ADMISSION, RECORD_COUNT, RECORD_LIMIT, key,
};
use sifr_frontend::persistence::{CompletedCheck, identity};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Hint {
    pub(super) schema: u32,
    pub(super) workspace: String,
    pub(super) context: String,
    generation: String,
}

pub(super) struct Store {
    pub(super) root: PathBuf,
    pub(super) workspace_root: PathBuf,
    pub(super) workspace: String,
    pub(super) context: String,
    pub(super) namespace_lease: std::sync::Arc<ProcessLease>,
    pub(super) validation: std::sync::Arc<RecordValidation>,
}
pub(super) struct Generation {
    pub(super) path: PathBuf,
    pub(super) manifest: Manifest,
    _lease: ProcessLease,
    _namespace_lease: std::sync::Arc<ProcessLease>,
    validation: std::sync::Arc<RecordValidation>,
}
pub(super) fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
pub(super) fn stamp<T: Serialize>(domain: &str, value: &T) -> io::Result<String> {
    identity(domain, value).map_err(io::Error::other)
}
pub(super) fn read(root: &Path, name: &str, limit: u64) -> io::Result<Vec<u8>> {
    storage::payload(root, Path::new(name))?;
    let file = storage::read_private_file(&root.join(name))?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(invalid("project record limit"));
    }
    read_bounded(file, metadata.len(), limit)
}
fn read_bounded(reader: impl Read, expected: u64, limit: u64) -> io::Result<Vec<u8>> {
    if expected > limit {
        return Err(invalid("project record limit"));
    }
    let bounded = limit
        .checked_add(1)
        .ok_or_else(|| invalid("project record limit"))?;
    let size =
        usize::try_from(expected).map_err(|_| invalid("project record exceeds address space"))?;
    let mut bytes = Vec::with_capacity(size);
    reader.take(bounded).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(invalid("project record grew past limit"));
    }
    Ok(bytes)
}
pub(super) fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = storage::new_private_file(path)?;
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
        storage::directory(&cache.join(PROJECTS))?;
        let workspace_root = workspace_root.canonicalize()?;
        let workspace = stamp("project-workspace-v1", &workspace_root)?;
        let namespace_lease =
            super::housekeeping::open_namespace(cache, &workspace_root, &workspace)?;
        let root = cache.join(PROJECTS).join(&workspace).join(context);
        storage::directory(&root)?;
        storage::directory(&root.join("generations"))?;
        Ok(Self {
            root,
            workspace_root,
            workspace,
            context: context.into(),
            namespace_lease,
            validation: std::sync::Arc::new(RecordValidation::new(context.into())),
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
    pub(super) fn generation_manifest(&self, id: &str) -> io::Result<Manifest> {
        if !key(id) {
            return Err(invalid("invalid generation identity"));
        }
        Manifest::read(
            &self.root.join("generations").join(id),
            &self.workspace,
            &self.context,
            id,
        )
    }
    fn generation(&self, id: &str) -> io::Result<Generation> {
        if !key(id) {
            return Err(invalid("invalid generation identity"));
        }
        let parent = self.root.join("generations");
        let admission = ProcessLease::shared(super::housekeeping::existing_lease(
            &self.root,
            READER_ADMISSION,
        )?)?;
        let lease = ProcessLease::shared(super::housekeeping::existing_lease(&parent, id)?)?;
        drop(admission);
        let path = parent.join(id);
        let manifest = self.generation_manifest(id)?;
        let generation = Generation {
            path,
            manifest,
            _lease: lease,
            _namespace_lease: self.namespace_lease.clone(),
            validation: self.validation.clone(),
        };
        for record in &generation.manifest.records {
            let bytes = generation.read_record(record)?;
            self.validation
                .validate(record, &bytes, "corrupt project record")?;
        }
        Ok(generation)
    }
    pub(super) fn publish(&self, bytes: &[u8], cancel: &AtomicBool) -> io::Result<String> {
        cancelled(cancel)?;
        let _writer = ProcessLease::exclusive(storage::process_entry_lock(&self.root, "writer")?)?;
        // Establish admission before publishing any generation. Merely opening
        // an empty store or pruning it must not create writer authority.
        let _admission_file = storage::process_entry_lock(&self.root, READER_ADMISSION)?;
        let previous = self.latest();
        if bytes.len() as u64 > RECORD_LIMIT {
            return Err(invalid("project result exceeds limit"));
        }
        let record_id = stamp("project-record-v1", &bytes)?;
        self.validation
            .validate(&record_id, bytes, "invalid project publication")?;
        let mut records = previous
            .as_ref()
            .map(|old| old.manifest.records.clone())
            .unwrap_or_default();
        // Refresh publication order, then retain a deterministic recent suffix.
        records.retain(|id| id != &record_id);
        records.push(record_id.clone());
        let mut retained_bytes = bytes.len() as u64;
        let mut retained = vec![record_id.clone()];
        let mut payloads = BTreeMap::from([(
            record_id.clone(),
            Payload {
                owner: None,
                length: bytes.len() as u64,
            },
        )]);
        if let Some(previous) = &previous {
            for id in records[..records.len() - 1].iter().rev() {
                let mut payload = previous.manifest.payloads[id].clone();
                let size = payload.length;
                if retained.len() == RECORD_COUNT || retained_bytes + size > HISTORY_BYTES {
                    break;
                }
                retained_bytes += size;
                retained.push(id.clone());
                if payload.owner.is_none() {
                    payload.owner = Some(previous.id()?.to_owned());
                }
                payloads.insert(id.clone(), payload);
            }
        }
        retained.reverse();
        records = retained;
        let manifest = Manifest {
            schema: sifr_compiler_services::project_history::SCHEMA,
            workspace: self.workspace.clone(),
            context: self.context.clone(),
            records,
            payloads,
        };
        let generation_id = manifest.identity()?;
        let parent = self.root.join("generations");
        let destination = parent.join(&generation_id);
        if destination.exists() {
            // Never trust an existing publication winner; validate all references.
            let _winner = self.generation(&generation_id)?;
            self.point(&generation_id)?;
            return Ok(generation_id);
        }
        // Establish the permanent reader lock before the generation is visible.
        let _generation_lock = storage::process_entry_lock(&parent, &generation_id)?;
        let stage_name = format!("{}.stage-{}", generation_id, token());
        let _stage_lease =
            ProcessLease::exclusive(storage::process_entry_lock(&parent, &stage_name)?)?;
        let stage = parent.join(&stage_name);
        let result = (|| {
            #[cfg(test)]
            if std::env::var_os("SIFR_DX13_STORAGE_FULL").is_some() {
                storage::new_private_file(&stage)?.write_all(b"partial")?;
                return Err(io::Error::other("injected storage full"));
            }
            sifr_compiler_services::project_history::write(&stage, &manifest, bytes)?;
            #[cfg(test)]
            super::tests::pause("before-rename");
            cancelled(cancel)?;
            storage::publish(&stage, &destination)?;
            #[cfg(test)]
            super::tests::pause("after-rename");

            let _validated = self.generation(&generation_id)?;
            self.point(&generation_id)?;
            Ok(generation_id.clone())
        })();
        if stage.exists() {
            let _ = fs::remove_file(stage);
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
        #[cfg(test)]
        super::tests::pause("pointer-scratch");
        storage::publish(&scratch, &self.root.join("latest"))?;
        // Hint failures never invalidate the complete user-cache generation.
        let scratch = self
            .workspace_root
            .join(format!(".sifrbuildinfo.stage-{}", token()));
        if write_advisory_hint(&scratch, &hint).is_ok() {
            let _ = fs::rename(&scratch, self.workspace_root.join(".sifrbuildinfo"));
            let _ = fs::remove_file(&scratch);
        }
        Ok(())
    }
    /// Explicit pressure cleanup retains the latest bounded history and leased
    /// predecessors. Evicted older records are permitted to miss.
    pub(super) fn prune(
        &self,
        pressure: bool,
        dry_run: bool,
    ) -> io::Result<super::ProjectPruneReport> {
        super::housekeeping::prune_store(self, pressure, dry_run)
    }
}
impl Generation {
    fn id(&self) -> io::Result<&str> {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| key(name))
            .ok_or_else(|| invalid("invalid generation identity"))
    }
    #[cfg(test)]
    pub(super) fn record_path(&self, record: &str) -> io::Result<PathBuf> {
        self.manifest.record_path(&self.path, record)
    }
    pub(super) fn read_record(&self, record: &str) -> io::Result<Vec<u8>> {
        self.manifest.read_record(&self.path, record)
    }
    pub(super) fn records(&self) -> impl Iterator<Item = io::Result<CompletedCheck>> + '_ {
        self.manifest.records.iter().rev().map(|record| {
            let bytes = self.read_record(record)?;
            self.validation
                .validate(record, &bytes, "changed immutable record")?;
            serde_json::from_slice(&bytes).map_err(io::Error::other)
        })
    }
}

fn write_advisory_hint(path: &Path, bytes: &[u8]) -> io::Result<()> {
    // This optional discovery hint cannot strengthen the committed cache
    // pointer. Atomic visibility is sufficient; loss after a crash is a miss.
    storage::new_private_file(path)?.write_all(bytes)
}

#[cfg(test)]
mod bounded_read_tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn bounded_reads_preserve_empty_exact_and_changed_lengths() {
        for (bytes, expected) in [
            (b"".as_slice(), 0),
            (b"0123456789abcdef".as_slice(), 16),
            (b"short".as_slice(), 16),
            (b"grew".as_slice(), 0),
        ] {
            assert_eq!(
                read_bounded(Cursor::new(bytes), expected, 16).unwrap(),
                bytes
            );
        }
        assert!(read_bounded(Cursor::new(b"0123456789abcdefg"), 0, 16).is_err());
        assert!(read_bounded(Cursor::new(b"short"), 17, 16).is_err());
        assert!(read_bounded(Cursor::new(b""), 0, u64::MAX).is_err());
    }

    #[test]
    fn bounded_reads_preserve_io_failure() {
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected read failure",
                ))
            }
        }
        assert_eq!(
            read_bounded(Broken, 1, 16).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
    }
}
