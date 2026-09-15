// Independent eager schema retained to certify lazy command construction.
use crate::cli_model_and_entrypoint::{
    BridgeCommands, DiagnosticFormat, SIFR_BUILD_VERSION, ToolsCommands,
};
use crate::formatter_cli::FmtArgs;
use crate::lint_cli::LintArgs;
use crate::python_cli::PythonArgs;
use crate::self_update_cli::SelfArgs;
use crate::sysroot_cli::PrintKind;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "sifr",
    version = SIFR_BUILD_VERSION,
    about = "The Sifr programming language compiler"
)]
struct Cli {
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
enum Commands {
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
        /// Require Cargo.lock to be unchanged
        #[arg(long)]
        locked: bool,
        /// Disable network access
        #[arg(long)]
        offline: bool,
        /// Combine --locked and --offline
        #[arg(long)]
        frozen: bool,
        /// Write the generated Cargo project without compiling it
        #[arg(long, hide = true)]
        materialize_only: bool,
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
    /// Manage declaration-first Python interop evidence
    Python(PythonArgs),
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
    /// Manage package-provided host tools
    Tools {
        #[command(subcommand)]
        command: ToolsCommands,
    },
    /// Manage a standalone Sifr installation
    #[command(name = "self")]
    SelfCommand(SelfArgs),
    /// Execute a package-provided host tool namespace
    #[command(external_subcommand)]
    HostTool(Vec<String>),
}

use clap::{CommandFactory, FromArgMatches};

fn equivalent(args: &[&str], update: bool) {
    let eager = if update {
        Cli::command_for_update()
    } else {
        Cli::command()
    };
    let deferred = if update {
        crate::cli_model_and_entrypoint::Cli::command_for_update()
    } else {
        crate::cli_model_and_entrypoint::Cli::command()
    };
    match (
        eager.try_get_matches_from(args),
        deferred.try_get_matches_from(args),
    ) {
        (Ok(mut before), Ok(mut after)) => {
            assert_eq!(before, after, "{args:?}, update={update}");
            if !update {
                assert!(Cli::from_arg_matches_mut(&mut before).is_ok(), "{args:?}");
                assert!(
                    crate::cli_model_and_entrypoint::Cli::from_arg_matches_mut(&mut after).is_ok(),
                    "{args:?}"
                );
                assert_eq!(before, after, "consumed: {args:?}");
            }
        }
        (Err(before), Err(after)) => {
            assert_eq!(before.kind(), after.kind(), "{args:?}, update={update}");
            assert_eq!(
                before.to_string(),
                after.to_string(),
                "{args:?}, update={update}"
            );
            assert_eq!(before.exit_code(), after.exit_code(), "{args:?}");
        }
        (before, after) => panic!("parser mismatch {args:?}: {before:?}, {after:?}"),
    }
}

#[test]
fn all_command_schemas_keep_eager_help_errors_groups_defaults_and_global_order() {
    for command in [
        "build", "run", "fetch", "doctor", "init", "repair", "bridge", "python", "check", "tree",
        "package", "publish", "vendor", "fmt", "lint", "lsp", "trace", "emit", "test", "tools",
        "self",
    ] {
        for update in [false, true] {
            for suffix in [
                vec![],
                vec!["--help"],
                vec!["--unknown"],
                vec!["--config", "a.toml", "--isolated"],
            ] {
                let mut args = vec!["sifr", command];
                args.extend(suffix);
                equivalent(&args, update);
            }
            equivalent(
                &[
                    "sifr", "--config", "a.toml", command, "--config", "b.toml", "--help",
                ],
                update,
            );
        }
    }
    for args in [
        &["sifr", "--help"][..],
        &["sifr", "--version"],
        &["sifr", "--print", "sysroot", "--json"],
        &["sifr", "--json"],
        &[
            "sifr",
            "build",
            "main.sifr",
            "-o",
            "out",
            "--locked",
            "--offline",
            "--quiet",
            "--materialize-only",
        ],
        &[
            "sifr",
            "run",
            "main.sifr",
            "--",
            "--application-flag",
            "value",
        ],
        &[
            "sifr",
            "run",
            "-p",
            "one",
            "--package",
            "two",
            "--bin",
            "app",
        ],
        &["sifr", "init", "project", "--lib", "--bin"],
        &["sifr", "init", "--lib", "--force", "--name", "project"],
        &["sifr", "doctor", "--json"],
        &["sifr", "repair", "--check"],
        &["sifr", "bridge", "check", "--workspace", "--locked"],
        &[
            "sifr",
            "check",
            "--workspace",
            "-p",
            "one",
            "--exclude",
            "two",
            "--message-format",
            "json",
            "--frozen",
        ],
        &["sifr", "tree", "--offline", "--", "--prefix", "depth"],
        &[
            "sifr",
            "package",
            "--workspace",
            "--list",
            "--no-verify",
            "--allow-dirty",
            "--exclude-lockfile",
        ],
        &[
            "sifr",
            "publish",
            "--dry-run",
            "-p",
            "one",
            "--exclude",
            "two",
            "--offline",
        ],
        &[
            "sifr",
            "vendor",
            "vendor-out",
            "--sync",
            "a.toml",
            "--no-delete",
            "--respect-source-config",
            "--versioned-dirs",
        ],
        &["sifr", "lsp", "--stdio", "--parent-pid", "42"],
        &["sifr", "lsp", "--parent-pid", "not-a-pid"],
        &["sifr", "trace", "main.sifr"],
        &["sifr", "emit", "main.sifr"],
        &["sifr", "test", "tests"],
        &["sifr", "tools", "lock", "--check"],
        &["sifr", "tools", "lock", "--help"],
        &["sifr", "bridge", "check", "--help"],
        &["sifr", "external-tool", "--arbitrary"],
        &[
            "sifr",
            "--config",
            "a.toml",
            "build",
            "main.sifr",
            "--config",
            "b.toml",
            "--isolated",
        ],
    ] {
        equivalent(args, false);
        equivalent(args, true);
    }
}
