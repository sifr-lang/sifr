//! Completion and fresh-output ownership for finite compiler invocations.
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u32,
    files: Vec<PathBuf>,
}

#[derive(Serialize)]
struct Report<T> {
    schema: u32,
    entries: Vec<T>,
}

pub(crate) fn execute<T: Serialize>(
    manifest: &Path,
    output: &Path,
    mut operation: impl FnMut(usize, &Path, &Path) -> Result<(i32, T), String>,
) -> Result<i32, String> {
    let manifest: Manifest = serde_json::from_slice(
        &fs::read(manifest).map_err(|error| format!("cannot read manifest: {error}"))?,
    )
    .map_err(|error| format!("invalid manifest: {error}"))?;
    if manifest.schema != 1 || manifest.files.is_empty() {
        return Err("expected schema 1 and at least one input".into());
    }
    if manifest.files.iter().any(|file| !file.is_absolute()) {
        return Err("batch input paths must be absolute".into());
    }
    // Never inherit a prior request's output or completion marker.
    fs::create_dir(output).map_err(|error| format!("cannot create fresh output: {error}"))?;
    let mut entries = Vec::with_capacity(manifest.files.len());
    let mut exit_code = 0;
    for (index, file) in manifest.files.iter().enumerate() {
        let (code, entry) = operation(index, file, output)?;
        exit_code = exit_code.max(code);
        entries.push(entry);
    }
    let report = serde_json::to_vec(&Report { schema: 1, entries })
        .map_err(|error| format!("cannot serialize report: {error}"))?;
    // Existence is the completion marker; partial writes still fail parsing.
    fs::write(output.join("report.json"), report)
        .map_err(|error| format!("cannot write report: {error}"))?;
    Ok(exit_code)
}
