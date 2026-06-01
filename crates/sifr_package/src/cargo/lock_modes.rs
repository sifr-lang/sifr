use crate::cargo::metadata::NormalizedCargoMetadata;
use crate::diag::PackageDiagnostic;
use sifr_frontend::{DiskSourceProvider, SourceProvider};

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

pub fn validate_offline_source_availability(
    metadata: &NormalizedCargoMetadata,
    lock_mode: CargoLockMode,
) -> Result<(), Vec<PackageDiagnostic>> {
    let mut provider = DiskSourceProvider::new();
    validate_offline_source_availability_with_provider(metadata, lock_mode, &mut provider)
}

pub fn validate_offline_source_availability_with_provider(
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
