//! Authenticate exact lock records and bounded path-package content.

use super::{
    HOST_TOOL_LOCK_FILE, HostToolGraph, host_tool_diagnostic, package_root,
    required_nonempty_string,
};
use crate::digest::lower_hex;
use crate::{CargoPackage, NormalizedCargoMetadata, PackageDiagnostic};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LockPackage {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
}

pub(super) fn parse_lock_packages(
    source: &str,
) -> Result<Vec<LockPackage>, Vec<PackageDiagnostic>> {
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

pub(super) fn exact_package_checksum(
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

pub(super) fn path_hash_exclusions(graph: &HostToolGraph) -> [PathBuf; 4] {
    [
        graph.target_directory.clone(),
        graph.workspace_root.join(".git"),
        graph.workspace_root.join("Cargo.lock"),
        graph.workspace_root.join(HOST_TOOL_LOCK_FILE),
    ]
}

pub(super) fn hash_path_package(
    root: &Path,
    exclusions: &[PathBuf],
) -> Result<String, PackageDiagnostic> {
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

pub(super) fn sha256(bytes: &[u8]) -> String {
    lower_hex(&Sha256::digest(bytes))
}
