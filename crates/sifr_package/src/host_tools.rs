use crate::digest::lower_hex;
use crate::{
    CargoPackage, CargoPackageId, NormalizedCargoMetadata, PackageClassification,
    PackageDiagnostic, PackageGraphSnapshot,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sifr_frontend::SourceProvider;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Write as _;
use std::path::{Path, PathBuf};

pub const HOST_TOOL_LOCK_FILE: &str = "sifr-tools.lock.json";
pub const HOST_TOOL_LOCK_VERSION: u32 = 1;

pub const HOST_TOOL_CAPABILITIES: &[&str] = &[
    "credentials",
    "environment",
    "network",
    "project-read",
    "project-write",
    "subprocess",
];

pub const RESERVED_TOOL_NAMESPACES: &[&str] = &[
    "bridge", "build", "check", "doctor", "emit", "fetch", "fmt", "help", "init", "lint", "lsp",
    "package", "publish", "python", "repair", "run", "self", "test", "trace", "tree", "vendor",
    "tools", "version",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostToolEntrypoint {
    pub namespace: String,
    pub package_id: CargoPackageId,
    pub package_name: String,
    pub package_version: String,
    pub package_source: Option<String>,
    pub package_checksum: String,
    pub package_root: PathBuf,
    pub entrypoint: String,
    pub capabilities: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostToolGraph {
    pub workspace_root: PathBuf,
    pub target_directory: PathBuf,
    pub tools_package_id: CargoPackageId,
    pub tools_package_name: String,
    pub tools_manifest: PathBuf,
    pub tools_manifest_fingerprint: String,
    pub lockfile: PathBuf,
    pub lockfile_fingerprint: String,
    pub entries: BTreeMap<String, HostToolEntrypoint>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostToolBuildPlan {
    pub args: Vec<String>,
    pub current_dir: PathBuf,
    pub namespace: String,
    pub package_id: CargoPackageId,
    pub package_checksum: String,
    pub capabilities: BTreeSet<String>,
    pub entrypoint: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HostToolLockArtifact {
    pub schema_version: u32,
    pub tools_package: String,
    pub tools_manifest_fingerprint: String,
    pub cargo_lock_fingerprint: String,
    pub entries: BTreeMap<String, LockedHostToolEntrypoint>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LockedHostToolEntrypoint {
    pub package_identity: String,
    pub package_checksum: String,
    pub entrypoint: String,
    pub capabilities: BTreeSet<String>,
}

impl HostToolGraph {
    pub fn build_plan(
        &self,
        namespace: &str,
        host_triple: &str,
    ) -> Result<HostToolBuildPlan, PackageDiagnostic> {
        if host_triple.is_empty() || host_triple.starts_with('-') {
            return Err(host_tool_diagnostic("host tool target triple is invalid"));
        }
        let entry = self.entries.get(namespace).ok_or_else(|| {
            host_tool_diagnostic(format!(
                "unknown tool namespace '{namespace}'; configure it in the tools package sifr.toml"
            ))
        })?;
        Ok(HostToolBuildPlan {
            args: vec![
                "build".to_string(),
                "--locked".to_string(),
                "--package".to_string(),
                entry.package_name.clone(),
                "--bin".to_string(),
                entry.entrypoint.clone(),
                "--target".to_string(),
                host_triple.to_string(),
            ]
            .into_iter()
            .collect(),
            current_dir: self.workspace_root.clone(),
            namespace: namespace.to_string(),
            package_id: entry.package_id.clone(),
            package_checksum: entry.package_checksum.clone(),
            capabilities: entry.capabilities.clone(),
            entrypoint: entry.entrypoint.clone(),
        })
    }
}

impl HostToolLockArtifact {
    #[must_use]
    pub fn from_graph(graph: &HostToolGraph) -> Self {
        Self {
            schema_version: HOST_TOOL_LOCK_VERSION,
            tools_package: graph.tools_package_name.clone(),
            tools_manifest_fingerprint: graph.tools_manifest_fingerprint.clone(),
            cargo_lock_fingerprint: graph.lockfile_fingerprint.clone(),
            entries: graph
                .entries
                .iter()
                .map(|(namespace, entry)| {
                    (
                        namespace.clone(),
                        LockedHostToolEntrypoint {
                            package_identity: normalized_package_identity(entry),
                            package_checksum: entry.package_checksum.clone(),
                            entrypoint: entry.entrypoint.clone(),
                            capabilities: entry.capabilities.clone(),
                        },
                    )
                })
                .collect(),
        }
    }

    pub fn from_json(source: &str) -> Result<Self, PackageDiagnostic> {
        let artifact: Self = serde_json::from_str(source).map_err(|error| {
            host_tool_diagnostic(format!("cannot parse {HOST_TOOL_LOCK_FILE}: {error}"))
        })?;
        if artifact.schema_version != HOST_TOOL_LOCK_VERSION {
            return Err(host_tool_diagnostic(format!(
                "unsupported {HOST_TOOL_LOCK_FILE} version {}",
                artifact.schema_version
            )));
        }
        Ok(artifact)
    }

    pub fn to_canonical_json(&self) -> Result<String, PackageDiagnostic> {
        serde_json::to_string_pretty(self).map_err(|error| {
            host_tool_diagnostic(format!("cannot serialize {HOST_TOOL_LOCK_FILE}: {error}"))
        })
    }

    pub fn verify_graph(&self, graph: &HostToolGraph) -> Result<(), PackageDiagnostic> {
        let observed = Self::from_graph(graph);
        if self != &observed {
            return Err(host_tool_diagnostic(format!(
                "{HOST_TOOL_LOCK_FILE} does not match the resolved tool graph; run `sifr tools lock` and review the change"
            )));
        }
        Ok(())
    }
}

pub fn resolve_host_tool_graph(
    snapshot: &PackageGraphSnapshot,
    provider: &mut impl SourceProvider,
) -> Result<HostToolGraph, Vec<PackageDiagnostic>> {
    let metadata = &snapshot.metadata;
    let Some(tools_package_name) = metadata.workspace_sifr.tools_package.as_deref() else {
        return Err(vec![host_tool_diagnostic(
            "workspace does not configure [workspace.metadata.sifr].tools-package",
        )]);
    };
    let tools_matches = metadata
        .workspace_members
        .iter()
        .filter_map(|id| metadata.packages.get(id))
        .filter(|package| package.name == tools_package_name)
        .collect::<Vec<_>>();
    if tools_matches.len() != 1 {
        return Err(vec![host_tool_diagnostic(format!(
            "tools package '{tools_package_name}' must name exactly one workspace member; found {}",
            tools_matches.len()
        ))]);
    }
    let tools_package = tools_matches[0];
    let Some(discovery) = tools_package.sifr_metadata.as_ref() else {
        return Err(vec![host_tool_diagnostic(format!(
            "tools package '{}' requires [package.metadata.sifr].manifest",
            tools_package.name
        ))]);
    };
    let tools_root = package_root(tools_package);
    let tools_manifest = tools_root.join(&discovery.manifest);
    let manifest_source = provider.read_file(&tools_manifest).map_err(|error| {
        vec![host_tool_diagnostic(format!(
            "cannot read tools manifest '{}': {error}",
            tools_manifest.display()
        ))]
    })?;
    let declarations = parse_tool_declarations(&tools_manifest, manifest_source.as_str())?;
    let tools_manifest_fingerprint = sha256(manifest_source.as_str().as_bytes());
    let lockfile = metadata.workspace_root.join("Cargo.lock");
    let lock_source = provider.read_file(&lockfile).map_err(|error| {
        vec![host_tool_diagnostic(format!(
            "host tools require the workspace Cargo.lock '{}': {error}",
            lockfile.display()
        ))]
    })?;
    let lockfile_fingerprint = sha256(lock_source.as_str().as_bytes());
    let lock_packages = parse_lock_packages(lock_source.as_str())?;
    let direct = direct_normal_dependencies(metadata, tools_package);
    let mut entries = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for (namespace, declaration) in declarations {
        let matches = direct
            .iter()
            .filter_map(|id| metadata.packages.get(*id))
            .filter(|package| package.name == declaration.package)
            .filter(|package| metadata.workspace_members.contains(&package.id))
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            diagnostics.push(host_tool_diagnostic(format!(
                "tool namespace '{namespace}' selects package '{}' but the tools member has {} exact normal workspace-member matches",
                declaration.package,
                matches.len()
            )));
            continue;
        }
        let package = matches[0];
        if !package
            .targets
            .iter()
            .any(|target| target.name == declaration.entrypoint && target.kind.contains("bin"))
        {
            diagnostics.push(host_tool_diagnostic(format!(
                "tool namespace '{namespace}' selects missing binary entrypoint '{}' in package '{}'",
                declaration.entrypoint, package.name
            )));
            continue;
        }
        let package_checksum = match exact_package_checksum(package, &lock_packages, metadata) {
            Ok(checksum) => checksum,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        entries.insert(
            namespace.clone(),
            HostToolEntrypoint {
                namespace,
                package_id: package.id.clone(),
                package_name: package.name.clone(),
                package_version: package.version.clone(),
                package_source: package.source.clone(),
                package_checksum,
                package_root: package_root(package),
                entrypoint: declaration.entrypoint,
                capabilities: declaration.capabilities,
            },
        );
    }
    diagnostics.extend(target_contamination_diagnostics(
        metadata,
        snapshot,
        &tools_package.id,
        entries.values().map(|entry| &entry.package_id),
    ));
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Ok(HostToolGraph {
        workspace_root: metadata.workspace_root.clone(),
        target_directory: metadata.target_directory.clone(),
        tools_package_id: tools_package.id.clone(),
        tools_package_name: tools_package.name.clone(),
        tools_manifest,
        tools_manifest_fingerprint,
        lockfile,
        lockfile_fingerprint,
        entries,
    })
}

pub fn validate_host_tool_application_isolation(
    metadata: &NormalizedCargoMetadata,
    provider: &mut impl SourceProvider,
) -> Result<(), Vec<PackageDiagnostic>> {
    let Some(tools_package_name) = metadata.workspace_sifr.tools_package.as_deref() else {
        return Ok(());
    };
    let tools_matches = metadata
        .workspace_members
        .iter()
        .filter_map(|id| metadata.packages.get(id))
        .filter(|package| package.name == tools_package_name)
        .collect::<Vec<_>>();
    if tools_matches.len() != 1 {
        return Err(vec![host_tool_diagnostic(format!(
            "tools package '{tools_package_name}' must name exactly one workspace member; found {}",
            tools_matches.len()
        ))]);
    }
    let tools_package = tools_matches[0];
    let Some(discovery) = tools_package.sifr_metadata.as_ref() else {
        return Err(vec![host_tool_diagnostic(format!(
            "tools package '{}' requires [package.metadata.sifr].manifest",
            tools_package.name
        ))]);
    };
    let manifest_path = package_root(tools_package).join(&discovery.manifest);
    let source = provider.read_file(&manifest_path).map_err(|error| {
        vec![host_tool_diagnostic(format!(
            "cannot read tools manifest '{}': {error}",
            manifest_path.display()
        ))]
    })?;
    let declarations = parse_tool_declarations(&manifest_path, source.as_str())?;
    let direct = direct_normal_dependencies(metadata, tools_package);
    let mut diagnostics = Vec::new();
    let mut tool_roots = BTreeSet::from([tools_package.id.clone()]);
    for (namespace, declaration) in declarations {
        let matches = direct
            .iter()
            .filter_map(|id| metadata.packages.get(*id))
            .filter(|package| package.name == declaration.package)
            .filter(|package| metadata.workspace_members.contains(&package.id))
            .collect::<Vec<_>>();
        if matches.len() == 1 {
            tool_roots.insert(matches[0].id.clone());
        } else {
            diagnostics.push(host_tool_diagnostic(format!(
                "tool namespace '{namespace}' requires one exact normal workspace-member package '{}'; found {}",
                declaration.package,
                matches.len()
            )));
        }
    }
    for application in metadata.workspace_members.iter().filter(|id| {
        !tool_roots.contains(*id)
            && metadata
                .packages
                .get(*id)
                .is_some_and(|package| package.sifr_metadata.is_some())
    }) {
        let closure = dependency_closure(metadata, application);
        for contaminated in closure.intersection(&tool_roots) {
            diagnostics.push(host_tool_diagnostic(format!(
                "application package '{}' reaches host-only tool package '{}'",
                application.0, contaminated.0
            )));
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub fn verify_host_tool_graph(
    graph: &HostToolGraph,
    provider: &mut impl SourceProvider,
) -> Result<(), PackageDiagnostic> {
    let source = provider.read_file(&graph.lockfile).map_err(|error| {
        host_tool_diagnostic(format!(
            "cannot verify host-tool lockfile '{}': {error}",
            graph.lockfile.display()
        ))
    })?;
    let observed = sha256(source.as_str().as_bytes());
    if observed != graph.lockfile_fingerprint {
        return Err(host_tool_diagnostic(format!(
            "host-tool lockfile hash drifted: expected {}, observed {observed}",
            graph.lockfile_fingerprint
        )));
    }
    let tools_source = provider.read_file(&graph.tools_manifest).map_err(|error| {
        host_tool_diagnostic(format!(
            "cannot verify tools manifest '{}': {error}",
            graph.tools_manifest.display()
        ))
    })?;
    let observed = sha256(tools_source.as_str().as_bytes());
    if observed != graph.tools_manifest_fingerprint {
        return Err(host_tool_diagnostic(format!(
            "tools manifest hash drifted: expected {}, observed {observed}",
            graph.tools_manifest_fingerprint
        )));
    }
    for entry in graph.entries.values() {
        if let Some(expected) = entry.package_checksum.strip_prefix("path-sha256:") {
            let observed = hash_path_package(&entry.package_root, &path_hash_exclusions(graph))?;
            if observed != expected {
                return Err(host_tool_diagnostic(format!(
                    "host-tool path package hash drifted for '{}': expected {expected}, observed {observed}",
                    entry.package_name
                )));
            }
        }
    }
    Ok(())
}

pub fn load_host_tool_lock(
    graph: &HostToolGraph,
    provider: &mut impl SourceProvider,
) -> Result<HostToolLockArtifact, PackageDiagnostic> {
    let path = graph.workspace_root.join(HOST_TOOL_LOCK_FILE);
    let source = provider.read_file(&path).map_err(|error| {
        host_tool_diagnostic(format!(
            "cannot read required host-tool lock '{}': {error}; run `sifr tools lock`",
            path.display()
        ))
    })?;
    let artifact = HostToolLockArtifact::from_json(source.as_str())?;
    artifact.verify_graph(graph)?;
    Ok(artifact)
}

pub fn write_host_tool_lock(graph: &HostToolGraph) -> Result<PathBuf, PackageDiagnostic> {
    let artifact = HostToolLockArtifact::from_graph(graph);
    let mut source = artifact.to_canonical_json()?;
    source.push('\n');
    let destination = graph.workspace_root.join(HOST_TOOL_LOCK_FILE);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| host_tool_diagnostic(format!("cannot identify lock write: {error}")))?
        .as_nanos();
    let temporary = graph.workspace_root.join(format!(
        ".{HOST_TOOL_LOCK_FILE}.{}.{nonce}.tmp",
        std::process::id(),
    ));
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| {
            host_tool_diagnostic(format!(
                "cannot create host-tool lock temporary file '{}': {error}",
                temporary.display()
            ))
        })?;
    file.write_all(source.as_bytes()).map_err(|error| {
        host_tool_diagnostic(format!(
            "cannot write host-tool lock temporary file '{}': {error}",
            temporary.display()
        ))
    })?;
    file.sync_all().map_err(|error| {
        host_tool_diagnostic(format!(
            "cannot sync host-tool lock temporary file '{}': {error}",
            temporary.display()
        ))
    })?;
    std::fs::rename(&temporary, &destination).map_err(|error| {
        host_tool_diagnostic(format!(
            "cannot replace host-tool lock '{}': {error}",
            destination.display()
        ))
    })?;
    Ok(destination)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ToolDeclaration {
    package: String,
    entrypoint: String,
    capabilities: BTreeSet<String>,
}

fn parse_tool_declarations(
    path: &Path,
    source: &str,
) -> Result<BTreeMap<String, ToolDeclaration>, Vec<PackageDiagnostic>> {
    let root = source.parse::<toml::Table>().map_err(|error| {
        vec![host_tool_diagnostic(format!(
            "cannot parse tools manifest '{}': {error}",
            path.display()
        ))]
    })?;
    let Some(tools) = root.get("tools").and_then(toml::Value::as_table) else {
        return Err(vec![host_tool_diagnostic(format!(
            "tools manifest '{}' requires a [tools] table",
            path.display()
        ))]);
    };
    let mut declarations = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for (namespace, value) in tools {
        if !valid_namespace(namespace) {
            diagnostics.push(host_tool_diagnostic(format!(
                "tool namespace '{namespace}' must use lowercase ASCII letters, digits, and single hyphens"
            )));
            continue;
        }
        if RESERVED_TOOL_NAMESPACES.contains(&namespace.as_str()) {
            diagnostics.push(host_tool_diagnostic(format!(
                "tool namespace '{namespace}' is reserved by Sifr"
            )));
            continue;
        }
        let Some(table) = value.as_table() else {
            diagnostics.push(host_tool_diagnostic(format!(
                "tools.{namespace} must be a table"
            )));
            continue;
        };
        if let Some(key) = table
            .keys()
            .find(|key| !matches!(key.as_str(), "package" | "entrypoint" | "capabilities"))
        {
            diagnostics.push(host_tool_diagnostic(format!(
                "tools.{namespace} contains unsupported key '{key}'"
            )));
            continue;
        }
        let Some(package) = required_nonempty_string(table, "package") else {
            diagnostics.push(host_tool_diagnostic(format!(
                "tools.{namespace}.package must be a non-empty string"
            )));
            continue;
        };
        let Some(entrypoint) = required_nonempty_string(table, "entrypoint") else {
            diagnostics.push(host_tool_diagnostic(format!(
                "tools.{namespace}.entrypoint must be a non-empty string"
            )));
            continue;
        };
        let capabilities = match parse_capabilities(namespace, table.get("capabilities")) {
            Ok(capabilities) => capabilities,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        declarations.insert(
            namespace.clone(),
            ToolDeclaration {
                package,
                entrypoint,
                capabilities,
            },
        );
    }
    if declarations.is_empty() && diagnostics.is_empty() {
        diagnostics.push(host_tool_diagnostic("tools manifest exports no namespaces"));
    }
    if diagnostics.is_empty() {
        Ok(declarations)
    } else {
        Err(diagnostics)
    }
}

fn parse_capabilities(
    namespace: &str,
    value: Option<&toml::Value>,
) -> Result<BTreeSet<String>, PackageDiagnostic> {
    let Some(values) = value.and_then(toml::Value::as_array) else {
        return Err(host_tool_diagnostic(format!(
            "tools.{namespace}.capabilities must be an explicit array"
        )));
    };
    let mut capabilities = BTreeSet::new();
    for value in values {
        let Some(capability) = value.as_str() else {
            return Err(host_tool_diagnostic(format!(
                "tools.{namespace}.capabilities entries must be strings"
            )));
        };
        if !HOST_TOOL_CAPABILITIES.contains(&capability) {
            return Err(host_tool_diagnostic(format!(
                "tools.{namespace} requests unknown capability '{capability}'"
            )));
        }
        if !capabilities.insert(capability.to_string()) {
            return Err(host_tool_diagnostic(format!(
                "tools.{namespace} repeats capability '{capability}'"
            )));
        }
    }
    Ok(capabilities)
}

fn valid_namespace(namespace: &str) -> bool {
    !namespace.is_empty()
        && !namespace.starts_with('-')
        && !namespace.ends_with('-')
        && !namespace.contains("--")
        && namespace
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn required_nonempty_string(table: &toml::Table, key: &str) -> Option<String> {
    table
        .get(key)
        .and_then(toml::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LockPackage {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
}

fn parse_lock_packages(source: &str) -> Result<Vec<LockPackage>, Vec<PackageDiagnostic>> {
    let value = source.parse::<toml::Table>().map_err(|error| {
        vec![host_tool_diagnostic(format!(
            "cannot parse Cargo.lock for host tools: {error}"
        ))]
    })?;
    let Some(packages) = value.get("package").and_then(toml::Value::as_array) else {
        return Err(vec![host_tool_diagnostic(
            "Cargo.lock contains no package records",
        )]);
    };
    packages
        .iter()
        .map(|value| {
            let table = value
                .as_table()
                .ok_or_else(|| host_tool_diagnostic("Cargo.lock package record is not a table"))?;
            let name = required_nonempty_string(table, "name")
                .ok_or_else(|| host_tool_diagnostic("Cargo.lock package has no name"))?;
            let version = required_nonempty_string(table, "version")
                .ok_or_else(|| host_tool_diagnostic("Cargo.lock package has no version"))?;
            Ok(LockPackage {
                name,
                version,
                source: table
                    .get("source")
                    .and_then(toml::Value::as_str)
                    .map(str::to_string),
                checksum: table
                    .get("checksum")
                    .and_then(toml::Value::as_str)
                    .map(str::to_string),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| vec![error])
}

fn exact_package_checksum(
    package: &CargoPackage,
    lock_packages: &[LockPackage],
    metadata: &NormalizedCargoMetadata,
) -> Result<String, PackageDiagnostic> {
    let matches = lock_packages
        .iter()
        .filter(|locked| {
            locked.name == package.name
                && locked.version == package.version
                && locked.source == package.source
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(host_tool_diagnostic(format!(
            "Cargo.lock must contain one exact record for host tool package '{} {}'; found {}",
            package.name,
            package.version,
            matches.len()
        )));
    }
    match &matches[0].checksum {
        Some(checksum) => Ok(checksum.clone()),
        None => hash_path_package(
            &package_root(package),
            &[
                metadata.target_directory.clone(),
                metadata.workspace_root.join(".git"),
                metadata.workspace_root.join("Cargo.lock"),
                metadata.workspace_root.join(HOST_TOOL_LOCK_FILE),
            ],
        )
        .map(|hash| format!("path-sha256:{hash}")),
    }
}

fn path_hash_exclusions(graph: &HostToolGraph) -> [PathBuf; 4] {
    [
        graph.target_directory.clone(),
        graph.workspace_root.join(".git"),
        graph.workspace_root.join("Cargo.lock"),
        graph.workspace_root.join(HOST_TOOL_LOCK_FILE),
    ]
}

fn hash_path_package(root: &Path, exclusions: &[PathBuf]) -> Result<String, PackageDiagnostic> {
    const MAX_FILES: usize = 10_000;
    const MAX_BYTES: u64 = 256 * 1024 * 1024;
    let mut pending = VecDeque::from([root.to_path_buf()]);
    let mut files = Vec::new();
    let mut total_bytes = 0u64;
    while let Some(directory) = pending.pop_front() {
        let entries = std::fs::read_dir(&directory).map_err(|error| {
            host_tool_diagnostic(format!(
                "cannot read host-tool package directory '{}': {error}",
                directory.display()
            ))
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                host_tool_diagnostic(format!("cannot read host-tool package entry: {error}"))
            })?;
            let path = entry.path();
            if exclusions.iter().any(|excluded| excluded == &path) {
                continue;
            }
            let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
                host_tool_diagnostic(format!(
                    "cannot inspect host-tool package path '{}': {error}",
                    path.display()
                ))
            })?;
            if metadata.file_type().is_symlink() {
                return Err(host_tool_diagnostic(format!(
                    "host-tool path package contains unsupported symlink '{}'",
                    path.display()
                )));
            }
            if metadata.is_dir() {
                pending.push_back(path);
            } else if metadata.is_file() {
                total_bytes = total_bytes.checked_add(metadata.len()).ok_or_else(|| {
                    host_tool_diagnostic("host-tool package byte count overflowed")
                })?;
                if total_bytes > MAX_BYTES {
                    return Err(host_tool_diagnostic(format!(
                        "host-tool path package exceeds {MAX_BYTES} bytes"
                    )));
                }
                files.push(path);
                if files.len() > MAX_FILES {
                    return Err(host_tool_diagnostic(format!(
                        "host-tool path package exceeds {MAX_FILES} files"
                    )));
                }
            }
        }
    }
    files.sort();
    let mut digest = Sha256::new();
    for path in files {
        let relative = path.strip_prefix(root).map_err(|_| {
            host_tool_diagnostic(format!(
                "host-tool package path '{}' escaped its root",
                path.display()
            ))
        })?;
        let bytes = std::fs::read(&path).map_err(|error| {
            host_tool_diagnostic(format!(
                "cannot read host-tool package file '{}': {error}",
                path.display()
            ))
        })?;
        digest.update(relative.as_os_str().as_encoded_bytes());
        digest.update([0]);
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(&bytes);
    }
    Ok(lower_hex(&digest.finalize()))
}

fn direct_normal_dependencies<'a>(
    metadata: &'a NormalizedCargoMetadata,
    from: &CargoPackage,
) -> Vec<&'a CargoPackageId> {
    metadata
        .resolve_edges
        .iter()
        .filter(|edge| edge.from == from.id)
        .filter(|edge| {
            let Some(target) = metadata.packages.get(&edge.to) else {
                return false;
            };
            from.dependencies.iter().any(|dependency| {
                dependency.kind.is_none()
                    && dependency
                        .package
                        .as_deref()
                        .unwrap_or(dependency.name.as_str())
                        == target.name
            })
        })
        .map(|edge| &edge.to)
        .collect()
}

fn target_contamination_diagnostics<'a>(
    metadata: &NormalizedCargoMetadata,
    snapshot: &PackageGraphSnapshot,
    tools_package: &CargoPackageId,
    entry_packages: impl Iterator<Item = &'a CargoPackageId>,
) -> Vec<PackageDiagnostic> {
    let tool_roots =
        BTreeSet::from_iter(std::iter::once(tools_package.clone()).chain(entry_packages.cloned()));
    let application_roots = snapshot
        .graph
        .classifications
        .iter()
        .filter_map(|(id, classification)| {
            matches!(
                classification,
                PackageClassification::SifrSource(_) | PackageClassification::RustBackedSifr(_)
            )
            .then_some(id.clone())
        })
        .collect::<Vec<_>>();
    let mut diagnostics = Vec::new();
    for application in application_roots {
        let closure = dependency_closure(metadata, &application);
        for contaminated in closure.intersection(&tool_roots) {
            diagnostics.push(host_tool_diagnostic(format!(
                "application package '{}' reaches host-only tool package '{}'",
                application.0, contaminated.0
            )));
        }
    }
    diagnostics
}

fn dependency_closure(
    metadata: &NormalizedCargoMetadata,
    root: &CargoPackageId,
) -> BTreeSet<CargoPackageId> {
    let mut closure = BTreeSet::new();
    let mut pending = VecDeque::from([root.clone()]);
    while let Some(current) = pending.pop_front() {
        if !closure.insert(current.clone()) {
            continue;
        }
        pending.extend(
            metadata
                .resolve_edges
                .iter()
                .filter(|edge| edge.from == current)
                .map(|edge| edge.to.clone()),
        );
    }
    closure
}

fn package_root(package: &CargoPackage) -> PathBuf {
    package
        .manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn normalized_package_identity(entry: &HostToolEntrypoint) -> String {
    format!(
        "{}@{}#{}",
        entry.package_name,
        entry.package_version,
        entry.package_source.as_deref().unwrap_or("path")
    )
}

fn sha256(bytes: &[u8]) -> String {
    lower_hex(&Sha256::digest(bytes))
}

fn host_tool_diagnostic(message: impl Into<String>) -> PackageDiagnostic {
    PackageDiagnostic::cargo_metadata_parse(&format!("host tool graph: {}", message.into()))
}
