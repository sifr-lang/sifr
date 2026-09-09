pub(crate) use super::bridge_cli::BridgeCommands;
use super::check_and_package_commands::{cmd_check, cmd_emit, cmd_fmt, cmd_test};
use super::cli_lock_modes::lock_mode_from_flags;
use super::deferred_cli_args::DeferredArgs;
use super::diagnostic_rendering_and_run::{
    RunCommandOptions, cmd_build, cmd_fetch, cmd_package, cmd_publish, cmd_run_with_options,
    cmd_tree, cmd_vendor, render_diagnostics,
};
use super::explain_cli::cmd_explain;
use super::formatter_cli::FmtArgs;
use super::host_tool_cli::{cmd_host_tool, cmd_host_tools_lock};
use super::lint_cli::{LintArgs, cmd_lint};
use super::python_cli::{PythonArgs, cmd_python};
use super::self_update_cli::{SelfArgs, cmd_self};
use super::sysroot_cli::{PrintKind, cmd_doctor, cmd_print};
use super::trace_cli::cmd_trace;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic, Severity};
use sifr_driver::find_workspace_root;
use sifr_frontend::{DiskSourceProvider, SourceProvider};
use std::collections::BTreeMap;
use std::io::{self, Write as _};
use std::panic::{AssertUnwindSafe, catch_unwind};
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
    Build(DeferredArgs<super::command_args::Build>),
    /// Compile and run a .sifr file
    Run(DeferredArgs<super::command_args::Run>),
    /// Fetch package dependencies
    Fetch(DeferredArgs<super::command_args::Fetch>),
    /// Inspect the resolved Sifr sysroot and install health
    Doctor(DeferredArgs<super::command_args::Doctor>),
    /// Create a new Sifr package
    Init(DeferredArgs<super::command_args::Init>),
    /// Repair Sifr-managed Cargo projection drift
    Repair(DeferredArgs<super::command_args::Repair>),
    /// Validate Rust bridge projections and interop probes for a package
    Bridge(DeferredArgs<super::command_args::Bridge>),
    /// Manage declaration-first Python interop evidence
    Python(DeferredArgs<PythonArgs>),
    /// Type-check a .sifr file without compiling
    Check(DeferredArgs<super::command_args::Check>),
    /// Show the package dependency tree
    Tree(DeferredArgs<super::command_args::Tree>),
    /// Assemble and verify a Cargo package archive for a Sifr package
    Package(DeferredArgs<super::command_args::Package>),
    /// Publish a Sifr package through Cargo
    Publish(DeferredArgs<super::command_args::Publish>),
    /// Vendor dependency sources through Cargo
    Vendor(DeferredArgs<super::command_args::Vendor>),
    /// Format Sifr source files
    Fmt(DeferredArgs<FmtArgs>),
    /// Run suppressible policy diagnostics
    Lint(DeferredArgs<LintArgs>),
    /// Run the native Sifr Language Server Protocol server
    Lsp(DeferredArgs<super::command_args::Lsp>),
    /// Print deterministic compiler-service trace and status output
    Trace(DeferredArgs<super::command_args::Trace>),
    /// Show the generated Rust source code
    Emit(DeferredArgs<super::command_args::Emit>),
    /// Run tests in a directory
    Test(DeferredArgs<super::command_args::Test>),
    /// Manage package-provided host tools
    Tools(DeferredArgs<super::command_args::Tools>),
    /// Manage a standalone Sifr installation
    #[command(name = "self")]
    SelfCommand(DeferredArgs<SelfArgs>),
    /// Execute a package-provided host tool namespace
    #[command(external_subcommand)]
    HostTool(Vec<String>),
}

