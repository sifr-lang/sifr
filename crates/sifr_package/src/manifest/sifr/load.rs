use super::SifrManifest;
use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use sifr_frontend::SourceProvider;
use std::path::Path;

impl SifrManifest {
    pub fn load(
        cargo_package_id: &CargoPackageId,
        manifest_path: &Path,
        provider: &mut impl SourceProvider,
    ) -> Result<Self, PackageDiagnostic> {
        let source = provider.read_file(manifest_path).map_err(|error| {
            PackageDiagnostic::missing_sifr_manifest(
                cargo_package_id,
                manifest_path.to_path_buf(),
                error.to_string(),
            )
        })?;
        Self::parse(cargo_package_id, manifest_path, source.as_str())
    }
}
