//! Explicit owner-scoped cleanup. The stable namespace lock is outside the
//! namespace; stores and detached readers hold it shared, orphan GC exclusively.
use super::storage::{Hint, Store, invalid, key, read, stamp, write_new};
use crate::cache_storage;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Component, Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Default, Serialize)]
pub struct ProjectPruneReport {
    /// Recognized candidate entries inspected, including protected entries.
    pub examined_entries: usize,
    /// Safe candidates under pressure; dry-run reports these without deletion.
    pub eligible_entries: usize,
    pub deleted_entries: usize,
    pub eligible_generations: usize,
    pub deleted_generations: usize,
}
impl ProjectPruneReport {
    fn add(&mut self, other: &Self) {
        self.examined_entries += other.examined_entries;
        self.eligible_entries += other.eligible_entries;
        self.deleted_entries += other.deleted_entries;
        self.eligible_generations += other.eligible_generations;
        self.deleted_generations += other.deleted_generations;
    }
    fn reclaim(&mut self, path: &Path, generation: bool, dry_run: bool) -> io::Result<()> {
        if !safe_tree(path) {
            return Ok(());
        }
        self.eligible_entries += 1;
        self.eligible_generations += usize::from(generation);
        if !dry_run {
            if fs::symlink_metadata(path)?.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
            self.deleted_entries += 1;
            self.deleted_generations += usize::from(generation);
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Owner {
    schema: u32,
    workspace: PathBuf,
    device: u64,
    inode: u64,
    created_ns: u128,
}

fn created_ns(metadata: &fs::Metadata) -> io::Result<u128> {
    Ok(metadata
        .created()?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos())
}

pub(super) fn open_namespace(cache: &Path, workspace: &Path, id: &str) -> io::Result<Arc<File>> {
    let projects = cache.join("projects");
    let lease = cache_storage::process_entry_lock(&projects, id)?;
    lease.try_lock_shared().map_err(io::Error::from)?;
    let namespace = projects.join(id);
    cache_storage::directory(&namespace)?;
    // Never retroactively claim unknown pre-existing payloads for this owner.
    // An ownerless interrupted/old namespace remains unavailable and protected.
    if namespace.join("owner.json").symlink_metadata().is_err()
        && fs::read_dir(&namespace)?.next().is_some()
    {
        return Err(invalid("unrecorded project namespace ownership"));
    }
    let metadata = fs::metadata(workspace)?;
    let expected = Owner {
        schema: 1,
        workspace: workspace.into(),
        device: metadata.dev(),
        inode: metadata.ino(),
        created_ns: created_ns(&metadata)?,
    };
    // Immutable owner records are never replaced. A competing creator may leave
    // a briefly incomplete record; optional persistence simply misses that run.
    match write_new(
        &namespace.join("owner.json"),
        &serde_json::to_vec(&expected)?,
    ) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    let owner: Owner = serde_json::from_slice(&read(&namespace, "owner.json", 16 * 1024)?)?;
    if owner.schema != 1
        || owner.workspace != expected.workspace
        || owner.device != expected.device
        || owner.inode != expected.inode
        || owner.created_ns != expected.created_ns
    {
        return Err(invalid("project namespace owner changed"));
    }
    Ok(Arc::new(lease))
}

/// Check existing ancestors without creating directories or following aliases.
fn safe_path(path: &Path) -> bool {
    path.is_absolute()
        && path.ancestors().all(|part| {
            fs::symlink_metadata(part).is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink())
        })
        && cache_storage::check_owned(path).is_ok()
}
fn safe_tree(path: &Path) -> bool {
    if cache_storage::check_owned(path).is_err() {
        return false;
    }
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() => true,
        Ok(meta) if meta.is_dir() => fs::read_dir(path).is_ok_and(|entries| {
            entries
                .into_iter()
                .all(|entry| entry.is_ok_and(|entry| safe_tree(&entry.path())))
        }),
        _ => false,
    }
}
pub(super) fn existing_lease(parent: &Path, name: &str) -> io::Result<File> {
    let locks = parent.join(".locks");
    if !safe_path(&locks) {
        return Err(invalid("unsafe lock directory"));
    }
    let path = locks.join(name);
    cache_storage::check_owned(&path)?;
    let lease = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    if !lease.metadata()?.is_file() {
        return Err(invalid("unsafe lock file"));
    }
    Ok(lease)
}
fn lock_tree(path: &Path, leases: &mut Vec<File>) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if entry.file_name() == ".locks" {
                for lock in fs::read_dir(entry.path())? {
                    let lock = lock?;
                    let name = lock.file_name();
                    let name = name.to_str().ok_or_else(|| invalid("invalid lock name"))?;
                    let lease = existing_lease(path, name)?;
                    lease.try_lock().map_err(io::Error::from)?;
                    leases.push(lease);
                }
            } else {
                lock_tree(&entry.path(), leases)?;
            }
        }
    }
    Ok(())
}
fn token(value: &str) -> bool {
    value.split_once('-').is_some_and(|(pid, time)| {
        !pid.is_empty()
            && !time.is_empty()
            && pid
                .bytes()
                .chain(time.bytes())
                .all(|byte| byte.is_ascii_digit())
    })
}
fn stage(name: &str) -> bool {
    name.split_once(".stage-")
        .is_some_and(|(id, suffix)| key(id) && token(suffix))
}

