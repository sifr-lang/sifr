use super::cli_model_and_entrypoint::{
    DiagnosticFormat, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG, diagnostic_with_code,
};
use super::diagnostic_rendering_and_run::render_diagnostics;
use clap::{Args, Subcommand};
use sifr_diagnostics::DiagnosticCode;
use sifr_driver::ArtifactCacheCleanPolicy;
use std::io::{self, Write as _};
use std::time::Duration;

#[derive(Args)]
pub(crate) struct CacheArgs {
    #[command(subcommand)]
    command: CacheCommands,
}

#[derive(Subcommand)]
enum CacheCommands {
    /// Report the generated-artifact cache footprint
    Status {
        /// Print machine-readable JSON
        #[arg(long)]
        json: bool,
        /// Stop after visiting this many filesystem nodes
        #[arg(
            long,
            value_name = "NODES",
            default_value_t = sifr_driver::DEFAULT_CACHE_SCAN_NODE_LIMIT
        )]
        scan_node_limit: usize,
    },
    /// Remove generated-artifact cache entries by an explicit policy
    Clean {
        /// Remove the complete Sifr-owned generated-artifact cache
        #[arg(long, conflicts_with_all = ["max_age_days", "max_size_mib"])]
        all: bool,
        /// Remove entries older than this number of days
        #[arg(long, value_name = "DAYS")]
        max_age_days: Option<u64>,
        /// Remove oldest entries until the cache is at most this size
        #[arg(long, value_name = "MIB")]
        max_size_mib: Option<u64>,
        /// Report removals without changing the cache
        #[arg(long)]
        dry_run: bool,
        /// Print machine-readable JSON
        #[arg(long)]
        json: bool,
        /// Stop after visiting this many filesystem nodes
        #[arg(
            long,
            value_name = "NODES",
            default_value_t = sifr_driver::DEFAULT_CACHE_SCAN_NODE_LIMIT
        )]
        scan_node_limit: usize,
    },
}

pub(super) fn cmd_cache(args: CacheArgs, diagnostic_format: DiagnosticFormat) -> i32 {
    let CacheArgs { command } = args;
    match command {
        CacheCommands::Status {
            json,
            scan_node_limit,
        } => match sifr_driver::artifact_cache_status_with_limit(scan_node_limit) {
            Ok(status) => {
                if json {
                    write_json(&status)
                } else {
                    let oldest = status
                        .oldest_entry_age_seconds
                        .map_or_else(|| "none".to_string(), |seconds| format!("{seconds}s"));
                    let _ = writeln!(
                        io::stdout(),
                        "Cache: {}\nEntries: {}\nBytes: {}\nOldest: {}\nScan complete: {}\nScanned nodes: {}",
                        status.root.display(),
                        status.entries,
                        status.bytes,
                        oldest,
                        status.scan_complete,
                        status.scanned_nodes
                    );
                    EXIT_SUCCESS
                }
            }
            Err(error) => cache_error(&error, diagnostic_format),
        },
        CacheCommands::Clean {
            all,
            max_age_days,
            max_size_mib,
            dry_run,
            json,
            scan_node_limit,
        } => {
            let policy =
                match clean_policy(all, max_age_days, max_size_mib, scan_node_limit, dry_run) {
                    Ok(policy) => policy,
                    Err(error) => return cache_error(&error, diagnostic_format),
                };
            match sifr_driver::clean_artifact_cache(&policy) {
                Ok(report) => {
                    if json {
                        write_json(&report)
                    } else if report.removed_all {
                        let action = if report.dry_run {
                            "Would remove"
                        } else {
                            "Removed"
                        };
                        let measurement = if report.scan_complete {
                            "complete"
                        } else {
                            "partial"
                        };
                        let _ = writeln!(
                            io::stdout(),
                            "{action} the complete cache. {measurement} scan measured {} entries ({} bytes).\nCache: {}",
                            report.removed_entries,
                            report.reclaimed_bytes,
                            report.root.display()
                        );
                        EXIT_SUCCESS
                    } else {
                        let action = if report.dry_run {
                            "Would remove"
                        } else {
                            "Removed"
                        };
                        let _ = writeln!(
                            io::stdout(),
                            "{action}: {} entries ({} bytes)\nRemaining: {} entries ({} bytes)\nScan complete: {}\nCache: {}",
                            report.removed_entries,
                            report.reclaimed_bytes,
                            report.remaining_entries,
                            report.remaining_bytes,
                            report.scan_complete,
                            report.root.display()
                        );
                        EXIT_SUCCESS
                    }
                }
                Err(error) => cache_error(&error, diagnostic_format),
            }
        }
    }
}

fn clean_policy(
    all: bool,
    max_age_days: Option<u64>,
    max_size_mib: Option<u64>,
    scan_node_limit: usize,
    dry_run: bool,
) -> Result<ArtifactCacheCleanPolicy, io::Error> {
    if !all && max_age_days.is_none() && max_size_mib.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cache clean requires --all, --max-age-days, or --max-size-mib",
        ));
    }
    if scan_node_limit == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cache scan node limit must be positive",
        ));
    }
    let max_age = max_age_days
        .map(|days| {
            days.checked_mul(86_400)
                .map(Duration::from_secs)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "cache age is too large")
                })
        })
        .transpose()?;
    let max_bytes = max_size_mib
        .map(|mib| {
            mib.checked_mul(1024 * 1024).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "cache size is too large")
            })
        })
        .transpose()?;
    Ok(ArtifactCacheCleanPolicy {
        remove_all: all,
        max_age,
        max_bytes,
        scan_node_limit,
        dry_run,
    })
}

fn write_json(value: &impl serde::Serialize) -> i32 {
    match serde_json::to_string_pretty(value) {
        Ok(rendered) => {
            let _ = writeln!(io::stdout(), "{rendered}");
            EXIT_SUCCESS
        }
        Err(error) => {
            let _ = writeln!(io::stderr(), "could not serialize cache report: {error}");
            EXIT_USAGE_OR_CONFIG
        }
    }
}

fn cache_error(error: &io::Error, diagnostic_format: DiagnosticFormat) -> i32 {
    let diagnostic = diagnostic_with_code(
        format!("cache operation failed: {error}"),
        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
    );
    render_diagnostics(&[diagnostic], diagnostic_format)
}

#[cfg(test)]
mod tests {
    use super::clean_policy;

    #[test]
    fn clean_policy_requires_an_explicit_bound() {
        assert!(clean_policy(false, None, None, 100, false).is_err());
        assert_eq!(
            clean_policy(false, Some(2), Some(4), 100, true)
                .expect("bounded policy should parse")
                .max_bytes,
            Some(4 * 1024 * 1024)
        );
    }
}
