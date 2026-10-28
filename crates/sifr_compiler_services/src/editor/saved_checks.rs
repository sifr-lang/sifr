//! Read-only access to complete driver-published check generations.
use crate::{
    CompilerContext,
    project_history::{Manifest, PROJECTS, READER_ADMISSION, key, latest_generation},
};
use serde::Deserialize;
use sifr_cache_storage::ProcessLease;
use sifr_frontend::{
    ModuleCheckDecision, WorkspaceSession,
    persistence::{CompletedCheck, SemanticInputs, identity},
};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Owner {
    schema: u32,
    workspace: PathBuf,
    device: u64,
    inode: u64,
    created_ns: u128,
}
pub(super) struct Generation {
    records: Vec<CompletedCheck>,
    _namespace_lease: ProcessLease,
    _generation_lease: ProcessLease,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn stamp<T: serde::Serialize>(domain: &str, value: &T) -> io::Result<String> {
    identity(domain, value).map_err(io::Error::other)
}
fn read(root: &Path, name: &str, limit: u64) -> io::Result<Vec<u8>> {
    sifr_cache_storage::payload(root, Path::new(name))?;
    let file = sifr_cache_storage::read_private_file(&root.join(name))?;
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
fn existing_lease(parent: &Path, name: &str) -> io::Result<File> {
    let locks = parent.join(".locks");
    if !locks.ancestors().all(sifr_cache_storage::real_directory) {
        return Err(invalid("unsafe lock directory"));
    }
    sifr_cache_storage::payload(&locks, Path::new(name))?;
    let lease = sifr_cache_storage::read_write_private_file(&locks.join(name))?;
    if !lease.metadata()?.is_file() {
        return Err(invalid("unsafe lock file"));
    }
    Ok(lease)
}
#[cfg(unix)]
fn file_identity(path: &Path) -> io::Result<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    let metadata = fs::metadata(path)?;
    Ok((metadata.dev(), metadata.ino()))
}
#[cfg(windows)]
fn file_identity(path: &Path) -> io::Result<(u64, u64)> {
    sifr_cache_storage::windows_storage_security::path_identity(path)
}
fn check_owner(cache: &Path, workspace: &Path, id: &str) -> io::Result<ProcessLease> {
    let projects = cache.join(PROJECTS);
    let lease = ProcessLease::shared(existing_lease(&projects, id)?)?;
    let namespace = projects.join(id);
    let owner: Owner = serde_json::from_slice(&read(&namespace, "owner.json", 16 * 1024)?)?;
    let (device, inode) = file_identity(workspace)?;
    let created_ns = fs::metadata(workspace)?
        .created()?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    if owner.schema != 1
        || owner.workspace != workspace
        || owner.device != device
        || owner.inode != inode
        || owner.created_ns != created_ns
    {
        return Err(invalid("project namespace owner changed"));
    }
    Ok(lease)
}
pub(super) fn load_generation(
    cache: &Path,
    workspace: &Path,
    context: &str,
) -> io::Result<Generation> {
    if !key(context) {
        return Err(invalid("invalid context identity"));
    }
    let workspace = workspace.canonicalize()?;
    let workspace_id = stamp("project-workspace-v1", &workspace)?;
    let namespace_lease = check_owner(cache, &workspace, &workspace_id)?;
    let root = cache.join(PROJECTS).join(&workspace_id).join(context);
    let parent = root.join("generations");
    let generation =
        latest_generation(&parent)?.ok_or_else(|| invalid("no complete project generation"))?;
    let admission = ProcessLease::shared(existing_lease(&root, READER_ADMISSION)?)?;
    let generation_lease = ProcessLease::shared(existing_lease(&parent, &generation)?)?;
    drop(admission);
    let records = read_complete_generation(
        &parent.join(&generation),
        &workspace_id,
        context,
        &generation,
    )?;
    Ok(Generation {
        records,
        _namespace_lease: namespace_lease,
        _generation_lease: generation_lease,
    })
}

pub(super) fn read_complete_generation(
    path: &Path,
    workspace: &str,
    context: &str,
    generation: &str,
) -> io::Result<Vec<CompletedCheck>> {
    let manifest = Manifest::read(path, workspace, context, generation)?;
    let mut records = Vec::new();
    for record_id in manifest.records.iter().rev() {
        let bytes = manifest.read_record(path, record_id)?;
        if stamp("project-record-v1", &bytes)? != *record_id {
            return Err(invalid("corrupt project record"));
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
                != Some(context)
            || record.diagnostics().is_err()
        {
            return Err(invalid("incomplete project record"));
        }
        records.push(record);
    }
    Ok(records)
}

pub fn saved_check_policy(file: &Path, cwd: &Path) -> Result<String, String> {
    identity("saved-check-policy-v1", &(file.parent(), cwd))
        .map_err(|error| format!("could not serialize saved-check policy: {error}"))
}

pub fn manifestless_inputs(
    compiler: &CompilerContext,
    metadata: &str,
    file: &Path,
) -> Result<SemanticInputs, String> {
    let cwd = std::env::current_dir()
        .map_err(|error| format!("could not read saved-check working directory: {error}"))?;
    let policy = saved_check_policy(file, &cwd)?;
    Ok(SemanticInputs {
        compiler: compiler.identity().as_str().into(),
        metadata: metadata.into(),
        target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        workspace_and_source_policy: policy,
        package_and_lock: "manifestless-owner-v1".into(),
        language_options: "ordinary-check-defaults-v1".into(),
        diagnostic_policy: "canonical-source-diagnostics-v1".into(),
        components: BTreeMap::new(),
        required_external: Default::default(),
        external: BTreeMap::new(),
    })
}

/// Reuse only checks from the current semantic, metadata, and cache owner.
/// The frontend validates current disk observations and editor overlay bytes.
pub fn restore_editor_checks(
    compiler: &CompilerContext,
    session: &mut WorkspaceSession,
) -> Vec<ModuleCheckDecision> {
    if !compiler.project_incremental() {
        return Vec::new();
    }
    let Some(frontend) = session.context() else {
        return Vec::new();
    };
    let graph = frontend.module_graph();
    let Some(entry) = graph
        .modules
        .iter()
        .find(|module| module.id == graph.entrypoint)
    else {
        return Vec::new();
    };
    let file = entry.canonical_path.as_path();
    let Some(workspace) = file.parent() else {
        return Vec::new();
    };
    let Ok(metadata) = compiler.metadata_provider() else {
        return Vec::new();
    };
    let Ok(inputs) = manifestless_inputs(compiler, &metadata.metadata.metadata_id, file) else {
        return Vec::new();
    };
    let Ok(context) = inputs.identity() else {
        return Vec::new();
    };
    let Ok(generation) = load_generation(compiler.cache_root(), workspace, &context) else {
        return Vec::new();
    };
    for record in &generation.records {
        if record.result.inputs.source.path == file {
            if let Some(decisions) = session.restore_saved_checks(record, &inputs) {
                return decisions;
            }
        }
    }
    Vec::new()
}

#[cfg(all(test, unix))]
#[path = "saved_check_lease_tests.rs"]
mod lease_tests;
