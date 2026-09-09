mod declarations;
mod fingerprint;
mod isolation;
mod lock;

use declarations::parse_tool_declarations;
use fingerprint::{exact_package_checksum, parse_lock_packages, sha256};
pub use isolation::validate_host_tool_application_isolation;
use isolation::{direct_normal_dependencies, target_contamination_diagnostics};
pub use lock::{load_host_tool_lock, verify_host_tool_graph, write_host_tool_lock};

use crate::{CargoPackage, CargoPackageId, PackageDiagnostic, PackageGraphSnapshot};
use serde::{Deserialize, Serialize};
use sifr_frontend::SourceProvider;
use std::collections::{BTreeMap, BTreeSet};
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

fn required_nonempty_string(table: &toml::Table, key: &str) -> Option<String> {
    table
        .get(key)
        .and_then(toml::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn package_root(package: &CargoPackage) -> PathBuf {
    package
        .manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn host_tool_diagnostic(message: impl Into<String>) -> PackageDiagnostic {
    PackageDiagnostic::cargo_metadata_parse(&format!("host tool graph: {}", message.into()))
}
