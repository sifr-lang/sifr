use std::io::{self, Write};
#[derive(clap::Args)]
pub(crate) struct CacheArgs {
    #[command(subcommand)]
    command: CacheCommand,
}
#[derive(clap::Subcommand)]
enum CacheCommand {
    /// Reclaim inactive project generations for one exact workspace under pressure
    PruneProject {
        workspace: std::path::PathBuf,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, default_value_t = 0)]
        reserve_bytes: u64,
    },
    /// Show sizes and protected entries in the selected owned cache
    Inspect {
        #[arg(long)]
        json: bool,
    },
    /// Reclaim inactive owned entries only under free-space pressure
    Prune {
        #[arg(long)]
        dry_run: bool,
        /// Required free bytes, selected from the operation's resource policy
        #[arg(long, default_value_t = 0)]
        reserve_bytes: u64,
    },
}
pub(crate) fn run(args: CacheArgs) -> i32 {
    let result = match args.command {
        CacheCommand::PruneProject { workspace, dry_run, reserve_bytes } => {
            sifr_driver::cache_storage::available_bytes().and_then(|available| {
                let generations = sifr_driver::project_cache::prune_project_cache(
                    &workspace, available < reserve_bytes, dry_run)?;
                Ok(serde_json::json!({"workspace": workspace, "dry_run": dry_run,
                    "reserve_bytes": reserve_bytes, "available_bytes": available,
                    "eligible_generations": generations}))
            })
        },
        CacheCommand::Inspect { json } => match sifr_driver::cache_storage::inspect() {
            Ok(report) if !json => {
                let _ = writeln!(io::stdout(), "cache: {}", report.root.display());
                for root in report.protected_roots {
                    let _ = writeln!(io::stdout(), "{} protected=true (auxiliary owner)", root.display());
                }
                for entry in report.entries {
                    let _ = writeln!(io::stdout(), "{} bytes={} protected={}",
                        entry.path.display(), entry.bytes, entry.protected);
                }
                return 0;
            }
            Ok(report) => serde_json::to_value(report).map_err(io::Error::other),
            Err(error) => Err(error),
        },
        CacheCommand::Prune { dry_run, reserve_bytes } => {
            sifr_driver::cache_storage::available_bytes().and_then(|available| {
                let report = sifr_driver::cache_storage::prune(reserve_bytes, available, dry_run)?;
                let after = sifr_driver::cache_storage::available_bytes()?;
                if !dry_run && after < reserve_bytes {
                    return Err(io::Error::other(format!(
                        "resource blocker: scope={} reserve_bytes={reserve_bytes} available_bytes={after}; protected entries retained; free owner-scoped storage or select another private cache filesystem",
                        std::env::current_dir()?.display())));
                }
                Ok(serde_json::json!({"dry_run": dry_run, "reason": "free_space_reserve",
                    "reserve_bytes": reserve_bytes, "available_bytes": available, "scope": std::env::current_dir()?,
                    "cache": report}))
            })
        }
    };
    match result {
        Ok(report) => {
            let _ = writeln!(io::stdout(), "{report}");
            0
        }
        Err(error) => {
            let _ = writeln!(
                io::stderr(),
                "cache: {error}; select a private SIFR_CACHE_DIR"
            );
            2
        }
    }
}
