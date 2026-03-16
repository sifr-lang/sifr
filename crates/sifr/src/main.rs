//! Sifr Compiler CLI
//!
//! Usage:
//!   sifr build <file.sifr>    Compile to native binary
//!   sifr run <file.sifr>      Compile and run
//!   sifr check <file.sifr>    Type-check only
//!   sifr emit <file.sifr>     Show generated Rust code
use clap::{Parser, Subcommand, ValueEnum};
use sifr_driver::{
    apply_diagnostic_recovery_limits, build, build_cached_project, build_cached_single_file,
    build_project, check, check_project, compile, compile_errors_to_diagnostics, run_tests,
    CachedBinaryArtifact, CompileError, CompilePhase, CompileResult, CompilerDiagnostic, Severity,
};
use sifr_python_ast::Stmt;
use sifr_python_parser::parse_module;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::{self, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(
    name = "sifr",
    version,
    about = "The Sifr programming language compiler"
)]
struct Cli {
    /// Diagnostic output format
    #[arg(long, value_enum, default_value_t = DiagnosticFormat::Human)]
    diagnostic_format: DiagnosticFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a .sifr file to a native binary
    Build {
        /// Input .sifr file
        file: PathBuf,
        /// Output directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
    /// Compile and run a .sifr file
    Run {
        /// Input .sifr file
        file: PathBuf,
    },
    /// Type-check a .sifr file without compiling
    Check {
        /// Input .sifr file
        file: PathBuf,
    },
    /// Show the generated Rust source code
    Emit {
        /// Input .sifr file
        file: PathBuf,
    },
    /// Run tests in a directory
    Test {
        /// Directory containing test files (default: current directory)
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum DiagnosticFormat {
    Human,
    Json,
    Compact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompilationMode {
    SingleFile,
    Project,
}

const EXIT_SUCCESS: i32 = 0;
const EXIT_USER_DIAGNOSTIC: i32 = 1;
const EXIT_USAGE_OR_CONFIG: i32 = 2;
const EXIT_INTERNAL_COMPILER_FAILURE: i32 = 3;
const MAX_COMPACT_REPRESENTATIVE_LOCATIONS: usize = 5;

fn main() {
    let cli = Cli::parse();
    process::exit(run_cli(cli));
}

fn run_cli(cli: Cli) -> i32 {
    let diagnostic_format = cli.diagnostic_format;
    match cli.command {
        Commands::Build { file, output } => cmd_build(&file, &output, diagnostic_format),
        Commands::Run { file } => cmd_run(&file, diagnostic_format),
        Commands::Check { file } => cmd_check(&file, diagnostic_format),
        Commands::Emit { file } => cmd_emit(&file, diagnostic_format),
        Commands::Test { dir } => cmd_test(&dir, diagnostic_format),
    }
}

fn resolve_compilation_mode(file: &Path) -> CompilationMode {
    let is_project_entry =
        file.file_stem().is_some_and(|stem| stem == "main") && has_local_project_imports(file);

    if is_project_entry {
        CompilationMode::Project
    } else {
        CompilationMode::SingleFile
    }
}

fn has_local_project_imports(file: &Path) -> bool {
    let Some(parent) = file.parent() else {
        return false;
    };
    let Ok(source) = std::fs::read_to_string(file) else {
        return false;
    };
    let parsed = match parse_module(&source) {
        Ok(parsed) if parsed.is_valid() => parsed,
        _ => return false,
    };

    parsed.suite().iter().any(|stmt| {
        let Stmt::ImportFrom(import_from) = stmt else {
            return false;
        };
        if import_from.level > 1 {
            return false;
        }
        let Some(module) = &import_from.module else {
            return false;
        };
        let module_name = module.to_string();
        if module_name == "typing"
            || module_name == "enum"
            || module_name.starts_with("sifr.")
            || module_name.starts_with("_sifr.")
        {
            return false;
        }
        parent.join(format!("{module_name}.sifr")).is_file()
    })
}

fn read_source(file: &Path) -> String {
    match std::fs::read_to_string(file) {
        Ok(source) => source,
        Err(e) => {
            let _ = writeln!(
                io::stderr(),
                "error: could not read file '{}': {e}",
                file.display()
            );
            process::exit(EXIT_USAGE_OR_CONFIG);
        }
    }
}

#[cfg(test)]
struct InvocationWorkspace {
    path: PathBuf,
}

#[cfg(test)]
impl InvocationWorkspace {
    fn create(prefix: &str) -> io::Result<Self> {
        let base_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir();
        for attempt in 0..8u8 {
            let unique = if attempt == 0 {
                format!("{}_{}_{}", prefix, process::id(), base_nanos)
            } else {
                format!("{}_{}_{}_{}", prefix, process::id(), base_nanos, attempt)
            };
            let path = root.join(unique);
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("failed to allocate unique workspace for prefix '{prefix}'"),
        ))
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
impl Drop for InvocationWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Note => 2,
        Severity::Help => 3,
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
        Severity::Help => "help",
    }
}

fn panic_payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(msg) = payload.downcast_ref::<&str>() {
        return (*msg).to_string();
    }
    if let Some(msg) = payload.downcast_ref::<String>() {
        return msg.clone();
    }
    "non-string panic payload".to_string()
}

fn run_with_panic_boundary<T>(
    context: impl Into<String>,
    phase: CompilePhase,
    f: impl FnOnce() -> T,
) -> Result<T, CompileError> {
    let context = context.into();
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => Ok(value),
        Err(payload) => Err(CompileError {
            message: format!("{context}: {}", panic_payload_message(payload.as_ref())),
            phase,
        }),
    }
}

fn is_internal_compile_error(error: &CompileError) -> bool {
    error.message.starts_with("internal compiler panic during ")
}

fn compile_error_exit_code(errors: &[CompileError]) -> i32 {
    if errors.iter().any(is_internal_compile_error) {
        EXIT_INTERNAL_COMPILER_FAILURE
    } else {
        EXIT_USER_DIAGNOSTIC
    }
}

fn compact_severity_summary(diagnostics: &[CompilerDiagnostic]) -> String {
    let mut error_count = 0usize;
    let mut warning_count = 0usize;
    let mut note_count = 0usize;
    let mut help_count = 0usize;
    for diagnostic in diagnostics {
        match diagnostic.severity {
            Severity::Error => error_count += 1,
            Severity::Warning => warning_count += 1,
            Severity::Note => note_count += 1,
            Severity::Help => help_count += 1,
        }
    }
    format!(
        "summary: {error_count} error(s), {warning_count} warning(s), {note_count} note(s), {help_count} help item(s)"
    )
}

fn compact_location_label(span: &sifr_driver::DiagnosticSpan) -> String {
    match (&span.file, span.line, span.column) {
        (Some(file), Some(line), Some(column)) => format!("{file}:{line}:{column}"),
        (Some(file), Some(line), None) => format!("{file}:{line}"),
        (Some(file), None, _) => file.clone(),
        (None, Some(line), Some(column)) => format!("<unknown>:{line}:{column}"),
        (None, Some(line), None) => format!("<unknown>:{line}"),
        (None, None, Some(column)) => format!("<unknown>:0:{column}"),
        (None, None, None) => "<unknown>".to_string(),
    }
}

fn render_compact_diagnostics(diagnostics: &[CompilerDiagnostic]) -> String {
    let mut grouped: BTreeMap<(u8, String, bool, String), Vec<&CompilerDiagnostic>> =
        BTreeMap::new();
    for diagnostic in diagnostics {
        let is_summary_group = diagnostic.message.starts_with("... +")
            && diagnostic.message.ends_with("more similar diagnostics");
        let key = (
            severity_rank(diagnostic.severity),
            diagnostic.code.clone(),
            is_summary_group,
            diagnostic.message.clone(),
        );
        grouped.entry(key).or_default().push(diagnostic);
    }

    let mut output = String::new();
    output.push_str(&compact_severity_summary(diagnostics));
    output.push('\n');

    for ((_severity_rank, code, _is_summary_group, message), group) in grouped {
        let severity = group[0].severity;
        let _ = writeln!(
            output,
            "{} [{code}] {message} (x{})",
            severity_label(severity),
            group.len()
        );

        let mut locations: BTreeSet<String> = BTreeSet::new();
        for diagnostic in &group {
            if let Some(span) = &diagnostic.primary_span {
                locations.insert(compact_location_label(span));
            }
        }

        let rendered_locations = locations
            .iter()
            .take(MAX_COMPACT_REPRESENTATIVE_LOCATIONS)
            .collect::<Vec<_>>();
        for location in rendered_locations {
            let _ = writeln!(output, "  at {location}");
        }
        if locations.len() > MAX_COMPACT_REPRESENTATIVE_LOCATIONS {
            let _ = writeln!(
                output,
                "  ... +{} more",
                locations.len() - MAX_COMPACT_REPRESENTATIVE_LOCATIONS
            );
        }

        if let Some(help) = group
            .iter()
            .find_map(|diagnostic| diagnostic.help.as_deref())
        {
            let _ = writeln!(output, "  help: {help}");
        }
        if let Some(url) = group
            .iter()
            .find_map(|diagnostic| (!diagnostic.url.is_empty()).then_some(diagnostic.url.as_str()))
        {
            let _ = writeln!(output, "  url: {url}");
        }
    }

    output
}

fn render_compile_errors(errors: &[CompileError], format: DiagnosticFormat) -> i32 {
    let diagnostics = apply_diagnostic_recovery_limits(&compile_errors_to_diagnostics(errors));
    match format {
        DiagnosticFormat::Human => {
            for diagnostic in diagnostics {
                let label = if diagnostic.code.starts_with("SIFR-PARSE-") {
                    "parse error"
                } else if diagnostic.code.starts_with("SIFR-TYPE-") {
                    "type error"
                } else if diagnostic.code.starts_with("SIFR-CODEGEN-") {
                    "codegen error"
                } else if diagnostic.code.starts_with("SIFR-BUILD-") {
                    "build error"
                } else {
                    match diagnostic.severity {
                        sifr_driver::Severity::Error => "error",
                        sifr_driver::Severity::Warning => "warning",
                        sifr_driver::Severity::Note => "note",
                        sifr_driver::Severity::Help => "help",
                    }
                };
                let _ = writeln!(
                    io::stderr(),
                    "{label}: {message}",
                    message = diagnostic.message
                );
            }
        }
        DiagnosticFormat::Json => match serde_json::to_string_pretty(&diagnostics) {
            Ok(json) => {
                let _ = writeln!(io::stderr(), "{json}");
            }
            Err(e) => {
                let _ = writeln!(
                    io::stderr(),
                    "build error: failed to serialize diagnostics as json: {e}"
                );
                return EXIT_INTERNAL_COMPILER_FAILURE;
            }
        },
        DiagnosticFormat::Compact => {
            let compact_output = render_compact_diagnostics(&diagnostics);
            let _ = write!(io::stderr(), "{compact_output}");
        }
    }
    compile_error_exit_code(errors)
}

fn cmd_build(file: &Path, output: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let result = match run_with_panic_boundary(
        "internal compiler panic during build command execution",
        CompilePhase::Build,
        || compile_entrypoint(file, output),
    ) {
        Ok(result) => result,
        Err(internal) => return render_compile_errors(&[internal], diagnostic_format),
    };

    match result {
        Ok(binary_path) => {
            let _ = writeln!(
                io::stderr(),
                "compiled successfully: {}",
                binary_path.display()
            );
            EXIT_SUCCESS
        }
        Err(errors) => render_compile_errors(&errors, diagnostic_format),
    }
}

fn cmd_run(file: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let result = match run_with_panic_boundary(
        "internal compiler panic during run command compilation",
        CompilePhase::Build,
        || build_run_artifact(file),
    ) {
        Ok(result) => result,
        Err(internal) => return render_compile_errors(&[internal], diagnostic_format),
    };

    match result {
        Ok(artifact) => {
            let _ = writeln!(io::stderr(), "{}", artifact.cache_status_line());
            let output = std::process::Command::new(artifact.binary_path())
                .output()
                .unwrap_or_else(|e| {
                    let _ = writeln!(io::stderr(), "error: could not run binary: {e}");
                    process::exit(EXIT_USAGE_OR_CONFIG);
                });

            // Forward stdout and stderr
            std::io::stdout().write_all(&output.stdout).ok();
            std::io::stderr().write_all(&output.stderr).ok();

            if !output.status.success() {
                return EXIT_USER_DIAGNOSTIC;
            }
            EXIT_SUCCESS
        }
        Err(errors) => render_compile_errors(&errors, diagnostic_format),
    }
}

fn cmd_check(file: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let errors = match run_with_panic_boundary(
        "internal compiler panic during check command execution",
        CompilePhase::TypeCheck,
        || check_entrypoint(file),
    ) {
        Ok(errors) => errors,
        Err(internal) => return render_compile_errors(&[internal], diagnostic_format),
    };

    if errors.is_empty() {
        match diagnostic_format {
            DiagnosticFormat::Human => {
                let _ = writeln!(io::stderr(), "no errors found");
            }
            DiagnosticFormat::Json => {
                let _ = writeln!(io::stdout(), "[]");
            }
            DiagnosticFormat::Compact => {
                let _ = writeln!(
                    io::stderr(),
                    "summary: 0 error(s), 0 warning(s), 0 note(s), 0 help item(s)"
                );
            }
        }
        EXIT_SUCCESS
    } else {
        render_compile_errors(&errors, diagnostic_format)
    }
}

fn cmd_test(dir: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let run_result = match run_with_panic_boundary(
        "internal compiler panic during test command execution",
        CompilePhase::Build,
        || run_tests(dir),
    ) {
        Ok(result) => result,
        Err(internal) => return render_compile_errors(&[internal], diagnostic_format),
    };
    match run_result {
        Ok(success) => {
            if success {
                EXIT_SUCCESS
            } else {
                EXIT_USER_DIAGNOSTIC
            }
        }
        Err(errors) => render_compile_errors(&errors, diagnostic_format),
    }
}

fn cmd_emit(file: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let compile_result = match run_with_panic_boundary(
        "internal compiler panic during emit command execution",
        CompilePhase::Codegen,
        || emit_entrypoint(file),
    ) {
        Ok(result) => result,
        Err(internal) => return render_compile_errors(&[internal], diagnostic_format),
    };
    match compile_result {
        CompileResult::Success { rust_source } => {
            let _ = write!(io::stdout(), "{rust_source}");
            EXIT_SUCCESS
        }
        CompileResult::Errors { errors } => render_compile_errors(&errors, diagnostic_format),
    }
}

fn compile_entrypoint(file: &Path, output: &Path) -> Result<PathBuf, Vec<CompileError>> {
    match resolve_compilation_mode(file) {
        CompilationMode::Project => build_project(file, output),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            build(&source, output)
        }
    }
}

