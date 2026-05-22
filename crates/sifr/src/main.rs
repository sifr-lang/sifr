//! Sifr Compiler CLI
//!
//! Usage:
//!   sifr build <file.sifr>    Compile to native binary
//!   sifr run <file.sifr>      Compile and run
//!   sifr check <file.sifr>    Type-check only
//!   sifr emit <file.sifr>     Show generated Rust code
//!   sifr fmt [--check] <path> Format Sifr source files
//!   sifr lint <path>          Run suppressible policy diagnostics
//!   sifr lsp --stdio          Run the native Language Server Protocol server
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]
use clap::{Parser, Subcommand, ValueEnum};
use sifr_diagnostics::{
    ChildSeverity, DiagnosticArg, DiagnosticCode, DiagnosticSpan, RenderedDiagnostic, Severity,
};
use sifr_driver::{
    apply_diagnostic_recovery_limits, build, build_cached_project, build_cached_single_file,
    build_project, check_project, check_single_file, compile, diagnostic_label_for_code_str,
    emit_project, find_workspace_root, run_tests, CachedBinaryArtifact, CompileResult,
};
use sifr_python_ast::Stmt;
use sifr_syntax::parse_module_suite;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::{self, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process;

const SIFR_BUILD_VERSION: &str = env!("SIFR_BUILD_VERSION");

#[derive(Parser)]
#[command(
    name = "sifr",
    version = SIFR_BUILD_VERSION,
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
    /// Format Sifr source files
    Fmt {
        /// Check formatting without writing changes
        #[arg(long)]
        check: bool,
        /// Input .sifr file or directory
        path: PathBuf,
    },
    /// Run suppressible policy diagnostics
    Lint {
        /// Input .sifr file or directory
        path: PathBuf,
    },
    /// Run the native Sifr Language Server Protocol server
    Lsp {
        /// Use stdio transport
        #[arg(long)]
        stdio: bool,
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

fn diagnostic_with_code(message: impl Into<String>, code: DiagnosticCode) -> RenderedDiagnostic {
    let message = message.into();
    let mut args = BTreeMap::new();
    args.insert(
        "message".to_string(),
        DiagnosticArg::String(message.clone()),
    );
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message,
        message_template: "{message}".to_string(),
        args,
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

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
        Commands::Fmt { check, path } => cmd_fmt(&path, check, diagnostic_format),
        Commands::Lint { path } => cmd_lint(&path, diagnostic_format),
        Commands::Lsp { stdio } => cmd_lsp(stdio),
        Commands::Emit { file } => cmd_emit(&file, diagnostic_format),
        Commands::Test { dir } => cmd_test(&dir, diagnostic_format),
    }
}

fn cmd_lsp(stdio: bool) -> i32 {
    if !stdio {
        let diagnostic = diagnostic_with_code(
            "sifr lsp requires --stdio in Phase 36",
            DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
        );
        render_diagnostics(&[diagnostic], DiagnosticFormat::Human);
        return EXIT_USAGE_OR_CONFIG;
    }
    match sifr_lsp::run_stdio() {
        Ok(()) => EXIT_SUCCESS,
        Err(error) => {
            let diagnostic = diagnostic_with_code(
                format!("language server failed: {error}"),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            );
            render_diagnostics(&[diagnostic], DiagnosticFormat::Human);
            EXIT_INTERNAL_COMPILER_FAILURE
        }
    }
}

fn resolve_compilation_mode(file: &Path) -> Result<CompilationMode, Vec<RenderedDiagnostic>> {
    if find_workspace_root(file)?.is_some() {
        return Ok(CompilationMode::Project);
    }

    let is_project_entry =
        file.file_stem().is_some_and(|stem| stem == "main") && has_local_project_imports(file);

    if is_project_entry {
        Ok(CompilationMode::Project)
    } else {
        Ok(CompilationMode::SingleFile)
    }
}

fn has_local_project_imports(file: &Path) -> bool {
    let Some(parent) = file.parent() else {
        return false;
    };
    let Ok(source) = std::fs::read_to_string(file) else {
        return false;
    };
    let suite = match parse_module_suite(&source, Some(&file.display().to_string())) {
        Ok(suite) => suite,
        _ => return false,
    };

    suite.iter().any(|stmt| {
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
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
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
    f: impl FnOnce() -> T,
) -> Result<T, Box<RenderedDiagnostic>> {
    let context = context.into();
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => Ok(value),
        Err(payload) => Err(Box::new(diagnostic_with_code(
            format!("{context}: {}", panic_payload_message(payload.as_ref())),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ))),
    }
}

fn is_internal_diagnostic(error: &RenderedDiagnostic) -> bool {
    error.code == DiagnosticCode::INTERNAL_COMPILER_PANIC.code()
}

fn diagnostic_exit_code(errors: &[RenderedDiagnostic]) -> i32 {
    if errors.iter().any(is_internal_diagnostic) {
        EXIT_INTERNAL_COMPILER_FAILURE
    } else if errors
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        EXIT_USER_DIAGNOSTIC
    } else {
        EXIT_SUCCESS
    }
}

#[cfg(test)]
fn legacy_diagnostic_display(diagnostic: &RenderedDiagnostic) -> String {
    format!("{}: {}", human_label(diagnostic), diagnostic.message)
}

fn human_label(diagnostic: &RenderedDiagnostic) -> &'static str {
    match diagnostic.severity {
        Severity::Error if diagnostic.code.starts_with("SIFR-") => {
            diagnostic_label_for_code_str(&diagnostic.code)
        }
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
    }
}

