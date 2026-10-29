//! Finite emission ownership: immutable compiler setup, fresh sources per request.
use crate::check_and_package_commands::emit_entrypoint_with_context;
use crate::cli_model_and_entrypoint::{
    DiagnosticFormat, EXIT_USAGE_OR_CONFIG, diagnostic_exit_code, run_with_panic_boundary,
};
use crate::diagnostic_rendering_and_run::render_diagnostic_output;
use serde::Serialize;
use sifr_driver::{CompileResult, CompilerContext};
use sifr_frontend::DiskSourceProvider;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct Entry {
    index: usize,
    source: PathBuf,
    exit_code: i32,
    stderr: String,
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
    crate::finite_batch::execute(manifest, output, |index, file, output| {
        let entry = emit_one(index, file, output, compiler)?;
        Ok((entry.exit_code, entry))
    })
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
    let result = match run_with_panic_boundary(
        "internal compiler panic during emit command execution",
        || emit_entrypoint_with_context(compiler, file, &mut provider),
    ) {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => {
            entry.exit_code = EXIT_USAGE_OR_CONFIG;
            entry.stderr = error;
            return Ok(entry);
        }
        Err(internal) => CompileResult::Errors {
            errors: vec![*internal],
        },
    };
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
