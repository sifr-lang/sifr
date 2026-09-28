//! Read-only access to complete driver-published check generations.
use crate::CompilerContext;
use serde::Deserialize;
use sifr_frontend::{
    ModuleCheckDecision, WorkspaceSession,
    persistence::{CompletedCheck, SemanticInputs, identity},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

const RECORD_LIMIT: u64 = 16 * 1024 * 1024;
const MANIFEST_LIMIT: u64 = 1024 * 1024;
const HISTORY_BYTES: u64 = 64 * 1024 * 1024;
const RECORD_COUNT: usize = 128;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Owner {
    schema: u32,
    workspace: PathBuf,
    device: u64,
    inode: u64,
    created_ns: u128,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Hint {
    schema: u32,
    workspace: String,
    context: String,
    generation: String,
}
#[derive(Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u32,
    workspace: String,
    context: String,
    records: Vec<String>,
}

pub(super) struct Generation {
    records: Vec<CompletedCheck>,
    _namespace_lease: File,
    _generation_lease: File,
}

fn key(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
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
fn check_owner(cache: &Path, workspace: &Path, id: &str) -> io::Result<File> {
    let projects = cache.join("projects");
    let lease = existing_lease(&projects, id)?;
    lease.try_lock_shared()?;
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
    let root = cache.join("projects").join(&workspace_id).join(context);
    let pointer = read(&root, "latest", 4096)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Hint>(&bytes).ok());
    let hint = read(&workspace, ".sifrbuildinfo", 4096)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Hint>(&bytes).ok());
    for candidate in pointer.into_iter().chain(hint) {
        if candidate.schema != 1
            || candidate.workspace != workspace_id
            || candidate.context != context
            || !key(&candidate.generation)
        {
            continue;
        }
        let parent = root.join("generations");
        let Ok(generation_lease) = existing_lease(&parent, &candidate.generation) else {
            continue;
        };
        if generation_lease.try_lock_shared().is_err() {
            continue;
        }
        let path = parent.join(&candidate.generation);
        let Ok(records) =
            read_complete_generation(&path, &workspace_id, context, &candidate.generation)
        else {
            continue;
        };
        return Ok(Generation {
            records,
            _namespace_lease: namespace_lease,
            _generation_lease: generation_lease,
        });
    }
    Err(invalid("no complete project generation"))
}
pub(super) fn read_complete_generation(
    path: &Path,
    workspace: &str,
    context: &str,
    generation: &str,
) -> io::Result<Vec<CompletedCheck>> {
    let manifest: Manifest = serde_json::from_slice(&read(path, "manifest.json", MANIFEST_LIMIT)?)?;
    if manifest.schema != 2
        || manifest.workspace != workspace
        || manifest.context != context
        || manifest.records.len() > RECORD_COUNT
        || manifest.records.iter().collect::<BTreeSet<_>>().len() != manifest.records.len()
        || stamp("project-generation-v1", &manifest)? != generation
    {
        return Err(invalid("incompatible project generation"));
    }
    let mut records = Vec::new();
    let mut total_bytes = 0;
    for record_id in manifest.records.iter().rev() {
        if !key(record_id) {
            return Err(invalid("invalid record reference"));
        }
        let bytes = read(path, record_id, RECORD_LIMIT)?;
        if stamp("project-record-v1", &bytes)? != *record_id {
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