pub(super) fn prune_store(
    store: &Store,
    pressure: bool,
    dry_run: bool,
) -> io::Result<ProjectPruneReport> {
    let mut report = ProjectPruneReport::default();
    if !pressure || !safe_path(&store.root) {
        return Ok(report);
    }
    let Ok(writer) = existing_lease(&store.root, "writer") else {
        return Ok(report); // No established writer authority: preserve the context.
    };
    writer.try_lock().map_err(io::Error::from)?;
    let latest = store.latest();
    let parent = store.root.join("generations");
    if !safe_path(&parent) {
        return Ok(report);
    }
    for entry in fs::read_dir(&parent)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !key(name) && !stage(name) {
            continue;
        }
        report.examined_entries += 1;
        if latest
            .as_ref()
            .is_some_and(|latest| latest.path == entry.path())
        {
            continue;
        }
        // If the latest pointer exists but cannot be validated, ownership of
        // current history is ambiguous. Staging can still be reclaimed safely.
        if key(name) && latest.is_none() && store.root.join("latest").symlink_metadata().is_ok() {
            continue;
        }
        if !safe_tree(&entry.path()) {
            continue;
        }
        let Ok(lease) = existing_lease(&parent, name) else {
            continue;
        };
        if lease.try_lock().is_err() {
            continue;
        }
        report.reclaim(&entry.path(), key(name), dry_run)?;
    }
    // Only writer-serialized staging locks are ephemeral. Generation locks
    // remain permanent because readers may open them without the writer lock.
    let locks = parent.join(".locks");
    if safe_path(&locks) {
        for entry in fs::read_dir(&locks)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if !stage(name) {
                continue;
            }
            report.examined_entries += 1;
            // Dry-run evaluates the same eligible stages without deleting them.
            if parent.join(name).symlink_metadata().is_ok()
                && (!dry_run || !safe_tree(&parent.join(name)))
            {
                continue;
            }
            if !safe_tree(&entry.path()) {
                continue;
            }
            let Ok(lease) = existing_lease(&parent, name) else {
                continue;
            };
            if lease.try_lock().is_err() {
                continue;
            }
            report.reclaim(&entry.path(), false, dry_run)?;
        }
    }
    for entry in fs::read_dir(&store.root)? {
        let entry = entry?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.strip_prefix("latest.stage-").is_some_and(token))
        {
            report.examined_entries += 1;
            report.reclaim(&entry.path(), false, dry_run)?;
        }
    }
    // Workspace scratch shares the directory with other semantic contexts.
    // Only a complete matching hint proves which writer owns it; partial or
    // foreign scratch stays untouched. No generic directory cleanup is allowed.
    if safe_path(&store.workspace_root) {
        for entry in fs::read_dir(&store.workspace_root)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if !name
                .strip_prefix(".sifrbuildinfo.stage-")
                .is_some_and(token)
            {
                continue;
            }
            report.examined_entries += 1;
            let hint = read(&store.workspace_root, name, 4096)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<Hint>(&bytes).ok());
            if hint.is_some_and(|hint| {
                hint.schema == 1
                    && hint.workspace == store.workspace
                    && hint.context == store.context
            }) {
                report.reclaim(&entry.path(), false, dry_run)?;
            }
        }
    }
    Ok(report)
}

