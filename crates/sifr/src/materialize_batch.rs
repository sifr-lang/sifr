//! Finite materialization with the public command authority and fresh request sources.
use crate::cli_model_and_entrypoint::{
    DiagnosticFormat, EXIT_USAGE_OR_CONFIG, run_with_panic_boundary,
};
use crate::diagnostic_rendering_and_run::{
    materialized_project_command::materialize_with_context, render_diagnostics,
};
use serde::Serialize;
use sifr_driver::CompilerContext;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct Entry {
    index: usize,
    source: PathBuf,
    project: PathBuf,
    exit_code: i32,
}

pub(crate) fn run(manifest: &Path, output: &Path) -> i32 {
    match execute(manifest, output, &crate::compiler_context()) {
        Ok(code) => code,
        Err(error) => {
            let _ = writeln!(io::stderr(), "materialization batch failed: {error}");
            EXIT_USAGE_OR_CONFIG
        }
    }
}

fn execute(manifest: &Path, output: &Path, compiler: &CompilerContext) -> Result<i32, String> {
    crate::finite_batch::execute(manifest, output, |index, file, output| {
        let root = output.join(index.to_string());
        // The command retains its canonical diagnostics. Mark their request owner
        // in the process log; the report records completion even after user errors.
        let _ = writeln!(io::stderr(), "[sifr-materialize-batch] begin index={index}");
        let exit_code = match run_with_panic_boundary(
            "internal compiler panic during Rust project materialization",
            || {
                materialize_with_context(
                    compiler,
                    file,
                    &root,
                    sifr_package::CargoLockMode::Normal,
                    true,
                    DiagnosticFormat::Human,
                )
            },
        ) {
            Ok(code) => code,
            Err(internal) => render_diagnostics(&[*internal], DiagnosticFormat::Human),
        };
        let _ = writeln!(
            io::stderr(),
            "[sifr-materialize-batch] end index={index} exit_code={exit_code}"
        );
        Ok((
            exit_code,
            Entry {
                index,
                source: file.to_owned(),
                project: root.join("sifr_output"),
                exit_code,
            },
        ))
    })
}

#[cfg(test)]
#[path = "materialize_batch_tests.rs"]
mod tests;