#[derive(Subcommand)]
pub(crate) enum ToolsCommands {
    /// Write or verify the committed host-tool lock artifact
    Lock {
        /// Verify the artifact without changing it
        #[arg(long)]
        check: bool,
    },
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
        Commands::Build(crate::deferred_cli_args::DeferredArgs(crate::command_args::Build {
            file,
            output,
            quiet,
            locked,
            offline,
            frozen,
            materialize_only,
        })) => cmd_build(
            &file,
            &output,
            lock_mode_from_flags(locked, offline, frozen),
            quiet,
            diagnostic_format,
            materialize_only,
        ),
        Commands::Run(crate::deferred_cli_args::DeferredArgs(crate::command_args::Run {
            target,
            packages,
            bin,
            script,
            locked,
            offline,
            frozen,
            quiet,
            args,
        })) => {
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
        Commands::Fetch(crate::deferred_cli_args::DeferredArgs(crate::command_args::Fetch {
            locked,
            offline,
            frozen,
        })) => cmd_fetch(
            lock_mode_from_flags(locked, offline, frozen),
            diagnostic_format,
        ),
        Commands::Doctor(crate::deferred_cli_args::DeferredArgs(crate::command_args::Doctor {
            json,
        })) => cmd_doctor(json, diagnostic_format),
        Commands::Init(crate::deferred_cli_args::DeferredArgs(crate::command_args::Init {
            path,
            lib,
            bin,
            name,
            force,
        })) => cmd_init(&path, lib, bin, name.as_deref(), force, diagnostic_format),
        Commands::Repair(crate::deferred_cli_args::DeferredArgs(crate::command_args::Repair {
            check,
        })) => cmd_repair(check, diagnostic_format),
        Commands::Bridge(crate::deferred_cli_args::DeferredArgs(crate::command_args::Bridge {
            command,
        })) => match command {
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
        Commands::Python(args) => cmd_python(args.0, diagnostic_format),
        Commands::Check(crate::deferred_cli_args::DeferredArgs(crate::command_args::Check {
            path,
            workspace,
            packages,
            exclude,
            message_format,
            locked,
            offline,
            frozen,
        })) => {
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
        Commands::Tree(crate::deferred_cli_args::DeferredArgs(crate::command_args::Tree {
            locked,
            offline,
            frozen,
            args,
        })) => cmd_tree(
            lock_mode_from_flags(locked, offline, frozen),
            &args,
            diagnostic_format,
        ),
        Commands::Package(crate::deferred_cli_args::DeferredArgs(
            crate::command_args::Package {
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
            },
        )) => {
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
        Commands::Publish(crate::deferred_cli_args::DeferredArgs(
            crate::command_args::Publish {
                dry_run,
                workspace,
                packages,
                exclude,
                no_verify,
                allow_dirty,
                locked,
                offline,
                frozen,
            },
        )) => {
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
        Commands::Vendor(crate::deferred_cli_args::DeferredArgs(crate::command_args::Vendor {
            path,
            sync,
            no_delete,
            respect_source_config,
            versioned_dirs,
            locked,
            offline,
            frozen,
        })) => {
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
        Commands::Fmt(args) => cmd_fmt(&args.0, &config, isolated, diagnostic_format),
        Commands::Lint(args) => cmd_lint(&args.0, &config, isolated, diagnostic_format),
        Commands::Lsp(crate::deferred_cli_args::DeferredArgs(crate::command_args::Lsp {
            stdio,
            parent_pid,
        })) => cmd_lsp(stdio, parent_pid),
        Commands::Trace(crate::deferred_cli_args::DeferredArgs(crate::command_args::Trace {
            file,
        })) => cmd_trace(&file, diagnostic_format),
        Commands::Emit(crate::deferred_cli_args::DeferredArgs(crate::command_args::Emit {
            file,
        })) => cmd_emit(&file, diagnostic_format),
        Commands::Test(crate::deferred_cli_args::DeferredArgs(crate::command_args::Test {
            dir,
        })) => cmd_test(&dir, diagnostic_format),
        Commands::Tools(crate::deferred_cli_args::DeferredArgs(crate::command_args::Tools {
            command,
        })) => match command {
            ToolsCommands::Lock { check } => cmd_host_tools_lock(check, diagnostic_format),
        },
        Commands::SelfCommand(args) => cmd_self(&args.0, diagnostic_format),
        Commands::HostTool(words) => cmd_host_tool(&words, diagnostic_format),
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
    let mut provider = DiskSourceProvider::new();
    let repair = sifr_package::repair_projection(&root, check, &mut provider);
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
    provider: &mut dyn SourceProvider,
) -> Result<CompilationMode, Vec<RenderedDiagnostic>> {
    if find_workspace_root(file, provider)?.is_some() {
        Ok(CompilationMode::Project)
    } else {
        Ok(CompilationMode::SingleFile)
    }
}

pub(super) fn read_source(file: &Path, provider: &mut dyn SourceProvider) -> String {
    match provider.read_file(file) {
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
