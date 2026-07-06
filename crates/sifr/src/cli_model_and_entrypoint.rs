pub(crate) use super::bridge_cli::BridgeCommands;
use super::check_and_package_commands::{cmd_check, cmd_emit, cmd_fmt, cmd_test};
use super::diagnostic_rendering_and_run::{
    cmd_build, cmd_fetch, cmd_package, cmd_publish, cmd_run_with_options, cmd_tree, cmd_vendor,
    render_diagnostics, RunCommandOptions,
};
use super::explain_cli::cmd_explain;
use super::formatter_cli::FmtArgs;
use super::lint_cli::{cmd_lint, LintArgs};
use super::self_update_cli::{cmd_self, SelfArgs};
use super::sysroot_cli::{cmd_doctor, cmd_print, PrintKind};
use super::trace_cli::cmd_trace;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic, Severity};
#[cfg(test)]
use sifr_driver::diagnostic_label_for_code_str;
use sifr_driver::find_workspace_root;
use sifr_frontend::{DiskSourceProvider, SourceProvider};
use sifr_python_ast::Stmt;
use sifr_syntax::parse_module_suite;
use std::collections::BTreeMap;
use std::io::{self, Write as _};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process;

pub(super) const SIFR_BUILD_VERSION: &str = env!("SIFR_BUILD_VERSION");

#[derive(Parser)]
#[command(
    name = "sifr",
    version = SIFR_BUILD_VERSION,
    about = "The Sifr programming language compiler"
)]
pub(crate) struct Cli {
    /// Diagnostic output format
    #[arg(long, value_enum, default_value_t = DiagnosticFormat::Human)]
    pub(crate) diagnostic_format: DiagnosticFormat,

    /// Explain a Sifr diagnostic code without running a package operation
    #[arg(long)]
    pub(crate) explain: Option<String>,

    /// Sifr config file path or KEY=VALUE override
    #[arg(long, global = true)]
    pub(crate) config: Vec<String>,

    /// Ignore Sifr configuration files
    #[arg(long, global = true)]
    pub(crate) isolated: bool,

    /// Developer override for the Sifr sysroot root
    #[arg(long, global = true, hide = true, value_name = "PATH")]
    pub(crate) sysroot: Option<PathBuf>,

    /// Print compiler metadata and exit
    #[arg(long = "print", value_enum)]
    pub(crate) print: Option<PrintKind>,

