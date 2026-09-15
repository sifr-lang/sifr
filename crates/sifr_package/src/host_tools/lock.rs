//! Canonical host-tool lock persistence and execution-time graph verification.

use super::fingerprint::{hash_path_package, path_hash_exclusions, sha256};
use super::{
    HOST_TOOL_LOCK_FILE, HOST_TOOL_LOCK_VERSION, HostToolEntrypoint, HostToolGraph,
    HostToolLockArtifact, LockedHostToolEntrypoint, host_tool_diagnostic,
};
use crate::PackageDiagnostic;
use sifr_frontend::SourceProvider;
use std::io::Write as _;
use std::path::PathBuf;

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

fn normalized_package_identity(entry: &HostToolEntrypoint) -> String {
    format!(
        "{}@{}#{}",
        entry.package_name,
        entry.package_version,
        entry.package_source.as_deref().unwrap_or("path")
    )
}
