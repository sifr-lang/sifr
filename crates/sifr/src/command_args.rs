//! Command-owned argument schemas, constructed only for the selected command.
use crate::cli_model_and_entrypoint::{BridgeCommands, ToolsCommands};
use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct Build {
    /// Input .sifr file
    pub(crate) file: PathBuf,
    /// Output directory (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub(crate) output: PathBuf,
    /// Suppress build phase details
    #[arg(long)]
    pub(crate) quiet: bool,
    /// Require Cargo.lock to be unchanged
    #[arg(long)]
    pub(crate) locked: bool,
    /// Disable network access
    #[arg(long)]
    pub(crate) offline: bool,
    /// Combine --locked and --offline
    #[arg(long)]
    pub(crate) frozen: bool,
    /// Write the generated Cargo project without compiling it
    #[arg(long, hide = true)]
    pub(crate) materialize_only: bool,
}

#[derive(clap::Args)]
pub(crate) struct Run {
    /// Input .sifr file, app target, or script name
    pub(crate) target: Option<String>,
    /// Select a workspace package by Cargo package name
    #[arg(short = 'p', long = "package")]
    pub(crate) packages: Vec<String>,
    /// Select a layout-discovered app target
    #[arg(long)]
    pub(crate) bin: Option<String>,
    /// Select a named package script
    #[arg(long)]
    pub(crate) script: Option<String>,
    /// Require Cargo.lock to be unchanged
    #[arg(long)]
    pub(crate) locked: bool,
    /// Disable network access
    #[arg(long)]
    pub(crate) offline: bool,
    /// Combine --locked and --offline
    #[arg(long)]
    pub(crate) frozen: bool,
    /// Suppress build phase details
    #[arg(long)]
    pub(crate) quiet: bool,
    /// Arguments passed to the selected app after --
    #[arg(last = true)]
    pub(crate) args: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct Fetch {
    /// Require Cargo.lock to be unchanged
    #[arg(long)]
    pub(crate) locked: bool,
    /// Disable network access
    #[arg(long)]
    pub(crate) offline: bool,
    /// Combine --locked and --offline
    #[arg(long)]
    pub(crate) frozen: bool,
}

#[derive(clap::Args)]
pub(crate) struct Doctor {
    /// Print doctor output as JSON
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(clap::Args)]
pub(crate) struct Init {
    /// Target directory
    #[arg(default_value = ".")]
    pub(crate) path: PathBuf,
    /// Create a library package
    #[arg(long, conflicts_with = "bin")]
    pub(crate) lib: bool,
    /// Create an app package
    #[arg(long)]
    pub(crate) bin: bool,
    /// Sifr package name
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// Create missing Sifr-owned files without overwriting existing files
    #[arg(long)]
    pub(crate) force: bool,
}

#[derive(clap::Args)]
pub(crate) struct Repair {
    /// Check projection drift without writing
    #[arg(long)]
    pub(crate) check: bool,
}

#[derive(clap::Args)]
pub(crate) struct Bridge {
    #[command(subcommand)]
    pub(crate) command: BridgeCommands,
}

#[derive(clap::Args)]
pub(crate) struct Check {
    /// Input .sifr file, or omit for package check
    pub(crate) path: Option<PathBuf>,
    /// Check all Sifr-capable workspace members through Cargo-compatible selection
    #[arg(long)]
    pub(crate) workspace: bool,
    /// Select one package by Cargo package spec or unambiguous package name
    #[arg(short = 'p', long = "package")]
    pub(crate) packages: Vec<String>,
    /// Exclude one package from workspace selection
    #[arg(long)]
    pub(crate) exclude: Vec<String>,
    /// Cargo-compatible package message format for package checks
    #[arg(long)]
    pub(crate) message_format: Option<String>,
    /// Require Cargo.lock to be unchanged
    #[arg(long)]
    pub(crate) locked: bool,
    /// Disable network access
    #[arg(long)]
    pub(crate) offline: bool,
    /// Combine --locked and --offline
    #[arg(long)]
    pub(crate) frozen: bool,
}

#[derive(clap::Args)]
pub(crate) struct Tree {
    /// Require Cargo.lock to be unchanged
    #[arg(long)]
    pub(crate) locked: bool,
    /// Disable network access
    #[arg(long)]
    pub(crate) offline: bool,
    /// Combine --locked and --offline
    #[arg(long)]
    pub(crate) frozen: bool,
    /// Cargo-compatible tree options
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub(crate) args: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct Package {
    /// Package all Sifr-capable workspace members through Cargo-compatible selection
    #[arg(long)]
    pub(crate) workspace: bool,
    /// Select one package by Cargo package spec or unambiguous package name
    #[arg(short = 'p', long = "package")]
    pub(crate) packages: Vec<String>,
    /// Exclude one package from workspace selection
    #[arg(long)]
    pub(crate) exclude: Vec<String>,
    /// Print packaged files without creating an archive
    #[arg(long)]
    pub(crate) list: bool,
    /// Skip Cargo's package verification build
    #[arg(long)]
    pub(crate) no_verify: bool,
    /// Skip Cargo package metadata warning checks
    #[arg(long)]
    pub(crate) no_metadata: bool,
    /// Allow dirty working tree contents
    #[arg(long)]
    pub(crate) allow_dirty: bool,
    /// Exclude Cargo.lock from the package archive
    #[arg(long)]
    pub(crate) exclude_lockfile: bool,
    /// Require Cargo.lock to be unchanged
    #[arg(long)]
    pub(crate) locked: bool,
    /// Disable network access
    #[arg(long)]
    pub(crate) offline: bool,
    /// Combine --locked and --offline
    #[arg(long)]
    pub(crate) frozen: bool,
}

#[derive(clap::Args)]
pub(crate) struct Publish {
    /// Validate publish without uploading
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Publish all Sifr-capable workspace members through Cargo-compatible selection
    #[arg(long)]
    pub(crate) workspace: bool,
    /// Select one package by Cargo package spec or unambiguous package name
    #[arg(short = 'p', long = "package")]
    pub(crate) packages: Vec<String>,
    /// Exclude one package from workspace selection
    #[arg(long)]
    pub(crate) exclude: Vec<String>,
    /// Skip Cargo's publish verification build
    #[arg(long)]
    pub(crate) no_verify: bool,
    /// Allow dirty working tree contents
    #[arg(long)]
    pub(crate) allow_dirty: bool,
    /// Require Cargo.lock to be unchanged
    #[arg(long)]
    pub(crate) locked: bool,
    /// Disable network access
    #[arg(long)]
    pub(crate) offline: bool,
    /// Combine --locked and --offline
    #[arg(long)]
    pub(crate) frozen: bool,
}

#[derive(clap::Args)]
pub(crate) struct Vendor {
    /// Output directory for vendored sources
    #[arg(default_value = "vendor")]
    pub(crate) path: PathBuf,
    /// Additional manifest to sync during vendoring
    #[arg(long)]
    pub(crate) sync: Vec<PathBuf>,
    /// Keep stale vendored sources
    #[arg(long)]
    pub(crate) no_delete: bool,
    /// Respect existing Cargo source configuration
    #[arg(long)]
    pub(crate) respect_source_config: bool,
    /// Use versioned vendor directory names
    #[arg(long)]
    pub(crate) versioned_dirs: bool,
    /// Require Cargo.lock to be unchanged
    #[arg(long)]
    pub(crate) locked: bool,
    /// Disable network access
    #[arg(long)]
    pub(crate) offline: bool,
    /// Combine --locked and --offline
    #[arg(long)]
    pub(crate) frozen: bool,
}

#[derive(clap::Args)]
pub(crate) struct Lsp {
    /// Use stdio transport
    #[arg(long)]
    pub(crate) stdio: bool,
    /// Exit the language server when the parent process is no longer alive
    #[arg(long = "parent-pid")]
    pub(crate) parent_pid: Option<u32>,
}

#[derive(clap::Args)]
pub(crate) struct Trace {
    /// Input .sifr file
    pub(crate) file: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct Emit {
    /// Input .sifr file
    pub(crate) file: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct Test {
    /// Directory containing test files (default: current directory)
    #[arg(default_value = ".")]
    pub(crate) dir: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct Tools {
    #[command(subcommand)]
    pub(crate) command: ToolsCommands,
}