    /// Print --print output as JSON
    #[arg(long, requires = "print")]
    pub(crate) json: bool,

    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Compile a .sifr file to a native binary
    Build {
        /// Input .sifr file
        file: PathBuf,
        /// Output directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
        /// Suppress build phase details
        #[arg(long)]
        quiet: bool,
    },
    /// Compile and run a .sifr file
    Run {
        /// Input .sifr file, app target, or script name
        target: Option<String>,
        /// Select a workspace package by Cargo package name
        #[arg(short = 'p', long = "package")]
        packages: Vec<String>,
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
        /// Suppress build phase details
        #[arg(long)]
        quiet: bool,
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
    /// Inspect the resolved Sifr sysroot and install health
    Doctor {
        /// Print doctor output as JSON
        #[arg(long)]
        json: bool,
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
    /// Validate Rust bridge projections and interop probes for a package
    Bridge {
        #[command(subcommand)]
        command: BridgeCommands,
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
    Fmt(FmtArgs),
    /// Run suppressible policy diagnostics
    Lint(LintArgs),
    /// Run the native Sifr Language Server Protocol server
    Lsp {
        /// Use stdio transport
        #[arg(long)]
        stdio: bool,
        /// Exit the language server when the parent process is no longer alive
        #[arg(long = "parent-pid")]
        parent_pid: Option<u32>,
    },
    /// Print deterministic compiler-service trace and status output
    Trace {
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
    /// Manage a standalone Sifr installation
    #[command(name = "self")]
    SelfCommand(SelfArgs),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum DiagnosticFormat {
    Human,
    Json,
    Compact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompilationMode {
    SingleFile,
    Project,
}

pub(crate) const EXIT_SUCCESS: i32 = 0;
pub(crate) const EXIT_USER_DIAGNOSTIC: i32 = 1;
pub(crate) const EXIT_USAGE_OR_CONFIG: i32 = 2;
pub(super) const EXIT_INTERNAL_COMPILER_FAILURE: i32 = 3;
pub(super) struct PackageCompilerContext {
    pub(super) graph: sifr_package::SifrPackageGraph,
    pub(super) source_map: sifr_package::PackageSourceMap,
    pub(super) package_id: sifr_package::SifrPackageId,
    pub(super) python_runtime: Option<sifr_driver::PackagePythonRuntime>,
}

pub(super) struct PackageGraphContext {
    pub(super) metadata: sifr_package::NormalizedCargoMetadata,
    pub(super) graph: sifr_package::SifrPackageGraph,
    pub(super) source_map: sifr_package::PackageSourceMap,
}

pub(crate) fn diagnostic_with_code(
    message: impl Into<String>,
    code: DiagnosticCode,
) -> RenderedDiagnostic {
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

pub(super) fn main() {
    let cli = Cli::parse();
    process::exit(run_cli(cli));
}

fn run_cli(cli: Cli) -> i32 {
    let diagnostic_format = cli.diagnostic_format;
    if let Some(sysroot) = cli.sysroot {
        if let Err(existing) = sifr_sysroot::set_process_sysroot_override(sysroot.clone()) {
            let diagnostic = diagnostic_with_code(
                format!(
                    "Sifr sysroot override was already set to {}; refusing second override {}",
                    existing.display(),
                    sysroot.display()
                ),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            );
            render_diagnostics(&[diagnostic], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    }
    if let Some(code) = cli.explain {
        return cmd_explain(&code, diagnostic_format);
    }
    if let Some(print) = cli.print {
        return cmd_print(print, cli.json, diagnostic_format);
    }
    let Some(command) = cli.command else {
        let mut cli_command = Cli::command();
        let _ = cli_command.write_help(&mut io::stderr());
        return EXIT_USAGE_OR_CONFIG;
    };
    let config = cli.config;
    let isolated = cli.isolated;
    match command {
        Commands::Build {
            file,
            output,
            quiet,
        } => cmd_build(&file, &output, quiet, diagnostic_format),
        Commands::Run {
            target,
            packages,
            bin,
            script,
            locked,
            offline,
            frozen,
            quiet,
            args,
        } => {
            let options = RunCommandOptions {
                target: target.as_deref(),
                bin: bin.as_deref(),
                script: script.as_deref(),
                packages: &packages,
                app_args: &args,
                lock_mode: lock_mode_from_flags(locked, offline, frozen),
                quiet,
                diagnostic_format,
            };
            cmd_run_with_options(&options)
        }
        Commands::Fetch {
            locked,
            offline,
            frozen,
        } => cmd_fetch(
            lock_mode_from_flags(locked, offline, frozen),
            diagnostic_format,
        ),
        Commands::Doctor { json } => cmd_doctor(json, diagnostic_format),
        Commands::Init {
            path,
            lib,
            bin,
            name,
            force,
        } => cmd_init(&path, lib, bin, name.as_deref(), force, diagnostic_format),
        Commands::Repair { check } => cmd_repair(check, diagnostic_format),
        Commands::Bridge { command } => match command {
            BridgeCommands::Check {
                workspace,
                packages,
                exclude,
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
                    None,
                    None,
                    &selection,
                    lock_mode_from_flags(locked, offline, frozen),
                    diagnostic_format,
                )
            }
        },
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
        Commands::Fmt(args) => cmd_fmt(&args, &config, isolated, diagnostic_format),
        Commands::Lint(args) => cmd_lint(&args, &config, isolated, diagnostic_format),
        Commands::Lsp { stdio, parent_pid } => cmd_lsp(stdio, parent_pid),
        Commands::Trace { file } => cmd_trace(&file, diagnostic_format),
        Commands::Emit { file } => cmd_emit(&file, diagnostic_format),
        Commands::Test { dir } => cmd_test(&dir, diagnostic_format),
        Commands::SelfCommand(args) => cmd_self(&args, diagnostic_format),
    }
}

pub(super) fn cmd_init(
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

pub(super) fn cmd_repair(check: bool, diagnostic_format: DiagnosticFormat) -> i32 {
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

pub(super) fn package_diagnostic(
    diagnostic: sifr_package::PackageDiagnostic,
) -> RenderedDiagnostic {
    sifr_driver::render_package_diagnostic(diagnostic)
}

pub(super) fn lock_mode_from_flags(
    locked: bool,
    offline: bool,
    frozen: bool,
) -> sifr_package::CargoLockMode {
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

pub(super) fn cmd_lsp(stdio: bool, parent_pid: Option<u32>) -> i32 {
    if !stdio {
        let diagnostic = diagnostic_with_code(
            "sifr lsp requires --stdio in editor tooling",
            DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
        );
        render_diagnostics(&[diagnostic], DiagnosticFormat::Human);
        return EXIT_USAGE_OR_CONFIG;
    }
    match sifr_lsp::run_stdio_with_options(sifr_lsp::LspServerOptions { parent_pid }) {
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

pub(super) fn resolve_compilation_mode(
    file: &Path,
) -> Result<CompilationMode, Vec<RenderedDiagnostic>> {
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

pub(super) fn has_local_project_imports(file: &Path) -> bool {
    let Some(parent) = file.parent() else {
        return false;
    };
    let mut provider = DiskSourceProvider::new();
    let Ok(source) = provider.read_file(file) else {
        return false;
    };
    let suite = match parse_module_suite(source.as_str(), Some(&file.display().to_string())) {
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
        provider.is_file(&parent.join(format!("{module_name}.sifr")))
    })
}

pub(super) fn read_source(file: &Path) -> String {
    match DiskSourceProvider::new().read_file(file) {
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
    .as_str()
    .to_string()
}

#[cfg(test)]
pub(crate) struct InvocationWorkspace {
    path: PathBuf,
}

#[cfg(test)]
impl InvocationWorkspace {
    pub(crate) fn create(prefix: &str) -> io::Result<Self> {
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

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
impl Drop for InvocationWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub(super) fn panic_payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(msg) = payload.downcast_ref::<&str>() {
        return (*msg).to_string();
    }
    if let Some(msg) = payload.downcast_ref::<String>() {
        return msg.clone();
    }
    "non-string panic payload".to_string()
}

pub(super) fn run_with_panic_boundary<T>(
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

pub(super) fn is_internal_diagnostic(error: &RenderedDiagnostic) -> bool {
    error.code == DiagnosticCode::INTERNAL_COMPILER_PANIC.code()
}

pub(super) fn diagnostic_exit_code(errors: &[RenderedDiagnostic]) -> i32 {
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
pub(super) fn legacy_diagnostic_display(diagnostic: &RenderedDiagnostic) -> String {
    format!("{}: {}", human_label(diagnostic), diagnostic.message)
}

#[cfg(test)]
pub(super) fn human_label(diagnostic: &RenderedDiagnostic) -> &'static str {
    match diagnostic.severity {
        Severity::Error if diagnostic.code.starts_with("SIFR-") => {
            diagnostic_label_for_code_str(&diagnostic.code)
        }
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
    }
}
