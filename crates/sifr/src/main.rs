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
    apply_diagnostic_recovery_limits, build, build_cached_package_project, build_cached_project,
    build_cached_single_file, build_project, check_package_project, check_project,
    check_single_file, compile, diagnostic_label_for_code_str, emit_project, find_workspace_root,
    run_tests, CachedBinaryArtifact, CompileResult, PackageEntrypoint,
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

    /// Explain a Sifr diagnostic code without running a package operation
    #[arg(long)]
    explain: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
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
        /// Input .sifr file, app target, or script name
        target: Option<String>,
        /// Select a layout-discovered app target
        #[arg(long)]
        bin: Option<String>,
        /// Select a named package script
        #[arg(long)]
        script: Option<String>,
        /// Require Cargo.lock to be unchanged
        #[arg(long)]
        locked: bool,
        /// Disable network access
        #[arg(long)]
        offline: bool,
        /// Combine --locked and --offline
        #[arg(long)]
        frozen: bool,
        /// Arguments passed to the selected app after --
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Fetch package dependencies
    Fetch {
        /// Require Cargo.lock to be unchanged
        #[arg(long)]
        locked: bool,
        /// Disable network access
        #[arg(long)]
        offline: bool,
        /// Combine --locked and --offline
        #[arg(long)]
        frozen: bool,
    },
    /// Create a new Sifr package
    Init {
        /// Target directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Create a library package
        #[arg(long, conflicts_with = "bin")]
        lib: bool,
        /// Create an app package
        #[arg(long)]
        bin: bool,
        /// Sifr package name
        #[arg(long)]
        name: Option<String>,
        /// Create missing Sifr-owned files without overwriting existing files
        #[arg(long)]
        force: bool,
    },
    /// Repair Sifr-managed Cargo projection drift
    Repair {
        /// Check projection drift without writing
        #[arg(long)]
        check: bool,
    },
    /// Type-check a .sifr file without compiling
    Check {
        /// Input .sifr file, or omit for package check
        path: Option<PathBuf>,
        /// Check all Sifr-capable workspace members through Cargo-compatible selection
        #[arg(long)]
        workspace: bool,
        /// Select one package by Cargo package spec or unambiguous package name
        #[arg(short = 'p', long = "package")]
        packages: Vec<String>,
        /// Exclude one package from workspace selection
        #[arg(long)]
        exclude: Vec<String>,
        /// Cargo-compatible package message format for package checks
        #[arg(long)]
        message_format: Option<String>,
        /// Require Cargo.lock to be unchanged
        #[arg(long)]
        locked: bool,
        /// Disable network access
        #[arg(long)]
        offline: bool,
        /// Combine --locked and --offline
        #[arg(long)]
        frozen: bool,
    },
    /// Show the package dependency tree
    Tree {
        /// Require Cargo.lock to be unchanged
        #[arg(long)]
        locked: bool,
        /// Disable network access
        #[arg(long)]
        offline: bool,
        /// Combine --locked and --offline
        #[arg(long)]
        frozen: bool,
        /// Cargo-compatible tree options
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Assemble and verify a Cargo package archive for a Sifr package
    Package {
        /// Package all Sifr-capable workspace members through Cargo-compatible selection
        #[arg(long)]
        workspace: bool,
        /// Select one package by Cargo package spec or unambiguous package name
        #[arg(short = 'p', long = "package")]
        packages: Vec<String>,
        /// Exclude one package from workspace selection
        #[arg(long)]
        exclude: Vec<String>,
        /// Print packaged files without creating an archive
        #[arg(long)]
        list: bool,
        /// Skip Cargo's package verification build
        #[arg(long)]
        no_verify: bool,
        /// Skip Cargo package metadata warning checks
        #[arg(long)]
        no_metadata: bool,
        /// Allow dirty working tree contents
        #[arg(long)]
        allow_dirty: bool,
        /// Exclude Cargo.lock from the package archive
        #[arg(long)]
        exclude_lockfile: bool,
        /// Require Cargo.lock to be unchanged
        #[arg(long)]
        locked: bool,
        /// Disable network access
        #[arg(long)]
        offline: bool,
        /// Combine --locked and --offline
        #[arg(long)]
        frozen: bool,
    },
    /// Publish a Sifr package through Cargo
    Publish {
        /// Validate publish without uploading
        #[arg(long)]
        dry_run: bool,
        /// Publish all Sifr-capable workspace members through Cargo-compatible selection
        #[arg(long)]
        workspace: bool,
        /// Select one package by Cargo package spec or unambiguous package name
        #[arg(short = 'p', long = "package")]
        packages: Vec<String>,
        /// Exclude one package from workspace selection
        #[arg(long)]
        exclude: Vec<String>,
        /// Skip Cargo's publish verification build
        #[arg(long)]
        no_verify: bool,
        /// Allow dirty working tree contents
        #[arg(long)]
        allow_dirty: bool,
        /// Require Cargo.lock to be unchanged
        #[arg(long)]
        locked: bool,
        /// Disable network access
        #[arg(long)]
        offline: bool,
        /// Combine --locked and --offline
        #[arg(long)]
        frozen: bool,
    },
    /// Vendor dependency sources through Cargo
    Vendor {
        /// Output directory for vendored sources
        #[arg(default_value = "vendor")]
        path: PathBuf,
        /// Additional manifest to sync during vendoring
        #[arg(long)]
        sync: Vec<PathBuf>,
        /// Keep stale vendored sources
        #[arg(long)]
        no_delete: bool,
        /// Respect existing Cargo source configuration
        #[arg(long)]
        respect_source_config: bool,
        /// Use versioned vendor directory names
        #[arg(long)]
        versioned_dirs: bool,
        /// Require Cargo.lock to be unchanged
        #[arg(long)]
        locked: bool,
        /// Disable network access
        #[arg(long)]
        offline: bool,
        /// Combine --locked and --offline
        #[arg(long)]
        frozen: bool,
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

struct PackageCompilerContext {
    graph: sifr_package::SifrPackageGraph,
    source_map: sifr_package::PackageSourceMap,
    package_id: sifr_package::SifrPackageId,
}

struct PackageGraphContext {
    metadata: sifr_package::NormalizedCargoMetadata,
    graph: sifr_package::SifrPackageGraph,
    source_map: sifr_package::PackageSourceMap,
}

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
    if let Some(code) = cli.explain {
        return cmd_explain(&code, diagnostic_format);
    }
    let Some(command) = cli.command else {
        let diagnostic = diagnostic_with_code(
            "no command provided",
            DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
        );
        render_diagnostics(&[diagnostic], diagnostic_format);
        return EXIT_USAGE_OR_CONFIG;
    };
    match command {
        Commands::Build { file, output } => cmd_build(&file, &output, diagnostic_format),
        Commands::Run {
            target,
            bin,
            script,
            locked,
            offline,
            frozen,
            args,
        } => cmd_run(
            target.as_deref(),
            bin.as_deref(),
            script.as_deref(),
            &args,
            lock_mode_from_flags(locked, offline, frozen),
            diagnostic_format,
        ),
        Commands::Fetch {
            locked,
            offline,
            frozen,
        } => cmd_fetch(
            lock_mode_from_flags(locked, offline, frozen),
            diagnostic_format,
        ),
        Commands::Init {
            path,
            lib,
            bin,
            name,
            force,
        } => cmd_init(&path, lib, bin, name.as_deref(), force, diagnostic_format),
        Commands::Repair { check } => cmd_repair(check, diagnostic_format),
        Commands::Check {
            path,
            workspace,
            packages,
            exclude,
            message_format,
            locked,
            offline,
            frozen,
        } => {
            let selection = sifr_package::CargoPackageSelection {
                workspace,
                packages,
                excludes: exclude,
            };
            cmd_check(
                path.as_deref(),
                message_format.as_deref(),
                &selection,
                lock_mode_from_flags(locked, offline, frozen),
                diagnostic_format,
            )
        }
        Commands::Tree {
            locked,
            offline,
            frozen,
            args,
        } => cmd_tree(
            lock_mode_from_flags(locked, offline, frozen),
            &args,
            diagnostic_format,
        ),
        Commands::Package {
            workspace,
            packages,
            exclude,
            list,
            no_verify,
            no_metadata,
            allow_dirty,
            exclude_lockfile,
            locked,
            offline,
            frozen,
        } => {
            let selection = sifr_package::CargoPackageSelection {
                workspace,
                packages,
                excludes: exclude,
            };
            let options = sifr_package::CargoPackageArchiveOptions {
                list,
                no_verify,
                no_metadata,
                allow_dirty,
                exclude_lockfile,
            };
            cmd_package(
                &selection,
                &options,
                lock_mode_from_flags(locked, offline, frozen),
                diagnostic_format,
            )
        }
        Commands::Publish {
            dry_run,
            workspace,
            packages,
            exclude,
            no_verify,
            allow_dirty,
            locked,
            offline,
            frozen,
        } => {
            let selection = sifr_package::CargoPackageSelection {
                workspace,
                packages,
                excludes: exclude,
            };
            let options = sifr_package::CargoPublishOptions {
                dry_run,
                no_verify,
                allow_dirty,
            };
            cmd_publish(
                &selection,
                &options,
                lock_mode_from_flags(locked, offline, frozen),
                diagnostic_format,
            )
        }
        Commands::Vendor {
            path,
            sync,
            no_delete,
            respect_source_config,
            versioned_dirs,
            locked,
            offline,
            frozen,
        } => {
            let options = sifr_package::CargoVendorOptions {
                sync,
                no_delete,
                respect_source_config,
                versioned_dirs,
            };
            cmd_vendor(
                &path,
                &options,
                lock_mode_from_flags(locked, offline, frozen),
                diagnostic_format,
            )
        }
        Commands::Fmt { check, path } => cmd_fmt(&path, check, diagnostic_format),
        Commands::Lint { path } => cmd_lint(&path, diagnostic_format),
        Commands::Lsp { stdio } => cmd_lsp(stdio),
        Commands::Emit { file } => cmd_emit(&file, diagnostic_format),
        Commands::Test { dir } => cmd_test(&dir, diagnostic_format),
    }
}

fn cmd_init(
    path: &Path,
    lib: bool,
    bin: bool,
    name: Option<&str>,
    force: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let kind = if lib && !bin {
        sifr_package::InitPackageKind::Lib
    } else {
        sifr_package::InitPackageKind::Bin
    };
    let sifr_name = name
        .map(str::to_string)
        .or_else(|| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "app".to_string());
    let options = sifr_package::InitPackageOptions {
        target_dir: path.to_path_buf(),
        sifr_name,
        kind,
        force,
    };
    match sifr_package::init_package(&options) {
        Ok(_) => EXIT_SUCCESS,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            EXIT_USAGE_OR_CONFIG
        }
    }
}