fn compact_severity_summary(diagnostics: &[RenderedDiagnostic]) -> String {
    let mut error_count = 0usize;
    let mut warning_count = 0usize;
    let mut note_count = 0usize;
    let mut help_count = 0usize;
    for diagnostic in diagnostics {
        match diagnostic.severity {
            Severity::Error => error_count += 1,
            Severity::Warning => warning_count += 1,
            Severity::Note => note_count += 1,
        }
        if diagnostic.help.is_some() {
            help_count += 1;
        }
    }
    format!(
        "summary: {error_count} error(s), {warning_count} warning(s), {note_count} note(s), {help_count} help item(s)"
    )
}

fn compact_location_label(span: &DiagnosticSpan) -> String {
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

fn render_compact_diagnostics(diagnostics: &[RenderedDiagnostic]) -> String {
    let mut grouped: BTreeMap<(u8, String, bool, String), Vec<&RenderedDiagnostic>> =
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
            if let Some(span) = diagnostic.spans.iter().find(|span| span.is_primary) {
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

fn canonical_diagnostic_stream(errors: &[RenderedDiagnostic]) -> Vec<RenderedDiagnostic> {
    apply_diagnostic_recovery_limits(errors)
}

fn render_diagnostic_stream(
    diagnostics: &[RenderedDiagnostic],
    format: DiagnosticFormat,
) -> Result<String, serde_json::Error> {
    let mut output = String::new();
    match format {
        DiagnosticFormat::Human => {
            for diagnostic in diagnostics {
                let label = human_label(diagnostic);
                let _ = writeln!(output, "{label}: {message}", message = diagnostic.message);
                for child in &diagnostic.children {
                    let child_label = match child.severity {
                        ChildSeverity::Note => "note",
                        ChildSeverity::Help => "help",
                    };
                    let _ = writeln!(output, "{child_label}: {}", child.message);
                }
            }
        }
        DiagnosticFormat::Json => {
            let json = serde_json::to_string_pretty(diagnostics)?;
            let _ = writeln!(output, "{json}");
        }
        DiagnosticFormat::Compact => {
            let _ = write!(output, "{}", render_compact_diagnostics(diagnostics));
        }
    }
    Ok(output)
}

fn render_diagnostic_output(
    errors: &[RenderedDiagnostic],
    format: DiagnosticFormat,
) -> Result<String, serde_json::Error> {
    let diagnostics = canonical_diagnostic_stream(errors);
    render_diagnostic_stream(&diagnostics, format)
}

fn render_diagnostics(errors: &[RenderedDiagnostic], format: DiagnosticFormat) -> i32 {
    match render_diagnostic_output(errors, format) {
        Ok(output) => {
            let _ = write!(io::stderr(), "{output}");
        }
        Err(e) => {
            let _ = writeln!(
                io::stderr(),
                "build error: failed to serialize diagnostics as json: {e}"
            );
            return EXIT_INTERNAL_COMPILER_FAILURE;
        }
    }
    diagnostic_exit_code(errors)
}

fn cmd_build(file: &Path, output: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let result = match run_with_panic_boundary(
        "internal compiler panic during build command execution",
        || compile_entrypoint(file, output),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
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
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn cmd_run(file: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let result = match run_with_panic_boundary(
        "internal compiler panic during run command compilation",
        || build_run_artifact(file),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
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
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn cmd_check(file: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let errors = match run_with_panic_boundary(
        "internal compiler panic during check command execution",
        || check_entrypoint(file),
    ) {
        Ok(errors) => errors,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
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
        render_diagnostics(&errors, diagnostic_format)
    }
}

fn cmd_fmt(path: &Path, check: bool, diagnostic_format: DiagnosticFormat) -> i32 {
    let result = match run_with_panic_boundary(
        "internal compiler panic during fmt command execution",
        || fmt_entrypoint(path, check),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    match result {
        Ok(changed) => {
            if check {
                if changed.is_empty() {
                    emit_success_message(diagnostic_format, "format check passed");
                    EXIT_SUCCESS
                } else {
                    render_diagnostics(&changed, diagnostic_format)
                }
            } else {
                emit_success_message(diagnostic_format, "formatted Sifr source files");
                EXIT_SUCCESS
            }
        }
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn cmd_lint(path: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let result = match run_with_panic_boundary(
        "internal compiler panic during lint command execution",
        || lint_entrypoint(path),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    match result {
        Ok(diagnostics) if diagnostics.is_empty() => {
            emit_success_message(diagnostic_format, "no lint diagnostics found");
            EXIT_SUCCESS
        }
        Ok(diagnostics) => render_diagnostics(&diagnostics, diagnostic_format),
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn emit_success_message(diagnostic_format: DiagnosticFormat, message: &str) {
    match diagnostic_format {
        DiagnosticFormat::Human => {
            let _ = writeln!(io::stderr(), "{message}");
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
}

fn cmd_test(dir: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let run_result = match run_with_panic_boundary(
        "internal compiler panic during test command execution",
        || run_tests(dir),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };
    match run_result {
        Ok(success) => {
            if success {
                EXIT_SUCCESS
            } else {
                EXIT_USER_DIAGNOSTIC
            }
        }
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn cmd_emit(file: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let compile_result = match run_with_panic_boundary(
        "internal compiler panic during emit command execution",
        || emit_entrypoint(file),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };
    match compile_result {
        CompileResult::Success { rust_source } => {
            let _ = write!(io::stdout(), "{rust_source}");
            EXIT_SUCCESS
        }
        CompileResult::Errors { errors } => render_diagnostics(&errors, diagnostic_format),
    }
}

fn compile_entrypoint(file: &Path, output: &Path) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    match resolve_compilation_mode(file)? {
        CompilationMode::Project => build_project(file, output),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            build(&source, output)
        }
    }
}

fn build_run_artifact(file: &Path) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    match resolve_compilation_mode(file)? {
        CompilationMode::Project => build_cached_project(file),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            build_cached_single_file(&source, file)
        }
    }
}

fn check_entrypoint(file: &Path) -> Vec<RenderedDiagnostic> {
    match resolve_compilation_mode(file) {
        Err(errors) => errors,
        Ok(CompilationMode::Project) => check_project(file),
        Ok(CompilationMode::SingleFile) => {
            let source = read_source(file);
            check_single_file(&source, file)
        }
    }
}

fn emit_entrypoint(file: &Path) -> CompileResult {
    let mode = match resolve_compilation_mode(file) {
        Ok(mode) => mode,
        Err(errors) => return CompileResult::Errors { errors },
    };
    match mode {
        CompilationMode::Project => emit_project(file),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            compile(&source)
        }
    }
}

fn fmt_entrypoint(
    path: &Path,
    check: bool,
) -> Result<Vec<RenderedDiagnostic>, Vec<RenderedDiagnostic>> {
    let files = sifr_format::collect_sifr_files(path)?;
    let mut diagnostics = Vec::new();
    for file in files {
        if check {
            diagnostics.extend(sifr_format::check_path(&file)?);
        } else {
            let _formatted = sifr_format::format_path(&file, false)?;
        }
    }
    Ok(diagnostics)
}

fn lint_entrypoint(path: &Path) -> Result<Vec<RenderedDiagnostic>, Vec<RenderedDiagnostic>> {
    let options = sifr_lint::LintOptions {
        explicit_target: path.is_file(),
        ..sifr_lint::LintOptions::default()
    };
    sifr_lint::lint_path(path, &options).map(|result| result.diagnostics)
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

    fn resolved_mode(file: &Path) -> CompilationMode {
        resolve_compilation_mode(file).expect("compilation mode should resolve")
    }

    fn test_diagnostic(
        code: &str,
        severity: Severity,
        message: &str,
        span: Option<DiagnosticSpan>,
        help: Option<&str>,
    ) -> RenderedDiagnostic {
        RenderedDiagnostic {
            code: code.to_string(),
            severity,
            message: message.to_string(),
            message_template: "{message}".to_string(),
            args: BTreeMap::new(),
            url: format!("https://sifr.sh/docs/errors/{code}"),
            spans: span.into_iter().collect(),
            children: Vec::new(),
            help: help.map(str::to_string),
            suggestions: Vec::new(),
        }
    }

    fn primary_test_span(file: &str, line: u32, column: u32) -> DiagnosticSpan {
        let byte_start = (line.saturating_sub(1) * 100) + column.saturating_sub(1);
        DiagnosticSpan {
            file: Some(file.to_string()),
            byte_start,
            byte_end: byte_start + 1,
            line: Some(line),
            column: Some(column),
            end_line: Some(line),
            end_column: Some(column),
            is_primary: true,
            label: None,
            lines: Vec::new(),
        }
    }

    #[test]
    fn test_json_diagnostic_format_uses_canonical_rendered_schema() {
        let diagnostics = vec![diagnostic_with_code(
            "sample diagnostic",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        )];
        let json = serde_json::to_value(&diagnostics)
            .expect("diagnostics should serialize to canonical JSON");
        let first = json
            .as_array()
            .and_then(|items| items.first())
            .and_then(serde_json::Value::as_object)
            .expect("diagnostic JSON should be an object");

        assert!(first.contains_key("message_template"));
        assert!(first.contains_key("args"));
        assert!(first.contains_key("spans"));
        assert!(!first.contains_key("primary_span"));
        assert!(!first.contains_key("related_spans"));
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

        assert_eq!(resolved_mode(&main), CompilationMode::Project);
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

        assert_eq!(resolved_mode(&app), CompilationMode::SingleFile);
    }

    #[test]
    fn test_manifest_less_run_explicit_non_main_file_stays_single_file() {
        let project = TestProject::new("manifest_less_non_main");
        let app = project.write(
            "app.sifr",
            "def main():\n    pass\n",
            "app file should be written",
        );
        project.write(
            "main.sifr",
            "from helper import value\n\ndef main():\n    print(value())\n",
            "project-like sibling should be written",
        );

        assert_eq!(resolved_mode(&app), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_project_for_non_main_entry_in_workspace() {
        let project = TestProject::new("workspace_non_main");
        project.write(
            "sifr.toml",
            "[source]\nroots = [\"src\"]\n",
            "manifest should be written",
        );
        project.write(
            "src/helper.sifr",
            "VALUE: int = 1\n",
            "helper should be written",
        );
        let app = project.write(
            "src/app.sifr",
            "from helper import VALUE\n\ndef main():\n    print(VALUE)\n",
            "app file should be written",
        );

        assert_eq!(resolved_mode(&app), CompilationMode::Project);
    }

    #[test]
    fn test_resolve_compilation_mode_reports_malformed_workspace_manifest() {
        let project = TestProject::new("workspace_malformed");
        project.write(
            "sifr.toml",
            "[source\nroots = [\".\"]\n",
            "manifest should be written",
        );
        let app = project.write(
            "app.sifr",
            "def main():\n    pass\n",
            "app should be written",
        );

        let errors = resolve_compilation_mode(&app)
            .expect_err("malformed manifest should prevent single-file fallback");

        assert!(errors[0].message.contains("could not parse sifr.toml"));
    }

    #[test]
    fn test_manifest_less_mode_does_not_ignore_malformed_package_manifest() {
        let project = TestProject::new("manifest_less_malformed_manifest");
        project.write(
            "sifr.toml",
            "[source\nroots = [\".\"]\n",
            "manifest should be written",
        );
        let app = project.write(
            "app.sifr",
            "def main():\n    pass\n",
            "app should be written",
        );

        let errors = resolve_compilation_mode(&app)
            .expect_err("package manifest should prevent manifest-less fallback");

        assert!(errors[0].message.contains("could not parse sifr.toml"));
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

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
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

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_missing_local_module() {
        let project = TestProject::new("missing_local");
        let main = project.write(
            "main.sifr",
            "from helper import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
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

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
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

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
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

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
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

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
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

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
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

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
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

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
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

        assert_eq!(resolved_mode(&main), CompilationMode::Project);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_relative_import_without_sibling() {
        let project = TestProject::new("relative_import_missing_sibling");
        let main = project.write(
            "main.sifr",
            "from .helper import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
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

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_bare_relative_import() {
        let project = TestProject::new("relative_import_bare");
        let main = project.write(
            "main.sifr",
            "from . import value\n\ndef main():\n    print(value)\n",
            "main file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
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
        let run_messages: Vec<String> = run_err.iter().map(legacy_diagnostic_display).collect();
        let build_messages: Vec<String> = build_err.iter().map(legacy_diagnostic_display).collect();
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
        let run_messages: Vec<String> = run_err.iter().map(legacy_diagnostic_display).collect();
        let build_messages: Vec<String> = build_err.iter().map(legacy_diagnostic_display).collect();
        assert_eq!(run_messages, build_messages);
        assert!(run_messages
            .iter()
            .any(|m| m.contains("unsupported import form: import helper")));

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
        let run_messages: Vec<String> = run_err.iter().map(legacy_diagnostic_display).collect();
        let build_messages: Vec<String> = build_err.iter().map(legacy_diagnostic_display).collect();
        assert_eq!(run_messages, build_messages);
        assert!(run_messages
            .iter()
            .any(|m| m.contains("unsupported import form: bare relative import")));

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
        let run_messages: Vec<String> = run_err.iter().map(legacy_diagnostic_display).collect();
        let build_messages: Vec<String> = build_err.iter().map(legacy_diagnostic_display).collect();
        assert_eq!(run_messages, build_messages);
        assert!(run_messages
            .iter()
            .any(|m| m.contains("unsupported import form: relative import level 2")));

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
    fn test_check_entrypoint_single_file_reveal_type_is_structured_spanned_note() {
        let dir = mktemp_dir("check_entrypoint_single_reveal_type");
        let main = dir.join("main.sifr");
        std::fs::write(&main, "def main():\n    reveal_type(1)\n")
            .expect("main file should be written");

        let diagnostics = check_entrypoint(&main);
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic.code, DiagnosticCode::TYPE_REVEAL_TYPE.code());
        assert_eq!(diagnostic.severity, Severity::Note);
        assert_eq!(
            diagnostic.message_template,
            "revealed type is {revealed_type}"
        );
        assert_eq!(
            diagnostic.args.get("revealed_type"),
            Some(&DiagnosticArg::String("int".to_string()))
        );

        let primary_span = diagnostic
            .spans
            .iter()
            .find(|span| span.is_primary)
            .expect("reveal_type diagnostic should carry a primary span");
        assert_eq!(
            primary_span.file.as_deref(),
            Some(main.to_string_lossy().as_ref())
        );
        assert_eq!(primary_span.line, Some(2));
        assert!(
            primary_span.byte_end > primary_span.byte_start,
            "reveal_type primary span should cover source bytes"
        );
        assert_eq!(diagnostic_exit_code(&diagnostics), EXIT_SUCCESS);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_check_entrypoint_single_file_arithmetic_warning_is_structured_spanned_warning() {
        let dir = mktemp_dir("check_entrypoint_single_arithmetic_warning");
        let main = dir.join("main.sifr");
        std::fs::write(
            &main,
            "def multiply(a: int, b: int) -> int:\n    return a * b\n\ndef main():\n    print(multiply(2, 3))\n",
        )
        .expect("main file should be written");

        let diagnostics = check_entrypoint(&main);
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];
        assert_eq!(
            diagnostic.code,
            DiagnosticCode::TYPE_ARITHMETIC_OVERFLOW_RISK.code()
        );
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert_eq!(
            diagnostic.message_template,
            "integer {operation} may overflow at runtime"
        );
        assert_eq!(
            diagnostic.args.get("operation"),
            Some(&DiagnosticArg::String("multiplication".to_string()))
        );

        let primary_span = diagnostic
            .spans
            .iter()
            .find(|span| span.is_primary)
            .expect("arithmetic warning should carry a primary span");
        assert_eq!(
            primary_span.file.as_deref(),
            Some(main.to_string_lossy().as_ref())
        );
        assert_eq!(primary_span.line, Some(2));
        assert_eq!(diagnostic_exit_code(&diagnostics), EXIT_SUCCESS);

        let human = render_diagnostic_output(&diagnostics, DiagnosticFormat::Human)
            .expect("human warning diagnostics should render");
        assert_eq!(
            human,
            "warning: integer multiplication may overflow at runtime\n"
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_check_entrypoint_single_file_unreachable_statement_warning_is_structured() {
        let dir = mktemp_dir("check_entrypoint_single_unreachable_warning");
        let main = dir.join("main.sifr");
        std::fs::write(
            &main,
            "def value() -> int:\n    return 1\n    return 2\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");

        let diagnostics = check_entrypoint(&main);
        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];
        assert_eq!(
            diagnostic.code,
            DiagnosticCode::FLOW_UNREACHABLE_STATEMENT.code()
        );
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert_eq!(diagnostic.message_template, "unreachable statement ignored");
        assert!(diagnostic.args.is_empty());
        let primary_span = diagnostic
            .spans
            .iter()
            .find(|span| span.is_primary)
            .expect("unreachable warning should carry a primary span");
        assert_eq!(
            primary_span.file.as_deref(),
            Some(main.to_string_lossy().as_ref())
        );
        assert_eq!(primary_span.line, Some(3));
        assert_eq!(diagnostic_exit_code(&diagnostics), EXIT_SUCCESS);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_check_entrypoint_reveal_type_notes_obey_recovery_cap() {
        let dir = mktemp_dir("check_entrypoint_reveal_type_cap");
        let main = dir.join("main.sifr");
        let mut source = String::new();
        for index in 0..60 {
            let _ = writeln!(source, "class T{index}:");
            let _ = writeln!(source, "    pass");
            let _ = writeln!(source);
        }
        let _ = writeln!(source, "def main():");
        for index in 0..60 {
            let _ = writeln!(source, "    reveal_type(T{index}())");
        }
        std::fs::write(&main, source).expect("main file should be written");

        let diagnostics = check_entrypoint(&main);
        assert_eq!(diagnostics.len(), 60);
        assert_eq!(diagnostic_exit_code(&diagnostics), EXIT_SUCCESS);

        let canonical = canonical_diagnostic_stream(&diagnostics);
        assert_eq!(canonical.len(), 50);
        assert_eq!(
            canonical
                .iter()
                .filter(|diagnostic| diagnostic.code == DiagnosticCode::TYPE_REVEAL_TYPE.code())
                .count(),
            49
        );
        let summary = canonical
            .last()
            .expect("recovery cap should append an omission summary");
        assert_eq!(
            summary.code,
            DiagnosticCode::INTERNAL_RECOVERY_OMISSION_SUMMARY.code()
        );
        // The summary occupies the final display slot, so 60 raw notes become
        // 49 explicit notes plus one summary for the 11 omitted notes.
        assert_eq!(
            summary.message,
            "11 additional reveal_type results omitted by recovery cap (top-level diagnostic stream)"
        );
        assert_eq!(
            summary.args.get("omitted_count"),
            Some(&DiagnosticArg::Unsigned(11))
        );
        assert_eq!(
            summary.args.get("omitted_kind"),
            Some(&DiagnosticArg::String("reveal_type results".to_string()))
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

        let check_messages: Vec<String> =
            check_errors.iter().map(legacy_diagnostic_display).collect();
        let build_messages: Vec<String> =
            build_errors.iter().map(legacy_diagnostic_display).collect();
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
    fn test_emit_entrypoint_uses_project_mode_for_project_like_main() {
        let dir = mktemp_dir("emit_project_boundary");
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
        let rust_source = match emit_result {
            CompileResult::Success { rust_source } => rust_source,
            CompileResult::Errors { errors } => {
                panic!("emit should use project mode successfully: {errors:?}")
            }
        };
        assert!(rust_source.contains("// src/main.rs"));
        assert!(rust_source.contains("// src/helper.rs"));

        let _ = std::fs::remove_dir_all(dir);
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

        let check_messages: Vec<String> =
            check_errors.iter().map(legacy_diagnostic_display).collect();
        let run_messages: Vec<String> = run_errors.iter().map(legacy_diagnostic_display).collect();
        let build_messages: Vec<String> =
            build_errors.iter().map(legacy_diagnostic_display).collect();
        assert_eq!(check_messages, run_messages);
        assert_eq!(run_messages, build_messages);

        let _ = std::fs::remove_dir_all(run_out);
        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_diagnostic_exit_code_contract_user_vs_internal() {
        let user_error = diagnostic_with_code("type mismatch", DiagnosticCode::TYPE_MISMATCH);
        assert_eq!(diagnostic_exit_code(&[user_error]), EXIT_USER_DIAGNOSTIC);

        let reveal_note = test_diagnostic(
            "SIFR-TYPE-0902",
            Severity::Note,
            "revealed type is int",
            None,
            None,
        );
        assert_eq!(diagnostic_exit_code(&[reveal_note]), EXIT_SUCCESS);

        let overflow_warning = test_diagnostic(
            "SIFR-TYPE-0901",
            Severity::Warning,
            "integer addition may overflow at runtime",
            None,
            None,
        );
        assert_eq!(diagnostic_exit_code(&[overflow_warning]), EXIT_SUCCESS);

        let internal_error = diagnostic_with_code(
            "internal compiler panic during single-file code generation: boom",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        );
        assert_eq!(
            diagnostic_exit_code(&[internal_error]),
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
    fn test_run_with_panic_boundary_converts_panic_to_internal_diagnostic() {
        let error = run_with_panic_boundary(
            "internal compiler panic during test boundary",
            || -> usize { panic!("boom") },
        )
        .expect_err("panic should convert to an internal compiler diagnostic");
        assert!(error
            .message
            .contains("internal compiler panic during test boundary: boom"));
        let error = *error;
        assert_eq!(
            diagnostic_exit_code(&[error]),
            EXIT_INTERNAL_COMPILER_FAILURE
        );
    }

    #[test]
    fn test_compact_renderer_invariants_summary_grouping_and_bounds() {
        let mut diagnostics = Vec::new();
        for idx in 0..8 {
            diagnostics.push(test_diagnostic(
                "SIFR-TYPE-0002",
                Severity::Error,
                "type mismatch: expected 'int', got 'str'",
                Some(primary_test_span("main.sifr", idx + 1, 1)),
                Some("fix assignment type"),
            ));
        }
        let compact = render_compact_diagnostics(&diagnostics);
        let mut lines = compact.lines();
        let first_line = lines.next().expect("compact output should have first line");
        assert!(
            first_line.starts_with("summary: "),
            "first line should be severity summary, got: {first_line}"
        );
        assert!(compact.contains("error [SIFR-TYPE-0002]"));
        assert!(compact.contains(" (x8)"));
        assert_eq!(compact.matches("help: ").count(), 1);
        assert_eq!(
            compact
                .matches("url: https://sifr.sh/docs/errors/SIFR-TYPE-0002")
                .count(),
            1
        );
        assert_eq!(compact.matches("  at main.sifr:").count(), 5);
        assert!(compact.contains("  ... +3 more"));
    }

    #[test]
    fn test_compact_renderer_never_drops_or_invents_relative_to_json_count() {
        let diagnostics = vec![
            test_diagnostic(
                "SIFR-TYPE-0002",
                Severity::Error,
                "mismatch one",
                None,
                None,
            ),
            test_diagnostic(
                "SIFR-TYPE-0002",
                Severity::Error,
                "mismatch one",
                None,
                None,
            ),
            test_diagnostic("SIFR-PARSE-0002", Severity::Error, "parse fail", None, None),
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
    fn test_diagnostic_formats_share_canonical_sorted_capped_stream() {
        let mut diagnostics = Vec::new();
        for idx in (0..49).rev() {
            diagnostics.push(test_diagnostic(
                "SIFR-TYPE-0002",
                Severity::Error,
                &format!("distinct diagnostic {idx:02}"),
                Some(primary_test_span(
                    &format!("zzz_distinct_{idx:02}.sifr"),
                    1,
                    1,
                )),
                None,
            ));
        }
        for idx in (0..8).rev() {
            diagnostics.push(test_diagnostic(
                "SIFR-TYPE-0002",
                Severity::Error,
                "aaa repeated mismatch",
                Some(primary_test_span("aaa_repeated.sifr", idx + 1, 1)),
                None,
            ));
        }

        let canonical = canonical_diagnostic_stream(&diagnostics);
        assert_eq!(canonical.len(), 50);
        assert!(canonical
            .iter()
            .take(5)
            .all(|diagnostic| diagnostic.code == "SIFR-TYPE-0002"
                && diagnostic.message == "aaa repeated mismatch"));
        assert_eq!(canonical[5].code, "SIFR-INTERNAL-0002");
        assert_eq!(
            canonical[5].message,
            "3 additional diagnostics omitted by recovery cap (similar-diagnostic group)"
        );
        assert!(canonical
            .iter()
            .any(|diagnostic| diagnostic.message == "distinct diagnostic 42"));
        assert!(!canonical
            .iter()
            .any(|diagnostic| diagnostic.message == "distinct diagnostic 43"));
        assert_eq!(canonical[49].code, "SIFR-INTERNAL-0002");
        assert_eq!(
            canonical[49].message,
            "6 additional diagnostics omitted by recovery cap (top-level diagnostic stream)"
        );

        let json_output = render_diagnostic_output(&diagnostics, DiagnosticFormat::Json)
            .expect("JSON diagnostics should render");
        let json_diagnostics: Vec<RenderedDiagnostic> =
            serde_json::from_str(&json_output).expect("JSON output should be diagnostic stream");
        assert_eq!(json_diagnostics, canonical);

        let human_output = render_diagnostic_output(&diagnostics, DiagnosticFormat::Human)
            .expect("human diagnostics should render");
        let expected_human = canonical
            .iter()
            .map(legacy_diagnostic_display)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        assert_eq!(human_output, expected_human);

        let compact_output = render_diagnostic_output(&diagnostics, DiagnosticFormat::Compact)
            .expect("compact diagnostics should render");
        let summary = compact_output
            .lines()
            .next()
            .expect("compact output should start with a summary");
        assert_eq!(
            summary,
            "summary: 48 error(s), 0 warning(s), 2 note(s), 0 help item(s)"
        );
        let compact_total: usize = compact_output
            .lines()
            .filter_map(|line| {
                let marker = " (x";
                let start = line.find(marker)?;
                let rest = &line[(start + marker.len())..];
                let end = rest.find(')')?;
                rest[..end].parse::<usize>().ok()
            })
            .sum();
        assert_eq!(compact_total, canonical.len());
        assert!(compact_output.contains("error [SIFR-TYPE-0002] distinct diagnostic 42 (x1)"));
        assert!(!compact_output.contains("distinct diagnostic 43"));
    }

    #[test]
    fn test_human_diagnostic_format_renders_child_notes() {
        let mut diagnostic = test_diagnostic(
            "SIFR-PARSE-0002",
            Severity::Error,
            "syntax error: expected expression",
            None,
            None,
        );
        diagnostic
            .children
            .push(sifr_diagnostics::render::RenderedDiagnosticChild {
                severity: ChildSeverity::Note,
                message: "while parsing helper".to_string(),
            });

        let human_output = render_diagnostic_output(&[diagnostic], DiagnosticFormat::Human)
            .expect("human diagnostics should render");
        assert_eq!(
            human_output,
            "parse error: syntax error: expected expression\nnote: while parsing helper\n"
        );
    }

    #[test]
    fn test_compact_renderer_snapshot_repeated_diagnostics_summary_group_last() {
        let mut diagnostics = Vec::new();
        for _ in 0..5 {
            diagnostics.push(test_diagnostic(
                "SIFR-TYPE-0002",
                Severity::Error,
                "type mismatch: expected 'int', got 'str'",
                None,
                None,
            ));
        }
        diagnostics.push(test_diagnostic(
            "SIFR-TYPE-0002",
            Severity::Error,
            "... +3 more similar diagnostics",
            None,
            None,
        ));

        let expected = concat!(
            "summary: 6 error(s), 0 warning(s), 0 note(s), 0 help item(s)\n",
            "error [SIFR-TYPE-0002] type mismatch: expected 'int', got 'str' (x5)\n",
            "  url: https://sifr.sh/docs/errors/SIFR-TYPE-0002\n",
            "error [SIFR-TYPE-0002] ... +3 more similar diagnostics (x1)\n",
            "  url: https://sifr.sh/docs/errors/SIFR-TYPE-0002\n",
        );
        assert_eq!(render_compact_diagnostics(&diagnostics), expected);
    }

    #[test]
    fn test_compact_renderer_snapshot_multi_severity_group_order() {
        let diagnostics = vec![
            test_diagnostic(
                "SIFR-TYPE-0002",
                Severity::Warning,
                "unused value",
                None,
                Some("remove the assignment"),
            ),
            test_diagnostic(
                "SIFR-PARSE-0002",
                Severity::Error,
                "parse failure",
                None,
                None,
            ),
            test_diagnostic(
                "SIFR-INTERNAL-0002",
                Severity::Note,
                "consider adding a type annotation",
                None,
                None,
            ),
        ];

        let expected = concat!(
            "summary: 1 error(s), 1 warning(s), 1 note(s), 1 help item(s)\n",
            "error [SIFR-PARSE-0002] parse failure (x1)\n",
            "  url: https://sifr.sh/docs/errors/SIFR-PARSE-0002\n",
            "warning [SIFR-TYPE-0002] unused value (x1)\n",
            "  help: remove the assignment\n",
            "  url: https://sifr.sh/docs/errors/SIFR-TYPE-0002\n",
            "note [SIFR-INTERNAL-0002] consider adding a type annotation (x1)\n",
            "  url: https://sifr.sh/docs/errors/SIFR-INTERNAL-0002\n",
        );
        assert_eq!(render_compact_diagnostics(&diagnostics), expected);
    }
}
