use clap::Args;
use std::path::PathBuf;

#[derive(Args, Clone, Debug)]
pub(crate) struct FmtArgs {
    /// Check formatting without writing changes
    #[arg(long)]
    pub(crate) check: bool,

    /// Print a unified diff without writing changes
    #[arg(long, conflicts_with = "check")]
    pub(crate) diff: bool,

    /// Filename context for formatting stdin
    #[arg(long)]
    pub(crate) stdin_filename: Option<PathBuf>,

    /// Byte range to format as START:END
    #[arg(long)]
    pub(crate) range: Option<String>,

    /// Preferred formatter line length
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    pub(crate) line_length: Option<u16>,

    /// Enable preview formatting behavior
    #[arg(long, conflicts_with = "no_preview")]
    pub(crate) preview: bool,

    /// Disable preview formatting behavior
    #[arg(long)]
    pub(crate) no_preview: bool,

    /// Exclude paths matching a formatter pattern
    #[arg(long, value_delimiter = ',')]
    pub(crate) exclude: Vec<String>,

    /// Respect VCS ignore files
    #[arg(long, default_value_t = true, conflicts_with = "no_respect_gitignore")]
    pub(crate) respect_gitignore: bool,

    /// Do not respect VCS ignore files
    #[arg(long)]
    pub(crate) no_respect_gitignore: bool,

    /// Apply excludes to explicit file targets
    #[arg(long, conflicts_with = "no_force_exclude")]
    pub(crate) force_exclude: bool,

    /// Do not apply excludes to explicit file targets
    #[arg(long)]
    pub(crate) no_force_exclude: bool,

    /// Disable formatter cache reads and writes
    #[arg(long)]
    pub(crate) no_cache: bool,

    /// Formatter cache directory
    #[arg(long)]
    pub(crate) cache_dir: Option<PathBuf>,

    /// Input .sifr files or directories; defaults to current directory
    #[arg(value_name = "FILES")]
    pub(crate) paths: Vec<PathBuf>,
}
