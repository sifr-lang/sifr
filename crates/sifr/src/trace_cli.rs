use crate::cli_model_and_entrypoint::{
    read_source, resolve_compilation_mode, run_with_panic_boundary, CompilationMode,
    DiagnosticFormat, EXIT_INTERNAL_COMPILER_FAILURE, EXIT_SUCCESS,
};
use crate::diagnostic_rendering_and_run::render_diagnostics;
use sifr_frontend::{
    FrontendInput, FrontendMode, ProjectRoot, SourcePath, SourceText, WorkspaceSession,
};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

pub(crate) fn cmd_trace(file: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let trace_result = match run_with_panic_boundary(
        "internal compiler panic during trace command execution",
        || trace_entrypoint(file),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };
    match trace_result {
        Ok(output) => {
            let _ = write!(io::stdout(), "{output}");
            EXIT_SUCCESS
        }
        Err(errors) => {
            if errors.is_empty() {
                EXIT_INTERNAL_COMPILER_FAILURE
            } else {
                render_diagnostics(&errors, diagnostic_format)
            }
        }
    }
}

fn trace_entrypoint(file: &Path) -> Result<String, Vec<sifr_diagnostics::RenderedDiagnostic>> {
    let mut session = match resolve_compilation_mode(file)? {
        CompilationMode::Project => {
            let root = trace_project_root(file);
            WorkspaceSession::open_project(root)?
        }
        CompilationMode::SingleFile => {
            let source = read_source(file);
            WorkspaceSession::open_single_file(FrontendInput {
                path: SourcePath::new(file.to_path_buf()),
                source: SourceText::new(source),
                mode: FrontendMode::SingleFile,
            })?
        }
    };
    Ok(session.snapshot().debug.render_text())
}

fn trace_project_root(file: &Path) -> ProjectRoot {
    let root = sifr_driver::find_workspace_root(file)
        .ok()
        .flatten()
        .map(|root| root.dir)
        .unwrap_or_else(|| {
            file.parent()
                .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
        });
    ProjectRoot {
        root: SourcePath::new(root),
        entrypoint: SourcePath::new(file.to_path_buf()),
    }
}

#[cfg(test)]
mod tests {
    use super::trace_entrypoint;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn trace_entrypoint_renders_status_and_trace_snapshot() {
        let dir = temp_dir("trace_cli_single_file");
        let file = dir.join("script.sifr");
        fs::write(&file, "def main():\n    return 1\n").expect("write source");

        let output = trace_entrypoint(&file).expect("trace should render");

        assert!(output.contains("[status]"));
        assert!(output.contains("target=single_file"));
        assert!(output.contains("[trace]"));
        assert!(output.contains("phase=parse"));

        let _ = fs::remove_dir_all(Path::new(&dir));
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("sifr_{name}_{}_{nonce}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }
}