fn build_run_artifact(file: &Path) -> Result<CachedBinaryArtifact, Vec<CompileError>> {
    match resolve_compilation_mode(file) {
        CompilationMode::Project => build_cached_project(file),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            build_cached_single_file(&source, file)
        }
    }
}

fn check_entrypoint(file: &Path) -> Vec<CompileError> {
    match resolve_compilation_mode(file) {
        CompilationMode::Project => check_project(file),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            check(&source)
        }
    }
}

fn emit_entrypoint(file: &Path) -> CompileResult {
    let source = read_source(file);
    compile(&source)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mktemp_dir(name: &str) -> PathBuf {
        let unique = format!(
            "sifr_cli_mode_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    struct TestProject {
        dir: PathBuf,
    }

    impl TestProject {
        fn new(name: &str) -> Self {
            Self {
                dir: mktemp_dir(name),
            }
        }

        /// Writes a test fixture and creates any missing parent directories first.
        fn write(&self, relative_path: &str, contents: &str, failure_message: &str) -> PathBuf {
            let path = self.dir.join(relative_path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("test fixture parent should exist");
            }
            std::fs::write(&path, contents).expect(failure_message);
            path
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn test_invocation_workspace_create_returns_unique_paths() {
        let first = InvocationWorkspace::create("sifr_run_workspace")
            .expect("first workspace should exist");
        let second = InvocationWorkspace::create("sifr_run_workspace")
            .expect("second workspace should exist");
        assert_ne!(first.path(), second.path());
        assert!(first.path().exists());
        assert!(second.path().exists());
    }

    #[test]
    fn test_resolve_compilation_mode_project_for_main_with_siblings() {
        let project = TestProject::new("project");
        let main = project.write(
            "main.sifr",
            "from helper import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def helper() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::Project);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_non_main_entry() {
        let project = TestProject::new("single");
        let app = project.write(
            "app.sifr",
            "def main():\n    pass\n",
            "app file should be written",
        );
        project.write(
            "helper.sifr",
            "def helper() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolve_compilation_mode(&app), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_main_without_local_imports() {
        let project = TestProject::new("main_no_imports");
        let main = project.write(
            "main.sifr",
            "def main():\n    print(\"ok\")\n",
            "main file should be written",
        );
        project.write(
            "scratch.sifr",
            "def nope(:\n",
            "scratch file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_stdlib_only_imports() {
        let project = TestProject::new("main_stdlib_only");
        let main = project.write(
            "main.sifr",
            "from sifr.math import floor\n\ndef main():\n    print(floor(3.9))\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def helper() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_missing_local_module() {
        let project = TestProject::new("missing_local");
        let main = project.write(
            "main.sifr",
            "from helper import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_regular_import_with_local_module() {
        let project = TestProject::new("regular_import_local_module");
        let main = project.write(
            "main.sifr",
            "import helper\n\ndef main():\n    print(\"ok\")\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def value() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_invalid_main_source() {
        let project = TestProject::new("invalid_main");
        let main = project.write("main.sifr", "def main(:\n", "main file should be written");
        project.write(
            "helper.sifr",
            "def helper() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_typing_import() {
        let project = TestProject::new("typing_import");
        let main = project.write(
            "main.sifr",
            "from typing import List\n\ndef main():\n    values: List[int] = [1]\n    print(values)\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def helper() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_typing_import_with_local_typing_file() {
        let project = TestProject::new("typing_import_local_file");
        let main = project.write(
            "main.sifr",
            "from typing import List\n\ndef main():\n    values: List[int] = [1]\n    print(values)\n",
            "main file should be written",
        );
        project.write(
            "typing.sifr",
            "def local() -> int:\n    return 1\n",
            "typing file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_enum_import() {
        let project = TestProject::new("enum_import");
        let main = project.write(
            "main.sifr",
            "from enum import Enum\n\ndef main():\n    print(\"ok\")\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def helper() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_enum_import_with_local_enum_file() {
        let project = TestProject::new("enum_import_local_file");
        let main = project.write(
            "main.sifr",
            "from enum import Enum\n\ndef main():\n    print(\"ok\")\n",
            "main file should be written",
        );
        project.write(
            "enum.sifr",
            "def local() -> int:\n    return 1\n",
            "enum file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_package_init_import() {
        let project = TestProject::new("pkg_import");
        let main = project.write(
            "main.sifr",
            "from pkg import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );
        project.write(
            "pkg/__init__.sifr",
            "def value() -> int:\n    return 1\n",
            "pkg init should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_project_for_relative_import_with_sibling() {
        let project = TestProject::new("relative_import");
        let main = project.write(
            "main.sifr",
            "from .helper import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def value() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::Project);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_relative_import_without_sibling() {
        let project = TestProject::new("relative_import_missing_sibling");
        let main = project.write(
            "main.sifr",
            "from .helper import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_multi_level_relative_import() {
        let project = TestProject::new("relative_import_multi_level");
        let main = project.write(
            "main.sifr",
            "from ..helper import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def value() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_bare_relative_import() {
        let project = TestProject::new("relative_import_bare");
        let main = project.write(
            "main.sifr",
            "from . import value\n\ndef main():\n    print(value)\n",
            "main file should be written",
        );

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_compile_entrypoint_error_consistency_for_project_mode() {
        let dir = mktemp_dir("entrypoint_consistency");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def value(:\n").expect("helper file should be written");

        let run_out = mktemp_dir("run_path");
        let build_out = mktemp_dir("build_path");
        let run_err = compile_entrypoint(&main, &run_out).expect_err("run compile should fail");
        let build_err =
            compile_entrypoint(&main, &build_out).expect_err("build compile should fail");
        let run_messages: Vec<String> = run_err.iter().map(ToString::to_string).collect();
        let build_messages: Vec<String> = build_err.iter().map(ToString::to_string).collect();
        assert_eq!(run_messages, build_messages);

        let _ = std::fs::remove_dir_all(run_out);
        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_compile_entrypoint_error_consistency_for_import_statement() {
        let dir = mktemp_dir("entrypoint_import_statement");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(&main, "import helper\n\ndef main():\n    print(\"ok\")\n")
            .expect("main file should be written");
        std::fs::write(&helper, "def value() -> int:\n    return 1\n")
            .expect("helper file should be written");

        let run_out = mktemp_dir("run_path_import_statement");
        let build_out = mktemp_dir("build_path_import_statement");
        let run_err = compile_entrypoint(&main, &run_out).expect_err("run compile should fail");
        let build_err =
            compile_entrypoint(&main, &build_out).expect_err("build compile should fail");
        let run_messages: Vec<String> = run_err.iter().map(ToString::to_string).collect();
        let build_messages: Vec<String> = build_err.iter().map(ToString::to_string).collect();
        assert_eq!(run_messages, build_messages);
        assert!(run_messages
            .iter()
            .any(|m| m.contains("unsupported import statement 'import helper'")));

        let _ = std::fs::remove_dir_all(run_out);
        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_compile_entrypoint_error_consistency_for_bare_relative_import() {
        let dir = mktemp_dir("entrypoint_bare_relative");
        let main = dir.join("main.sifr");
        std::fs::write(
            &main,
            "from . import helper\n\ndef main():\n    print(helper)\n",
        )
        .expect("main file should be written");

        let run_out = mktemp_dir("run_path_bare_relative");
        let build_out = mktemp_dir("build_path_bare_relative");
        let run_err = compile_entrypoint(&main, &run_out).expect_err("run compile should fail");
        let build_err =
            compile_entrypoint(&main, &build_out).expect_err("build compile should fail");
        let run_messages: Vec<String> = run_err.iter().map(ToString::to_string).collect();
        let build_messages: Vec<String> = build_err.iter().map(ToString::to_string).collect();
        assert_eq!(run_messages, build_messages);
        assert!(run_messages
            .iter()
            .any(|m| m.contains("unsupported bare relative import")));

        let _ = std::fs::remove_dir_all(run_out);
        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_compile_entrypoint_error_consistency_for_multi_level_relative_import() {
        let dir = mktemp_dir("entrypoint_multi_level_relative");
        let main = dir.join("main.sifr");
        std::fs::write(
            &main,
            "from ..helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");

        let run_out = mktemp_dir("run_path_multi_level_relative");
        let build_out = mktemp_dir("build_path_multi_level_relative");
        let run_err = compile_entrypoint(&main, &run_out).expect_err("run compile should fail");
        let build_err =
            compile_entrypoint(&main, &build_out).expect_err("build compile should fail");
        let run_messages: Vec<String> = run_err.iter().map(ToString::to_string).collect();
        let build_messages: Vec<String> = build_err.iter().map(ToString::to_string).collect();
        assert_eq!(run_messages, build_messages);
        assert!(run_messages
            .iter()
            .any(|m| m.contains("unsupported relative import level 2")));

        let _ = std::fs::remove_dir_all(run_out);
        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_check_entrypoint_project_mode_resolves_local_imports() {
        let dir = mktemp_dir("check_entrypoint_project_imports");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def value() -> int:\n    return 42\n")
            .expect("helper file should be written");

        let errors = check_entrypoint(&main);
        assert!(
            errors.is_empty(),
            "project-aware check should succeed for valid local imports: {errors:?}"
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_check_entrypoint_project_mode_error_parity_with_compile_entrypoint() {
        let dir = mktemp_dir("check_entrypoint_error_parity");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def value() -> int:\n    return \"bad\"\n")
            .expect("helper file should be written");

        let check_errors = check_entrypoint(&main);
        let build_out = mktemp_dir("check_entrypoint_build_out");
        let build_errors = compile_entrypoint(&main, &build_out)
            .err()
            .expect("build path should fail for helper type mismatch");

        let check_messages: Vec<String> = check_errors.into_iter().map(|e| e.to_string()).collect();
        let build_messages: Vec<String> = build_errors.into_iter().map(|e| e.to_string()).collect();
        assert_eq!(check_messages, build_messages);
        assert!(check_messages
            .iter()
            .any(|m| m.contains("[helper] return type mismatch")));

        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_compile_entrypoint_single_file_ignores_unrelated_sibling_parse_errors() {
        let dir = mktemp_dir("single_file_sibling_isolation");
        let main = dir.join("main.sifr");
        let output = mktemp_dir("single_file_sibling_isolation_out");
        std::fs::write(&main, "def main():\n    print(\"solo\")\n")
            .expect("main file should be written");
        std::fs::write(dir.join("scratch.sifr"), "def broken(:\n")
            .expect("unrelated sibling should be written");

        let binary = compile_entrypoint(&main, &output)
            .expect("single-file build should ignore unrelated sibling parse errors");
        assert!(binary.exists());

        let _ = std::fs::remove_dir_all(output);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_compile_entrypoint_non_main_input_stays_single_file() {
        let dir = mktemp_dir("non_main_single_file_boundary");
        let app = dir.join("app.sifr");
        let output = mktemp_dir("non_main_single_file_boundary_out");
        std::fs::write(&app, "def main():\n    print(\"app\")\n").expect("app should be written");
        std::fs::write(
            dir.join("main.sifr"),
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("project-like main should be written");
        std::fs::write(dir.join("helper.sifr"), "def value(:\n").expect("helper should be written");

        let binary =
            compile_entrypoint(&app, &output).expect("non-main entry should stay single-file");
        assert!(binary.exists());

        let _ = std::fs::remove_dir_all(output);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_emit_entrypoint_preserves_single_file_boundary_for_project_like_main() {
        let dir = mktemp_dir("emit_single_file_boundary");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main should be written");
        std::fs::write(&helper, "def value() -> int:\n    return 42\n")
            .expect("helper should be written");

        let check_errors = check_entrypoint(&main);
        assert!(
            check_errors.is_empty(),
            "check should preserve project-mode behavior: {check_errors:?}"
        );

        let emit_result = emit_entrypoint(&main);
        let emit_errors = match emit_result {
            CompileResult::Success { .. } => {
                panic!("emit should stay single-file and fail on local project imports")
            }
            CompileResult::Errors { errors } => errors,
        };
        let emit_messages: Vec<String> = emit_errors.iter().map(ToString::to_string).collect();
        assert!(emit_messages
            .iter()
            .any(|message| message.contains("unknown module 'helper'")));

        let _ = std::fs::remove_dir_all(dir);
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("crate should live under repo_root/crates/sifr")
            .to_path_buf()
    }

    fn emit_demo_rust(relative_path: &str) -> String {
        let demo_path = repo_root().join(relative_path);
        match emit_entrypoint(&demo_path) {
            CompileResult::Success { rust_source } => rust_source,
            CompileResult::Errors { errors } => {
                panic!("emit should succeed for {relative_path}: {errors:?}")
            }
        }
    }

    #[test]
    fn test_emit_entrypoint_downshifts_phase24_analysis_demos() {
        let m24_2 =
            emit_demo_rust("demos/m24_2_semantic_query_layer_standardization_demo/main.sifr");
        assert!(m24_2.contains("fn recurse(n: i64) -> i64"));
        assert!(m24_2.contains("if !_broke"));

        let m24_3 =
            emit_demo_rust("demos/m24_3_control_flow_effect_query_unification_demo/main.sifr");
        assert!(m24_3.contains("return Err(ValueError::new(\"non-positive\".to_string()));"));
        assert!(m24_3.contains("return 99 as i64;"));

        let m24_4 =
            emit_demo_rust("demos/m24_4_analysis_emission_boundary_hardening_demo/main.sifr");
        assert!(m24_4.contains("fn summarize(values: &Vec<i64>) -> i64"));
        assert!(m24_4.contains("if value > (10 as i64)"));
    }

    #[test]
    fn test_emit_entrypoint_downshifts_phase25_analysis_demos() {
        let m25_3 = emit_demo_rust("demos/m25_3_canonical_flow_truth_queries_demo/main.sifr");
        assert!(m25_3.contains("return Err(ValueError::new(\"bad value\".to_string()));"));
        assert!(m25_3.contains("return 77 as i64;"));
        assert!(!m25_3.contains("11 as i64"));

        let m25_4 =
            emit_demo_rust("demos/m25_4_diagnostics_and_consumer_integration_demo/main.sifr");
        assert!(m25_4.contains("fn inferred(flag: bool) -> i64"));
        assert!(m25_4.contains("return 2 as i64;"));
        assert!(!m25_4.contains("never"));
    }

    #[test]
    fn test_frontend_error_messages_match_across_check_build_and_run_paths() {
        let dir = mktemp_dir("frontend_error_mode_parity");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def value() -> int:\n    return \"bad\"\n")
            .expect("helper file should be written");

        let check_errors = check_entrypoint(&main);
        let run_out = mktemp_dir("frontend_parity_run_out");
        let build_out = mktemp_dir("frontend_parity_build_out");
        let run_errors = compile_entrypoint(&main, &run_out)
            .err()
            .expect("run path should fail on helper type error");
        let build_errors = compile_entrypoint(&main, &build_out)
            .err()
            .expect("build path should fail on helper type error");

        let check_messages: Vec<String> = check_errors.into_iter().map(|e| e.to_string()).collect();
        let run_messages: Vec<String> = run_errors.into_iter().map(|e| e.to_string()).collect();
        let build_messages: Vec<String> = build_errors.into_iter().map(|e| e.to_string()).collect();
        assert_eq!(check_messages, run_messages);
        assert_eq!(run_messages, build_messages);

        let _ = std::fs::remove_dir_all(run_out);
        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_compile_error_exit_code_contract_user_vs_internal() {
        let user_error = CompileError {
            message: "type mismatch".to_string(),
            phase: CompilePhase::TypeCheck,
        };
        assert_eq!(compile_error_exit_code(&[user_error]), EXIT_USER_DIAGNOSTIC);

        let internal_error = CompileError {
            message: "internal compiler panic during single-file code generation: boom".to_string(),
            phase: CompilePhase::Codegen,
        };
        assert_eq!(
            compile_error_exit_code(&[internal_error]),
            EXIT_INTERNAL_COMPILER_FAILURE
        );
    }

    #[test]
    fn test_diagnostic_format_cli_rejects_unknown_value_with_usage_exit_code() {
        let parse_result = Cli::try_parse_from([
            "sifr",
            "--diagnostic-format",
            "not-a-format",
            "check",
            "main.sifr",
        ]);
        match parse_result {
            Ok(_) => panic!("unknown diagnostic format should fail"),
            Err(error) => assert_eq!(error.exit_code(), EXIT_USAGE_OR_CONFIG),
        }
    }

    #[test]
    fn test_diagnostic_format_cli_accepts_compact_value() {
        let parse_result = Cli::try_parse_from([
            "sifr",
            "--diagnostic-format",
            "compact",
            "check",
            "main.sifr",
        ]);
        assert!(parse_result.is_ok(), "compact format should parse");
    }

    #[test]
    fn test_run_with_panic_boundary_converts_panic_to_internal_compile_error() {
        let error = run_with_panic_boundary(
            "internal compiler panic during test boundary",
            CompilePhase::Build,
            || -> usize { panic!("boom") },
        )
        .expect_err("panic should convert to compile error");
        assert!(error
            .message
            .contains("internal compiler panic during test boundary: boom"));
        assert_eq!(
            compile_error_exit_code(&[error]),
            EXIT_INTERNAL_COMPILER_FAILURE
        );
    }

    #[test]
    fn test_compact_renderer_invariants_summary_grouping_and_bounds() {
        let mut diagnostics = Vec::new();
        for idx in 0..8 {
            diagnostics.push(CompilerDiagnostic {
                code: "SIFR-TYPE-0001".to_string(),
                severity: Severity::Error,
                message: "type mismatch: expected 'int', got 'str'".to_string(),
                url: "https://sifr.dev/docs/errors/SIFR-TYPE-0001".to_string(),
                primary_span: Some(sifr_driver::DiagnosticSpan {
                    file: Some("main.sifr".to_string()),
                    line: Some(idx + 1),
                    column: Some(1),
                }),
                related_spans: Vec::new(),
                children: Vec::new(),
                help: Some("fix assignment type".to_string()),
                suggestions: Vec::new(),
            });
        }
        let compact = render_compact_diagnostics(&diagnostics);
        let mut lines = compact.lines();
        let first_line = lines.next().expect("compact output should have first line");
        assert!(
            first_line.starts_with("summary: "),
            "first line should be severity summary, got: {first_line}"
        );
        assert!(compact.contains("error [SIFR-TYPE-0001]"));
        assert!(compact.contains(" (x8)"));
        assert_eq!(compact.matches("help: ").count(), 1);
        assert_eq!(
            compact
                .matches("url: https://sifr.dev/docs/errors/SIFR-TYPE-0001")
                .count(),
            1
        );
        assert_eq!(compact.matches("  at main.sifr:").count(), 5);
        assert!(compact.contains("  ... +3 more"));
    }

    #[test]
    fn test_compact_renderer_never_drops_or_invents_relative_to_json_count() {
        let diagnostics = vec![
            CompilerDiagnostic {
                code: "SIFR-TYPE-0001".to_string(),
                severity: Severity::Error,
                message: "mismatch one".to_string(),
                url: "https://sifr.dev/docs/errors/SIFR-TYPE-0001".to_string(),
                primary_span: None,
                related_spans: Vec::new(),
                children: Vec::new(),
                help: None,
                suggestions: Vec::new(),
            },
            CompilerDiagnostic {
                code: "SIFR-TYPE-0001".to_string(),
                severity: Severity::Error,
                message: "mismatch one".to_string(),
                url: "https://sifr.dev/docs/errors/SIFR-TYPE-0001".to_string(),
                primary_span: None,
                related_spans: Vec::new(),
                children: Vec::new(),
                help: None,
                suggestions: Vec::new(),
            },
            CompilerDiagnostic {
                code: "SIFR-PARSE-0001".to_string(),
                severity: Severity::Error,
                message: "parse fail".to_string(),
                url: "https://sifr.dev/docs/errors/SIFR-PARSE-0001".to_string(),
                primary_span: None,
                related_spans: Vec::new(),
                children: Vec::new(),
                help: None,
                suggestions: Vec::new(),
            },
        ];
        let compact = render_compact_diagnostics(&diagnostics);
        let grouped_total: usize = compact
            .lines()
            .filter_map(|line| {
                let marker = " (x";
                let start = line.find(marker)?;
                let rest = &line[(start + marker.len())..];
                let end = rest.find(')')?;
                rest[..end].parse::<usize>().ok()
            })
            .sum();
        assert_eq!(grouped_total, diagnostics.len());
    }

    #[test]
    fn test_compact_renderer_snapshot_repeated_diagnostics_summary_group_last() {
        let mut diagnostics = Vec::new();
        for _ in 0..5 {
            diagnostics.push(CompilerDiagnostic {
                code: "SIFR-TYPE-0001".to_string(),
                severity: Severity::Error,
                message: "type mismatch: expected 'int', got 'str'".to_string(),
                url: "https://sifr.dev/docs/errors/SIFR-TYPE-0001".to_string(),
                primary_span: None,
                related_spans: Vec::new(),
                children: Vec::new(),
                help: None,
                suggestions: Vec::new(),
            });
        }
        diagnostics.push(CompilerDiagnostic {
            code: "SIFR-TYPE-0001".to_string(),
            severity: Severity::Error,
            message: "... +3 more similar diagnostics".to_string(),
            url: "https://sifr.dev/docs/errors/SIFR-TYPE-0001".to_string(),
            primary_span: None,
            related_spans: Vec::new(),
            children: Vec::new(),
            help: None,
            suggestions: Vec::new(),
        });

        let expected = concat!(
            "summary: 6 error(s), 0 warning(s), 0 note(s), 0 help item(s)\n",
            "error [SIFR-TYPE-0001] type mismatch: expected 'int', got 'str' (x5)\n",
            "  url: https://sifr.dev/docs/errors/SIFR-TYPE-0001\n",
            "error [SIFR-TYPE-0001] ... +3 more similar diagnostics (x1)\n",
            "  url: https://sifr.dev/docs/errors/SIFR-TYPE-0001\n",
        );
        assert_eq!(render_compact_diagnostics(&diagnostics), expected);
    }

    #[test]
    fn test_compact_renderer_snapshot_multi_severity_group_order() {
        let diagnostics = vec![
            CompilerDiagnostic {
                code: "SIFR-TYPE-0001".to_string(),
                severity: Severity::Warning,
                message: "unused value".to_string(),
                url: "https://sifr.dev/docs/errors/SIFR-TYPE-0001".to_string(),
                primary_span: None,
                related_spans: Vec::new(),
                children: Vec::new(),
                help: Some("remove the assignment".to_string()),
                suggestions: Vec::new(),
            },
            CompilerDiagnostic {
                code: "SIFR-PARSE-0001".to_string(),
                severity: Severity::Error,
                message: "parse failure".to_string(),
                url: "https://sifr.dev/docs/errors/SIFR-PARSE-0001".to_string(),
                primary_span: None,
                related_spans: Vec::new(),
                children: Vec::new(),
                help: None,
                suggestions: Vec::new(),
            },
            CompilerDiagnostic {
                code: "SIFR-CODEGEN-0001".to_string(),
                severity: Severity::Help,
                message: "consider adding a type annotation".to_string(),
                url: "https://sifr.dev/docs/errors/SIFR-CODEGEN-0001".to_string(),
                primary_span: None,
                related_spans: Vec::new(),
                children: Vec::new(),
                help: None,
                suggestions: Vec::new(),
            },
        ];

        let expected = concat!(
            "summary: 1 error(s), 1 warning(s), 0 note(s), 1 help item(s)\n",
            "error [SIFR-PARSE-0001] parse failure (x1)\n",
            "  url: https://sifr.dev/docs/errors/SIFR-PARSE-0001\n",
            "warning [SIFR-TYPE-0001] unused value (x1)\n",
            "  help: remove the assignment\n",
            "  url: https://sifr.dev/docs/errors/SIFR-TYPE-0001\n",
            "help [SIFR-CODEGEN-0001] consider adding a type annotation (x1)\n",
            "  url: https://sifr.dev/docs/errors/SIFR-CODEGEN-0001\n",
        );
        assert_eq!(render_compact_diagnostics(&diagnostics), expected);
    }
}
