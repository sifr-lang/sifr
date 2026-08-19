use crate::cargo::metadata::NormalizedCargoMetadata;
use crate::diag::PackageDiagnostic;
use sifr_frontend::SourceProvider;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CargoLockMode {
    #[default]
    Normal,
    Locked,
    Offline,
    Frozen,
}

impl CargoLockMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Locked => "locked",
            Self::Offline => "offline",
            Self::Frozen => "frozen",
        }
    }

    #[must_use]
    pub const fn cargo_arg(self) -> Option<&'static str> {
        match self {
            Self::Normal => None,
            Self::Locked => Some("--locked"),
            Self::Offline => Some("--offline"),
            Self::Frozen => Some("--frozen"),
        }
    }

    #[must_use]
    pub const fn is_network_disallowed(self) -> bool {
        matches!(self, Self::Offline | Self::Frozen)
    }

    #[must_use]
    pub const fn is_lock_mutation_disallowed(self) -> bool {
        matches!(self, Self::Locked | Self::Frozen)
    }
}

#[must_use]
pub fn cargo_lock_failure_reason(stderr: &str) -> Option<&'static str> {
    let normalized = stderr.to_ascii_lowercase();
    if normalized.contains("checksum for") && normalized.contains("changed between lock files") {
        return Some("checksum drift");
    }
    if normalized.contains("cannot create the lock file") {
        return Some("missing lockfile");
    }
    if normalized.contains("cannot update the lock file")
        || normalized.contains("lock file needs to be updated")
    {
        return Some("stale lockfile or feature/source drift");
    }
    if normalized.contains("failed to select a version for the requirement")
        || normalized.contains("no matching package named")
    {
        return Some("locked package selection drift");
    }
    if normalized.contains("attempting to make an http request")
        || (normalized.contains("offline") && normalized.contains("failed"))
    {
        return Some("offline source unavailable");
    }
    None
}

pub fn validate_offline_source_availability(
    metadata: &NormalizedCargoMetadata,
    lock_mode: CargoLockMode,
    provider: &mut impl SourceProvider,
) -> Result<(), Vec<PackageDiagnostic>> {
    if !lock_mode.is_network_disallowed() {
        return Ok(());
    }

    let diagnostics = metadata
        .packages
        .values()
        .filter(|package| package.sifr_metadata.is_some())
        .filter_map(|package| {
            let package_root = package.manifest_path.parent()?;
            if provider.is_dir(package_root) {
                None
            } else {
                Some(PackageDiagnostic::source_unavailable_offline(
                    &package.id,
                    package_root,
                    lock_mode,
                ))
            }
        })
        .collect::<Vec<_>>();

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

#[cfg(test)]
mod tests {
    use super::{cargo_lock_failure_reason, CargoLockMode};

    #[test]
    fn lock_modes_expose_exact_cargo_arguments() {
        assert_eq!(CargoLockMode::Normal.cargo_arg(), None);
        assert_eq!(CargoLockMode::Locked.cargo_arg(), Some("--locked"));
        assert_eq!(CargoLockMode::Offline.cargo_arg(), Some("--offline"));
        assert_eq!(CargoLockMode::Frozen.cargo_arg(), Some("--frozen"));
    }

    #[test]
    fn cargo_lock_failures_are_classified_before_rust_resolution() {
        for (stderr, expected) in [
            (
                "checksum for `indexmap` changed between lock files",
                "checksum drift",
            ),
            (
                "cannot create the lock file in frozen mode",
                "missing lockfile",
            ),
            (
                "the lock file needs to be updated but --locked was passed",
                "stale lockfile or feature/source drift",
            ),
            (
                "failed to select a version for the requirement `indexmap`",
                "locked package selection drift",
            ),
            (
                "attempting to make an HTTP request, but --offline was specified",
                "offline source unavailable",
            ),
        ] {
            assert_eq!(cargo_lock_failure_reason(stderr), Some(expected));
        }
    }
}