// An absent original path is proof only when every existing ancestor is a real
// directory. Symlinks, replacement objects, traversal and inaccessible paths
// are ambiguous, including dangling symlinks that Path::exists calls absent.
fn definitely_absent(path: &Path) -> bool {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::CurDir | Component::ParentDir))
    {
        return false;
    }
    let mut current = PathBuf::new();
    for part in path.components() {
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return true,
            _ => return false,
        }
    }
    false
}

pub(super) fn prune_workspace(
    cache: &Path,
    workspace: &Path,
    pressure: bool,
    dry_run: bool,
) -> io::Result<ProjectPruneReport> {
    let mut report = ProjectPruneReport::default();
    if !pressure {
        return Ok(report);
    }
    let requested_absolute = workspace.is_absolute();
    let absolute = if workspace.is_absolute() {
        workspace.to_path_buf()
    } else {
        std::env::current_dir()?.join(workspace)
    };
    let workspace = absolute.as_path();
    let orphan = definitely_absent(workspace);
    if orphan && !requested_absolute {
        return Err(invalid(
            "deleted workspace requires its original absolute canonical path",
        ));
    }
    if !orphan && !safe_path(workspace) {
        return Ok(report);
    }
    let workspace = if orphan {
        workspace.to_path_buf()
    } else {
        workspace.canonicalize()?
    };
    let id = stamp("project-workspace-v1", &workspace)?;
    let projects = cache.join("projects");
    let namespace = projects.join(&id);
    if !safe_path(cache) || !safe_path(&projects) || !safe_path(&namespace) {
        return Ok(report);
    }
    let Ok(lease) = existing_lease(&projects, &id) else {
        return Ok(report);
    };
    if orphan {
        if lease.try_lock().is_err() {
            return Ok(report);
        }
    } else if lease.try_lock_shared().is_err() {
        return Ok(report);
    }
    let owner = read(&namespace, "owner.json", 16 * 1024)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Owner>(&bytes).ok());
    let Some(owner) = owner else {
        return Ok(report);
    };
    if owner.schema != 1 || owner.workspace != workspace {
        return Ok(report);
    }
    if !orphan
        && !fs::metadata(&workspace).is_ok_and(|m| {
            m.dev() == owner.device
                && m.ino() == owner.inode
                && created_ns(&m).ok() == Some(owner.created_ns)
        })
    {
        return Ok(report);
    }
    for entry in fs::read_dir(&namespace)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(context) = name.to_str().filter(|name| key(name)) else {
            continue;
        };
        if orphan {
            report.examined_entries += 1;
            // Recheck absence immediately before each deletion. The namespace
            // lease blocks all cooperating new stores and detached readers.
            let mut leases = Vec::new();
            if safe_tree(&entry.path())
                && lock_tree(&entry.path(), &mut leases).is_ok()
                && definitely_absent(&workspace)
            {
                report.reclaim(&entry.path(), false, dry_run)?;
            }
        } else {
            if !safe_path(&entry.path()) {
                continue;
            }
            // Pruning is read-only until a selected deletion, including dry-run:
            // do not use Store::open, which establishes missing owner directories.
            let store = Store {
                root: entry.path(),
                workspace_root: workspace.clone(),
                workspace: id.clone(),
                context: context.into(),
                namespace_lease: Arc::new(lease.try_clone()?),
            };
            match store.prune(true, dry_run) {
                Ok(part) => report.add(&part),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => return Err(error),
            }
        }
    }
    // Keep owner.json and the external namespace lock as a stable tombstone.
    // Never recursively remove another workspace or guess at unrecorded owners.
    Ok(report)
}
