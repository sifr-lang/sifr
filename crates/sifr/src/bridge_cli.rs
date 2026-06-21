use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum BridgeCommands {
    /// Check Rust bridge projections and interop probe diagnostics
    Check {
        /// Check all Sifr-capable workspace members through Cargo-compatible selection
        #[arg(long)]
        workspace: bool,
        /// Select one package by Cargo package spec or unambiguous package name
        #[arg(short = 'p', long = "package")]
        packages: Vec<String>,
        /// Exclude one package from workspace selection
        #[arg(long)]
        exclude: Vec<String>,
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
}