fn cmd_repair(check: bool, diagnostic_format: DiagnosticFormat) -> i32 {
    let root = match std::env::current_dir() {
        Ok(root) => root,
        Err(error) => {
            let diagnostic = diagnostic_with_code(
                format!("could not read current directory: {error}"),
                DiagnosticCode::PACKAGE_PROJECTION_MANIFEST_POINTER_DRIFT,
            );
            render_diagnostics(&[diagnostic], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let repair = sifr_package::repair_projection(&root, check);
    if repair.diagnostics.is_empty() {
        EXIT_SUCCESS
    } else {
        let diagnostics = repair
            .diagnostics
            .into_iter()
            .map(package_diagnostic)
            .collect::<Vec<_>>();
        render_diagnostics(&diagnostics, diagnostic_format);
        EXIT_USER_DIAGNOSTIC
    }
}

fn package_diagnostic(diagnostic: sifr_package::PackageDiagnostic) -> RenderedDiagnostic {
    diagnostic_with_code(diagnostic.message, diagnostic.code)
}

fn lock_mode_from_flags(locked: bool, offline: bool, frozen: bool) -> sifr_package::CargoLockMode {
    if frozen {
        sifr_package::CargoLockMode::Frozen
    } else if offline {
        sifr_package::CargoLockMode::Offline
    } else if locked {
        sifr_package::CargoLockMode::Locked
    } else {
        sifr_package::CargoLockMode::Normal
    }
}

fn cmd_explain(code: &str, diagnostic_format: DiagnosticFormat) -> i32 {
    let explanation = diagnostic_explanation(code);
    if let Some(text) = explanation {
        match diagnostic_format {
            DiagnosticFormat::Human | DiagnosticFormat::Compact => {
                let _ = writeln!(io::stdout(), "{text}");
            }
            DiagnosticFormat::Json => {
                let value = serde_json::json!({ "code": code, "explanation": text });
                let _ = writeln!(
                    io::stdout(),
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
                );
            }
        }
        EXIT_SUCCESS
    } else {
        let diagnostic = diagnostic_with_code(
            format!("unknown diagnostic code '{code}'"),
            DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
        );
        render_diagnostics(&[diagnostic], diagnostic_format);
        EXIT_USAGE_OR_CONFIG
    }
}

fn diagnostic_explanation(code: &str) -> Option<String> {
    if code == "SIFR-PACKAGE-0105" {
        return Some(
            "SIFR-PACKAGE-0105 is retired. Cargo credential failures are reported as SIFR-PACKAGE-0101 so Sifr preserves Cargo's underlying error text with credential redaction."
                .to_string(),
        );
    }
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .parent()?
        .to_path_buf();
    let path = repo_root.join("docs/errors").join(format!("{code}.md"));
    let text = std::fs::read_to_string(path).ok()?;
    let mut lines = text
        .lines()
        .filter(|line| !line.starts_with("<!--") && !line.starts_with('|'));
    let title = lines.find(|line| line.starts_with("# "))?;
    let summary = lines.find(|line| !line.trim().is_empty()).unwrap_or("");
    Some(format!(
        "{}\n\n{}\n\nDocs: https://sifr.sh/docs/errors/{code}",
        title.trim_start_matches("# "),
        summary,
    ))
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

fn cmd_run(
    target: Option<&str>,
    bin: Option<&str>,
    script: Option<&str>,
    app_args: &[String],
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let current_dir = match std::env::current_dir() {
        Ok(current_dir) => current_dir,
        Err(error) => {
            let diagnostic = diagnostic_with_code(
                format!("could not read current directory: {error}"),
                DiagnosticCode::PACKAGE_CARGO_COMMAND_FAILED,
            );
            render_diagnostics(&[diagnostic], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let session =
        match sifr_package::PackageSession::discover(sifr_package::PackageSessionOptions {
            current_dir,
            lock_mode,
        }) {
            Ok(session) => session,
            Err(error) => {
                render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
                return EXIT_USAGE_OR_CONFIG;
            }
        };
    let plan = session.plan_run(&sifr_package::PackageRunRequest {
        target_or_path: target.map(str::to_string),
        bin: bin.map(str::to_string),
        script: script.map(str::to_string),
        app_args: app_args.to_vec(),
        script_depth: 0,
    });
    let plan = match plan {
        Ok(plan) => plan,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USER_DIAGNOSTIC;
        }
    };
    if let Some(origin) = plan.script_origin {
        return cmd_script(&origin, diagnostic_format);
    }
    if let Some(
        sifr_package::ResolvedRunTarget::File(path)
        | sifr_package::ResolvedRunTarget::App { path, .. },
    ) = plan.run_target
    {
        if !session.manifest_less_mode {
            return cmd_run_package_file(&path, &session, lock_mode, app_args, diagnostic_format);
        }
        return cmd_run_file(&path, app_args, diagnostic_format);
    }
    if let Some(cargo) = plan.cargo {
        return execute_cargo_plan(&cargo, lock_mode, diagnostic_format);
    }
    EXIT_SUCCESS
}

fn cmd_script(origin: &sifr_package::ScriptOrigin, diagnostic_format: DiagnosticFormat) -> i32 {
    match origin.command.as_str() {
        "run" => cmd_run(
            origin.args.first().map(String::as_str),
            None,
            None,
            &[],
            sifr_package::CargoLockMode::Normal,
            diagnostic_format,
        ),
        "check" => cmd_check(
            origin
                .args
                .first()
                .filter(|arg| !arg.starts_with('-'))
                .map(Path::new),
            None,
            &sifr_package::CargoPackageSelection::default(),
            sifr_package::CargoLockMode::Normal,
            diagnostic_format,
        ),
        "fetch" => cmd_fetch(sifr_package::CargoLockMode::Normal, diagnostic_format),
        "tree" => cmd_tree(
            sifr_package::CargoLockMode::Normal,
            &origin.args,
            diagnostic_format,
        ),
        "package" => cmd_package(
            &sifr_package::CargoPackageSelection::default(),
            &sifr_package::CargoPackageArchiveOptions::default(),
            sifr_package::CargoLockMode::Normal,
            diagnostic_format,
        ),
        "publish" => cmd_publish(
            &sifr_package::CargoPackageSelection::default(),
            &sifr_package::CargoPublishOptions {
                dry_run: origin.args.iter().any(|arg| arg == "--dry-run"),
                ..sifr_package::CargoPublishOptions::default()
            },
            sifr_package::CargoLockMode::Normal,
            diagnostic_format,
        ),
        "vendor" => cmd_vendor(
            origin
                .args
                .first()
                .map(Path::new)
                .unwrap_or_else(|| Path::new("vendor")),
            &sifr_package::CargoVendorOptions::default(),
            sifr_package::CargoLockMode::Normal,
            diagnostic_format,
        ),
        _ => {
            let diagnostic = diagnostic_with_code(
                format!("unsupported package script command '{}'", origin.command),
                DiagnosticCode::PACKAGE_SCRIPT_RECURSION,
            );
            render_diagnostics(&[diagnostic], diagnostic_format);
            EXIT_USAGE_OR_CONFIG
        }
    }
}

fn cmd_run_file(file: &Path, app_args: &[String], diagnostic_format: DiagnosticFormat) -> i32 {
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
                .args(app_args)
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

fn cmd_run_package_file(
    file: &Path,
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    app_args: &[String],
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let context = match package_compiler_context(session, lock_mode, diagnostic_format) {
        Ok(Some(context)) => context,
        Ok(None) => return cmd_run_file(file, app_args, diagnostic_format),
        Err(exit_code) => return exit_code,
    };
    let entrypoint = PackageEntrypoint {
        main_file: file.to_path_buf(),
        package_id: context.package_id,
        graph: context.graph,
        source_map: context.source_map,
    };
    let result = match run_with_panic_boundary(
        "internal compiler panic during package run command compilation",
        || build_cached_package_project(&entrypoint),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    match result {
        Ok(artifact) => run_binary_artifact(&artifact, app_args),
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn run_binary_artifact(artifact: &CachedBinaryArtifact, app_args: &[String]) -> i32 {
    let _ = writeln!(io::stderr(), "{}", artifact.cache_status_line());
    let output = std::process::Command::new(artifact.binary_path())
        .args(app_args)
        .output()
        .unwrap_or_else(|e| {
            let _ = writeln!(io::stderr(), "error: could not run binary: {e}");
            process::exit(EXIT_USAGE_OR_CONFIG);
        });

    std::io::stdout().write_all(&output.stdout).ok();
    std::io::stderr().write_all(&output.stderr).ok();

    if output.status.success() {
        EXIT_SUCCESS
    } else {
        EXIT_USER_DIAGNOSTIC
    }
}

fn cmd_fetch(lock_mode: sifr_package::CargoLockMode, diagnostic_format: DiagnosticFormat) -> i32 {
    let session = match package_session_for_cwd(lock_mode) {
        Ok(session) => session,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let Some(cargo) = session.plan_fetch().cargo else {
        return EXIT_SUCCESS;
    };
    execute_cargo_plan(&cargo, lock_mode, diagnostic_format)
}

fn cmd_tree(
    lock_mode: sifr_package::CargoLockMode,
    args: &[String],
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let session = match package_session_for_cwd(lock_mode) {
        Ok(session) => session,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let Some(cargo) = session.plan_tree(args).cargo else {
        return EXIT_SUCCESS;
    };
    execute_cargo_plan(&cargo, lock_mode, diagnostic_format)
}

fn cmd_package(
    selection: &sifr_package::CargoPackageSelection,
    options: &sifr_package::CargoPackageArchiveOptions,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let session = match package_session_for_cwd(lock_mode) {
        Ok(session) => session,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    if let Err(exit_code) = run_package_release_preflight(
        &session,
        selection,
        options.allow_dirty,
        lock_mode,
        diagnostic_format,
    ) {
        return exit_code;
    }
    let plan = session.plan_package(
        &sifr_package::CargoFeatureSelection::default(),
        selection,
        options,
    );
    let Some(cargo) = plan.cargo else {
        return EXIT_SUCCESS;
    };
    execute_cargo_plan(&cargo, lock_mode, diagnostic_format)
}

fn cmd_publish(
    selection: &sifr_package::CargoPackageSelection,
    options: &sifr_package::CargoPublishOptions,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let session = match package_session_for_cwd(lock_mode) {
        Ok(session) => session,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    if let Err(exit_code) = run_package_release_preflight(
        &session,
        selection,
        options.allow_dirty,
        lock_mode,
        diagnostic_format,
    ) {
        return exit_code;
    }
    let plan = session.plan_publish(
        &sifr_package::CargoFeatureSelection::default(),
        selection,
        options,
    );
    let Some(cargo) = plan.cargo else {
        return EXIT_SUCCESS;
    };
    execute_cargo_plan(&cargo, lock_mode, diagnostic_format)
}

fn cmd_vendor(
    output_dir: &Path,
    options: &sifr_package::CargoVendorOptions,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let session = match package_session_for_cwd(lock_mode) {
        Ok(session) => session,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let plan = session.plan_vendor(output_dir, options);
    let Some(cargo) = plan.cargo else {
        return EXIT_SUCCESS;
    };
    execute_cargo_plan(&cargo, lock_mode, diagnostic_format)
}

fn package_session_for_cwd(
    lock_mode: sifr_package::CargoLockMode,
) -> Result<sifr_package::PackageSession, sifr_package::PackageDiagnostic> {
    let current_dir = std::env::current_dir().map_err(|error| {
        sifr_package::PackageDiagnostic::cargo_command_failed(
            sifr_package::CargoAction::Metadata,
            format!("could not read current directory: {error}"),
        )
    })?;
    sifr_package::PackageSession::discover(sifr_package::PackageSessionOptions {
        current_dir,
        lock_mode,
    })
}

fn run_package_release_preflight(
    session: &sifr_package::PackageSession,
    selection: &sifr_package::CargoPackageSelection,
    allow_dirty: bool,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<(), i32> {
    let Some(context) = load_package_graph_context(session, lock_mode, diagnostic_format)? else {
        return Ok(());
    };
    let package_ids = selected_release_package_ids(&context, session, selection)
        .map_err(|diagnostics| render_package_diagnostics(diagnostics, diagnostic_format))?;
    if let Err(diagnostics) = sifr_package::validate_backend_trust(&context.graph) {
        return Err(render_package_diagnostics(diagnostics, diagnostic_format));
    }
    let mut diagnostics = Vec::new();
    for package_id in package_ids {
        let Some(package) = context.graph.packages.get(&package_id) else {
            continue;
        };
        let entries = cargo_package_list_entries(
            session,
            lock_mode,
            &package.cargo_package_name,
            allow_dirty,
            diagnostic_format,
        )?;
        if let Err(errors) =
            sifr_package::validate_package_archive(package, &context.source_map, &entries)
        {
            diagnostics.extend(errors);
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(render_package_diagnostics(diagnostics, diagnostic_format))
    }
}

fn selected_release_package_ids(
    context: &PackageGraphContext,
    session: &sifr_package::PackageSession,
    selection: &sifr_package::CargoPackageSelection,
) -> Result<BTreeSet<sifr_package::SifrPackageId>, Vec<sifr_package::PackageDiagnostic>> {
    let mut selected = BTreeSet::new();
    if selection.workspace {
        selected.extend(
            sifr_package::select_sifr_workspace_members(&context.metadata, &context.graph)?
                .selected_sifr_packages,
        );
    }
    if !selection.packages.is_empty() {
        selected.extend(
            sifr_package::explicit_package_selection(
                &context.metadata,
                &context.graph,
                &selection.packages,
            )?
            .selected_sifr_packages,
        );
    }
    if !selection.workspace && selection.packages.is_empty() {
        selected.extend(current_session_package_id(session, &context.graph));
    }
    for excluded in &selection.excludes {
        selected.retain(|package_id| {
            context
                .graph
                .packages
                .get(package_id)
                .is_none_or(|package| {
                    package.cargo_package_name != *excluded
                        && package.package_id.0 != *excluded
                        && package.sifr_name.0 != *excluded
                })
        });
    }
    Ok(selected)
}

fn current_session_package_id(
    session: &sifr_package::PackageSession,
    graph: &sifr_package::SifrPackageGraph,
) -> Option<sifr_package::SifrPackageId> {
    let manifest_path = session.manifest_path.as_ref()?;
    graph
        .packages
        .values()
        .find(|package| paths_equal(&package.sifr_manifest, manifest_path))
        .map(|package| package.package_id.clone())
}

fn cargo_package_list_entries(
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    cargo_package_name: &str,
    allow_dirty: bool,
    diagnostic_format: DiagnosticFormat,
) -> Result<Vec<sifr_package::PackageArchiveEntry>, i32> {
    let selection = sifr_package::CargoPackageSelection {
        workspace: false,
        packages: vec![cargo_package_name.to_string()],
        excludes: Vec::new(),
    };
    let options = sifr_package::CargoPackageArchiveOptions {
        list: true,
        allow_dirty,
        ..sifr_package::CargoPackageArchiveOptions::default()
    };
    let plan = sifr_package::CargoCommandPlan::package_with_options(
        session.workspace_root.clone(),
        lock_mode,
        &sifr_package::CargoFeatureSelection::default(),
        &selection,
        &options,
    );
    let output = match std::process::Command::new(&plan.program)
        .args(&plan.args)
        .current_dir(&plan.current_dir)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            let diagnostic = cargo_failure_diagnostic(&plan, lock_mode, None, &error.to_string());
            render_diagnostics(&[diagnostic], diagnostic_format);
            return Err(EXIT_USAGE_OR_CONFIG);
        }
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let excerpt = if stderr.trim().is_empty() {
            stdout.as_ref()
        } else {
            stderr.as_ref()
        };
        let diagnostic = cargo_failure_diagnostic(
            &plan,
            lock_mode,
            output.status.code(),
            &bounded_excerpt(excerpt),
        );
        render_diagnostics(&[diagnostic], diagnostic_format);
        return Err(EXIT_USER_DIAGNOSTIC);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| sifr_package::PackageArchiveEntry {
            relative_path: PathBuf::from(line),
        })
        .collect())
}

fn render_package_diagnostics(
    diagnostics: Vec<sifr_package::PackageDiagnostic>,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let diagnostics = diagnostics
        .into_iter()
        .map(package_diagnostic)
        .collect::<Vec<_>>();
    render_diagnostics(&diagnostics, diagnostic_format)
}

fn execute_cargo_plan(
    plan: &sifr_package::CargoCommandPlan,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let output = match std::process::Command::new(&plan.program)
        .args(&plan.args)
        .current_dir(&plan.current_dir)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            let diagnostic = cargo_failure_diagnostic(plan, lock_mode, None, &error.to_string());
            render_diagnostics(&[diagnostic], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    if output.status.success() {
        let _ = io::stdout().write_all(&output.stdout);
        let _ = io::stderr().write_all(&output.stderr);
        return EXIT_SUCCESS;
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let excerpt = if stderr.trim().is_empty() {
        stdout.as_ref()
    } else {
        stderr.as_ref()
    };
    let diagnostic = cargo_failure_diagnostic(
        plan,
        lock_mode,
        output.status.code(),
        &bounded_excerpt(excerpt),
    );
    render_diagnostics(&[diagnostic], diagnostic_format);
    EXIT_USER_DIAGNOSTIC
}

fn cargo_failure_diagnostic(
    plan: &sifr_package::CargoCommandPlan,
    lock_mode: sifr_package::CargoLockMode,
    exit_status: Option<i32>,
    excerpt: &str,
) -> RenderedDiagnostic {
    let stderr_redacted = sifr_package::cargo::errors::redact_cargo_stderr(excerpt);
    let package = sifr_package::map_cargo_failure(plan.action, &stderr_redacted);
    let mut diagnostic = package_diagnostic(package);
    diagnostic.args.insert(
        "action".to_string(),
        DiagnosticArg::String(plan.action.as_str().to_string()),
    );
    diagnostic.args.insert(
        "current_dir".to_string(),
        DiagnosticArg::String(plan.current_dir.display().to_string()),
    );
    diagnostic.args.insert(
        "args_redacted".to_string(),
        DiagnosticArg::String(redacted_args(&plan.args).join(" ")),
    );
    diagnostic.args.insert(
        "lock_mode".to_string(),
        DiagnosticArg::String(format!("{lock_mode:?}").to_ascii_lowercase()),
    );
    diagnostic.args.insert(
        "network_mode".to_string(),
        DiagnosticArg::String(if lock_mode.is_network_disallowed() {
            "offline".to_string()
        } else {
            "online".to_string()
        }),
    );
    diagnostic.args.insert(
        "stderr_redacted".to_string(),
        DiagnosticArg::String(stderr_redacted),
    );
    if let Some(status) = exit_status {
        diagnostic.args.insert(
            "exit_status".to_string(),
            DiagnosticArg::String(status.to_string()),
        );
    }
    diagnostic
}

fn redacted_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| sifr_package::cargo::errors::redact_cargo_stderr(arg))
        .collect()
}

fn bounded_excerpt(text: &str) -> String {
    const MAX_LINES: usize = 12;
    const MAX_BYTES: usize = 4096;
    let mut excerpt = text.lines().take(MAX_LINES).collect::<Vec<_>>().join("\n");
    if excerpt.len() > MAX_BYTES {
        excerpt.truncate(MAX_BYTES);
    }
    excerpt
}

fn cmd_check(
    file: Option<&Path>,
    message_format: Option<&str>,
    selection: &sifr_package::CargoPackageSelection,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let session = match package_session_for_cwd(lock_mode) {
        Ok(session) => session,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let mut plan = match session.plan_check(
        file,
        &sifr_package::CargoFeatureSelection::default(),
        selection,
    ) {
        Ok(plan) => plan,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USER_DIAGNOSTIC;
        }
    };
    if let Some(mut cargo) = plan.cargo.take() {
        if let Some(format) = message_format {
            cargo.extend_forwarded_args(&["--message-format".to_string(), format.to_string()]);
        }
        return execute_cargo_plan(&cargo, lock_mode, diagnostic_format);
    }
    if let Some(sifr_package::ResolvedRunTarget::File(path)) = plan.run_target {
        if !session.manifest_less_mode {
            return cmd_check_package_file(&path, &session, lock_mode, diagnostic_format);
        }
        return cmd_check_file(&path, diagnostic_format);
    }
    EXIT_SUCCESS
}

fn cmd_check_file(file: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
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

fn cmd_check_package_file(
    file: &Path,
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let context = match package_compiler_context(session, lock_mode, diagnostic_format) {
        Ok(Some(context)) => context,
        Ok(None) => return cmd_check_file(file, diagnostic_format),
        Err(exit_code) => return exit_code,
    };
    let entrypoint = PackageEntrypoint {
        main_file: file.to_path_buf(),
        package_id: context.package_id,
        graph: context.graph,
        source_map: context.source_map,
    };
    let errors = match run_with_panic_boundary(
        "internal compiler panic during package check command execution",
        || check_package_project(&entrypoint),
    ) {
        Ok(errors) => errors,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    if errors.is_empty() {
        emit_success_message(diagnostic_format, "no errors found");
        EXIT_SUCCESS
    } else {
        render_diagnostics(&errors, diagnostic_format)
    }
}

fn package_compiler_context(
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<Option<PackageCompilerContext>, i32> {
    let Some(context) = load_package_graph_context(session, lock_mode, diagnostic_format)? else {
        return Ok(None);
    };
    let Some(package_id) = current_session_package_id(session, &context.graph) else {
        return Ok(None);
    };
    Ok(Some(PackageCompilerContext {
        graph: context.graph,
        source_map: context.source_map,
        package_id,
    }))
}

fn load_package_graph_context(
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<Option<PackageGraphContext>, i32> {
    if session.manifest_less_mode {
        return Ok(None);
    }
    let metadata_plan =
        sifr_package::CargoCommandPlan::metadata(session.workspace_root.clone(), lock_mode);
    let output = match std::process::Command::new(&metadata_plan.program)
        .args(&metadata_plan.args)
        .current_dir(&metadata_plan.current_dir)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            let diagnostic =
                cargo_failure_diagnostic(&metadata_plan, lock_mode, None, &error.to_string());
            render_diagnostics(&[diagnostic], diagnostic_format);
            return Err(EXIT_USAGE_OR_CONFIG);
        }
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let excerpt = if stderr.trim().is_empty() {
            stdout.as_ref()
        } else {
            stderr.as_ref()
        };
        let diagnostic = cargo_failure_diagnostic(
            &metadata_plan,
            lock_mode,
            output.status.code(),
            &bounded_excerpt(excerpt),
        );
        render_diagnostics(&[diagnostic], diagnostic_format);
        return Err(EXIT_USER_DIAGNOSTIC);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let metadata = match sifr_package::parse_metadata_json(&stdout) {
        Ok(metadata) => metadata,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return Err(EXIT_USAGE_OR_CONFIG);
        }
    };
    let normalized = metadata.clone().normalize();
    let graph = match sifr_package::derive_package_graph(metadata) {
        Ok(graph) => graph,
        Err(errors) => {
            let diagnostics = errors
                .into_iter()
                .map(package_diagnostic)
                .collect::<Vec<_>>();
            render_diagnostics(&diagnostics, diagnostic_format);
            return Err(EXIT_USER_DIAGNOSTIC);
        }
    };
    let source_map = match sifr_package::PackageSourceMap::build(&graph) {
        Ok(source_map) => source_map,
        Err(errors) => {
            let diagnostics = errors
                .into_iter()
                .map(package_diagnostic)
                .collect::<Vec<_>>();
            render_diagnostics(&diagnostics, diagnostic_format);
            return Err(EXIT_USER_DIAGNOSTIC);
        }
    };
    Ok(Some(PackageGraphContext {
        metadata: normalized,
        graph,
        source_map,
    }))
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    let left = left.canonicalize().unwrap_or_else(|_| left.to_path_buf());
    let right = right.canonicalize().unwrap_or_else(|_| right.to_path_buf());
    left == right
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
    use std::sync::{Mutex, MutexGuard, OnceLock};

    static CWD_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct CurrentDirGuard {
        previous: PathBuf,
        _lock: MutexGuard<'static, ()>,
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.previous).expect("restore cwd");
        }
    }

    fn enter_test_cwd(path: &Path) -> CurrentDirGuard {
        let lock = CWD_TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("cwd test lock should not be poisoned");
        let previous = std::env::current_dir().expect("cwd exists");
        std::env::set_current_dir(path).expect("chdir to test cwd");
        CurrentDirGuard {
            previous,
            _lock: lock,
        }
    }

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

    fn write_real_sifr_package(
        root: &Path,
        cargo_name: &str,
        sifr_name: &str,
        cargo_dependencies: &str,
    ) {
        std::fs::create_dir_all(root.join("src")).expect("package src dir should exist");
        std::fs::write(root.join("src/lib.rs"), "").expect("pure marker should be written");
        std::fs::write(
            root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{cargo_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n{cargo_dependencies}\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n"
            ),
        )
        .expect("cargo manifest should be written");
        std::fs::write(
            root.join("sifr.toml"),
            format!(
                "[package]\nname = \"{sifr_name}\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n"
            ),
        )
        .expect("sifr manifest should be written");
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
    fn test_package_cli_init_lib_creates_projection() {
        let dir = mktemp_dir("package_cli_init_lib");
        let package = dir.join("demo_json");

        let exit = cmd_init(
            &package,
            true,
            false,
            Some("demo_json"),
            false,
            DiagnosticFormat::Compact,
        );

        assert_eq!(exit, EXIT_SUCCESS);
        assert!(package.join("sifr.toml").is_file());
        assert!(package.join("Cargo.toml").is_file());
        assert!(package.join("src/__init__.sifr").is_file());
        assert!(package.join("src/lib.rs").is_file());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_package_cli_repair_check_reports_projection_drift() {
        let dir = mktemp_dir("package_cli_repair_check");
        let package = dir.join("demo_json");
        assert_eq!(
            cmd_init(
                &package,
                true,
                false,
                Some("demo_json"),
                false,
                DiagnosticFormat::Compact,
            ),
            EXIT_SUCCESS
        );
        std::fs::write(
            package.join("Cargo.toml"),
            "[package]\nname = \"sifr-demo-json\"\n",
        )
        .expect("break projection");
        let exit = {
            let _cwd = enter_test_cwd(&package);
            cmd_repair(true, DiagnosticFormat::Compact)
        };
        assert_eq!(exit, EXIT_USER_DIAGNOSTIC);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_package_cli_parses_run_script_bin_and_app_args() {
        let cli = Cli::try_parse_from([
            "sifr", "run", "--script", "dev", "--locked", "--", "--port", "8080",
        ])
        .expect("run script cli parses");

        let Some(Commands::Run {
            script,
            locked,
            args,
            ..
        }) = cli.command
        else {
            panic!("expected run command");
        };
        assert_eq!(script.as_deref(), Some("dev"));
        assert!(locked);
        assert_eq!(args, ["--port", "8080"]);

        let cli =
            Cli::try_parse_from(["sifr", "run", "--bin", "admin"]).expect("run bin cli parses");
        let Some(Commands::Run { bin, .. }) = cli.command else {
            panic!("expected run command");
        };
        assert_eq!(bin.as_deref(), Some("admin"));
    }

    #[test]
    fn test_package_cli_check_explicit_file_uses_package_imports() {
        let dir = mktemp_dir("package_cli_check_package_imports");
        let app_root = dir.join("app");
        let json_root = dir.join("json");
        write_real_sifr_package(
            &app_root,
            "sifr-demo-app",
            "demo_app",
            "demo_json = { path = \"../json\", package = \"sifr-demo-json\" }\n",
        );
        let cargo_toml = app_root.join("Cargo.toml");
        let cargo_source =
            std::fs::read_to_string(&cargo_toml).expect("app cargo manifest should be readable");
        std::fs::write(
            &cargo_toml,
            cargo_source.replace(
                "[dependencies]",
                "[package.metadata.sifr.aliases]\ndemo_json_v1 = { dependency = \"demo_json\", import = \"demo_json_v1\" }\n\n[dependencies]",
            ),
        )
        .expect("app cargo manifest should be updated with alias");
        write_real_sifr_package(&json_root, "sifr-demo-json", "demo_json", "");
        std::fs::write(
            app_root.join("src/main.sifr"),
            "from demo_json_v1 import parse_json\n\n\
def main():\n    assert parse_json() == 1\n",
        )
        .expect("app source should be written");
        std::fs::write(
            json_root.join("src/__init__.sifr"),
            "from .parse import parse_json\n",
        )
        .expect("json namespace should be written");
        std::fs::write(
            json_root.join("src/parse.sifr"),
            "def parse_json() -> int:\n    return 1\n",
        )
        .expect("json implementation should be written");
        let exit = {
            let _cwd = enter_test_cwd(&app_root);
            cmd_check(
                Some(Path::new("src/main.sifr")),
                None,
                &sifr_package::CargoPackageSelection::default(),
                sifr_package::CargoLockMode::Normal,
                DiagnosticFormat::Compact,
            )
        };
        assert_eq!(exit, EXIT_SUCCESS);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_package_cli_check_explicit_file_falls_back_for_legacy_workspace_manifest() {
        let project = TestProject::new("package_cli_check_legacy_workspace_manifest");
        project.write(
            "Cargo.toml",
            "[workspace]\nmembers = [\"rust_member\"]\nresolver = \"2\"\n",
            "workspace manifest should be written",
        );
        project.write(
            "rust_member/Cargo.toml",
            "[package]\nname = \"rust-member\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            "rust member manifest should be written",
        );
        project.write(
            "rust_member/src/lib.rs",
            "pub fn value() -> i32 { 1 }\n",
            "rust member source should be written",
        );
        project.write(
            "sifr.toml",
            "[package]\nname = \"legacy-workspace\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroots = [\".\"]\n",
            "legacy workspace manifest should be written",
        );
        project.write(
            "helper.sifr",
            "def value() -> int:\n    return 1\n",
            "helper source should be written",
        );
        project.write(
            "main.sifr",
            "from helper import value\n\n\
def main():\n    assert value() == 1\n",
            "main source should be written",
        );
        let exit = {
            let _cwd = enter_test_cwd(&project.dir);
            cmd_check(
                Some(Path::new("main.sifr")),
                None,
                &sifr_package::CargoPackageSelection::default(),
                sifr_package::CargoLockMode::Normal,
                DiagnosticFormat::Compact,
            )
        };
        assert_eq!(exit, EXIT_SUCCESS);
    }

    #[test]
    fn test_package_cli_parses_check_message_format_and_tree_args() {
        let cli = Cli::try_parse_from([
            "sifr",
            "check",
            "--locked",
            "--workspace",
            "-p",
            "demo-app",
            "--exclude",
            "demo-tools",
            "--message-format",
            "json",
        ])
        .expect("check cli parses");
        let Some(Commands::Check {
            message_format,
            locked,
            workspace,
            packages,
            exclude,
            ..
        }) = cli.command
        else {
            panic!("expected check command");
        };
        assert_eq!(message_format.as_deref(), Some("json"));
        assert!(locked);
        assert!(workspace);
        assert_eq!(packages, ["demo-app"]);
        assert_eq!(exclude, ["demo-tools"]);

        let cli = Cli::try_parse_from(["sifr", "tree", "--offline", "--depth", "1"])
            .expect("tree cli parses");
        let Some(Commands::Tree { offline, args, .. }) = cli.command else {
            panic!("expected tree command");
        };
        assert!(offline);
        assert_eq!(args, ["--depth", "1"]);

        let cli = Cli::try_parse_from([
            "sifr",
            "package",
            "--workspace",
            "-p",
            "demo-app",
            "--exclude",
            "demo-tools",
            "--list",
            "--no-verify",
            "--no-metadata",
            "--allow-dirty",
            "--exclude-lockfile",
            "--frozen",
        ])
        .expect("package cli parses");
        let Some(Commands::Package {
            workspace,
            packages,
            exclude,
            list,
            no_verify,
            no_metadata,
            allow_dirty,
            exclude_lockfile,
            frozen,
            ..
        }) = cli.command
        else {
            panic!("expected package command");
        };
        assert!(workspace);
        assert_eq!(packages, ["demo-app"]);
        assert_eq!(exclude, ["demo-tools"]);
        assert!(list);
        assert!(no_verify);
        assert!(no_metadata);
        assert!(allow_dirty);
        assert!(exclude_lockfile);
        assert!(frozen);

        let cli = Cli::try_parse_from([
            "sifr",
            "publish",
            "--dry-run",
            "-p",
            "demo-app",
            "--no-verify",
            "--allow-dirty",
            "--locked",
        ])
        .expect("publish cli parses");
        let Some(Commands::Publish {
            dry_run,
            packages,
            no_verify,
            allow_dirty,
            locked,
            ..
        }) = cli.command
        else {
            panic!("expected publish command");
        };
        assert!(dry_run);
        assert_eq!(packages, ["demo-app"]);
        assert!(no_verify);
        assert!(allow_dirty);
        assert!(locked);

        let cli = Cli::try_parse_from([
            "sifr",
            "vendor",
            "third_party/vendor",
            "--sync",
            "member/Cargo.toml",
            "--no-delete",
            "--respect-source-config",
            "--versioned-dirs",
            "--offline",
        ])
        .expect("vendor cli parses");
        let Some(Commands::Vendor {
            path,
            sync,
            no_delete,
            respect_source_config,
            versioned_dirs,
            offline,
            ..
        }) = cli.command
        else {
            panic!("expected vendor command");
        };
        assert_eq!(path, PathBuf::from("third_party/vendor"));
        assert_eq!(sync, [PathBuf::from("member/Cargo.toml")]);
        assert!(no_delete);
        assert!(respect_source_config);
        assert!(versioned_dirs);
        assert!(offline);
    }

    #[test]
    fn test_package_cli_explain_retired_credential_code() {
        let text =
            diagnostic_explanation("SIFR-PACKAGE-0105").expect("retired code explanation exists");
        assert!(text.contains("retired"));
        assert!(text.contains("SIFR-PACKAGE-0101"));
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
