//! Finite emission ownership: immutable compiler setup, fresh sources per request.
use crate::check_and_package_commands::emit_entrypoint_with_context;
use crate::cli_model_and_entrypoint::{
    DiagnosticFormat, EXIT_USAGE_OR_CONFIG, diagnostic_exit_code, run_with_panic_boundary,
};
use crate::diagnostic_rendering_and_run::render_diagnostic_output;
use serde::{Deserialize, Serialize};
use sifr_driver::{CompileResult, CompilerContext};
use sifr_frontend::{DiskSourceProvider, SourceProvider};
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u32,
    files: Vec<PathBuf>,
}

#[derive(Serialize)]
struct Entry {
    index: usize,
    source: PathBuf,
    exit_code: i32,
    stderr: String,
}

#[derive(Serialize)]
struct Report {
    schema: u32,
    entries: Vec<Entry>,
}

pub(crate) fn run(manifest: &Path, output: &Path) -> i32 {
    match execute(manifest, output, &crate::compiler_context()) {
        Ok(code) => code,
        Err(error) => {
            let _ = writeln!(io::stderr(), "emission batch failed: {error}");
            EXIT_USAGE_OR_CONFIG
        }
    }
}

fn execute(manifest: &Path, output: &Path, compiler: &CompilerContext) -> Result<i32, String> {
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
    // A fresh output directory prevents a failed request from inheriting old Rust.
    fs::create_dir(output).map_err(|error| format!("cannot create fresh output: {error}"))?;
    let mut entries = Vec::with_capacity(manifest.files.len());
    let mut exit_code = 0;
    for (index, file) in manifest.files.iter().enumerate() {
        let entry = emit_one(index, file, output, compiler)?;
        exit_code = exit_code.max(entry.exit_code);
        entries.push(entry);
    }
    let report = serde_json::to_vec(&Report { schema: 1, entries })
        .map_err(|error| format!("cannot serialize report: {error}"))?;
    // Report existence is the completion marker; a partial write still fails parsing.
    fs::write(output.join("report.json"), report)
        .map_err(|error| format!("cannot write report: {error}"))?;
    Ok(exit_code)
}

fn emit_one(
    index: usize,
    file: &Path,
    output: &Path,
    compiler: &CompilerContext,
) -> Result<Entry, String> {
    let mut entry = Entry {
        index,
        source: file.to_owned(),
        exit_code: 0,
        stderr: String::new(),
    };
    let mut provider = DiskSourceProvider::new();
    // Scalar emission exits on an unreadable source. Diagnose it here so unrelated
    // requests still execute. This same fresh provider owns the source snapshot.
    if let Err(error) = provider.read_file(file) {
        entry.exit_code = EXIT_USAGE_OR_CONFIG;
        entry.stderr = format!("error: could not read file '{}': {error}\n", file.display());
        return Ok(entry);
    }
    let result = run_with_panic_boundary(
        "internal compiler panic during emit command execution",
        || emit_entrypoint_with_context(compiler, file, &mut provider),
    )
    .unwrap_or_else(|internal| CompileResult::Errors {
        errors: vec![*internal],
    });
    match result {
        CompileResult::Success { rust_source } => {
            fs::write(output.join(format!("{index}.rs")), rust_source)
                .map_err(|error| format!("cannot write emission {index}: {error}"))?;
        }
        CompileResult::Errors { errors } => {
            entry.exit_code = diagnostic_exit_code(&errors);
            entry.stderr = render_diagnostic_output(&errors, DiagnosticFormat::Json)
                .map_err(|error| format!("cannot serialize diagnostics {index}: {error}"))?;
        }
    }
    Ok(entry)
}

#[cfg(test)]
#[path = "emit_batch_tests.rs"]
mod tests;
